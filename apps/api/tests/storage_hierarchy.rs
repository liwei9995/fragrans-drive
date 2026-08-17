use axum::http::StatusCode;
use common::{json_auth_request, response_bytes, setup};
use serde_json::json;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn move_folder_cycle_returns_400() {
    let ctx = setup().await;
    let folder_a = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            json!({
                "name": "A",
                "parentId": "root",
                "type": "folder"
            }),
        ))
        .await
        .unwrap();
    let folder_a: serde_json::Value =
        serde_json::from_slice(&response_bytes(folder_a).await).unwrap();
    let folder_a_id = folder_a["id"].as_str().unwrap();

    let folder_b = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            json!({
                "name": "B",
                "parentId": folder_a_id,
                "type": "folder"
            }),
        ))
        .await
        .unwrap();
    let folder_b: serde_json::Value =
        serde_json::from_slice(&response_bytes(folder_b).await).unwrap();
    let folder_b_id = folder_b["id"].as_str().unwrap();

    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/move",
            &ctx.auth_token,
            json!({
                "fileId": folder_a_id,
                "parentId": folder_b_id
            }),
        ))
        .await
        .expect("request");

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    ctx.teardown().await;
}
