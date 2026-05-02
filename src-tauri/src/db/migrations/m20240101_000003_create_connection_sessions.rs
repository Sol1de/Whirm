use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
enum Proxies {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConnectionSessions {
    Table,
    Id,
    ProxyId,
    ConnectedAt,
    DisconnectedAt,
    IpAddress,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut fk = ForeignKey::create()
            .from(ConnectionSessions::Table, ConnectionSessions::ProxyId)
            .to(Proxies::Table, Proxies::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .to_owned();

        manager
            .create_table(
                Table::create()
                    .table(ConnectionSessions::Table)
                    .if_not_exists()
                    .col(string(ConnectionSessions::Id).primary_key())
                    .col(string_null(ConnectionSessions::ProxyId))
                    .col(string(ConnectionSessions::ConnectedAt))
                    .col(string_null(ConnectionSessions::DisconnectedAt))
                    .col(string_null(ConnectionSessions::IpAddress))
                    .foreign_key(&mut fk)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ConnectionSessions::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}
