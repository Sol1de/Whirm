use sea_orm_migration::prelude::*;

mod m20240101_000001_create_proxies;
mod m20240101_000002_create_settings;
mod m20240101_000003_create_connection_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_proxies::Migration),
            Box::new(m20240101_000002_create_settings::Migration),
            Box::new(m20240101_000003_create_connection_sessions::Migration),
        ]
    }
}
