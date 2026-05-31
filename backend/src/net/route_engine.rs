//! Per-request routing: should this target go DIRECT or through the UPSTREAM
//! proxy?
//!
//! The engine always forces DIRECT for developer-local traffic (loopback,
//! `*.local`, private IP ranges) so connecting to a proxy never breaks
//! `localhost` or LAN dev servers. Beyond that, two modes are supported:
//! [`RouteMode::Global`] (everything else goes upstream) and
//! [`RouteMode::DomainRules`] (explicit per-domain DIRECT/UPSTREAM list with a
//! default fallback). The decision function is pure and unit-testable.

use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDecision {
    Direct,
    Upstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteMode {
    /// Everything not force-bypassed is routed through the upstream proxy.
    Global,
    /// Only domains matched by a rule are routed; unmatched falls back to the
    /// engine's `default_action`.
    DomainRules,
}

/// A single domain routing rule. `pattern` matches a host exactly or as a
/// suffix when prefixed with `*.` (e.g. `*.example.com`).
#[derive(Debug, Clone)]
pub struct DomainRule {
    pub pattern: String,
    pub action: RouteDecision,
}

impl DomainRule {
    fn matches(&self, host: &str) -> bool {
        let host = host.to_ascii_lowercase();
        let pat = self.pattern.to_ascii_lowercase();
        if let Some(suffix) = pat.strip_prefix("*.") {
            host == suffix || host.ends_with(&format!(".{suffix}"))
        } else {
            host == pat
        }
    }
}

/// Returns true for hosts that must never leave the machine through a proxy:
/// loopback names/IPs, `*.local` (mDNS), and RFC 1918 / link-local private IPs.
pub fn is_bypass(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_ascii_lowercase();

    if h == "localhost" || h.ends_with(".localhost") {
        return true;
    }
    if h == "local" || h.ends_with(".local") {
        return true;
    }

    if let Ok(ip) = h.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => {
                v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified()
            }
            IpAddr::V6(v6) => {
                v6.is_loopback()
                    || v6.is_unspecified()
                    // unique-local fc00::/7
                    || (v6.segments()[0] & 0xfe00) == 0xfc00
                    // link-local fe80::/10
                    || (v6.segments()[0] & 0xffc0) == 0xfe80
            }
        };
    }

    false
}

/// Routing engine: holds the active mode, optional domain rules, and the
/// fallback action used by [`RouteMode::DomainRules`] for unmatched hosts.
pub struct RouteEngine {
    pub mode: RouteMode,
    pub rules: Vec<DomainRule>,
    pub default_action: RouteDecision,
}

impl RouteEngine {
    /// Engine used right after a plain connect: route everything (except local
    /// traffic) through the upstream proxy.
    pub fn global() -> Self {
        Self {
            mode: RouteMode::Global,
            rules: Vec::new(),
            default_action: RouteDecision::Upstream,
        }
    }

    /// Decide how to route a request for `host`.
    pub fn decide(&self, host: &str) -> RouteDecision {
        if is_bypass(host) {
            return RouteDecision::Direct;
        }
        match self.mode {
            RouteMode::Global => RouteDecision::Upstream,
            RouteMode::DomainRules => self
                .rules
                .iter()
                .find(|r| r.matches(host))
                .map(|r| r.action)
                .unwrap_or(self.default_action),
        }
    }
}
