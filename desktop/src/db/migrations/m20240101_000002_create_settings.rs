use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum Settings {
    Table,
    Id,
    GlobalTimeout,
    DnsLeakProtection,
    ProxyProtocol,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(string(Settings::Id).primary_key())
                    .col(integer(Settings::GlobalTimeout).default(5000))
                    .col(boolean(Settings::DnsLeakProtection).default(true).extra("CHECK(dns_leak_protection IN (0, 1))"))
                    .col(string(Settings::ProxyProtocol).default("SOCKS5").extra("CHECK(proxy_protocol IN ('SOCKS5','HTTP','HTTPS'))"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}
