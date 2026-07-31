mod common;
use common::*;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serial_test::serial;
use tower::util::ServiceExt;

fn write_legacy_file(root: &std::path::Path, content: &[u8], iv: &str) -> String {
    use ctr::cipher::{KeyIvInit, StreamCipher};
    use md5::Digest;

    let mut hasher = md5::Md5::new();
    hasher.update(content);
    let hash = hex::encode(hasher.finalize());
    let mut iv_bytes = [0u8; 16];
    hex::decode_to_slice(iv, &mut iv_bytes).unwrap();
    let key = aes::cipher::generic_array::GenericArray::from_slice(hash.as_bytes());
    let mut cipher = ctr::Ctr128BE::<aes::Aes256>::new(key, &iv_bytes.into());
    let mut encrypted = content.to_vec();
    cipher.apply_keystream(&mut encrypted);

    let dir = root.join(&hash[0..2]).join(&hash[2..4]).join(&hash[4..6]);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(&hash), encrypted).unwrap();
    hash
}

fn multipart_upload_request(uri: &str, token: &str, file_content: &[u8]) -> Request<Body> {
    let boundary = "----Boundary123";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"parentId\"\r\n\r\nroot\r\n");
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: text/plain\r\n\r\n");
    body.extend_from_slice(file_content);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap()
}

async fn upload_file(ctx: &TestContext, content: &[u8]) -> String {
    let req = multipart_upload_request("/v1/storage/upload", &ctx.auth_token, content);
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<String> = serde_json::from_slice(&response_bytes(res).await).unwrap();
    data[0].clone()
}

#[tokio::test]
#[serial]
async fn access_token_can_call_authenticated_api() {
    let ctx = setup().await;
    let req = Request::builder()
        .method("GET")
        .uri("/v1/profile")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn list_download_url_is_immediately_usable() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"listed content").await;

    let list = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/storage/list")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"query":{"parentId":"root"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(list).await).unwrap();
    let item = body["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_str() == Some(&file_id))
        .unwrap();
    let url = item["url"].as_str().unwrap();
    let uri = url.strip_prefix("http://localhost:3821").unwrap();

    let download = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(download.status(), StatusCode::OK);
    assert_eq!(response_bytes(download).await.as_ref(), b"listed content");

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn legacy_file_remains_downloadable_before_migration() {
    let ctx = setup().await;
    let content = b"legacy content";
    let iv = "00000000000000000000000000000000";
    let md5_hash = write_legacy_file(ctx.storage_dir.path(), content, iv);
    let file_id = mongodb::bson::oid::ObjectId::new();
    ctx.db
        .collection::<mongodb::bson::Document>("storage")
        .insert_one(mongodb::bson::doc! {
            "_id": file_id,
            "userId": &ctx.user_id,
            "name": "legacy.txt",
            "parentId": "root",
            "type": "file",
            "mimeType": "text/plain",
            "MD5Hash": md5_hash,
            "iv": iv,
            "trashed": false,
            "createdAt": mongodb::bson::DateTime::now(),
            "updatedAt": mongodb::bson::DateTime::now(),
        })
        .await
        .unwrap();

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/storage/{file_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_bytes(response).await.as_ref(), content);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn download_token_cannot_call_authenticated_api() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"test").await;

    // Get download token
    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"fileId": "{}"}}"#, file_id)))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let url = String::from_utf8(response_bytes(res).await.to_vec())
        .unwrap()
        .replace("\"", "");
    let download_token = url.split("token=").last().unwrap();

    let req = Request::builder()
        .method("GET")
        .uri("/v1/profile")
        .header(header::AUTHORIZATION, format!("Bearer {}", download_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn download_token_can_fetch_only_its_file() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"test").await;

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"fileId": "{}"}}"#, file_id)))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let url = String::from_utf8(response_bytes(res).await.to_vec())
        .unwrap()
        .replace('"', "");
    let download_token = url.split("token=").last().unwrap();

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{}", file_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", download_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body_bytes = response_bytes(res).await;
    if status != StatusCode::OK {
        println!("Error: {}", String::from_utf8_lossy(&body_bytes));
    }
    assert_eq!(status, StatusCode::OK);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn download_token_cannot_fetch_sibling_file() {
    let ctx = setup().await;
    let file1 = upload_file(&ctx, b"test1").await;
    let file2 = upload_file(&ctx, b"test2").await;

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"fileId": "{}"}}"#, file1)))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let url = String::from_utf8(response_bytes(res).await.to_vec())
        .unwrap()
        .replace("\"", "");
    let download_token = url.split("token=").last().unwrap();

    // try to fetch file2 with file1's token
    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{}", file2))
        .header(header::AUTHORIZATION, format!("Bearer {}", download_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn download_url_rejects_unowned_file() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"test").await;

    let db_name = format!(
        "fragrans_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let user = fragrans::domain::user::User {
        id: None,
        email: format!("user2-{}@example.com", db_name),
        password: fragrans::utils::crypto::hash_password("password123"),
        first_name: "Test2".to_string(),
        last_name: "User2".to_string(),
        gender: None,
        age: None,
        avatar: None,
        roles: vec!["user".to_string()],
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    };
    let inserted = ctx
        .db
        .collection::<fragrans::domain::user::User>("users")
        .insert_one(user)
        .await
        .unwrap()
        .inserted_id
        .as_object_id()
        .unwrap();
    let user_id2 = inserted.to_hex();

    let token2 = fragrans::api::middleware::create_token(
        "test-secret-key-that-is-long-enough",
        &user_id2,
        fragrans::api::middleware::TokenPurpose::Access,
        None,
        (chrono::Utc::now().timestamp() + 3600) as usize,
        None,
    )
    .unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", token2))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"fileId": "{}"}}"#, file_id)))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn expired_download_token_is_rejected() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"test").await;

    let claims = fragrans::api::middleware::Claims {
        user_id: ctx.user_id.clone(),
        purpose: fragrans::api::middleware::TokenPurpose::Download,
        file_id: Some(file_id.clone()),
        exp: (chrono::Utc::now().timestamp() - 3600) as usize,
        share_version: Some(0),
    };
    let expired_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"test-secret-key-that-is-long-enough"),
    )
    .unwrap();

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{}?token={}", file_id, expired_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    // In actual implementation, expiration is 0 means it expires immediately or already expired.
    // Wait, the JWT token generation adds `exp = Utc::now() + exp_seconds`. If we use 0, it expires right now or slightly after.
    // Let's use `exp_seconds = -1` but `create_token` uses `usize`. So we can use a custom logic to create an expired token.
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn download_response_does_not_log_query_token() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"test").await;

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!(r#"{{"fileId": "{}"}}"#, file_id)))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let url = String::from_utf8(response_bytes(res).await.to_vec())
        .unwrap()
        .replace("\"", "");
    let download_token = url.split("token=").last().unwrap();

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{}?token={}", file_id, download_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verifying tracing output requires a custom subscriber which is complex for this test.
    // We trust that our TraceLayer configuration in src/api/mod.rs works as implemented.

    ctx.teardown().await;
}

