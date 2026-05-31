use app_lib::net::route_engine::is_bypass;
use app_lib::net::{DomainRule, RouteDecision, RouteEngine, RouteMode};

// ─── dev-safe bypass ─────────────────────────────────────────────────────────

#[test]
fn bypass_covers_loopback_names_and_ips() {
    assert!(is_bypass("localhost"));
    assert!(is_bypass("foo.localhost"));
    assert!(is_bypass("127.0.0.1"));
    assert!(is_bypass("::1"));
}

#[test]
fn bypass_covers_mdns_local() {
    assert!(is_bypass("myhost.local"));
    assert!(is_bypass("printer.local."));
}

#[test]
fn bypass_covers_private_ipv4_ranges() {
    assert!(is_bypass("10.0.0.5"));
    assert!(is_bypass("172.16.4.9"));
    assert!(is_bypass("192.168.1.1"));
    assert!(is_bypass("169.254.0.1")); // link-local
}

#[test]
fn bypass_covers_private_ipv6_ranges() {
    assert!(is_bypass("fc00::1")); // unique-local
    assert!(is_bypass("fe80::1")); // link-local
}

#[test]
fn bypass_excludes_public_hosts() {
    assert!(!is_bypass("example.com"));
    assert!(!is_bypass("8.8.8.8"));
    assert!(!is_bypass("1.1.1.1"));
}

// ─── global mode ─────────────────────────────────────────────────────────────

#[test]
fn global_routes_public_upstream_and_local_direct() {
    let engine = RouteEngine::global();
    assert_eq!(engine.decide("example.com"), RouteDecision::Upstream);
    assert_eq!(engine.decide("localhost"), RouteDecision::Direct);
    assert_eq!(engine.decide("192.168.0.10"), RouteDecision::Direct);
}

// ─── domain rules mode ───────────────────────────────────────────────────────

fn rules_engine() -> RouteEngine {
    RouteEngine {
        mode: RouteMode::DomainRules,
        rules: vec![
            DomainRule { pattern: "blocked.com".into(), action: RouteDecision::Upstream },
            DomainRule { pattern: "*.internal.dev".into(), action: RouteDecision::Direct },
        ],
        default_action: RouteDecision::Direct,
    }
}

#[test]
fn domain_rules_match_exact_and_wildcard() {
    let engine = rules_engine();
    assert_eq!(engine.decide("blocked.com"), RouteDecision::Upstream);
    assert_eq!(engine.decide("api.internal.dev"), RouteDecision::Direct);
    assert_eq!(engine.decide("internal.dev"), RouteDecision::Direct);
}

#[test]
fn domain_rules_fall_back_to_default() {
    let engine = rules_engine();
    // Unmatched public host → default_action (Direct here).
    assert_eq!(engine.decide("unmatched.com"), RouteDecision::Direct);
}

#[test]
fn domain_rules_still_force_local_direct() {
    let mut engine = rules_engine();
    engine.default_action = RouteDecision::Upstream;
    // Even with an upstream default, local traffic is forced DIRECT.
    assert_eq!(engine.decide("localhost"), RouteDecision::Direct);
    assert_eq!(engine.decide("10.1.2.3"), RouteDecision::Direct);
}
