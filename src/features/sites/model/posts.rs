use crate::features::sites::model::site;
use crate::features::users::model::{api_key, user};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub body: Option<String>,
    pub image: Option<String>,
    pub video: Option<String>,
    pub url: Option<String>,
    pub retry: i8,
    pub status: PostStatus,
    pub site_id: i64,
    pub user_id: i64,
    pub api_key_id: i64,
    pub created_at: DateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum PostStatus {
    #[sea_orm(string_value = "PENDING")]
    PENDING,

    #[sea_orm(string_value = "COMPLETED")]
    COMPLETED,

    #[sea_orm(string_value = "FAILED")]
    FAILED,

    #[sea_orm(string_value = "CANCELLED")]
    CANCELLED,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation, Serialize)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "api_key::Entity",
        from = "Column::ApiKeyId",
        to = "api_key::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    ApiKey,
    #[sea_orm(
        belongs_to = "site::Entity",
        from = "Column::SiteId",
        to = "site::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Site,
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::UserId",
        to = "user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<api_key::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ApiKey.def()
    }
}

impl Related<site::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Site.def()
    }
}

impl Related<user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
