mod common;
use common::*;

use axum::http::StatusCode;
use fragrans::{api, config::Config};
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn test_auth_config_endpoint() {
    let ctx = setup().await;

    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "GET",
            "/v1/auth/config",
            "",
            serde_json::Value::Null,
        ))
        .await
        .expect("auth config request");

    assert_eq!(res.status(), StatusCode::OK);
    let data: serde_json::Value =
        serde_json::from_slice(&response_bytes(res).await).expect("parse response");
    assert!(data.get("allowRegistration").is_some());
    assert!(data.get("emailVerificationRequired").is_some());
    assert!(data.get("captchaRequired").is_some());

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_captcha_endpoint() {
    let ctx = setup().await;

    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "GET",
            "/v1/auth/captcha",
            "",
            serde_json::Value::Null,
        ))
        .await
        .expect("captcha request");

    assert_eq!(res.status(), StatusCode::OK);
    let data: serde_json::Value =
        serde_json::from_slice(&response_bytes(res).await).expect("parse response");
    let id = data["id"].as_str().expect("id exists");
    let svg = data["svg"].as_str().expect("svg exists");
    assert!(!id.is_empty());
    assert!(svg.starts_with("<svg"));

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_registration_disabled_rejects_requests() {
    let ctx = setup().await;

    let custom_config = Config {
        mongo_uri: "mongodb://localhost:27017".to_string(),
        jwt_secret: "a-very-long-secret-key-that-is-at-least-32-chars".to_string(),
        port: 3821,
        domain: "http://localhost:3821".to_string(),
        storage_destination: ctx.storage_dir.path().to_path_buf(),
        storage_master_key: [0u8; 32],
        max_upload_bytes: 10 * 1024 * 1024,
        allow_registration: false, // Registration disabled
        email_verification_required: false,
        captcha_required: false,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
    };
    let app = api::router(ctx.db.clone(), custom_config);

    // 1. Check auth config reflects disabled registration
    let config_res = app
        .clone()
        .oneshot(json_auth_request(
            "GET",
            "/v1/auth/config",
            "",
            serde_json::Value::Null,
        ))
        .await
        .expect("config request");
    let config_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(config_res).await).expect("parse response");
    assert_eq!(config_data["allowRegistration"], false);

    // 2. Attempting to send register email code fails with 403 Forbidden
    let send_code_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": "closed@example.com",
                "purpose": "register",
                "captchaId": "test",
                "captchaCode": "8888" // bypass code for test
            }),
        ))
        .await
        .expect("send code request");
    assert_eq!(send_code_res.status(), StatusCode::FORBIDDEN);

    // 3. Attempting to register fails with 403 Forbidden
    let register_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": "closed@example.com",
                "password": "password123",
                "firstName": "Closed",
                "lastName": "User"
            }),
        ))
        .await
        .expect("register request");
    assert_eq!(register_res.status(), StatusCode::FORBIDDEN);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_captcha_and_email_verification_flow() {
    let ctx = setup().await;

    let custom_config = Config {
        mongo_uri: "mongodb://localhost:27017".to_string(),
        jwt_secret: "a-very-long-secret-key-that-is-at-least-32-chars".to_string(),
        port: 3821,
        domain: "http://localhost:3821".to_string(),
        storage_destination: ctx.storage_dir.path().to_path_buf(),
        storage_master_key: [0u8; 32],
        max_upload_bytes: 10 * 1024 * 1024,
        allow_registration: true,
        email_verification_required: true, // Email verification required
        captcha_required: true,            // Captcha required
        smtp_host: None,                   // Mock email mode
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
    };
    let app = api::router(ctx.db.clone(), custom_config);

    let test_email = format!("secure-user-{}@example.com", uuid::Uuid::new_v4());

    // 1. Send code with invalid captcha fails
    let fail_captcha_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register",
                "captchaId": "wrong_id",
                "captchaCode": "0000"
            }),
        ))
        .await
        .expect("send code fail");
    assert_eq!(fail_captcha_res.status(), StatusCode::BAD_REQUEST);

    // 2. Send code with valid test captcha succeeds
    let send_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register",
                "captchaId": "test",
                "captchaCode": "8888"
            }),
        ))
        .await
        .expect("send code ok");
    assert_eq!(send_res.status(), StatusCode::OK);

    // 3. Register with wrong email verification code fails
    let reg_fail_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": test_email,
                "password": "password123",
                "firstName": "Verified",
                "lastName": "User",
                "captchaId": "test",
                "captchaCode": "8888",
                "emailCode": "000000"
            }),
        ))
        .await
        .expect("reg fail");
    assert_eq!(reg_fail_res.status(), StatusCode::BAD_REQUEST);

    // 4. Register with valid verification code (888888 in test mode) succeeds
    let reg_ok_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": test_email,
                "password": "password123",
                "firstName": "Verified",
                "lastName": "User",
                "captchaId": "test",
                "captchaCode": "8888",
                "emailCode": "888888"
            }),
        ))
        .await
        .expect("reg ok");
    assert_eq!(reg_ok_res.status(), StatusCode::CREATED);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_forgot_and_reset_password() {
    let ctx = setup().await;

    let app = ctx.app.clone();
    let email = format!("reset-user-{}@example.com", uuid::Uuid::new_v4());

    // 1. Create user
    let reg_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": email,
                "password": "old_password_123",
                "firstName": "Reset",
                "lastName": "Test"
            }),
        ))
        .await
        .expect("register request");
    assert_eq!(reg_res.status(), StatusCode::CREATED);

    // 2. Request reset password code
    let send_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "reset_password",
                "captchaId": "test",
                "captchaCode": "8888"
            }),
        ))
        .await
        .expect("send reset code");
    assert_eq!(send_res.status(), StatusCode::OK);

    // 3. Reset password with wrong code fails
    let reset_fail = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/reset-password",
            "",
            serde_json::json!({
                "email": email,
                "code": "111111",
                "password": "new_password_123",
                "changePassword": "new_password_123"
            }),
        ))
        .await
        .expect("reset fail");
    assert_eq!(reset_fail.status(), StatusCode::BAD_REQUEST);

    // 4. Reset password with valid code succeeds
    let reset_ok = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/reset-password",
            "",
            serde_json::json!({
                "email": email,
                "code": "888888",
                "password": "new_password_123",
                "changePassword": "new_password_123"
            }),
        ))
        .await
        .expect("reset ok");
    assert_eq!(reset_ok.status(), StatusCode::OK);

    // 5. Old password login fails
    let old_login = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": email,
                "password": "old_password_123"
            }),
        ))
        .await
        .expect("old login");
    assert_eq!(old_login.status(), StatusCode::UNAUTHORIZED);

    // 6. New password login succeeds
    let new_login = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": email,
                "password": "new_password_123"
            }),
        ))
        .await
        .expect("new login");
    assert_eq!(new_login.status(), StatusCode::OK);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_forgot_password_user_enumeration_protection() {
    let ctx = setup().await;
    let app = ctx.app.clone();
    let nonexistent_email = format!("nonexistent-{}@example.com", uuid::Uuid::new_v4());

    // Requesting reset code for non-existent email returns 200 generic success without leaking error
    let res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": nonexistent_email,
                "purpose": "reset_password",
                "captchaId": "test",
                "captchaCode": "8888"
            }),
        ))
        .await
        .expect("send reset code");

    assert_eq!(res.status(), StatusCode::OK);
    let data: serde_json::Value =
        serde_json::from_slice(&response_bytes(res).await).expect("parse response");
    assert!(
        data["message"]
            .as_str()
            .unwrap()
            .contains("If this email is registered")
    );

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_email_rate_limiting_cooldown() {
    let ctx = setup().await;
    let app = ctx.app.clone();
    let email = format!("ratelimit-{}@example.com", uuid::Uuid::new_v4());

    // First send succeeds
    let res1 = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "register",
                "captchaId": "test",
                "captchaCode": "8888"
            }),
        ))
        .await
        .expect("send code 1");
    assert_eq!(res1.status(), StatusCode::OK);

    // Immediate second send to the same email fails with 429 Too Many Requests
    let res2 = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "register",
                "captchaId": "test",
                "captchaCode": "8888"
            }),
        ))
        .await
        .expect("send code 2");
    assert_eq!(res2.status(), StatusCode::TOO_MANY_REQUESTS);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_case_insensitive_email_registration_and_login() {
    let ctx = setup().await;
    let app = ctx.app.clone();
    let unique_id = uuid::Uuid::new_v4();
    let mixed_email = format!("MixCase-{}@Example.COM", unique_id);
    let lower_email = mixed_email.to_lowercase();
    let upper_email = mixed_email.to_uppercase();

    // 1. Register with mixed-case email
    let reg_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": mixed_email,
                "password": "secure_password_123",
                "firstName": "Case",
                "lastName": "Sensitive"
            }),
        ))
        .await
        .expect("register request");
    assert_eq!(reg_res.status(), StatusCode::CREATED);

    // 2. Registering with lowercase version of the same email fails with 400 User already exists
    let dup_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": lower_email,
                "password": "secure_password_123",
                "firstName": "Case",
                "lastName": "Sensitive"
            }),
        ))
        .await
        .expect("duplicate register request");
    assert_eq!(dup_res.status(), StatusCode::BAD_REQUEST);

    // 3. Login with lowercase email succeeds
    let login_lower = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": lower_email,
                "password": "secure_password_123"
            }),
        ))
        .await
        .expect("login lower");
    assert_eq!(login_lower.status(), StatusCode::OK);

    // 4. Login with uppercase email succeeds
    let login_upper = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": upper_email,
                "password": "secure_password_123"
            }),
        ))
        .await
        .expect("login upper");
    assert_eq!(login_upper.status(), StatusCode::OK);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_input_validation_limits() {
    let ctx = setup().await;
    let app = ctx.app.clone();

    // 1. Password > 72 chars fails
    let long_password = "a".repeat(73);
    let res_long_pw = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": format!("longpw-{}@example.com", uuid::Uuid::new_v4()),
                "password": long_password,
                "firstName": "Long",
                "lastName": "Password"
            }),
        ))
        .await
        .expect("long pw");
    assert_eq!(res_long_pw.status(), StatusCode::BAD_REQUEST);

    // 2. Empty or blank first name fails
    let res_blank_name = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": format!("blankname-{}@example.com", uuid::Uuid::new_v4()),
                "password": "password123",
                "firstName": "   ",
                "lastName": "Valid"
            }),
        ))
        .await
        .expect("blank name");
    assert_eq!(res_blank_name.status(), StatusCode::BAD_REQUEST);

    ctx.teardown().await;
}
