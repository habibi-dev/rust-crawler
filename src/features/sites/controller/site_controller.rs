use crate::core::dto::pagination::PaginationParams;
use crate::core::response::{json_error, json_success};
use crate::features::sites::repository::site_repository::SiteRepository;
use crate::features::sites::validation::site_form::SiteForm;
use crate::features::users::service::api_key_user::ApiKey;
use crate::features::users::service::auth_user::AuthUser;
use axum::Form;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use validator::Validate;

pub struct SiteController;

impl SiteController {
    pub async fn list(Query(p): Query<PaginationParams>) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match SiteRepository::list(page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    pub async fn list_by_user(
        Query(p): Query<PaginationParams>,
        AuthUser(user): AuthUser,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match SiteRepository::list_by_user(user.id, page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    pub async fn list_by_token(
        Query(p): Query<PaginationParams>,
        ApiKey(api_key): ApiKey,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match SiteRepository::list_by_api_key(api_key.id, page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    pub async fn list_all_by_token(ApiKey(api_key): ApiKey) -> impl IntoResponse {
        match SiteRepository::list_all_by_api_key(api_key.id).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // POST /sites
    pub async fn create(Form(form): Form<SiteForm>) -> Response {
        if let Err(e) = form.validate() {
            return json_error(StatusCode::BAD_REQUEST, e.to_string());
        }

        match SiteRepository::create(form).await {
            Ok(site) => json_success(site),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // PUT /sites/:id
    pub async fn update(Path(site_id): Path<i64>, Form(form): Form<SiteForm>) -> Response {
        if let Err(e) = form.validate() {
            return json_error(StatusCode::BAD_REQUEST, e.to_string());
        }

        match SiteRepository::update(site_id, form).await {
            Ok(Some(site)) => json_success(site),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Site not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    // GET /sites/:id
    pub async fn show(Path(site_id): Path<i64>) -> Response {
        match SiteRepository::find_by_id(site_id).await {
            Ok(Some(site)) => json_success(site),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Site not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    // DELETE /sites/:id
    pub async fn delete(Path(site_id): Path<i64>) -> Response {
        match SiteRepository::delete(site_id).await {
            Ok(result) => json_success(result),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e),
        }
    }
}
