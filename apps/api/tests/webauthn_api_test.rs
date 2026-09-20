mod common;
use common::*;

use axum::http::StatusCode;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn test_webauthn_api_flow() {
    let ctx = setup().await;

    // 1. Test public login-start without email
    let login_start_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/webauthn/login-start",
            "",
            serde_json::json!({ "email": null }),
        ))
        .await
        .expect("login-start request");

    assert_eq!(login_start_res.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&response_bytes(login_start_res).await).expect("parse response");
    assert!(body.get("sessionId").is_some());
    assert!(body.get("challenge").is_some());

    // 2. Test list passkeys (initially empty for existing test user)
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "GET",
            "/v1/auth/webauthn/passkeys",
            &ctx.auth_token,
            serde_json::json!({}),
        ))
        .await
        .expect("list passkeys request");

    assert_eq!(list_res.status(), StatusCode::OK);
    let list_body: Vec<serde_json::Value> =
        serde_json::from_slice(&response_bytes(list_res).await).expect("parse list response");
    assert_eq!(list_body.len(), 0);

    // 3. Test register-start with authenticated user
    let reg_start_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/webauthn/register-start",
            &ctx.auth_token,
            serde_json::json!({}),
        ))
        .await
        .expect("register-start request");

    assert_eq!(reg_start_res.status(), StatusCode::OK);
    let reg_start_body: serde_json::Value =
        serde_json::from_slice(&response_bytes(reg_start_res).await).expect("parse reg response");
    assert!(reg_start_body.get("sessionId").is_some());
    assert!(reg_start_body.get("challenge").is_some());

    ctx.teardown().await;
}
