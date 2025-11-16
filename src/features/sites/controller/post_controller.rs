use crate::core::dto::pagination::PaginationParams;
use crate::core::response::{json_error, json_success};
use crate::features::sites::repository::post_repository::PostRepository;
use crate::features::sites::validation::post_form::{PostForm, PostFormCreate};
use crate::features::users::service::api_key_user::ApiKey;
use crate::features::users::service::auth_user::AuthUser;
use axum::Form;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use validator::Validate;

pub struct PostController;

impl PostController {
    // GET /posts
    pub async fn list(Query(p): Query<PaginationParams>) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match PostRepository::list(page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // GET /sites/:site_id/posts
    pub async fn list_by_site(
        Path(site_id): Path<i64>,
        Query(p): Query<PaginationParams>,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match PostRepository::list_by_site(site_id, page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // GET /me/posts
    pub async fn list_by_user(
        Query(p): Query<PaginationParams>,
        AuthUser(user): AuthUser,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match PostRepository::list_by_user(user.id, page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // GET /token/posts
    pub async fn list_by_token(
        Query(p): Query<PaginationParams>,
        ApiKey(api_key): ApiKey,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match PostRepository::list_by_api_key(api_key.id, page, per_page).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // POST /posts
    pub async fn create(Form(form): Form<PostFormCreate>) -> Response {
        if let Err(e) = form.validate() {
            return json_error(StatusCode::BAD_REQUEST, e.to_string());
        }

        match PostRepository::create(form).await {
            Ok(Some(post)) => json_success(post),
            Ok(None) => json_error(StatusCode::BAD_REQUEST, "Failed to create post".to_string()),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // PUT /posts/:id
    pub async fn update(Path(post_id): Path<i64>, Form(form): Form<PostForm>) -> Response {
        if let Err(e) = form.validate() {
            return json_error(StatusCode::BAD_REQUEST, e.to_string());
        }

        match PostRepository::update(post_id, form).await {
            Ok(Some(post)) => json_success(post),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Post not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    // GET /posts/:id
    pub async fn show(Path(post_id): Path<i64>) -> Response {
        match PostRepository::find_by_id(post_id).await {
            Ok(Some(post)) => json_success(post),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Post not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    // GET /posts/by-url/:url
    pub async fn show_by_url(Path(url): Path<String>) -> Response {
        match PostRepository::find_by_url(&url).await {
            Ok(Some(post)) => json_success(post),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Post not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    // DELETE /posts/:id
    pub async fn delete(Path(post_id): Path<i64>) -> Response {
        match PostRepository::delete(post_id).await {
            Ok(result) => json_success(result),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e),
        }
    }
}
