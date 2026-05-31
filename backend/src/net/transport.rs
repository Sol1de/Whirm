//! Upstream connection strategies.
//!
//! A [`Transport`] knows how to open a byte stream to a target `host:port`.
//! The relay asks the [`super::RouteEngine`] for a decision and then calls the
//! matching transport: [`DirectTransport`] for DIRECT, or the configured
//! upstream transport ([`Socks5Transport`] / [`HttpProxyTransport`] /
//! [`HttpsProxyTransport`]) for UPSTREAM.
//!
//! ## Future: `WhirmGatewayTransport`
//! A hosted "Whirm Gateway" would be added here as another `Transport`
//! implementation speaking **standard HTTPS** (e.g. CONNECT-over-TLS or an
//! HTTP/2 tunnel). It is intentionally not implemented yet — this trait is the
//! single seam it would plug into, with no changes required in the relay.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;
use tokio_socks::tcp::Socks5Stream;

use super::{BoxedStream, NetError};

// ─── PURE HELPERS (unit-testable without sockets) ────────────────────────────

/// Build the `Proxy-Authorization` header value for HTTP CONNECT auth.
/// Returns e.g. `Basic dXNlcjpwYXNz`.
pub fn basic_auth_header(username: &str, password: &str) -> String {
    let token = STANDARD.encode(format!("{username}:{password}"));
    format!("Basic {token}")
}

/// Build a full HTTP `CONNECT` request head (terminated by a blank line).
/// `auth` is `Some((user, pass))` when the upstream proxy requires credentials.
pub fn build_connect_request(host: &str, port: u16, auth: Option<(&str, &str)>) -> String {
    let mut req = format!("CONNECT {host}:{port} HTTP/1.1\r\nHost: {host}:{port}\r\n");
    if let Some((u, p)) = auth {
        if !u.is_empty() {
            req.push_str(&format!("Proxy-Authorization: {}\r\n", basic_auth_header(u, p)));
        }
    }
    req.push_str("\r\n");
    req
}

/// Parse the status code out of an HTTP response status line such as
/// `HTTP/1.1 200 Connection Established`. Returns `None` if malformed.
pub fn parse_status_code(status_line: &str) -> Option<u16> {
    status_line.split_whitespace().nth(1)?.parse().ok()
}

// ─── TRANSPORT TRAIT ─────────────────────────────────────────────────────────

#[async_trait]
pub trait Transport: Send + Sync {
    /// Open a stream carrying traffic to `host:port`.
    async fn connect(&self, host: &str, port: u16) -> Result<BoxedStream, NetError>;
}

/// Resolve `host:port` to a concrete socket address (local DNS resolution).
/// Used when DNS-leak protection is OFF so the upstream receives an IP.
async fn resolve_one(host: &str, port: u16) -> Result<SocketAddr, NetError> {
    tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| NetError::Resolve(format!("{host}: {e}")))?
        .next()
        .ok_or_else(|| NetError::Resolve(host.to_string()))
}

// ─── DIRECT ──────────────────────────────────────────────────────────────────

/// Connects straight to the target. Used for DIRECT routing decisions
/// (loopback, `*.local`, private IPs, and DomainRules marked DIRECT).
pub struct DirectTransport;

#[async_trait]
impl Transport for DirectTransport {
    async fn connect(&self, host: &str, port: u16) -> Result<BoxedStream, NetError> {
        let stream = TcpStream::connect((host, port)).await?;
        Ok(Box::new(stream))
    }
}

// ─── SOCKS5 ──────────────────────────────────────────────────────────────────

/// Routes through an upstream SOCKS5 proxy (with optional username/password
/// auth per RFC 1929). When `remote_dns` is true the hostname is handed to the
/// proxy for remote resolution (SOCKS5h); otherwise it is resolved locally.
pub struct Socks5Transport {
    pub proxy_addr: String, // "host:port" of the upstream proxy
    pub username: Option<String>,
    pub password: Option<String>,
    pub remote_dns: bool,
}

#[async_trait]
impl Transport for Socks5Transport {
    async fn connect(&self, host: &str, port: u16) -> Result<BoxedStream, NetError> {
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() => Some((u.as_str(), p.as_str())),
            _ => None,
        };

