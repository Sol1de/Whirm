use std::sync::Mutex;
use std::time::{Duration, Instant};
use reqwest::Proxy;
use sysproxy::Sysproxy;
use tauri::{Manager, State};
use urlencoding::encode as url_encode;

struct AppState {
    saved_proxy: Mutex<Option<Sysproxy>>,
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

#[tauri::command]
async fn connect_proxy(
    host: String,
    port: u16,
    protocol: String,
    username: Option<String>,
    password: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let proxy_url = build_proxy_url(
        &protocol,
        &host,
        port,
        username.as_deref(),
        password.as_deref(),
    );

    let ip = check_connectivity(&proxy_url).await?;

    save_original_proxy(&state)?;

    let new_proxy = Sysproxy {
        enable: true,
        host,
        port,
        bypass: String::from("localhost,127.0.0.1,<local>"),
    };
    new_proxy
        .set_system_proxy()
        .map_err(|e| format!("Failed to apply system proxy: {e}"))?;

    Ok(ip)
}

#[tauri::command]
async fn disconnect_proxy(state: State<'_, AppState>) -> Result<(), String> {
    let saved = state.saved_proxy.lock()
        .map_err(|_| "Corrupted mutex (poisoned lock)".to_string())?
        .take();

    match saved {
        Some(proxy) => {
            proxy
                .set_system_proxy()
                .map_err(|e| format!("Failed to restore system proxy: {e}"))?;
        }
        None => {
            return Ok(());
        }
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

    let latency_ms = start.elapsed().as_millis() as u64;

    Ok(latency_ms)
}

// ─── TAURI ENTRY POINT ─────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            saved_proxy: Mutex::new(None),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_proxy,
            disconnect_proxy,
            test_proxy,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let proxy = {
                    let state = app_handle.state::<AppState>();
                    state.saved_proxy.lock().ok().and_then(|mut g| g.take())
                };
                if let Some(p) = proxy {
                    let _ = p.set_system_proxy();
                }
            }
        });
}
