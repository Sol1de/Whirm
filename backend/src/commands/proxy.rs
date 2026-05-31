use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use reqwest::Proxy;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use urlencoding::encode as url_encode;

use crate::crypto;
use crate::db::entities::{
    proxy,
    settings,
    enums::proxy::{ProxyProtocol, ProxyStatus},
};
use crate::net::{
    relay, system_proxy, HttpProxyTransport, HttpsProxyTransport, RouteEngine, Socks5Transport,
    Transport,
};
use crate::AppState;

/// Friendlier message when a stored password cannot be decrypted — almost
/// always because the machine identity (and thus the derived key) changed.
const DECRYPT_HELP: &str =
    "Saved password could not be decrypted (machine identity may have changed). \
Re-enter the password for this proxy.";

fn map_decrypt_err(e: String) -> String {
    if e == "Decryption failed" {
        DECRYPT_HELP.to_string()
    } else {
        e
    }
}

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

fn build_proxy_client(proxy_url: &str, timeout_secs: u64) -> Result<reqwest::Client, String> {
    let proxy = Proxy::all(proxy_url).map_err(|e| format!("Invalid proxy URL: {e}"))?;
    reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

async fn check_connectivity(proxy_url: &str, timeout_secs: u64) -> Result<String, String> {
    let client = build_proxy_client(proxy_url, timeout_secs)?;
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
        *saved = Some(system_proxy::capture()?);
    }
    Ok(())
}

fn protocol_str(protocol: &ProxyProtocol) -> &'static str {
    match protocol {
        ProxyProtocol::Socks5 => "SOCKS5",
        ProxyProtocol::Http => "HTTP",
        ProxyProtocol::Https => "HTTPS",
    }
}

/// Build the upstream [`Transport`] for a proxy. `remote_dns` mirrors the
/// `dnsLeakProtection` setting: when true the hostname is resolved by the
/// upstream proxy (remote resolution) instead of locally.
fn build_transport(
    protocol: &ProxyProtocol,
    host: &str,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    remote_dns: bool,
) -> Arc<dyn Transport> {
    match protocol {
        ProxyProtocol::Socks5 => Arc::new(Socks5Transport {
            proxy_addr: format!("{host}:{port}"),
            username,
            password,
            remote_dns,
        }),
        ProxyProtocol::Http => Arc::new(HttpProxyTransport {
            proxy_addr: format!("{host}:{port}"),
            username,
            password,
            remote_dns,
        }),
        ProxyProtocol::Https => Arc::new(HttpsProxyTransport::new(
            host.to_string(),
            port,
            username,
            password,
            remote_dns,
        )),
    }
}

/// Load the singleton settings row (timeout + DNS behaviour).
async fn load_settings(state: &AppState) -> Result<settings::Model, String> {
    settings::Entity::find()
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Settings row not found".to_string())
}

/// Set a proxy's status column without touching any other field.
async fn set_proxy_status(state: &AppState, id: &str, status: ProxyStatus) -> Result<(), String> {
    let active = proxy::ActiveModel {
        id: Set(id.to_string()),
        status: Set(status),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };
    active.update(&state.db).await.map_err(|e| e.to_string())?;
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
    // Backend-enforced concurrency guard: refuse a second connect.
    {
        let active = state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner());
        if active.is_some() {
            return Err("A proxy is already connected".to_string());
        }
    }

    let proxy = proxy::Entity::find_by_id(&proxy_id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proxy not found: {proxy_id}"))?;

    let key = crypto::derive_key()?;
    let password = proxy
        .password
        .as_deref()
        .map(|enc| crypto::decrypt(enc, &key))
        .transpose()
        .map_err(map_decrypt_err)?;

    let settings = load_settings(state).await?;
    let timeout_secs = ((settings.global_timeout.max(0) as u64) / 1000).max(1);
    let remote_dns = settings.dns_leak_protection;

    // Pre-flight reachability through the upstream (validates host/auth/protocol).
    let proxy_url = build_proxy_url(
        protocol_str(&proxy.protocol),
        &proxy.host,
        proxy.port as u16,
        proxy.username.as_deref(),
        password.as_deref(),
    );
    let ip = check_connectivity(&proxy_url, timeout_secs).await?;

    // Start the local relay fronting this upstream + a global route engine.
    let transport = build_transport(
        &proxy.protocol,
        &proxy.host,
        proxy.port as u16,
        proxy.username.clone(),
        password,
        remote_dns,
    );
    let engine = Arc::new(RouteEngine::global());
    let handle = relay::start(transport, engine).await?;
    let relay_port = handle.port;

    // Point the OS system proxy at the relay; capture the original first.
    save_original_proxy(state)?;
    if let Err(e) = system_proxy::apply(relay_port) {
        handle.shutdown();
        return Err(e);
    }

    // Mark active. On a status-write failure, unwind cleanly.
    if let Err(e) = set_proxy_status(state, &proxy_id, ProxyStatus::Active).await {
        handle.shutdown();
        let saved = state.saved_proxy.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(p) = saved {
            let _ = system_proxy::restore(p);
        }
        return Err(e);
    }

    *state.relay.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
    *state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()) = Some(proxy_id);

    Ok(ip)
}

