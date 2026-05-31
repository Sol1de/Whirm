use app_lib::net::relay::{
    parse_absolute_target, parse_connect_target, parse_request_line, RequestLine,
};

#[test]
fn request_line_parses_method_target_version() {
    let parsed = parse_request_line("CONNECT example.com:443 HTTP/1.1").unwrap();
    assert_eq!(
        parsed,
        RequestLine {
            method: "CONNECT".into(),
            target: "example.com:443".into(),
            version: "HTTP/1.1".into(),
        }
    );
}

#[test]
fn request_line_rejects_incomplete() {
    assert!(parse_request_line("CONNECT").is_none());
    assert!(parse_request_line("").is_none());
}

#[test]
fn connect_target_splits_host_and_port() {
    assert_eq!(parse_connect_target("example.com:443"), Some(("example.com".into(), 443)));
    assert_eq!(parse_connect_target("10.0.0.1:8080"), Some(("10.0.0.1".into(), 8080)));
}

#[test]
fn connect_target_defaults_port_443() {
    assert_eq!(parse_connect_target("example.com"), Some(("example.com".into(), 443)));
}

#[test]
fn connect_target_rejects_bad_port() {
    assert!(parse_connect_target("example.com:notaport").is_none());
}

#[test]
fn absolute_target_extracts_host_port_path() {
    assert_eq!(
        parse_absolute_target("http://example.com/foo/bar"),
        Some(("example.com".into(), 80, "/foo/bar".into()))
    );
}

#[test]
fn absolute_target_with_explicit_port() {
    assert_eq!(
        parse_absolute_target("http://example.com:8080/"),
        Some(("example.com".into(), 8080, "/".into()))
    );
}

#[test]
fn absolute_target_without_path_defaults_root() {
    assert_eq!(
        parse_absolute_target("http://example.com"),
        Some(("example.com".into(), 80, "/".into()))
    );
}

#[test]
fn absolute_target_rejects_non_http_scheme() {
    assert!(parse_absolute_target("https://example.com/").is_none());
}