        let stream = if self.remote_dns {
            // Hand the domain to the proxy → remote resolution (SOCKS5h).
            match auth {
                Some((u, p)) => {
                    Socks5Stream::connect_with_password(self.proxy_addr.as_str(), (host, port), u, p)
                        .await
                }
                None => Socks5Stream::connect(self.proxy_addr.as_str(), (host, port)).await,
            }
        } else {
            // Resolve locally first, then send the IP to the proxy.
            let addr = resolve_one(host, port).await?;
            match auth {
                Some((u, p)) => {
                    Socks5Stream::connect_with_password(self.proxy_addr.as_str(), addr, u, p).await
                }
                None => Socks5Stream::connect(self.proxy_addr.as_str(), addr).await,
            }
        }
        .map_err(|e| NetError::Proxy(e.to_string()))?;

        Ok(Box::new(stream))
    }
}

// ─── HTTP CONNECT (shared) ───────────────────────────────────────────────────

/// Send a CONNECT request over `stream` and validate the `2xx` response.
async fn perform_connect<S>(
    stream: &mut S,
    target_host: &str,
    target_port: u16,
    auth: Option<(&str, &str)>,
) -> Result<(), NetError>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let req = build_connect_request(target_host, target_port, auth);
    stream.write_all(req.as_bytes()).await?;
    stream.flush().await?;

    // Read the response head up to the terminating CRLFCRLF.
    let mut buf = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    loop {
        let n = stream.read(&mut byte).await?;
        if n == 0 {
            return Err(NetError::Proxy("upstream closed during CONNECT".into()));
        }
        buf.push(byte[0]);
        if buf.ends_with(b"\r\n\r\n") {
            break;
        }
        if buf.len() > 8192 {
            return Err(NetError::Proxy("CONNECT response too large".into()));
        }
    }

    let head = String::from_utf8_lossy(&buf);
    let status_line = head.lines().next().unwrap_or_default();
    match parse_status_code(status_line) {
        Some(code) if (200..300).contains(&code) => Ok(()),
        Some(code) => Err(NetError::Proxy(format!("CONNECT rejected: {code}"))),
        None => Err(NetError::Proxy(format!("malformed CONNECT response: {status_line}"))),
    }
}

/// Resolve to an IP string when remote DNS is disabled, else keep the hostname.
async fn connect_target(host: &str, port: u16, remote_dns: bool) -> Result<(String, u16), NetError> {
    if remote_dns {
        Ok((host.to_string(), port))
    } else {
        let addr = resolve_one(host, port).await?;
        Ok((addr.ip().to_string(), addr.port()))
    }
}

// ─── HTTP PROXY ──────────────────────────────────────────────────────────────

/// Routes through an upstream HTTP proxy via the CONNECT method.
pub struct HttpProxyTransport {
    pub proxy_addr: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub remote_dns: bool,
}

#[async_trait]
impl Transport for HttpProxyTransport {
    async fn connect(&self, host: &str, port: u16) -> Result<BoxedStream, NetError> {
        let mut stream = TcpStream::connect(&self.proxy_addr).await?;
        let (t_host, t_port) = connect_target(host, port, self.remote_dns).await?;
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() => Some((u.as_str(), p.as_str())),
            _ => None,
        };
        perform_connect(&mut stream, &t_host, t_port, auth).await?;
        Ok(Box::new(stream))
    }
}

// ─── HTTPS PROXY ─────────────────────────────────────────────────────────────

/// Routes through an upstream HTTPS proxy: TLS to the proxy, then CONNECT
/// through the encrypted channel.
pub struct HttpsProxyTransport {
    pub proxy_host: String,
    pub proxy_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub remote_dns: bool,
    tls: Arc<ClientConfig>,
}

impl HttpsProxyTransport {
    pub fn new(
        proxy_host: String,
        proxy_port: u16,
        username: Option<String>,
        password: Option<String>,
        remote_dns: bool,
    ) -> Self {
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        Self {
            proxy_host,
            proxy_port,
            username,
            password,
            remote_dns,
            tls: Arc::new(config),
        }
    }
}

#[async_trait]
impl Transport for HttpsProxyTransport {
    async fn connect(&self, host: &str, port: u16) -> Result<BoxedStream, NetError> {
        let tcp = TcpStream::connect((self.proxy_host.as_str(), self.proxy_port)).await?;
        let connector = TlsConnector::from(self.tls.clone());
        let server_name = ServerName::try_from(self.proxy_host.clone())
            .map_err(|_| NetError::Tls(format!("invalid proxy hostname: {}", self.proxy_host)))?;
        let mut stream = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| NetError::Tls(e.to_string()))?;

        let (t_host, t_port) = connect_target(host, port, self.remote_dns).await?;
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) if !u.is_empty() => Some((u.as_str(), p.as_str())),
            _ => None,
        };
        perform_connect(&mut stream, &t_host, t_port, auth).await?;
        Ok(Box::new(stream))
    }
}
