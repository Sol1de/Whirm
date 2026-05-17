mod fixtures;
use fixtures::{make_test_app, mem_db};

use app_lib::commands::proxy::{
    add_proxy, build_proxy_url, delete_proxy, get_proxies, update_proxy, AddProxyInput,
    UpdateProxyInput,
};
use app_lib::db::entities::{enums::proxy::{ProxyProtocol, ProxyStatus}, proxy};
use pretty_assertions::assert_eq;
use sea_orm::EntityTrait;

// ─── build_proxy_url (pure function) ────────────────────────────────────────

#[test]
fn build_url_socks5_no_auth() {
    let url = build_proxy_url("SOCKS5", "myhost", 1080, None, None);
    assert_eq!(url, "socks5://myhost:1080");
}

#[test]
fn build_url_http_with_user_and_pass() {
    let url = build_proxy_url("HTTP", "myhost", 8080, Some("user"), Some("pass"));
    assert_eq!(url, "http://user:pass@myhost:8080");
}

#[test]
fn build_url_https_with_user_only() {
    let url = build_proxy_url("HTTPS", "myhost", 443, Some("user"), None);
    assert_eq!(url, "https://user@myhost:443");
}

#[test]
fn build_url_url_encodes_special_chars() {
    let url = build_proxy_url("SOCKS5", "myhost", 1080, Some("u@ser"), Some("p@ss/word"));
    assert_eq!(url, "socks5://u%40ser:p%40ss%2Fword@myhost:1080");
}

#[test]
fn build_url_unknown_protocol_falls_back_to_http() {
    let url = build_proxy_url("UNKNOWN", "myhost", 3128, None, None);
    assert_eq!(url, "http://myhost:3128");
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn make_add_input(port: i32, password: Option<&str>) -> AddProxyInput {
    AddProxyInput {
        name: "Test Proxy".to_string(),
        host: "127.0.0.1".to_string(),
        port,
        protocol: ProxyProtocol::Socks5,
        country: "France".to_string(),
        country_code: "FR".to_string(),
        username: None,
        password: password.map(|p| p.to_string()),
        category: None,
    }
}

// new_password: None = don't touch, Some("") = clear, Some("x") = re-encrypt with "x"
fn make_update_input(id: &str, new_password: Option<&str>) -> UpdateProxyInput {
    UpdateProxyInput {
        id: id.to_string(),
        name: "Test Proxy".to_string(),
        host: "127.0.0.1".to_string(),
        port: 1080,
        protocol: ProxyProtocol::Socks5,
        country: "France".to_string(),
        country_code: "FR".to_string(),
        username: None,
        new_password: new_password.map(|s| s.to_string()),
        status: ProxyStatus::Inactive,
        category: None,
    }
}

// ─── CRUD ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn add_proxy_rejects_port_zero() {
    let app = make_test_app(mem_db().await);
    let err = add_proxy(make_add_input(0, None), app.state_ref())
        .await
        .unwrap_err();
    assert!(err.contains("Port must be between 1 and 65535"), "got: {err}");
}

#[tokio::test]
async fn add_proxy_rejects_port_above_65535() {
    let app = make_test_app(mem_db().await);
    let err = add_proxy(make_add_input(65536, None), app.state_ref())
        .await
        .unwrap_err();
    assert!(err.contains("Port must be between 1 and 65535"), "got: {err}");
}

#[tokio::test]
async fn add_proxy_encrypts_password_and_strips_on_return() {
    let app = make_test_app(mem_db().await);
    let result = add_proxy(make_add_input(1080, Some("my-plaintext-password")), app.state_ref())
        .await
        .expect("add_proxy");

    // Return value strips the password
    assert_eq!(result.password, None);

    // Raw DB row has encrypted (non-plaintext) password
    let raw = proxy::Entity::find_by_id(&result.id)
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("row present");

    assert!(raw.password.is_some(), "password should be stored");
    assert_ne!(
        raw.password.as_deref(),
        Some("my-plaintext-password"),
        "password must not be stored as plaintext"
    );
}

#[tokio::test]
async fn add_proxy_with_no_password_persists_null() {
    let app = make_test_app(mem_db().await);
    let result = add_proxy(make_add_input(1080, None), app.state_ref())
        .await
        .expect("add_proxy");

    let raw = proxy::Entity::find_by_id(&result.id)
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("row present");

    assert_eq!(raw.password, None);
}

