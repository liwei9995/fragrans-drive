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
    let login_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(login_res).await).expect("parse login");
    let token = login_data["access_token"]
        .as_str()
        .expect("token exists")
        .to_string();
    assert!(!token.is_empty());

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
