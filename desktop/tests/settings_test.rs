mod fixtures;
use fixtures::{make_test_app, mem_db, mem_db_no_seed};

use app_lib::commands::settings::{get_settings, save_settings, SettingsInput};
use app_lib::db::entities::enums::proxy::ProxyProtocol;

#[tokio::test]
async fn get_settings_returns_seeded_defaults() {
    let app = make_test_app(mem_db().await);

    let row = get_settings(app.state_ref()).await.expect("get_settings");

    assert_eq!(row.global_timeout, 5000);
    assert!(row.dns_leak_protection);
    assert_eq!(row.proxy_protocol, ProxyProtocol::Socks5);
}

#[tokio::test]
async fn save_settings_updates_existing_row() {
    let app = make_test_app(mem_db().await);

    let existing = get_settings(app.state_ref()).await.expect("get_settings");
    let input = SettingsInput {
        id: existing.id,
        global_timeout: 10000,
        dns_leak_protection: false,
        proxy_protocol: ProxyProtocol::Http,
    };
    save_settings(input, app.state_ref()).await.expect("save_settings");

    let updated = get_settings(app.state_ref()).await.expect("get after save");
    assert_eq!(updated.global_timeout, 10000);
    assert!(!updated.dns_leak_protection);
    assert_eq!(updated.proxy_protocol, ProxyProtocol::Http);
}

#[tokio::test]
async fn save_settings_errors_when_row_missing() {
    let app = make_test_app(mem_db_no_seed().await);

    let input = SettingsInput {
        id: "any-id".to_string(),
        global_timeout: 5000,
        dns_leak_protection: true,
        proxy_protocol: ProxyProtocol::Socks5,
    };
    let err = save_settings(input, app.state_ref()).await.unwrap_err();
    assert!(
        err.contains("Settings row not found"),
        "unexpected error: {err}"
    );
}