#[tokio::test]
async fn update_proxy_with_some_nonempty_re_encrypts() {
    let app = make_test_app(mem_db().await);
    let added = add_proxy(make_add_input(1080, Some("old-password")), app.state_ref())
        .await
        .expect("add");

    let old_enc = proxy::Entity::find_by_id(&added.id)
        .one(&app.state.db)
        .await
        .expect("q1")
        .expect("row")
        .password;

    update_proxy(
        make_update_input(&added.id, Some("new-password")),
        app.state_ref(),
    )
    .await
    .expect("update");

    let new_enc = proxy::Entity::find_by_id(&added.id)
        .one(&app.state.db)
        .await
        .expect("q2")
        .expect("row")
        .password;

    assert!(new_enc.is_some(), "password should still be set");
    assert_ne!(old_enc, new_enc, "ciphertext should differ after re-encryption");
    assert_ne!(new_enc.as_deref(), Some("new-password"));
}

#[tokio::test]
async fn update_proxy_with_none_leaves_password_unchanged() {
    let app = make_test_app(mem_db().await);
    let added = add_proxy(make_add_input(1080, Some("password")), app.state_ref())
        .await
        .expect("add");

    let old_enc = proxy::Entity::find_by_id(&added.id)
        .one(&app.state.db)
        .await
        .expect("q1")
        .expect("row")
        .password;

    // new_password: None → NotSet (password column not touched in DB)
    update_proxy(make_update_input(&added.id, None), app.state_ref()).await.expect("update");

    let new_enc = proxy::Entity::find_by_id(&added.id)
        .one(&app.state.db)
        .await
        .expect("q2")
        .expect("row")
        .password;

    assert_eq!(old_enc, new_enc, "password should remain unchanged");
}

#[tokio::test]
async fn update_proxy_with_empty_string_clears_password() {
    let app = make_test_app(mem_db().await);
    let added = add_proxy(make_add_input(1080, Some("password")), app.state_ref())
        .await
        .expect("add");

    // Some("") → Set(None) → clears password in DB
    update_proxy(
        make_update_input(&added.id, Some("")),
        app.state_ref(),
    )
    .await
    .expect("update");

    let row = proxy::Entity::find_by_id(&added.id)
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("row");

    assert_eq!(row.password, None, "password should be cleared");
}

#[tokio::test]
async fn get_proxies_strips_password_field() {
    let app = make_test_app(mem_db().await);
    add_proxy(make_add_input(1080, Some("secret")), app.state_ref())
        .await
        .expect("add");

    let proxies = get_proxies(app.state_ref()).await.expect("get_proxies");
    assert_eq!(proxies.len(), 1);
    assert_eq!(proxies[0].password, None, "password must be stripped");
}

#[tokio::test]
async fn delete_proxy_succeeds_when_inactive() {
    let app = make_test_app(mem_db().await);
    let added = add_proxy(make_add_input(1080, None), app.state_ref())
        .await
        .expect("add");
    let id = added.id.clone();

    delete_proxy(id.clone(), app.state_ref()).await.expect("delete");

    let row = proxy::Entity::find_by_id(&id)
        .one(&app.state.db)
        .await
        .expect("query");
    assert!(row.is_none(), "proxy should be deleted");
}

#[tokio::test]
async fn delete_proxy_refuses_when_active_id_matches() {
    let app = make_test_app(mem_db().await);
    let added = add_proxy(make_add_input(1080, None), app.state_ref())
        .await
        .expect("add");
    let id = added.id.clone();

    // Mark as active
    *app.state.active_proxy_id.lock().unwrap() = Some(id.clone());

    let err = delete_proxy(id, app.state_ref()).await.unwrap_err();
    assert!(
        err.contains("Cannot delete proxy while it is active"),
        "got: {err}"
    );
}

#[tokio::test]
async fn add_proxy_rejects_negative_port() {
    let app = make_test_app(mem_db().await);
    let err = add_proxy(make_add_input(-1, None), app.state_ref())
        .await
        .unwrap_err();
    assert!(err.contains("Port must be between 1 and 65535"), "got: {err}");
}

#[tokio::test]
async fn update_proxy_errors_when_id_not_found() {
    let app = make_test_app(mem_db().await);
    let err = update_proxy(make_update_input("nonexistent-id", None), app.state_ref())
        .await
        .unwrap_err();
    assert!(!err.is_empty(), "should return an error for missing proxy: {err}");
}

#[tokio::test]
async fn add_proxy_rejects_empty_host() {
    let app = make_test_app(mem_db().await);
    let mut input = make_add_input(1080, None);
    input.host = "".to_string();

    let err = add_proxy(input, app.state_ref()).await.unwrap_err();
    assert!(err.contains("Host must not be empty"), "got: {err}");
}
