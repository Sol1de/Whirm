use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "connection_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub proxy_id: Option<String>,
    pub connected_at: DateTimeUtc,
    pub disconnected_at: Option<DateTimeUtc>,
    pub ip_address: Option<String>,
    #[sea_orm(belongs_to, from = "proxy_id", to = "id")]
    pub proxy: HasOne<super::proxy::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
