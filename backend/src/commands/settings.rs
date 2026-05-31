use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::db::entities::{settings, enums::proxy::ProxyProtocol};
use crate::AppState;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub id: String,
    pub global_timeout: i32,
    pub dns_leak_protection: bool,
    pub proxy_protocol: ProxyProtocol,
}

pub async fn get_settings(state: &AppState) -> Result<settings::Model, String> {
    settings::Entity::find()
        .one(&state.db)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Settings row not found".to_string())
}

pub async fn save_settings(input: SettingsInput, state: &AppState) -> Result<(), String> {
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
