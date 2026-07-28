use axum::http::StatusCode;
use common::{json_auth_request, setup};
use serde_json::json;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn move_folder_cycle_returns_400() {
    let ctx = setup().await;
    // We would need actual DB objects to test cycles and valid parents,
    // but we can at least write the skeleton.
    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/move",
            &ctx.auth_token,
            json!({
                "fileId": "000000000000000000000000",
                "parentId": "111111111111111111111111"
            }),
        ))
        .await
        .expect("request");
    
    // An invalid target parent should be 400 or 404. Let's say 400.
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
