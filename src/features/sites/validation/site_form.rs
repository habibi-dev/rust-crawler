use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct SiteForm {
    pub name: String,
    pub url: String,
    pub url_list: String,

    pub path_link: Option<String>,
    pub path_title: Option<String>,
    pub path_content: Option<String>,
    pub path_image: Option<String>,
    pub path_video: Option<String>,
    pub path_remove: Option<String>,

    pub screenshot: Option<bool>,
    pub status: Option<bool>,

    pub user_id: i64,
    pub api_key_id: i64,
}