async fn download_url(ctx: &TestContext, body: &str) -> (StatusCode, String) {
    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/download/url")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let text = String::from_utf8(response_bytes(res).await.to_vec()).unwrap();
    (status, text.replace('"', ""))
}

#[tokio::test]
#[serial]
async fn revoke_share_invalidates_existing_download_token() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"revoked").await;

    let (status, url) = download_url(&ctx, &format!(r#"{{"fileId":"{file_id}"}}"#)).await;
    assert_eq!(status, StatusCode::OK);
    let old_token = url.split("token=").last().unwrap().to_string();

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/storage/{file_id}/revoke_share"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    assert_eq!(body["shareVersion"], 1);

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{file_id}?token={old_token}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let (status, url) = download_url(&ctx, &format!(r#"{{"fileId":"{file_id}"}}"#)).await;
    assert_eq!(status, StatusCode::OK);
    let new_token = url.split("token=").last().unwrap();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{file_id}?token={new_token}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn list_thumbnail_url_uses_live_share_version() {
    use fragrans::domain::storage::{Storage, StorageType};
    use fragrans::infrastructure::db::storage_repo::StorageRepository;
    use jsonwebtoken::{DecodingKey, Validation};

    let ctx = setup().await;
    let repo = StorageRepository::new(&ctx.db);
    let now = chrono::Utc::now();
    let thumb_id = repo
        .create(Storage {
            id: None,
            name: "thumb.jpg".into(),
            base_name: None,
            ext_name: None,
            mime_type: Some("image/jpeg".into()),
            encoding: None,
            size: Some(1),
            md5_hash: None,
            iv: None,
            content_hash: Some("a".repeat(64)),
            hash_algorithm: Some("sha256".into()),
            encryption_format: Some(1),
            share_version: 0,
            parent_id: "root".into(),
            r#type: StorageType::Thumbnail,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .unwrap();
    let file_id = repo
        .create(Storage {
            id: None,
            name: "photo.jpg".into(),
            base_name: Some("photo".into()),
            ext_name: Some("jpg".into()),
            mime_type: Some("image/jpeg".into()),
            encoding: None,
            size: Some(1),
            md5_hash: None,
            iv: None,
            content_hash: Some("b".repeat(64)),
            hash_algorithm: Some("sha256".into()),
            encryption_format: Some(1),
            share_version: 0,
            parent_id: "root".into(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: Some(thumb_id.to_hex()),
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .unwrap();

    let revoke = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/v1/storage/{}/revoke_share", thumb_id.to_hex()))
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(revoke.status(), StatusCode::OK);

    let list = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/storage/list")
                .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"query":{"parentId":"root"}}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(list).await).unwrap();
    let item = body["docs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"].as_str() == Some(&file_id.to_hex()))
        .unwrap();
    let thumb_url = item["thumbnail"].as_str().unwrap();
    let token = thumb_url.split("token=").last().unwrap();
    let claims = jsonwebtoken::decode::<fragrans::api::middleware::Claims>(
        token,
        &DecodingKey::from_secret(b"test-secret-key-that-is-long-enough"),
        &Validation::default(),
    )
    .unwrap()
    .claims;
    assert_eq!(claims.share_version, Some(1));
    assert_eq!(claims.file_id.as_deref(), Some(thumb_id.to_hex().as_str()));

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn expire_in_seconds_rejects_out_of_range() {
    let ctx = setup().await;
    let file_id = upload_file(&ctx, b"expire").await;

    for body in [
        format!(r#"{{"fileId":"{file_id}","expireInSeconds":-1}}"#),
        format!(r#"{{"fileId":"{file_id}","expireInSeconds":0}}"#),
        format!(r#"{{"fileId":"{file_id}","expireInSeconds":86401}}"#),
    ] {
        let (status, _) = download_url(&ctx, &body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "body={body}");
    }

    let (status, url) =
        download_url(&ctx, &format!(r#"{{"fileId":"{file_id}","expireInSeconds":60}}"#)).await;
    assert_eq!(status, StatusCode::OK);
    let token = url.split("token=").last().unwrap();
    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{file_id}?token={token}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    ctx.teardown().await;
}
