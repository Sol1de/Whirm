pub mod commands;
pub mod crypto;
pub mod db;

use std::sync::Mutex;

use chrono::Utc;
use db::entities::connection_session;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use sysproxy::Sysproxy;
use tauri::{Manager, State};

// ─── TAURI COMMAND ─────────────────────────────────────────────────

#[tauri::command]
async fn connect_proxy(proxy_id: String, state: State<'_, AppState>) -> Result<String, String> {
    commands::proxy::connect_proxy(proxy_id, &state).await
}

#[tauri::command]
async fn disconnect_proxy(state: State<'_, AppState>) -> Result<(), String> {
    commands::proxy::disconnect_proxy(&state).await
}

#[tauri::command]
async fn get_proxies(state: State<'_, AppState>) -> Result<Vec<db::entities::proxy::Model>, String> {
    commands::proxy::get_proxies(&state).await
}

#[tauri::command]
async fn add_proxy(
    input: commands::proxy::AddProxyInput,
    state: State<'_, AppState>,
) -> Result<db::entities::proxy::Model, String> {
    commands::proxy::add_proxy(input, &state).await
}

#[tauri::command]
async fn update_proxy(
    input: commands::proxy::UpdateProxyInput,
    state: State<'_, AppState>,
) -> Result<db::entities::proxy::Model, String> {
    commands::proxy::update_proxy(input, &state).await
}

#[tauri::command]
async fn delete_proxy(id: String, state: State<'_, AppState>) -> Result<(), String> {
    commands::proxy::delete_proxy(id, &state).await
}

#[tauri::command]
async fn test_proxy(
    host: String,
    port: u16,
    protocol: String,
    username: Option<String>,
    password: Option<String>,
    timeout: Option<u64>,
) -> Result<u64, String> {
    commands::proxy::test_proxy(host, port, protocol, username, password, timeout).await
}

#[tauri::command]
async fn get_settings(
    state: State<'_, AppState>,
) -> Result<db::entities::settings::Model, String> {
    commands::settings::get_settings(&state).await
}

#[tauri::command]
async fn save_settings(
    input: commands::settings::SettingsInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    commands::settings::save_settings(input, &state).await
}

#[tauri::command]
async fn open_session(
    proxy_id: String,
    ip_address: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    commands::session::open_session(proxy_id, ip_address, &state).await
}

#[tauri::command]
async fn close_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    commands::session::close_session(session_id, &state).await
}

#[tauri::command]
async fn get_sessions(
    limit: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<db::entities::connection_session::Model>, String> {
    commands::session::get_sessions(limit, &state).await
}

pub struct AppState {
    pub saved_proxy: Mutex<Option<Sysproxy>>,
    pub db: DatabaseConnection,
    pub active_session_id: Mutex<Option<String>>,
    pub active_proxy_id: Mutex<Option<String>>,
}

fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    let db = tauri::async_runtime::block_on(db::init(&handle))
        .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) as Box<dyn std::error::Error>)?;

    app.manage(AppState {
        saved_proxy: Mutex::new(None),
        db,
        active_session_id: Mutex::new(None),
        active_proxy_id: Mutex::new(None),
    });

    if cfg!(debug_assertions) {
        app.handle().plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )?;
    }

    Ok(())
}

fn handle_exit(app_handle: &tauri::AppHandle) {
    let proxy = {
        let state = app_handle.state::<AppState>();
        state.saved_proxy.lock().ok().and_then(|mut g| g.take())
    };
    if let Some(p) = proxy {
        p.set_system_proxy().ok();
    }

    let session_id = {
        let state = app_handle.state::<AppState>();
        state.active_session_id.lock().ok().and_then(|mut g| g.take())
    };
    if let Some(id) = session_id {
        let state = app_handle.state::<AppState>();
        tauri::async_runtime::block_on(async {
            if let Ok(Some(session)) = connection_session::Entity::find_by_id(&id)
                .one(&state.db)
                .await
            {
                let active = connection_session::ActiveModel {
                    id: Set(session.id),
                    disconnected_at: Set(Some(Utc::now())),
                    ..Default::default()
                };

                active.update(&state.db).await.ok();
            }
        });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup_app)
        .invoke_handler(tauri::generate_handler![
            connect_proxy,
            disconnect_proxy,
            test_proxy,
            get_proxies,
            add_proxy,
            update_proxy,
            delete_proxy,
            get_settings,
            save_settings,
            open_session,
            close_session,
            get_sessions,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                handle_exit(app_handle);
            }
        });
}
