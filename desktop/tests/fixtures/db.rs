#![allow(dead_code)]

use app_lib::db::{migrations::Migrator, seed_settings};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub async fn mem_db() -> DatabaseConnection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);

    let db = Database::connect(options).await.expect("connect mem db");
    Migrator::up(&db, None).await.expect("run migrations");
    seed_settings(&db).await.expect("seed settings");
    db
}

pub async fn mem_db_no_seed() -> DatabaseConnection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);

    let db = Database::connect(options).await.expect("connect mem db");
    Migrator::up(&db, None).await.expect("run migrations");
    db
}
