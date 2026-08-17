#![allow(dead_code)]

use axum::{
    body::Body,
    http::{Request, header},
};
use chrono::Utc;
use fragrans::{
    api::{self},
    config::Config,
    domain::user::User,
    utils::crypto::hash_password,
};
use http_body_util::BodyExt;
use mongodb::{Client, Database, bson::doc, options::ClientOptions};
use std::{env, time::Duration};
use tempfile::TempDir;

pub struct TestContext {
    pub app: axum::Router,
    pub db: Database,
    pub storage_dir: TempDir,
    pub user_id: String,
    pub auth_token: String,
    pub download_token: String,
}

impl TestContext {
    pub async fn teardown(self) {
        let _ = self.db.drop().await;
    }
}

pub async fn setup() -> TestContext {
    let mongo_uri = env::var("TEST_MONGO_URI")
        .or_else(|_| env::var("MONGO_URI"))
        .unwrap_or_else(|_| "mongodb://test:nest@127.0.0.1:25018/?authSource=admin".to_string());

    let mut options = ClientOptions::parse(&mongo_uri)
        .await
        .expect("Failed to parse mongo URI");
    options.server_selection_timeout = Some(Duration::from_secs(1));
    options.connect_timeout = Some(Duration::from_secs(1));
    let client = Client::with_options(options).expect("Failed to create mongo client");

    let db_name = format!("fragrans_test_{}", uuid::Uuid::new_v4());
    let db = client.database(&db_name);
    if let Err(e) = db.run_command(doc! { "ping": 1 }).await {
        panic!("Skipping integration tests: MongoDB is unavailable: {}", e);
    }

    let storage_dir = TempDir::new().expect("Failed to create temp dir");
    fragrans::infrastructure::db::ensure_indexes(&db)
        .await
        .expect("create indexes");

    let jwt_secret = "test-secret-key-that-is-long-enough".to_string();
    let config = Config {
        mongo_uri: mongo_uri.clone(),
        jwt_secret: jwt_secret.clone(),
        port: 3821,
        domain: "http://localhost:3821".to_string(),
        storage_destination: storage_dir.path().to_path_buf(),
        storage_master_key: [0u8; 32],
        max_upload_bytes: 10 * 1024 * 1024,
    };
    let app = api::router(db.clone(), config);

    let user = User {
        id: None,
        email: format!("user-{}@example.com", db_name),
        password: hash_password("password123"),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        gender: None,
        age: None,
        avatar: None,
        roles: vec!["user".to_string()],
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };
    let inserted = db
        .collection::<User>("users")
        .insert_one(user)
        .await
        .expect("Failed to insert user")
        .inserted_id
        .as_object_id()
        .expect("Inserted ID is not ObjectId");
    let user_id = inserted.to_hex();

    let token = api::middleware::create_token(
        &jwt_secret,
        &user_id,
        api::middleware::TokenPurpose::Access,
        None,
        (chrono::Utc::now().timestamp() + 3600) as usize,
        None,
    )
    .expect("Failed to encode token");

    let download_token = api::middleware::create_token(
        &jwt_secret,
        &user_id,
        api::middleware::TokenPurpose::Download,
        Some("dummy_file_id".to_string()),
        (chrono::Utc::now().timestamp() + 3600) as usize,
        None,
    )
    .expect("Failed to encode token");

    TestContext {
        app,
        db,
        storage_dir,
        user_id,
        auth_token: token.clone(),
        download_token,
    }
}

pub fn auth_request(method: &str, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .expect("request")
}

pub async fn response_bytes(response: axum::response::Response) -> bytes::Bytes {
    response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes()
}

pub fn json_auth_request(
    method: &str,
    uri: &str,
    token: &str,
    body: serde_json::Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("json request")
}
