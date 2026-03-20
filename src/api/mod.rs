pub mod middleware;
pub mod storage;
pub mod users;

use crate::config::Config;
use axum::Router;
use mongodb::Database;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        users::login,
        users::create_user,
        users::get_all_users,
        users::get_user,
        users::update_profile,
        users::update_password,
        users::delete_user,
        users::get_profile,
        storage::upload_file,
        storage::create_folder,
        storage::get_files,
        storage::get_trashed_files,
        storage::get_file,
        storage::move_file,
        storage::get_download_url,
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
    let state = AppState {
        db,
        config: Arc::new(config),
    };

    let auth_routes = Router::new()
        .route("/login", axum::routing::post(users::login))
        .with_state(state.clone());

    // 创建用户（注册）无需登录，其余用户接口需 JWT
    let user_routes_public = Router::new()
        .route("/", axum::routing::post(users::create_user))
        .with_state(state.clone());

    let user_routes_protected = Router::new()
        .route("/", axum::routing::get(users::get_all_users))
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

    let user_routes = user_routes_public.merge(user_routes_protected);

    let storage_routes = Router::new()
        .route("/upload", axum::routing::post(storage::upload_file))
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
            axum::routing::get(users::get_profile).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                middleware::auth_guard,
            )),
        )
        .with_state(state.clone());

    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/v1", v1)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