pub async fn disconnect_proxy(state: &AppState) -> Result<(), String> {
    // Flip the previously active proxy back to inactive.
    let active_id = state.active_proxy_id.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(id) = active_id {
        let _ = set_proxy_status(state, &id, ProxyStatus::Inactive).await;
    }

    // Stop the relay.
    if let Some(handle) = state.relay.lock().unwrap_or_else(|e| e.into_inner()).take() {
        handle.shutdown();
    }

    // Restore the original system proxy.
    let saved = state
        .saved_proxy
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let Some(proxy) = saved {
        system_proxy::restore(proxy)?;
    }

    Ok(())
}

pub async fn test_proxy(
    host: String,
    port: u16,
    protocol: String,
    username: Option<String>,
    password: Option<String>,
    timeout: Option<u64>,
) -> Result<u64, String> {
    let proxy_url = build_proxy_url(
        &protocol,
        &host,
        port,
        username.as_deref(),
        password.as_deref(),
    );

    let timeout_secs = timeout.map(|ms| (ms / 1000).max(1)).unwrap_or(10);
    let client = build_proxy_client(&proxy_url, timeout_secs)?;
    let start = Instant::now();

    client
        .get("https://api.ipify.org")
        .send()
        .await
        .map_err(|e| format!("Proxy unreachable: {e}"))?;

    Ok(start.elapsed().as_millis() as u64)
}

/// Test a saved proxy by id. Decrypts the stored password server-side so the
/// ciphertext never has to round-trip through the frontend.
pub async fn test_proxy_by_id(id: String, state: &AppState) -> Result<u64, String> {
    let proxy = proxy::Entity::find_by_id(&id)
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proxy not found: {id}"))?;

    let key = crypto::derive_key()?;
    let password = proxy
        .password
        .as_deref()
        .map(|enc| crypto::decrypt(enc, &key))
        .transpose()
        .map_err(map_decrypt_err)?;

    let settings = load_settings(state).await?;
    let timeout = Some((settings.global_timeout.max(0) as u64).max(1));

    test_proxy(
        proxy.host,
        proxy.port as u16,
        protocol_str(&proxy.protocol).to_string(),
        proxy.username,
        password,
        timeout,
    )
    .await
}

/// Cumulative bytes moved through the relay since the current connection began.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Throughput {
    pub down_bytes: u64,
    pub up_bytes: u64,
}

pub async fn get_throughput(state: &AppState) -> Result<Throughput, String> {
    let guard = state.relay.lock().unwrap_or_else(|e| e.into_inner());
    let (down, up) = match guard.as_ref() {
        Some(handle) => (
            handle.stats.down.load(Ordering::Relaxed),
            handle.stats.up.load(Ordering::Relaxed),
        ),
        None => (0, 0),
    };
    Ok(Throughput { down_bytes: down, up_bytes: up })
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
