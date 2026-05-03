use std::time::{Duration, Instant};

use chrono::Utc;
use reqwest::Proxy;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::Deserialize;
use urlencoding::encode as url_encode;

use crate::crypto;
use crate::db::entities::{proxy, enums::proxy::{ProxyProtocol, ProxyStatus}};
use crate::AppState;

// ─── NETWORKING HELPERS ─────────────────────────────────────────────────────

pub fn build_proxy_url(
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
    let mut saved = state.saved_proxy.lock().unwrap_or_else(|e| e.into_inner());
    if saved.is_none() {
        *saved = Some(
            sysproxy::Sysproxy::get_system_proxy()
                .map_err(|e| format!("Failed to read current system proxy: {e}"))?,
        );
    }
    Ok(())
}

// ─── INPUT STRUCTS ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddProxyInput {
    pub name: String,
    pub host: String,
    pub port: i32,
    pub protocol: ProxyProtocol,
    pub country: String,
    pub country_code: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProxyInput {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub protocol: ProxyProtocol,
    pub country: String,
    pub country_code: String,
    pub username: Option<String>,
    pub new_password: Option<String>,
    pub status: ProxyStatus,
    pub category: Option<String>,
}

// ─── TUNNEL COMMANDS ────────────────────────────────────────────────────────

pub async fn connect_proxy(proxy_id: String, state: &AppState) -> Result<String, String> {
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
    save_original_proxy(state)?;

    let new_proxy = sysproxy::Sysproxy {
        enable: true,
        host: proxy.host,
        port: proxy.port as u16,
        bypass: String::from("localhost,127.0.0.1,<local>"),
    };
    new_proxy
        .set_system_proxy()
        .map_err(|e| format!("Failed to apply system proxy: {e}"))?;

    *state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(proxy_id);

    Ok(ip)
}

pub async fn disconnect_proxy(state: &AppState) -> Result<(), String> {
    let saved = state
        .saved_proxy
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();

    if let Some(proxy) = saved {
        proxy
            .set_system_proxy()
            .map_err(|e| format!("Failed to restore system proxy: {e}"))?;
    }

    *state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()) = None;

    Ok(())
}

pub async fn test_proxy(
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

// ─── CRUD COMMANDS ──────────────────────────────────────────────────────────

pub async fn get_proxies(state: &AppState) -> Result<Vec<proxy::Model>, String> {
    let proxies = proxy::Entity::find()
        .all(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    Ok(proxies.into_iter().map(|mut p| { p.password = None; p }).collect())
}

pub async fn add_proxy(input: AddProxyInput, state: &AppState) -> Result<proxy::Model, String> {
    if input.port < 1 || input.port > 65535 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    if input.host.trim().is_empty() {
        return Err("Host must not be empty".to_string());
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
            _ => None,
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

pub async fn update_proxy(
    input: UpdateProxyInput,
    state: &AppState,
) -> Result<proxy::Model, String> {
    if input.port < 1 || input.port > 65535 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    if input.host.trim().is_empty() {
        return Err("Host must not be empty".to_string());
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
            None => sea_orm::ActiveValue::NotSet,
            Some(_) => Set(None),
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

pub async fn delete_proxy(id: String, state: &AppState) -> Result<(), String> {
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
