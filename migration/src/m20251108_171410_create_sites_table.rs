use crate::m20251016_092534_create_users_table::User;
use crate::m20251016_173133_create_api_keys_table::ApiKey;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Site::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Site::Id)
                            .big_unsigned()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Site::Name).string().not_null())
                    .col(ColumnDef::new(Site::Url).string().not_null())
                    .col(ColumnDef::new(Site::UrlList).string().not_null())
                    .col(ColumnDef::new(Site::PathLink).string().null())
                    .col(ColumnDef::new(Site::PathTitle).string().null())
                    .col(ColumnDef::new(Site::PathContent).string().null())
                    .col(ColumnDef::new(Site::PathImage).string().null())
                    .col(ColumnDef::new(Site::PathVideo).string().null())
                    .col(ColumnDef::new(Site::PathRemove).text().null())
                    .col(ColumnDef::new(Site::Screenshot).boolean().default(false))
                    .col(ColumnDef::new(Site::Status).boolean().default(true))
                    .col(ColumnDef::new(Site::UserId).big_unsigned().not_null())
                    .col(ColumnDef::new(Site::ApiKeyId).big_unsigned().not_null())
                    .col(
                        ColumnDef::new(Site::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Site::Table, Site::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Site::Table, Site::ApiKeyId)
                            .to(ApiKey::Table, ApiKey::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Site::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Site {
    Table,
    Id,
    Name,
    Url,
    PathLink,
    PathTitle,
    PathContent,
    PathImage,
    PathVideo,
    PathRemove,
    Screenshot,
    Status,
    UserId,
    ApiKeyId,
    CreatedAt,
    UrlList,
}
