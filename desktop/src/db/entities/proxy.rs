use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::proxy::{ProxyProtocol, ProxyStatus};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "proxies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub protocol: ProxyProtocol,
    pub country: String,
    pub country_code: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub status: ProxyStatus,
    pub category: Option<String>,
    pub last_used_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    #[sea_orm(has_many)]
    pub connection_sessions: HasMany<super::connection_session::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
