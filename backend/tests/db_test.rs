mod fixtures;
use fixtures::mem_db_no_seed;

use app_lib::db::{entities::settings, seed_settings};
use app_lib::db::entities::enums::proxy::ProxyProtocol;
use sea_orm::EntityTrait;

#[tokio::test]
async fn migrations_apply_to_empty_sqlite_memory() {
    let db = mem_db_no_seed().await;
    // If migrations failed, mem_db_no_seed would have panicked.
    // Querying each table verifies the schema was created.
    let settings_result = settings::Entity::find().all(&db).await;
    assert!(settings_result.is_ok(), "settings table missing after migration");

    let proxy_result = app_lib::db::entities::proxy::Entity::find().all(&db).await;
    assert!(proxy_result.is_ok(), "proxies table missing after migration");

    let session_result = app_lib::db::entities::connection_session::Entity::find().all(&db).await;
    assert!(session_result.is_ok(), "connection_sessions table missing after migration");
}

#[tokio::test]
async fn seed_settings_inserts_default_row_when_absent() {
    let db = mem_db_no_seed().await;
    seed_settings(&db).await.expect("seed");

    let row = settings::Entity::find()
        .one(&db)
        .await
        .expect("query")
        .expect("row present");

    assert_eq!(row.global_timeout, 5000);
    assert!(row.dns_leak_protection);
    assert_eq!(row.proxy_protocol, ProxyProtocol::Socks5);
}

#[tokio::test]
async fn seed_settings_is_idempotent_on_second_call() {
    let db = mem_db_no_seed().await;
    seed_settings(&db).await.expect("first seed");
    seed_settings(&db).await.expect("second seed");

    let rows = settings::Entity::find().all(&db).await.expect("query");
    assert_eq!(rows.len(), 1, "should still have exactly one settings row");
}
