use app_lib::net::transport::{basic_auth_header, build_connect_request, parse_status_code};

#[test]
fn basic_auth_header_base64_encodes_credentials() {
    // base64("user:pass") == "dXNlcjpwYXNz"
    assert_eq!(basic_auth_header("user", "pass"), "Basic dXNlcjpwYXNz");
}

#[test]
fn connect_request_without_auth_has_no_authorization_header() {
    let req = build_connect_request("example.com", 443, None);
    assert!(req.starts_with("CONNECT example.com:443 HTTP/1.1\r\n"));
    assert!(req.contains("Host: example.com:443\r\n"));
    assert!(!req.contains("Proxy-Authorization"));
    assert!(req.ends_with("\r\n\r\n"));
}

#[test]
fn connect_request_with_auth_includes_authorization_header() {
    let req = build_connect_request("example.com", 8080, Some(("user", "pass")));
    assert!(req.contains("Proxy-Authorization: Basic dXNlcjpwYXNz\r\n"));
    assert!(req.ends_with("\r\n\r\n"));
}

#[test]
fn connect_request_with_empty_username_skips_auth() {
    let req = build_connect_request("example.com", 443, Some(("", "")));
    assert!(!req.contains("Proxy-Authorization"));
}

#[test]
fn parse_status_code_extracts_2xx() {
    assert_eq!(parse_status_code("HTTP/1.1 200 Connection Established"), Some(200));
    assert_eq!(parse_status_code("HTTP/1.1 407 Proxy Authentication Required"), Some(407));
}

#[test]
fn parse_status_code_rejects_malformed() {
    assert_eq!(parse_status_code("garbage"), None);
    assert_eq!(parse_status_code(""), None);
}
