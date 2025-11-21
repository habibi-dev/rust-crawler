use crate::features::sites::controller::post_controller::PostController;
use crate::features::sites::controller::site_controller::SiteController;
use crate::middleware::auth::auth;
use crate::middleware::is_admin::is_admin;
use crate::utility::state::app_state;
use axum::routing::get;
use axum::{Router, middleware};

pub fn site_route() -> (&'static str, Router) {
    let state = app_state();

    let mw_auth = middleware::from_fn_with_state(state.clone(), auth);
    let mw_admin = middleware::from_fn_with_state(state.clone(), is_admin);

    let admin_router = Router::new()
        .route("/", get(SiteController::list).post(SiteController::create))
        .route(
            "/{site_id}",
            get(SiteController::show)
                .put(SiteController::update)
                .delete(SiteController::delete),
        )
        .route_layer(mw_admin);

    (
        "api/v1/sites",
        Router::new()
            .route("/by-user", get(SiteController::list_by_user))
            .route("/by-token", get(SiteController::list_by_token))
            .route("/by-token/all", get(SiteController::list_all_by_token))
            .merge(admin_router)
            .route_layer(mw_auth),
    )
}

pub fn post_route() -> (&'static str, Router) {
    let state = app_state();

    let mw_auth = middleware::from_fn_with_state(state.clone(), auth);
    let mw_admin = middleware::from_fn_with_state(state.clone(), is_admin);

    let admin_router = Router::new()
        .route("/", get(PostController::list).post(PostController::create))
        .route(
            "/{post_id}",
            get(PostController::show)
                .put(PostController::update)
                .delete(PostController::delete),
        )
        .route("/by-url/{url}", get(PostController::show_by_url))
        .route_layer(mw_admin);

    (
        "api/v1/posts",
        Router::new()
            .route("/by-user", get(PostController::list_by_user))
            .route("/by-token", get(PostController::list_by_token))
            .route("/by-site/{site_id}", get(PostController::list_by_site))
            .merge(admin_router)
            .route_layer(mw_auth),
    )
}
