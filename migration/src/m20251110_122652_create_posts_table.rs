use crate::m20251016_092534_create_users_table::User;
use crate::m20251016_173133_create_api_keys_table::ApiKey;
use crate::m20251108_171410_create_sites_table::Site;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // table
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .big_integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().null())
                    .col(ColumnDef::new(Posts::Body).text().null())
                    .col(ColumnDef::new(Posts::Image).string().null())
                    .col(ColumnDef::new(Posts::Video).string().null())
                    .col(ColumnDef::new(Posts::Url).string().not_null())
                    .col(ColumnDef::new(Posts::SiteId).big_integer().not_null())
                    .col(ColumnDef::new(Posts::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Posts::ApiKeyId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Posts::Status)
                            .string()
                            .not_null()
                            .default("PENDING")
                            .check(Expr::col(Posts::Status).is_in([
                                "PENDING",
                                "COMPLETED",
                                "FAILED",
                                "CANCELLED",
                            ])),
                    )
                    .col(ColumnDef::new(Posts::Retry).integer().not_null().default(0))
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::ApiKeyId)
                            .to(ApiKey::Table, ApiKey::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Posts::Table, Posts::SiteId)
                            .to(Site::Table, Site::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_user_id")
                    .table(Posts::Table)
                    .col(Posts::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_api_key_id")
                    .table(Posts::Table)
                    .col(Posts::ApiKeyId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_user_api_key")
                    .table(Posts::Table)
                    .col(Posts::UserId)
                    .col(Posts::ApiKeyId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_status")
                    .table(Posts::Table)
                    .col(Posts::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_post_site_id_url")
                    .table(Posts::Table)
                    .col(Posts::SiteId)
                    .col(Posts::Url)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Posts {
    Table,
    Id,
    Title,
    Body,
    Image,
    Video,
    Url,
    SiteId,
    UserId,
    ApiKeyId,
    Retry,
    Status,
    CreatedAt,
}
