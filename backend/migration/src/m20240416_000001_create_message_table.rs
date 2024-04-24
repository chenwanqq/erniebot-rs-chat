use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Message::MessageId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Message::SessionId).integer().not_null())
                    .col(
                        ColumnDef::new(Message::Role)
                            .enumeration(
                                Alias::new("role"),
                                vec![
                                    Alias::new("user"),
                                    Alias::new("assistant"),
                                    Alias::new("function"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Message::Content).text().not_null())
                    .col(
                        ColumnDef::new(Message::MessageType)
                            .enumeration(
                                Alias::new("message_type"),
                                vec![Alias::new("text"), Alias::new("image"), Alias::new("file")],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(Message::CreateTime).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Message {
    Table,
    MessageId,
    SessionId,
    Role,
    Content,
    MessageType,
    CreateTime,
}
