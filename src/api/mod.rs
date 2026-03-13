pub mod middleware;
pub mod storage;
pub mod users;

use crate::config::Config;
use axum::Router;
use mongodb::Database;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
}

pub fn router(db: Database, config: Config) -> Router {
    let state = AppState {
        db,
        config: Arc::new(config),
    };

    let auth_routes = Router::new()
        .route("/login", axum::routing::post(users::login))
        .with_state(state.clone());

    let user_routes = Router::new()
        .route(
            "/",
            axum::routing::get(users::get_all_users).post(users::create_user),
        )
        .route(
            "/{id}",
            axum::routing::get(users::get_user).delete(users::delete_user),
        )
        .route("/profile/{id}", axum::routing::post(users::update_profile))
        .route("/password", axum::routing::post(users::update_password))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_guard,
        ))
        .with_state(state.clone());

    let storage_routes = Router::new()
        .route("/upload", axum::routing::post(storage::upload_file))
        .route("/folder", axum::routing::post(storage::create_folder))
        .route("/list", axum::routing::post(storage::get_files))
        .route("/path", axum::routing::post(storage::get_path))
        .route("/move", axum::routing::post(storage::move_file))
        .route(
            "/download/url",
            axum::routing::post(storage::get_download_url),
        )
        .route(
            "/{id}",
            axum::routing::put(storage::update_file).delete(storage::remove_file),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_guard,
        ))
        .route("/{id}", axum::routing::get(storage::get_file)) // Move public Get here or keep it outside layer? Legacy had @Public()
        .with_state(state.clone());

    let v1 = Router::new()
        .nest("/auth", auth_routes)
        .nest("/users", user_routes)
        .nest("/storage", storage_routes)
        .route(
            "/profile",
            axum::routing::get(users::get_profile).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                middleware::auth_guard,
            )),
        )
        .with_state(state.clone());

    Router::new().nest("/v1", v1).with_state(state)
}
