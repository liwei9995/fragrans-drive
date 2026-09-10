mod common;
use common::*;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serial_test::serial;
use tower::util::ServiceExt;

fn multipart_upload_request(
    uri: &str,
    token: &str,
    file_content: &[u8],
    filename: &str,
    content_type: &str,
) -> Request<Body> {
    let boundary = "----Boundary123";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"parentId\"\r\n\r\nroot\r\n");
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", content_type).as_bytes());
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

async fn upload_custom_file(
    ctx: &TestContext,
    content: &[u8],
    filename: &str,
    content_type: &str,
) -> String {
    let req = multipart_upload_request(
        "/v1/storage/upload",
        &ctx.auth_token,
        content,
        filename,
        content_type,
    );
    let res = ctx.app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<String> = serde_json::from_slice(&response_bytes(res).await).unwrap();
    data[0].clone()
}

#[tokio::test]
#[serial]
async fn test_public_direct_link_lifecycle() {
    let ctx = setup().await;
    let file_id =
        upload_custom_file(&ctx, b"hello public direct link", "test.png", "image/png").await;

    // 1. Initially, file is not public
    let enable_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(enable_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    assert_eq!(body["isPublic"], true);
    let slug = body["publicSlug"].as_str().unwrap().to_string();
    assert!(!slug.is_empty());
    let public_url = body["publicUrl"].as_str().unwrap().to_string();
    assert!(public_url.ends_with(&format!("/v1/p/{}", slug)));

    // 2. Anonymous client can access via /v1/p/{slug} with NO auth headers
    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let headers = res.headers().clone();
    assert!(
        headers
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("sandbox")
    );
    assert_eq!(
        headers
            .get("x-content-type-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "nosniff"
    );
    let disposition = headers
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.starts_with("inline; filename="));
    let etag = headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let content = response_bytes(res).await;
    assert_eq!(content.as_ref(), b"hello public direct link");

    // 3. Anonymous client can access with filename suffix: /v1/p/{slug}/test.png
    let get_with_name_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}/test.png"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(get_with_name_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        response_bytes(res).await.as_ref(),
        b"hello public direct link"
    );

    // 4. Force download with ?download=1
    let get_download_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}?download=1"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(get_download_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let disposition = res
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.starts_with("attachment; filename="));

    // 5. 304 Not Modified when If-None-Match matches ETag
    let inm_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .header(header::IF_NONE_MATCH, etag)
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(inm_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_MODIFIED);

    // 6. Refresh slug
    let refresh_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true, "refresh": true}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(refresh_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    let new_slug = body["publicSlug"].as_str().unwrap().to_string();
    assert_ne!(slug, new_slug);

    // Old slug is now 404
    let old_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(old_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // New slug works
    let new_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{new_slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(new_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 7. Disable public status
    let disable_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": false}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(disable_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Slug is now 404
    let disabled_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{new_slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(disabled_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_dangerous_mime_forces_attachment() {
    let ctx = setup().await;
    let file_id = upload_custom_file(
        &ctx,
        b"<h1>evil</h1><script>alert(1)</script>",
        "evil.html",
        "text/html",
    )
    .await;

    let enable_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(enable_req).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    let slug = body["publicSlug"].as_str().unwrap();

    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let disposition = res
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    // Must be attachment, NEVER inline for HTML
    assert!(disposition.starts_with("attachment; filename="));
    assert!(
        res.headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("sandbox")
    );

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_unicode_chinese_filename_rfc5987() {
    let ctx = setup().await;
    let file_id =
        upload_custom_file(&ctx, b"chinese test content", "测试报告.png", "image/png").await;

    let enable_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(enable_req).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    let slug = body["publicSlug"].as_str().unwrap();

    let get_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let disposition = res
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.contains("filename=\"____.png\""));
    assert!(disposition.contains("filename*=UTF-8''%E6%B5%8B%E8%AF%95%E6%8A%A5%E5%91%8A.png"));

    // Verify anti-enumeration: tokenless direct /v1/storage/{id} is strictly rejected
    let direct_id_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/storage/{file_id}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(direct_id_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_public_link_expiration_and_head() {
    let ctx = setup().await;
    let file_id = upload_custom_file(&ctx, b"temporary public file", "temp.png", "image/png").await;

    // 1. Enable with expiresIn: 1 (expires in 1 second)
    let enable_req = Request::builder()
        .method("PUT")
        .uri(format!("/v1/storage/{file_id}/public"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true, "expiresIn": 1}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(enable_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    let slug = body["publicSlug"].as_str().unwrap().to_string();
    assert!(body["publicExpiresAt"].is_string());

    // 2. HEAD request: zero-IO check returns 200 with matching Content-Length and empty body
    let head_req = Request::builder()
        .method("HEAD")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(head_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap(),
        "21"
    );
    let head_body = response_bytes(res).await;
    assert!(head_body.is_empty());

    // 3. Test list filter by isPublic
    let list_req = Request::builder()
        .method("POST")
        .uri("/v1/storage/list")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"isPublic": true}"#))
        .unwrap();
    let res = ctx.app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let list_data: serde_json::Value = serde_json::from_slice(&response_bytes(res).await).unwrap();
    assert!(list_data["docs"].as_array().unwrap().len() >= 1);
    for doc in list_data["docs"].as_array().unwrap() {
        assert_eq!(doc["isPublic"], true);
    }

    // 4. Wait for expiration (1.2s)
    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;

    let expired_req = Request::builder()
        .method("GET")
        .uri(format!("/v1/p/{slug}"))
        .body(Body::empty())
        .unwrap();
    let res = ctx.app.clone().oneshot(expired_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    ctx.teardown().await;
}
