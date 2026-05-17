use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum Proxies {
    Table,
    Id,
    Name,
    Host,
    Port,
    Protocol,
    Country,
    CountryCode,
    Username,
    Password,
    Status,
    Category,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Proxies::Table)
                    .if_not_exists()
                    .col(string(Proxies::Id).primary_key())
                    .col(string(Proxies::Name))
                    .col(string(Proxies::Host))
                    .col(integer(Proxies::Port))
                    .col(string(Proxies::Protocol).extra("CHECK(protocol IN ('SOCKS5','HTTP','HTTPS'))"))
                    .col(string(Proxies::Country).default(""))
                    .col(string(Proxies::CountryCode).default(""))
                    .col(string_null(Proxies::Username))
                    .col(string_null(Proxies::Password))
                    .col(string(Proxies::Status).default("inactive").extra("CHECK(status IN ('active','inactive','error'))"))
                    .col(string_null(Proxies::Category))
                    .col(string_null(Proxies::LastUsedAt))
                    .col(string(Proxies::CreatedAt).extra("DEFAULT (datetime('now'))"))
                    .col(string(Proxies::UpdatedAt).extra("DEFAULT (datetime('now'))"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Proxies::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}
