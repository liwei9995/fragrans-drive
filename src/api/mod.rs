pub mod error;
pub mod middleware;
pub mod storage;
pub mod users;

use crate::config::Config;
use axum::{Router, extract::State};
use mongodb::Database;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
    pub local_storage: crate::infrastructure::storage::local::LocalStorage,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        users::login,
        users::create_user,
        users::update_password,
        users::update_profile,
        users::get_profile,
        storage::upload_file,
        storage::create_folder,
        storage::get_files,
        storage::get_trashed_files,
        storage::get_file,
        storage::move_file,
        storage::get_download_url,
        storage::revoke_share,
        storage::update_file,
        storage::remove_file,
        storage::restore_file,
        storage::restore_trashed_files,
        storage::empty_trash,
        storage::get_path,
    ),
    components(
        schemas(
            users::CreateUserDto, users::UpdateUserDto, users::UpdatePasswordDto, users::LoginDto, users::LoginResponse, users::CreateUserResponse,
            storage::CreateFolderDto, storage::GetFilesDto, storage::GetPathDto, storage::MoveFileDto, storage::RestoreTrashDto,
            crate::domain::user::User, crate::domain::user::UserResponse, crate::domain::storage::Storage, crate::domain::storage::StorageListResponse, crate::domain::storage::StorageListPaginatedResponse, crate::domain::storage::StoragePathNode, crate::domain::storage::CreateFolderResponse, crate::domain::storage::UpdateStorageResponse, crate::domain::storage::TrashCleanupResponse, crate::domain::storage::TrashRestoreResponse,
            middleware::UserContext
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "storage", description = "File storage endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

pub fn router(db: Database, config: Config) -> Router {
    let local_storage = crate::infrastructure::storage::local::LocalStorage::new(
        config.storage_destination.clone(),
        config.storage_master_key,
    )
    .expect("Failed to initialize local storage");

    let state = AppState {
        db,
        config: Arc::new(config),
        local_storage,
    };

    let auth_routes = Router::new()
        .route("/login", axum::routing::post(users::login))
        .with_state(state.clone());

    // User registration does not require auth; other user routes require JWT.
    let user_routes_public = Router::new()
        .route("/", axum::routing::post(users::create_user))
        .with_state(state.clone());

    let user_routes_protected = Router::new()
        .route("/password", axum::routing::post(users::update_password))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_guard,
        ))
        .with_state(state.clone());

    let user_routes = user_routes_public.merge(user_routes_protected);

    let storage_routes = Router::new()
        .route("/upload", axum::routing::post(storage::upload_file))
        .layer(axum::extract::DefaultBodyLimit::max(
            state.config.max_upload_bytes,
        ))
        .route("/folder", axum::routing::post(storage::create_folder))
        .route("/list", axum::routing::post(storage::get_files))
        .route(
            "/trash/list",
            axum::routing::post(storage::get_trashed_files),
        )
        .route(
            "/trash/restore",
            axum::routing::post(storage::restore_trashed_files),
        )
        .route("/path", axum::routing::post(storage::get_path))
        .route("/move", axum::routing::post(storage::move_file))
        .route("/trash", axum::routing::delete(storage::empty_trash))
        .route(
            "/download/url",
            axum::routing::post(storage::get_download_url),
        )
        .route(
            "/{id}/revoke_share",
            axum::routing::post(storage::revoke_share),
        )
        .route(
            "/{id}",
            axum::routing::put(storage::update_file).delete(storage::remove_file),
        )
        .route("/{id}/restore", axum::routing::post(storage::restore_file))
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
            axum::routing::get(users::get_profile)
                .patch(users::update_profile)
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    middleware::auth_guard,
                )),
        )
        .with_state(state.clone());

    let health_routes = Router::new()
        .route("/live", axum::routing::get(|| async { "OK" }))
        .route(
            "/ready",
            axum::routing::get(|State(state): State<AppState>| async move {
                // Ping mongo
                if let Err(e) = state
                    .db
                    .run_command(mongodb::bson::doc! { "ping": 1 })
                    .await
                {
                    tracing::error!("Health check failed (mongo ping): {}", e);
                    return axum::http::StatusCode::SERVICE_UNAVAILABLE;
                }

                // Check if storage destination is writable
                let test_file = state
                    .config
                    .storage_destination
                    .join(format!(".healthcheck-{}", uuid::Uuid::new_v4()));
                let write_result = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&test_file)
                    .await;
                if let Err(e) = write_result {
                    tracing::error!("Health check failed (storage write): {}", e);
                    return axum::http::StatusCode::SERVICE_UNAVAILABLE;
                }
                if let Err(e) = tokio::fs::remove_file(&test_file).await {
                    tracing::error!("Health check failed (storage cleanup): {}", e);
                    return axum::http::StatusCode::SERVICE_UNAVAILABLE;
                }

                axum::http::StatusCode::OK
            }),
        )
        .with_state(state.clone());

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/health", health_routes)
        .nest("/v1", v1)
        .layer(TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
            tracing::info_span!("http_request", method = %request.method(), uri = %request.uri().path())
        }))
        .with_state(state)
}
