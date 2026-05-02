use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::proxy::ProxyProtocol;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub global_timeout: i32,
    pub dns_leak_protection: bool,
    pub proxy_protocol: ProxyProtocol,
}

impl ActiveModelBehavior for ActiveModel {}
