use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BoardData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BoardData::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BoardData::ServerId).big_integer().not_null())
                    .col(
                        ColumnDef::new(BoardData::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BoardData::MessageId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BoardData::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BoardData::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BoardData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BoardData {
    Table,
    Id,
    ServerId,
    ChannelId,
    MessageId,
    CreatedAt,
    UpdatedAt,
}
