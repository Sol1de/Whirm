mod commands;
mod crypto;
mod db;

use std::sync::Mutex;

use chrono::Utc;
use db::entities::connection_session;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use sysproxy::Sysproxy;
use tauri::Manager;

pub(crate) struct AppState {
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
            commands::proxy::connect_proxy,
            commands::proxy::disconnect_proxy,
            commands::proxy::test_proxy,
            commands::proxy::get_proxies,
            commands::proxy::add_proxy,
            commands::proxy::update_proxy,
            commands::proxy::delete_proxy,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::session::open_session,
            commands::session::close_session,
            commands::session::get_sessions,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                handle_exit(app_handle);
            }
        });
}
