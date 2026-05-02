mod crypto;
mod db;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use chrono::Utc;
use db::entities::{connection_session, proxy, settings, enums::proxy::{ProxyProtocol, ProxyStatus}};
use reqwest::Proxy;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use sysproxy::Sysproxy;
use tauri::{Manager, State};
use urlencoding::encode as url_encode;

// ─── APP STATE ───────────────────────────────────────────────────────────────

struct AppState {
    saved_proxy: Mutex<Option<Sysproxy>>,
    db: DatabaseConnection,
    active_session_id: Mutex<Option<String>>,
    active_proxy_id: Mutex<Option<String>>,
}

// ─── PROXY NETWORKING HELPERS ────────────────────────────────────────────────

fn build_proxy_url(
    protocol: &str,
    host: &str,
    port: u16,
    username: Option<&str>,
    password: Option<&str>,
) -> String {
    let scheme = match protocol {
        "SOCKS5" => "socks5",
        "HTTPS" => "https",
        _ => "http",
    };

    let auth = match (username, password) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {
            format!("{}:{}@", url_encode(u), url_encode(p))
        }
        (Some(u), _) if !u.is_empty() => format!("{}@", url_encode(u)),
        _ => String::new(),
    };

    format!("{scheme}://{auth}{host}:{port}")
}

