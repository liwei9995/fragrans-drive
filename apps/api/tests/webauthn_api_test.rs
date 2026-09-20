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

    let wrong_password = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/webauthn/register-start",
            &ctx.auth_token,
            serde_json::json!({ "password": "wrong" }),
        ))
        .await
        .unwrap();
    assert_eq!(wrong_password.status(), StatusCode::UNAUTHORIZED);

    // 3. Test register-start with authenticated user
    let reg_start_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/webauthn/register-start",
            &ctx.auth_token,
            serde_json::json!({ "password": "password123" }),
        ))
        .await
        .expect("register-start request");

    assert_eq!(reg_start_res.status(), StatusCode::OK);
    let reg_start_body: serde_json::Value =
        serde_json::from_slice(&response_bytes(reg_start_res).await).expect("parse reg response");
    assert!(reg_start_body.get("sessionId").is_some());
    assert!(reg_start_body.get("challenge").is_some());

    // Deleting a passkey revokes sessions, including the current access token.
    let user_id = mongodb::bson::oid::ObjectId::parse_str(&ctx.user_id).unwrap();
    let repo = fragrans::infrastructure::db::user_repo::UserRepository::new(&ctx.db);
    repo.add_passkey(
        user_id,
        fragrans::domain::user::StoredPasskey {
            id: "test-passkey".to_string(),
            name: "Test".to_string(),
            passkey_json: "{}".to_string(),
            created_at: Some(chrono::Utc::now()),
        },
    )
    .await
    .unwrap();
    assert!(
        repo.update_passkey(user_id, "test-passkey", "{}", "{}")
            .await
            .unwrap()
    );
    let delete = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "DELETE",
            "/v1/auth/webauthn/passkeys/test-passkey",
            &ctx.auth_token,
            serde_json::json!({ "password": "password123" }),
        ))
        .await
        .unwrap();
    let status = delete.status();
    let body = response_bytes(delete).await;
    assert_eq!(
        status,
        StatusCode::NO_CONTENT,
        "{}",
        String::from_utf8_lossy(&body)
    );
    assert!(
        !repo
            .update_passkey(user_id, "test-passkey", "{}", "{}")
            .await
            .unwrap()
    );
    let profile = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", &ctx.auth_token))
        .await
        .unwrap();
    assert_eq!(profile.status(), StatusCode::UNAUTHORIZED);

    ctx.teardown().await;
}
