use crate::features::sites::model::posts::PostStatus;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PostForm {
    pub title: Option<String>,
    pub body: Option<String>,
    pub image: Option<String>,
    pub video: Option<String>,
    pub status: PostStatus,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PostFormCreate {
    pub url: Option<String>,
    pub site_id: i64,
    pub user_id: i64,
    pub api_key_id: i64,
}
