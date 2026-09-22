use fragrans::{api, config, infrastructure};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,tower_http=info,fragrans=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env().expect("Invalid configuration");
    let db = infrastructure::db::init_db(&config)
        .await
        .expect("Failed to initialize database");

    let app = api::router(db.clone(), config.clone());

    let db_bg = db.clone();
    let config_bg = config.clone();
    tokio::spawn(async move {
        fragrans::service::storage::backfill_video_thumbnails(&db_bg, &config_bg).await;
    });

    let local_storage = infrastructure::storage::local::LocalStorage::new(
        config.storage_destination.clone(),
        config.storage_master_key,
    )
    .expect("Failed to initialize storage for cleanup task");
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(12 * 3600));
        loop {
            interval.tick().await;
            tracing::info!("Starting periodic cleanup of stale temporary uploads...");
            match local_storage
                .cleanup_stale_temp_uploads(std::time::Duration::from_secs(24 * 3600))
                .await
            {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Cleaned up {} stale temp upload directories", count);
                    }
                }
                Err(e) => {
                    tracing::warn!("Error during temp upload cleanup: {}", e);
                }
            }
        }
    });

    let port = config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
