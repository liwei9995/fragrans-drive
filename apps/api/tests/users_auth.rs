mod common;
use common::*;

use axum::http::StatusCode;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn test_user_registration_and_login() {
    let ctx = setup().await;

    // 1. Create a new user
    let email = "newuser@example.com";
    let password = "newpassword123";
    let register_payload = serde_json::json!({
        "email": email,
        "password": password,
        "firstName": "John",
        "lastName": "Doe"
    });

    let register_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request("POST", "/v1/users", "", register_payload))
        .await
        .expect("register request");

    assert_eq!(register_res.status(), StatusCode::CREATED);
    let register_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(register_res).await).expect("parse register");
    let user_id = register_data["id"].as_str().expect("id exists").to_string();
    assert!(!user_id.is_empty());

    // 2. Login with the new user
    let login_payload = serde_json::json!({
        "email": email,
        "password": password
    });

    let login_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            login_payload,
        ))
        .await
        .expect("login request");

    assert_eq!(login_res.status(), StatusCode::OK);
    let refresh_cookie = refresh_cookie(&login_res);
    let login_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(login_res).await).expect("parse login");
    let token = login_data["access_token"]
        .as_str()
        .expect("token exists")
        .to_string();
    assert!(!token.is_empty());
    assert!(!refresh_cookie.is_empty());
    assert!(login_data.get("refresh_token").is_none());

    // 3. Get profile using the token
    let profile_res = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", &token))
        .await
        .expect("profile request");

    assert_eq!(profile_res.status(), StatusCode::OK);
    let profile: serde_json::Value =
        serde_json::from_slice(&response_bytes(profile_res).await).expect("parse profile");
    assert_eq!(profile["email"].as_str().unwrap(), email);
    assert_eq!(profile["firstName"].as_str().unwrap(), "John");
    assert_eq!(profile["lastName"].as_str().unwrap(), "Doe");

    // 4. Update profile
    let update_payload = serde_json::json!({
        "firstName": "Jane",
        "age": 30
    });
    let update_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "PATCH",
            "/v1/profile",
            &token,
            update_payload,
        ))
        .await
        .expect("update request");

    assert_eq!(update_res.status(), StatusCode::OK);
    let updated_profile: serde_json::Value =
        serde_json::from_slice(&response_bytes(update_res).await).expect("parse updated profile");
    assert_eq!(updated_profile["firstName"].as_str().unwrap(), "Jane");
    assert_eq!(updated_profile["lastName"].as_str().unwrap(), "Doe"); // unchanged
    assert_eq!(updated_profile["age"].as_u64().unwrap(), 30);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn refresh_rotates_tokens_and_rejects_access_token() {
    let ctx = setup().await;

    let login_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": format!("user-{}@example.com", ctx.db.name()),
                "password": "password123"
            }),
        ))
        .await
        .expect("login");
    assert_eq!(login_res.status(), StatusCode::OK);
    let cookie = refresh_cookie(&login_res);
    let login_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(login_res).await).expect("parse login");
    let access = login_data["access_token"].as_str().unwrap().to_string();
    let refresh = cookie.split_once('=').unwrap().1.to_string();
    let access_claims = jsonwebtoken::decode::<fragrans::api::middleware::Claims>(
        &access,
        &jsonwebtoken::DecodingKey::from_secret(b"test-secret-key-that-is-long-enough"),
        &jsonwebtoken::Validation::default(),
    )
    .unwrap()
    .claims;
    let refresh_claims = jsonwebtoken::decode::<fragrans::api::middleware::Claims>(
        &refresh,
        &jsonwebtoken::DecodingKey::from_secret(b"test-secret-key-that-is-long-enough"),
        &jsonwebtoken::Validation::default(),
    )
    .unwrap()
    .claims;
    let now = chrono::Utc::now().timestamp() as usize;
    assert!((access_claims.exp - now).abs_diff(900) <= 2);
    assert!((refresh_claims.exp - now).abs_diff(3600 * 24 * 7) <= 2);

    let access_as_refresh = ctx
        .app
        .clone()
        .oneshot(cookie_request(
            "POST",
            "/v1/auth/refresh",
            &format!("__Host-fragrans-refresh={access}"),
        ))
        .await
        .expect("refresh with access");
    assert_eq!(access_as_refresh.status(), StatusCode::UNAUTHORIZED);

    let refresh_res = ctx
        .app
        .clone()
        .oneshot(cookie_request("POST", "/v1/auth/refresh", &cookie))
        .await
        .expect("refresh");
    assert_eq!(refresh_res.status(), StatusCode::OK);
    let new_cookie = refresh_cookie(&refresh_res);
    let refresh_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(refresh_res).await).expect("parse refresh");
    let new_access = refresh_data["access_token"].as_str().unwrap().to_string();
    let new_refresh = new_cookie.split_once('=').unwrap().1.to_string();
    assert!(!new_access.is_empty());
    assert_ne!(new_refresh, refresh);

    let replay_res = ctx
        .app
        .clone()
        .oneshot(cookie_request("POST", "/v1/auth/refresh", &cookie))
        .await
        .expect("replay old refresh token");
    assert_eq!(replay_res.status(), StatusCode::UNAUTHORIZED);

    let profile_res = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", &new_access))
        .await
        .expect("profile");
    assert_eq!(profile_res.status(), StatusCode::OK);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn password_change_invalidates_refresh_token() {
    let ctx = setup().await;

    let login_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": format!("user-{}@example.com", ctx.db.name()),
                "password": "password123"
            }),
        ))
        .await
        .expect("login");
    let cookie = refresh_cookie(&login_res);
    let login_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(login_res).await).expect("parse login");
    let access = login_data["access_token"].as_str().unwrap().to_string();

    // Attempt password change with incorrect old password
    let wrong_old_pwd_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users/password",
            &access,
            serde_json::json!({
                "oldPassword": "wrongpassword",
                "password": "newpassword123",
                "changePassword": "newpassword123"
            }),
        ))
        .await
        .expect("update password with wrong old password");
    assert_eq!(wrong_old_pwd_res.status(), StatusCode::BAD_REQUEST);

    // Successful password change with correct old password
    let update_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users/password",
            &access,
            serde_json::json!({
                "oldPassword": "password123",
                "password": "newpassword123",
                "changePassword": "newpassword123"
            }),
        ))
        .await
        .expect("update password");
    assert_eq!(update_res.status(), StatusCode::OK);

    let refresh_res = ctx
        .app
        .clone()
        .oneshot(cookie_request("POST", "/v1/auth/refresh", &cookie))
        .await
        .expect("refresh after password change");
    assert_eq!(refresh_res.status(), StatusCode::UNAUTHORIZED);

    // Old access token must also be immediately rejected because token_version was incremented
    let old_access_res = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", &access))
        .await
        .expect("access profile with old access token");
    assert_eq!(old_access_res.status(), StatusCode::UNAUTHORIZED);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn logout_revokes_both_tokens() {
    let ctx = setup().await;
    let login = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/auth/login",
            "",
            serde_json::json!({
                "email": format!("user-{}@example.com", ctx.db.name()),
                "password": "password123"
            }),
        ))
        .await
        .unwrap();
    let cookie = refresh_cookie(&login);
    let data: serde_json::Value = serde_json::from_slice(&response_bytes(login).await).unwrap();
    let access = data["access_token"].as_str().unwrap();

    let logout = ctx
        .app
        .clone()
        .oneshot(cookie_request("POST", "/v1/auth/logout", &cookie))
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    assert!(
        logout
            .headers()
            .get(axum::http::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    let profile = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", access))
        .await
        .unwrap();
    assert_eq!(profile.status(), StatusCode::UNAUTHORIZED);
    let refresh = ctx
        .app
        .clone()
        .oneshot(cookie_request("POST", "/v1/auth/refresh", &cookie))
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::UNAUTHORIZED);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn refresh_rejects_foreign_origin_and_legacy_access_token() {
    let ctx = setup().await;
    let mut foreign = cookie_request(
        "POST",
        "/v1/auth/refresh",
        "__Host-fragrans-refresh=invalid",
    );
    foreign.headers_mut().insert(
        axum::http::header::ORIGIN,
        "https://evil.example".parse().unwrap(),
    );
    assert_eq!(
        ctx.app.clone().oneshot(foreign).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    let legacy = fragrans::api::middleware::create_token(
        "test-secret-key-that-is-long-enough",
        &ctx.user_id,
        fragrans::api::middleware::TokenPurpose::Access,
        None,
        (chrono::Utc::now().timestamp() + 3600) as usize,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        ctx.app
            .clone()
            .oneshot(auth_request("GET", "/v1/profile", &legacy))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn login_rate_limit_blocks_after_burst() {
    let ctx = setup().await;

    let payload = serde_json::json!({
        "email": format!("test-{}@example.com", uuid::Uuid::new_v4()),
        "password": "wrongpassword"
    });

    let mut hit_rate_limit = false;
    // Account limiter blocks repeated attempts even before the IP limit is reached.
    for _ in 0..12 {
        let res = ctx
            .app
            .clone()
            .oneshot(json_auth_request(
                "POST",
                "/v1/auth/login",
                "",
                payload.clone(),
            ))
            .await
            .expect("login attempt");
        if res.status() == StatusCode::TOO_MANY_REQUESTS {
            hit_rate_limit = true;
            break;
        }
    }
    assert!(hit_rate_limit, "Expected to hit 429 Too Many Requests");

    ctx.teardown().await;
}
