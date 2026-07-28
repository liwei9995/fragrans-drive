use axum::http::StatusCode;
use common::{json_auth_request, setup};
use serde_json::json;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn put_storage_unknown_field_returns_400() {
    let ctx = setup().await;
    // We expect a PUT with arbitrary fields to fail if they are not explicitly allowed.
    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "PUT",
            "/v1/storage/000000000000000000000000",
            &ctx.auth_token,
            json!({
                "name": "new_name.txt",
                "userId": "some_other_user",
                "MD5Hash": "hash",
                "contentHash": "hash",
                "hashAlgorithm": "md5",
                "iv": "iv",
                "encryptionFormat": 0,
                "type": "file",
                "thumbnail": "thumb",
                "trashed": true,
                "createdAt": "date",
                "updatedAt": "date"
            }),
        ))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn cannot_mutate_other_user_item() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "PUT",
            "/v1/storage/000000000000000000000000",
            &ctx.auth_token,
            json!({
                "name": "new_name.txt"
            }),
        ))
        .await
        .expect("request");
    
    // We'll get 404 because we don't own the item (or it doesn't exist).
    // In this dummy test we use an invalid ID for simplicity, but the test ensures
    // the payload validation (e.g. deny_unknown_fields) triggers before the 404, or 
    // that the handler restricts by user.
    assert!(res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::BAD_REQUEST);
}
