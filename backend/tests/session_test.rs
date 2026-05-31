mod fixtures;
use fixtures::{make_test_app, mem_db};

use app_lib::commands::session::{close_session, get_sessions, open_session};
use app_lib::db::entities::{
    connection_session,
    enums::proxy::{ProxyProtocol, ProxyStatus},
    proxy,
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};

async fn insert_test_proxy(db: &sea_orm::DatabaseConnection, id: &str) {
    let now = Utc::now();
    proxy::ActiveModel {
        id: Set(id.to_string()),
        name: Set("Test Proxy".to_string()),
        host: Set("127.0.0.1".to_string()),
        port: Set(1080),
        protocol: Set(ProxyProtocol::Socks5),
        country: Set("France".to_string()),
        country_code: Set("FR".to_string()),
        username: Set(None),
        password: Set(None),
        status: Set(ProxyStatus::Inactive),
        category: Set(None),
        last_used_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
    .expect("insert test proxy");
}

#[tokio::test]
async fn open_session_inserts_row_and_touches_proxy_last_used_at() {
    let db = mem_db().await;
    insert_test_proxy(&db, "p-1").await;
    let app = make_test_app(db);

    open_session("p-1".to_string(), "1.2.3.4".to_string(), app.state_ref())
        .await
        .expect("open_session");

    let p = proxy::Entity::find_by_id("p-1")
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("proxy present");

    assert!(
        p.last_used_at.is_some(),
        "last_used_at should be updated by open_session"
    );
}

#[tokio::test]
async fn open_session_returns_inserted_id_and_stores_active_session_id() {
    let db = mem_db().await;
    insert_test_proxy(&db, "p-test").await;
    let app = make_test_app(db);

    let session_id = open_session("p-test".to_string(), "1.2.3.4".to_string(), app.state_ref())
        .await
        .expect("open_session");

    assert!(!session_id.is_empty());

    let active = app.state.active_session_id.lock().unwrap().clone();
    assert_eq!(active, Some(session_id));
}

#[tokio::test]
async fn close_session_sets_disconnected_at() {
    let db = mem_db().await;
    insert_test_proxy(&db, "p-test").await;
    let app = make_test_app(db);

    let session_id = open_session("p-test".to_string(), "1.2.3.4".to_string(), app.state_ref())
        .await
        .expect("open_session");

    close_session(session_id.clone(), app.state_ref())
        .await
        .expect("close_session");

    let row = connection_session::Entity::find_by_id(&session_id)
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("session present");

    assert!(
        row.disconnected_at.is_some(),
        "disconnected_at should be set after close"
    );
}

#[tokio::test]
async fn close_session_clears_active_id_with_real_session() {
    let db = mem_db().await;
    insert_test_proxy(&db, "p-close").await;
    let app = make_test_app(db);

    let session_id = open_session("p-close".to_string(), "5.6.7.8".to_string(), app.state_ref())
        .await
        .expect("open_session");

    close_session(session_id.clone(), app.state_ref())
        .await
        .expect("close_session");

    let active = app.state.active_session_id.lock().unwrap().clone();
    assert_eq!(active, None, "active_session_id should be cleared");

    let row = connection_session::Entity::find_by_id(&session_id)
        .one(&app.state.db)
        .await
        .expect("query")
        .expect("session present");
    assert!(
        row.disconnected_at.is_some(),
        "disconnected_at should be set"
    );
}

#[tokio::test]
async fn close_session_clears_active_id_even_when_row_missing() {
    let app = make_test_app(mem_db().await);

    *app.state.active_session_id.lock().unwrap() = Some("session-1".to_string());

    close_session("session-1".to_string(), app.state_ref())
        .await
        .expect("close_session");

    let active = app.state.active_session_id.lock().unwrap().clone();
    assert_eq!(active, None, "active_session_id should be cleared");
}

#[tokio::test]
async fn close_session_keeps_active_id_when_id_differs() {
    let app = make_test_app(mem_db().await);

    *app.state.active_session_id.lock().unwrap() = Some("session-1".to_string());

    close_session("session-2".to_string(), app.state_ref())
        .await
        .expect("close_session");

    let active = app.state.active_session_id.lock().unwrap().clone();
    assert_eq!(
        active,
        Some("session-1".to_string()),
        "active_session_id should be unchanged"
    );
}

#[tokio::test]
async fn get_sessions_orders_by_connected_at_desc() {
    let db = mem_db().await;
    let now = Utc::now();

    connection_session::ActiveModel {
        id: Set("s-old".to_string()),
        proxy_id: Set(None),
        connected_at: Set(now - Duration::seconds(60)),
        disconnected_at: Set(None),
        ip_address: Set(None),
    }
    .insert(&db)
    .await
    .expect("insert old");

    connection_session::ActiveModel {
        id: Set("s-new".to_string()),
        proxy_id: Set(None),
        connected_at: Set(now),
        disconnected_at: Set(None),
        ip_address: Set(None),
    }
    .insert(&db)
    .await
    .expect("insert new");

    let app = make_test_app(db);
    let sessions = get_sessions(None, app.state_ref()).await.expect("get_sessions");

    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, "s-new", "most recent first");
    assert_eq!(sessions[1].id, "s-old");
}

#[tokio::test]
async fn get_sessions_respects_limit() {
    let db = mem_db().await;
    let now = Utc::now();

    for i in 0..5u64 {
        connection_session::ActiveModel {
            id: Set(format!("s-{i}")),
            proxy_id: Set(None),
            connected_at: Set(now - Duration::seconds(i as i64)),
            disconnected_at: Set(None),
            ip_address: Set(None),
        }
        .insert(&db)
        .await
        .expect("insert");
    }

    let app = make_test_app(db);
    let sessions = get_sessions(Some(3), app.state_ref())
        .await
        .expect("get_sessions");

    assert_eq!(sessions.len(), 3, "should respect limit of 3");
}

#[tokio::test]
async fn open_session_rejects_nonexistent_proxy() {
    let app = make_test_app(mem_db().await);

    let result = open_session("no-such-proxy".to_string(), "9.9.9.9".to_string(), app.state_ref())
        .await;

    let err = result.unwrap_err();
    assert!(
        err.contains("Proxy not found"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn get_sessions_with_limit_zero_returns_empty() {
    let db = mem_db().await;
    let now = Utc::now();

    connection_session::ActiveModel {
        id: Set("s-exists".to_string()),
        proxy_id: Set(None),
        connected_at: Set(now),
        disconnected_at: Set(None),
        ip_address: Set(None),
    }
    .insert(&db)
    .await
    .expect("insert");

    let app = make_test_app(db);
    let sessions = get_sessions(Some(0), app.state_ref())
        .await
        .expect("get_sessions");

    assert_eq!(sessions.len(), 0, "limit=0 should return empty");
}
