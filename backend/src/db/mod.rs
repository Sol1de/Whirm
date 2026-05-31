pub mod entities;
pub mod migrations;

use entities::enums::proxy::ProxyProtocol;
use entities::settings;
use migrations::Migrator;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, DatabaseConnection, EntityTrait};
use sea_orm_migration::MigratorTrait;
use tauri::Manager;

pub async fn init(app_handle: &tauri::AppHandle) -> Result<DatabaseConnection, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Cannot create app data dir: {e}"))?;

    let db_path = data_dir.join("whirm.db");
    let path_str = db_path
        .to_str()
        .ok_or("Invalid DB path (non-UTF8 characters)")?
        .replace('\\', "/");

    let connection_string = format!("sqlite://{}?mode=rwc", path_str);

    let db = Database::connect(&connection_string)
        .await
        .map_err(|e| format!("Failed to connect to database: {e}"))?;

    Migrator::up(&db, None)
        .await
        .map_err(|e| format!("Failed to run migrations: {e}"))?;

    seed_settings(&db).await?;

    Ok(db)
}

pub async fn seed_settings(db: &DatabaseConnection) -> Result<(), String> {
    let exists = settings::Entity::find()
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if exists.is_none() {
        settings::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            global_timeout: Set(5000),
            dns_leak_protection: Set(true),
            proxy_protocol: Set(ProxyProtocol::Socks5),
        }
        .insert(db)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