fn build_proxy_client(proxy_url: &str) -> Result<reqwest::Client, String> {
    let proxy = Proxy::all(proxy_url).map_err(|e| format!("Invalid proxy URL: {e}"))?;
    reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

async fn check_connectivity(proxy_url: &str) -> Result<String, String> {
    let client = build_proxy_client(proxy_url)?;
    let ip = client
        .get("https://api.ipify.org")
        .send()
        .await
        .map_err(|e| format!("Proxy unreachable (connection failed): {e}"))?
        .text()
        .await
        .map_err(|e| format!("Failed to read IP response: {e}"))?;
    Ok(ip.trim().to_string())
}

fn save_original_proxy(state: &AppState) -> Result<(), String> {
    let mut saved = state
        .saved_proxy
        .lock()
        .map_err(|e| format!("Corrupted mutex (poisoned lock): {e}"))?;
    if saved.is_none() {
        *saved = Some(
            Sysproxy::get_system_proxy()
                .map_err(|e| format!("Failed to read current system proxy: {e}"))?,
        );
    }
    Ok(())
}

// ─── EXISTING PROXY TUNNEL COMMANDS ──────────────────────────────────────────

#[tauri::command]
async fn connect_proxy(
    proxy_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let proxy = proxy::Entity::find_by_id(&proxy_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proxy not found: {proxy_id}"))?;

    let key = crypto::derive_key()?;
    let password = proxy.password.as_deref()
        .map(|enc| crypto::decrypt(enc, &key))
        .transpose()?;

    let protocol_str = match proxy.protocol {
        ProxyProtocol::Socks5 => "SOCKS5",
        ProxyProtocol::Http => "HTTP",
        ProxyProtocol::Https => "HTTPS",
    };

    let proxy_url = build_proxy_url(
        protocol_str,
        &proxy.host,
        proxy.port as u16,
        proxy.username.as_deref(),
        password.as_deref(),
    );

    let ip = check_connectivity(&proxy_url).await?;
    save_original_proxy(&state)?;

    let new_proxy = Sysproxy {
        enable: true,
        host: proxy.host,
        port: proxy.port as u16,
        bypass: String::from("localhost,127.0.0.1,<local>"),
    };
    new_proxy
        .set_system_proxy()
        .map_err(|e| format!("Failed to apply system proxy: {e}"))?;

    Ok(ip)
}

#[tauri::command]
async fn disconnect_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let saved = state
        .saved_proxy
        .lock()
        .map_err(|_| "Corrupted mutex (poisoned lock)".to_string())?
        .take();

    if let Some(proxy) = saved {
        proxy
            .set_system_proxy()
            .map_err(|e| format!("Failed to restore system proxy: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
async fn test_proxy(
    host: String,
    port: u16,
    protocol: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<u64, String> {
    let proxy_url = build_proxy_url(
        &protocol,
        &host,
        port,
        username.as_deref(),
        password.as_deref(),
    );

    let client = build_proxy_client(&proxy_url)?;
    let start = Instant::now();

    client
        .get("https://api.ipify.org")
        .send()
        .await
        .map_err(|e| format!("Proxy unreachable: {e}"))?;

    Ok(start.elapsed().as_millis() as u64)
}

// ─── DB INPUT STRUCTS ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddProxyInput {
    name: String,
    host: String,
    port: i32,
    protocol: ProxyProtocol,
    country: String,
    country_code: String,
    username: Option<String>,
    password: Option<String>,
    category: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProxyInput {
    id: String,
    name: String,
    host: String,
    port: i32,
    protocol: ProxyProtocol,
    country: String,
    country_code: String,
    username: Option<String>,
    new_password: Option<String>,
    status: ProxyStatus,
    category: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsInput {
    id: String,
    global_timeout: i32,
    dns_leak_protection: bool,
    proxy_protocol: ProxyProtocol,
}

// ─── PROXY CRUD COMMANDS ─────────────────────────────────────────────────────

#[tauri::command]
async fn get_proxies(state: State<'_, AppState>) -> Result<Vec<proxy::Model>, String> {
    let proxies = proxy::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(proxies.into_iter().map(|mut p| { p.password = None; p }).collect())
}

#[tauri::command]
async fn add_proxy(
    input: AddProxyInput,
    state: State<'_, AppState>,
) -> Result<proxy::Model, String> {
    if input.port < 1 || input.port > 65535 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    let now = Utc::now();
    let new_proxy = proxy::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        name: Set(input.name),
        host: Set(input.host),
        port: Set(input.port),
        protocol: Set(input.protocol),
        country: Set(input.country),
        country_code: Set(input.country_code),
        username: Set(input.username),
        password: Set(match &input.password {
            Some(p) if !p.is_empty() => {
                let key = crypto::derive_key()?;
                Some(crypto::encrypt(p, &key)?)
            }
            other => other.clone(),
        }),
        status: Set(ProxyStatus::Inactive),
        category: Set(input.category),
        last_used_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let mut result = new_proxy
        .insert(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    result.password = None;
    Ok(result)
}

#[tauri::command]
async fn update_proxy(
    input: UpdateProxyInput,
    state: State<'_, AppState>,
) -> Result<proxy::Model, String> {
    if input.port < 1 || input.port > 65535 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    let updated_proxy = proxy::ActiveModel {
        id: Set(input.id),
        name: Set(input.name),
        host: Set(input.host),
        port: Set(input.port),
        protocol: Set(input.protocol),
        country: Set(input.country),
        country_code: Set(input.country_code),
        username: Set(input.username),
        password: match input.new_password {
            Some(ref p) if !p.is_empty() => {
                let key = crypto::derive_key()?;
                Set(Some(crypto::encrypt(p, &key)?))
            }
            Some(_) => Set(None),
            None => sea_orm::ActiveValue::NotSet,
        },
        status: Set(input.status),
        category: Set(input.category),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let mut result = updated_proxy.update(&state.db).await.map_err(|e| e.to_string())?;
    result.password = None;
    Ok(result)
}

#[tauri::command]
async fn delete_proxy(id: String, state: State<'_, AppState>) -> Result<(), String> {
    {
        let active = state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner());
        if active.as_deref() == Some(id.as_str()) {
            return Err("Cannot delete proxy while it is active".to_string());
        }
    }

    proxy::Entity::delete_by_id(&id)
        .exec(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── SETTINGS COMMANDS ────────────────────────────────────────────────────────

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<settings::Model, String> {
    settings::Entity::find()
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Settings row not found".to_string())
}

#[tauri::command]
async fn save_settings(
    input: SettingsInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let row = settings::Entity::find()
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Settings row not found".to_string())?;

    let active = settings::ActiveModel {
        id: Set(row.id),
        global_timeout: Set(input.global_timeout),
        dns_leak_protection: Set(input.dns_leak_protection),
        proxy_protocol: Set(input.proxy_protocol),
    };

    active.update(&state.db).await.map_err(|e| e.to_string())?;
    Ok(())
}

// ─── SESSION COMMANDS ─────────────────────────────────────────────────────────

#[tauri::command]
async fn open_session(
    proxy_id: String,
    ip_address: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let now = Utc::now();

    // Update proxy last_used_at
    if let Some(proxy_model) = proxy::Entity::find_by_id(&proxy_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
    {
        let active = proxy::ActiveModel {
            id: Set(proxy_model.id),
            last_used_at: Set(Some(now)),
            updated_at: Set(now),
            ..Default::default()
        };

        active.update(&state.db).await.map_err(|e| e.to_string())?;
    }

    let session = connection_session::ActiveModel {
        id: Set(uuid::Uuid::new_v4().to_string()),
        proxy_id: Set(Some(proxy_id)),
        connected_at: Set(now),
        disconnected_at: Set(None),
        ip_address: Set(Some(ip_address)),
    };

    let result = session
        .insert(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let id = result.id;

    *state.active_session_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(id.clone());
    *state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()) = result.proxy_id.clone();

    Ok(id)
}

#[tauri::command]
async fn close_session(session_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if let Some(session) = connection_session::Entity::find_by_id(&session_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
    {
        let active = connection_session::ActiveModel {
            id: Set(session.id),
            disconnected_at: Set(Some(Utc::now())),
            ..Default::default()
        };

        active.update(&state.db).await.map_err(|e| e.to_string())?;
    }

    state.active_session_id.lock().unwrap_or_else(|e| e.into_inner())
        .take_if(|id| id == &session_id);
    *state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()) = None;

    Ok(())
}

#[tauri::command]
async fn get_sessions(
    limit: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<connection_session::Model>, String> {
    let mut query = connection_session::Entity::find()
        .order_by_desc(connection_session::Column::ConnectedAt);

    if let Some(n) = limit {
        query = query.limit(n);
    }

    Ok(query.all(&state.db).await.map_err(|e| e.to_string())?)
}

// ─── APP SETUP ───────────────────────────────────────────────────────────────

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

// ─── EXIT HANDLER ────────────────────────────────────────────────────────────

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
                let active: connection_session::ActiveModel = connection_session::ActiveModel {
                    id: Set(session.id),
                    disconnected_at:  Set(Some(Utc::now())),
                    ..Default::default()
                };

                active.update(&state.db).await.ok();
            }
        });
    }
}

// ─── TAURI ENTRY POINT ───────────────────────────────────────────────────────

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
