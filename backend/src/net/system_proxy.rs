//! Thin wrapper over the `sysproxy` crate.
//!
//! Centralizes the OS system-proxy mutation so the command layer never builds a
//! `Sysproxy` by hand. The relay always listens on loopback, so the OS proxy is
//! always pointed at `127.0.0.1:<relay_port>` with a fixed bypass list.

use sysproxy::Sysproxy;

/// Hosts the OS should reach without going through the relay. The relay also
/// enforces this via [`super::RouteEngine`], but setting it at the OS level
/// avoids an unnecessary loopback hop for local traffic.
pub const BYPASS: &str = "localhost,127.0.0.1,<local>";

/// Read the current system proxy so it can be restored on disconnect/exit.
pub fn capture() -> Result<Sysproxy, String> {
    Sysproxy::get_system_proxy().map_err(|e| format!("Failed to read current system proxy: {e}"))
}

/// Point the OS system proxy at the local relay on `127.0.0.1:port`.
pub fn apply(port: u16) -> Result<(), String> {
    let proxy = Sysproxy {
        enable: true,
        host: "127.0.0.1".to_string(),
        port,
        bypass: BYPASS.to_string(),
    };
    proxy
        .set_system_proxy()
        .map_err(|e| format!("Failed to apply system proxy: {e}"))
}

/// Restore a previously captured system proxy state.
pub fn restore(saved: Sysproxy) -> Result<(), String> {
    saved
        .set_system_proxy()
        .map_err(|e| format!("Failed to restore system proxy: {e}"))
}
