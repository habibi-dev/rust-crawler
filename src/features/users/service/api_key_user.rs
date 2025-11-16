use crate::features::users::model::api_key;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use std::sync::Arc;

pub struct ApiKey(pub Arc<api_key::Model>);

impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Arc<api_key::Model>>()
            .cloned()
            .map(ApiKey)
            .ok_or((StatusCode::UNAUTHORIZED, "Not ApiKey".to_string()))
    }
}
