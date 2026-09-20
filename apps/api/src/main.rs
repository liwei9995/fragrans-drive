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
