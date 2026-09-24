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
        trust_proxy_headers: false,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
        turnstile_secret_key: None,
        turnstile_site_key: None,
    };
    let app =
        api::router_with_auth_security(ctx.db.clone(), custom_config, ctx.auth_security.clone());

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
    let (captcha_id, captcha_code) = ctx.captcha();
    let send_code_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": "closed@example.com",
                "purpose": "register",
                "captchaId": captcha_id,
                "captchaCode": captcha_code
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
        trust_proxy_headers: false,
        smtp_host: None, // Delivery must fail closed without SMTP.
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
        turnstile_secret_key: None,
        turnstile_site_key: None,
    };
    let app =
        api::router_with_auth_security(ctx.db.clone(), custom_config, ctx.auth_security.clone());

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

    // 2. Valid captcha reaches mail delivery; absent SMTP fails closed.
    let (captcha_id, captcha_code) = ctx.captcha();
    let send_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register",
                "captchaId": captcha_id,
                "captchaCode": captcha_code
            }),
        ))
        .await
        .expect("send code ok");
    assert_eq!(send_res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let email_code = ctx
        .auth_security
        .generate_email_code("register", &test_email);

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
                "emailCode": "000000"
            }),
        ))
        .await
        .expect("reg fail");
    assert_eq!(reg_fail_res.status(), StatusCode::BAD_REQUEST);

    // 4. Register with a real verification code succeeds.
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
                "emailCode": email_code
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

    // 2. Sending fails closed without SMTP; seed the code for reset validation.
    let (captcha_id, captcha_code) = ctx.captcha();
    let send_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "reset_password",
                "captchaId": captcha_id,
                "captchaCode": captcha_code
            }),
        ))
        .await
        .expect("send reset code");
    assert_eq!(send_res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let email_code = ctx
        .auth_security
        .generate_email_code("reset_password", &email);

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
                "code": email_code,
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
    let (captcha_id, captcha_code) = ctx.captcha();

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
                "captchaId": captcha_id,
                "captchaCode": captcha_code
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

    // First attempt reaches the mailer and fails closed without SMTP.
    let (captcha_id, captcha_code) = ctx.captcha();
    let res1 = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "register",
                "captchaId": captcha_id,
                "captchaCode": captcha_code
            }),
        ))
        .await
        .expect("send code 1");
    assert_eq!(res1.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Immediate second send to the same email fails with 429 Too Many Requests
    let (captcha_id, captcha_code) = ctx.captcha();
    let res2 = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": email,
                "purpose": "register",
                "captchaId": captcha_id,
                "captchaCode": captcha_code
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

#[tokio::test]
#[serial]
async fn test_turnstile_verification_flow() {
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
        email_verification_required: true,
        captcha_required: true,
        trust_proxy_headers: false,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
        turnstile_secret_key: Some("1x0000000000000000000000000000000AA".to_string()),
        turnstile_site_key: Some("1x00000000000000000000AA".to_string()),
    };
    let app =
        api::router_with_auth_security(ctx.db.clone(), custom_config, ctx.auth_security.clone());

    // 1. Auth config returns turnstileSiteKey
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
    assert_eq!(config_data["turnstileSiteKey"], "1x00000000000000000000AA");

    let test_email = format!("turnstile-user-{}@example.com", uuid::Uuid::new_v4());

    // 2. Requesting send-code without turnstile token fails
    let missing_token_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register"
            }),
        ))
        .await
        .expect("send code without token");
    assert_eq!(missing_token_res.status(), StatusCode::BAD_REQUEST);

    // 3. Requesting send-code with empty turnstile token fails
    let empty_token_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register",
                "turnstileToken": "   "
            }),
        ))
        .await
        .expect("send code with empty token");
    assert_eq!(empty_token_res.status(), StatusCode::BAD_REQUEST);

    // 4. Requesting send-code with valid dummy test turnstile token passes human verification
    // (reaches mail delivery stage which returns 500 when SMTP is not configured)
    let valid_token_res = app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/send-code",
            "",
            serde_json::json!({
                "email": test_email,
                "purpose": "register",
                "turnstileToken": "dummy-pass-token"
            }),
        ))
        .await
        .expect("send code with valid token");
    assert_eq!(valid_token_res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 5. Test direct registration when email verification is disabled
    let no_email_config = Config {
        mongo_uri: "mongodb://localhost:27017".to_string(),
        jwt_secret: "a-very-long-secret-key-that-is-at-least-32-chars".to_string(),
        port: 3821,
        domain: "http://localhost:3821".to_string(),
        storage_destination: ctx.storage_dir.path().to_path_buf(),
        storage_master_key: [0u8; 32],
        max_upload_bytes: 10 * 1024 * 1024,
        allow_registration: true,
        email_verification_required: false,
        captcha_required: true,
        trust_proxy_headers: false,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_pass: None,
        smtp_from: None,
        turnstile_secret_key: Some("1x0000000000000000000000000000000AA".to_string()),
        turnstile_site_key: Some("1x00000000000000000000AA".to_string()),
    };
    let app2 =
        api::router_with_auth_security(ctx.db.clone(), no_email_config, ctx.auth_security.clone());

    // Register without turnstile token fails
    let reg_no_token = app2
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": format!("no-token-{}@example.com", uuid::Uuid::new_v4()),
                "password": "password123",
                "firstName": "Turn",
                "lastName": "Stile"
            }),
        ))
        .await
        .expect("register no token");
    assert_eq!(reg_no_token.status(), StatusCode::BAD_REQUEST);

    // Register with valid turnstile token succeeds
    let reg_with_token = app2
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users",
            "",
            serde_json::json!({
                "email": format!("with-token-{}@example.com", uuid::Uuid::new_v4()),
                "password": "password123",
                "firstName": "Turn",
                "lastName": "Stile",
                "turnstileToken": "dummy-pass-token"
            }),
        ))
        .await
        .expect("register with token");
    assert_eq!(reg_with_token.status(), StatusCode::CREATED);

    ctx.teardown().await;
}
