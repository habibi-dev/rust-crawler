use crate::core::dto::pagination::PaginationParams;
use crate::core::response::{json_error, json_success};
use crate::features::sites::model::site::Model;
use crate::features::sites::repository::site_repository::SiteRepository;
use crate::features::sites::validation::site_form::SiteForm;
use crate::features::users::model::user;
use crate::features::users::service::api_key_user::ApiKey;
use crate::features::users::service::auth_user::AuthUser;
use axum::Form;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use validator::Validate;

pub struct SiteController;

impl SiteController {
    pub async fn list(
        Query(p): Query<PaginationParams>,
        ApiKey(api_key): ApiKey,
        AuthUser(user): AuthUser,
    ) -> impl IntoResponse {
        let page = p.page();
        let per_page = p.per_page();

        match SiteRepository::list(page, per_page, user, api_key).await {
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

    pub async fn list_all_by_user(AuthUser(user): AuthUser) -> impl IntoResponse {
        match SiteRepository::list_all_by_user_id(user.id).await {
            Ok(items) => json_success(items),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // POST /sites
    pub async fn create(
        ApiKey(api_key): ApiKey,
        AuthUser(user): AuthUser,
        Form(mut form): Form<SiteForm>,
    ) -> Response {
        if let Err(e) = form.validate() {
            return json_error(StatusCode::BAD_REQUEST, e.to_string());
        }

        // Non-admins can't choose ownership; force to themselves and current api_key.
        if !user.is_admin {
            form.user_id = Some(user.id);
            form.api_key_id = Some(api_key.id);
        } else {
            // Admins may omit; default to current context.
            if form.user_id.is_none() {
                form.user_id = Some(user.id);
            }
            if form.api_key_id.is_none() {
                form.api_key_id = Some(api_key.id);
            }
        }

        match SiteRepository::create(form).await {
            Ok(site) => json_success(site),
            Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }

    // PUT /sites/:id
    pub async fn update(
        AuthUser(user): AuthUser,
        Path(site_id): Path<i64>,
        Form(form): Form<SiteForm>,
    ) -> Response {
        let _site = match Self::check_access(site_id, &user).await {
            Ok(site) => site,
            Err(resp) => return resp,
        };

        if let Err((code, msg)) = Self::check_update(&form) {
            return json_error(code, msg);
        }

        match SiteRepository::update(site_id, form).await {
            Ok(Some(site)) => json_success(site),
            Ok(None) => json_error(StatusCode::NOT_FOUND, "Site not found".to_string()),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }

    pub async fn show(AuthUser(user): AuthUser, Path(site_id): Path<i64>) -> Response {
        match Self::check_access(site_id, &user).await {
            Ok(site) => json_success(site),
            Err(resp) => resp,
        }
    }

    pub async fn delete(AuthUser(user): AuthUser, Path(site_id): Path<i64>) -> Response {
        let _site = match Self::check_access(site_id, &user).await {
            Ok(site) => site,
            Err(resp) => return resp,
        };

        match SiteRepository::delete(site_id).await {
            Ok(result) => json_success(result),
            Err(e) => json_error(StatusCode::BAD_REQUEST, e),
        }
    }

    fn check_update(form: &SiteForm) -> Result<(), (StatusCode, String)> {
        form.validate()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        Ok(())
    }

    async fn check_access(site_id: i64, user: &user::Model) -> Result<Model, Response> {
        let site = match SiteRepository::find_by_id(site_id).await {
            Ok(Some(site)) => site,
            Ok(None) => {
                return Err(json_error(
                    StatusCode::NOT_FOUND,
                    "Site not found".to_string(),
                ));
            }
            Err(e) => {
                return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
        };

        if !user.is_admin && site.user_id != user.id {
            return Err(json_error(
                StatusCode::FORBIDDEN,
                "You do not have permission to access this site".to_string(),
            ));
        }

        Ok(site)
    }
}
