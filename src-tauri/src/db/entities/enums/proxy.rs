use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "proxy_protocol")]
#[serde(rename_all = "UPPERCASE")]
pub enum ProxyProtocol {
    #[sea_orm(string_value = "SOCKS5")]
    Socks5,
    #[sea_orm(string_value = "HTTP")]
    Http,
    #[sea_orm(string_value = "HTTPS")]
    Https,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "proxy_status")]
#[serde(rename_all = "lowercase")]
pub enum ProxyStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "error")]
    Error,
}
