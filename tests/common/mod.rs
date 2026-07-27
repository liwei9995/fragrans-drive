use axum::{
    body::Body,
    http::{Request, header},
};
use chrono::Utc;
use fragrans::{
    api::{self, middleware::Claims},
    config::Config,
    domain::user::User,
    utils::{crypto::hash_password},
};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use mongodb::{Client, Database, bson::doc, options::ClientOptions};
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
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

pub async fn setup() -> Option<TestContext> {
    let mongo_uri = env::var("TEST_MONGO_URI")
        .or_else(|_| env::var("MONGO_URI"))
        .unwrap_or_else(|_| "mongodb://test:nest@127.0.0.1:25018/?authSource=admin".to_string());

    let mut options = ClientOptions::parse(&mongo_uri).await.ok()?;
    options.server_selection_timeout = Some(Duration::from_secs(1));
    options.connect_timeout = Some(Duration::from_secs(1));
    let client = Client::with_options(options).ok()?;

    let db_name = format!(
        "fragrans_test_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos()
    );
    let db = client.database(&db_name);
    if db.run_command(doc! { "ping": 1 }).await.is_err() {
        eprintln!("Skipping integration tests: MongoDB is unavailable");
        return None;
    }

    let storage_dir = TempDir::new().ok()?;
    unsafe {
        env::set_var("STORAGE_DESTINATION", storage_dir.path());
    }

    let jwt_secret = "test-secret-key-that-is-long-enough".to_string();
    let config = Config {
        mongo_uri: mongo_uri.clone(),
        jwt_secret: jwt_secret.clone(),
        port: 3821,
        domain: "http://localhost:3821".to_string(),
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
        .ok()?
        .inserted_id
        .as_object_id()?;
    let user_id = inserted.to_hex();

    let claims = Claims {
        user_id: user_id.clone(),
        exp: (Utc::now().timestamp() + 3600) as usize,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .ok()?;

    Some(TestContext {
        app,
        db,
        storage_dir,
        user_id,
        auth_token: token.clone(),
        download_token: token,
    })
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
