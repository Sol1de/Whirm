//! Local forwarding relay.
//!
//! Binds an ephemeral loopback port the OS system proxy points at, speaks the
//! HTTP-proxy protocol the OS emits (`CONNECT` tunnels, plus best-effort
//! absolute-form HTTP), asks the [`RouteEngine`] for a DIRECT/UPSTREAM
//! decision, opens the matching [`Transport`], and pumps bytes both ways while
//! counting them into [`RelayStats`].

use std::pin::Pin;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

use super::route_engine::RouteDecision;
use super::transport::DirectTransport;
use super::{RelayHandle, RelayStats, RouteEngine, Transport};

const MAX_HEAD: usize = 16 * 1024;

/// Parsed first line of an inbound proxy request.
#[derive(Debug, PartialEq, Eq)]
pub struct RequestLine {
    pub method: String,
    pub target: String,
    pub version: String,
}

/// Parse an HTTP request line like `CONNECT example.com:443 HTTP/1.1`.
pub fn parse_request_line(line: &str) -> Option<RequestLine> {
    let mut parts = line.trim_end().split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    Some(RequestLine { method, target, version })
}

/// Split a CONNECT authority (`host:port`) into parts, defaulting to 443.
pub fn parse_connect_target(target: &str) -> Option<(String, u16)> {
    let (host, port) = match target.rsplit_once(':') {
        Some((h, p)) => (h, p.parse().ok()?),
        None => (target, 443),
    };
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port))
}

/// Extract `host:port` from an absolute-form request URI (`http://host/path`),
/// defaulting to port 80. Returns the host, port, and origin-form path.
pub fn parse_absolute_target(uri: &str) -> Option<(String, u16, String)> {
    let rest = uri.strip_prefix("http://")?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().ok()?),
        None => (authority.to_string(), 80),
    };
    if host.is_empty() {
        return None;
    }
    Some((host, port, path.to_string()))
}

// ─── BYTE-COUNTING WRAPPER ───────────────────────────────────────────────────

/// Wraps the client-side stream so traffic is metered live into the shared
/// [`RelayStats`]: bytes read from the client count as `up`, bytes written to
/// the client count as `down`.
struct Counting<S> {
    inner: S,
    stats: Arc<RelayStats>,
}

impl<S: AsyncRead + Unpin> AsyncRead for Counting<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let res = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &res {
            let read = buf.filled().len() - before;
            if read > 0 {
                self.stats.up.fetch_add(read as u64, Ordering::Relaxed);
            }
        }
        res
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for Counting<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let res = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(Ok(n)) = &res {
            self.stats.down.fetch_add(*n as u64, Ordering::Relaxed);
        }
        res
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

// ─── RELAY LIFECYCLE ─────────────────────────────────────────────────────────

/// Bind the relay on `127.0.0.1:0`, spawn the accept loop, and return a handle
/// carrying the assigned port, a cancellation token, and live stats.
pub async fn start(upstream: Arc<dyn Transport>, engine: Arc<RouteEngine>) -> Result<RelayHandle, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| format!("Failed to bind relay: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to read relay port: {e}"))?
        .port();

    let cancel = CancellationToken::new();
    let stats = Arc::new(RelayStats::default());

    let task_cancel = cancel.clone();
    let task_stats = stats.clone();
    tokio::spawn(async move {
        accept_loop(listener, upstream, engine, task_cancel, task_stats).await;
    });

    Ok(RelayHandle { port, cancel, stats })
}

async fn accept_loop(
    listener: TcpListener,
    upstream: Arc<dyn Transport>,
    engine: Arc<RouteEngine>,
    cancel: CancellationToken,
    stats: Arc<RelayStats>,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            accepted = listener.accept() => {
                let Ok((client, _)) = accepted else { continue };
                let upstream = upstream.clone();
                let engine = engine.clone();
                let stats = stats.clone();
                let conn_cancel = cancel.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = conn_cancel.cancelled() => {}
                        r = handle_conn(client, upstream, engine, stats) => {
                            if let Err(e) = r {
                                log::debug!("relay connection ended: {e}");
                            }
                        }
                    }
                });
            }
        }
    }
}

/// Pick the transport for a host based on the routing decision.
fn select_transport<'a>(
    engine: &RouteEngine,
    upstream: &'a Arc<dyn Transport>,
    direct: &'a DirectTransport,
    host: &str,
) -> &'a dyn Transport {
    match engine.decide(host) {
        RouteDecision::Direct => direct,
        RouteDecision::Upstream => upstream.as_ref(),
    }
}

async fn handle_conn(
    mut client: TcpStream,
    upstream: Arc<dyn Transport>,
    engine: Arc<RouteEngine>,
    stats: Arc<RelayStats>,
) -> Result<(), String> {
    let direct = DirectTransport;

    // Read the request head up to CRLFCRLF.
    let mut head = Vec::with_capacity(512);
    let mut byte = [0u8; 1];
    loop {
        let n = client.read(&mut byte).await.map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(()); // client gone
        }
        head.push(byte[0]);
        if head.ends_with(b"\r\n\r\n") {
            break;
        }
        if head.len() > MAX_HEAD {
            return Err("request head too large".into());
        }
    }

    let head_str = String::from_utf8_lossy(&head);
    let first_line = head_str.lines().next().unwrap_or_default();
    let req = parse_request_line(first_line).ok_or("malformed request line")?;

    if req.method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) = parse_connect_target(&req.target).ok_or("bad CONNECT target")?;
        let transport = select_transport(&engine, &upstream, &direct, &host);
        let upstream_stream = transport.connect(&host, port).await.map_err(|e| e.to_string())?;

        client
            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await
            .map_err(|e| e.to_string())?;

        pump(client, upstream_stream, &stats).await
    } else {
        // Best-effort plain-HTTP: rewrite absolute-form to origin-form and
        // replay the head into a tunnel to the target host on port 80.
        let (host, port, path) =
            parse_absolute_target(&req.target).ok_or("unsupported request target")?;
        let transport = select_transport(&engine, &upstream, &direct, &host);
        let mut upstream_stream = transport.connect(&host, port).await.map_err(|e| e.to_string())?;

        let rewritten = head_str.replacen(
            &format!("{} {} {}", req.method, req.target, req.version),
            &format!("{} {} {}", req.method, path, req.version),
            1,
        );
        upstream_stream
            .write_all(rewritten.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        pump(client, upstream_stream, &stats).await
    }
}

/// Pump bytes both ways between the (metered) client and the upstream stream.
/// The [`Counting`] wrapper updates the shared [`RelayStats`] live as bytes flow.
async fn pump(
    client: TcpStream,
    mut upstream_stream: super::BoxedStream,
    stats: &Arc<RelayStats>,
) -> Result<(), String> {
    let mut counted = Counting {
        inner: client,
        stats: stats.clone(),
    };

    tokio::io::copy_bidirectional(&mut counted, &mut upstream_stream)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
