//! Network core for Whirm.
//!
//! Whirm is an app-level proxy router, **not** a VPN. When the user connects to
//! an upstream proxy we start a small local relay (`relay.rs`) bound to an
//! ephemeral loopback port and point the OS system proxy at it. The relay
//! fronts a pluggable [`Transport`] (the upstream connection strategy) and a
//! [`RouteEngine`] (per-request DIRECT vs UPSTREAM decision), so authenticated
//! and SOCKS5 proxies route correctly — something the raw OS `host:port` proxy
//! setting cannot express.
//!
//! There is no TUN / WireGuard / OpenVPN here and none is planned. A future
//! "Whirm Gateway" may be added as another [`Transport`] over standard HTTPS.

pub mod relay;
pub mod route_engine;
pub mod system_proxy;
pub mod transport;

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::sync::CancellationToken;

pub use route_engine::{DomainRule, RouteDecision, RouteEngine, RouteMode};
pub use transport::{
    DirectTransport, HttpProxyTransport, HttpsProxyTransport, Socks5Transport, Transport,
};

/// Any bidirectional byte stream the relay can pump through
/// [`tokio::io::copy_bidirectional`]. Every concrete transport erases to this.
pub trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> AsyncStream for T {}

/// Boxed [`AsyncStream`] returned by [`Transport::connect`].
pub type BoxedStream = Box<dyn AsyncStream>;

/// Errors raised by the network core. Convertible to `String` to match the
/// `Result<_, String>` convention used by the Tauri command layer.
#[derive(Debug)]
pub enum NetError {
    Io(std::io::Error),
    Resolve(String),
    Proxy(String),
    Tls(String),
    Protocol(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetError::Io(e) => write!(f, "I/O error: {e}"),
            NetError::Resolve(h) => write!(f, "Could not resolve host: {h}"),
            NetError::Proxy(m) => write!(f, "Upstream proxy error: {m}"),
            NetError::Tls(m) => write!(f, "TLS error: {m}"),
            NetError::Protocol(m) => write!(f, "Protocol error: {m}"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<std::io::Error> for NetError {
    fn from(e: std::io::Error) -> Self {
        NetError::Io(e)
    }
}

impl From<NetError> for String {
    fn from(e: NetError) -> Self {
        e.to_string()
    }
}

/// Live byte counters shared between the relay tasks and the throughput command.
/// `down` = bytes sent from upstream to the client, `up` = client to upstream.
#[derive(Debug, Default)]
pub struct RelayStats {
    pub down: AtomicU64,
    pub up: AtomicU64,
}

/// Handle to a running relay. Dropping it does not stop the relay — call
/// [`CancellationToken::cancel`] (via [`RelayHandle::shutdown`]) first.
pub struct RelayHandle {
    /// Ephemeral loopback port the OS system proxy is pointed at.
    pub port: u16,
    /// Cancels the accept loop and all in-flight tunnels.
    pub cancel: CancellationToken,
    /// Live throughput counters.
    pub stats: Arc<RelayStats>,
}

impl RelayHandle {
    /// Signal the relay to stop accepting and tear down active tunnels.
    pub fn shutdown(&self) {
        self.cancel.cancel();
    }
}
