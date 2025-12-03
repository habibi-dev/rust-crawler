use crate::core::dto::pagination::PaginationParams;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PostListParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub post_id: Option<i64>,
}
