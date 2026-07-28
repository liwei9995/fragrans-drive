use axum::http::StatusCode;
use common::{auth_request, json_auth_request, setup};
use serde_json::json;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn ordinary_user_cannot_list_all_users() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/users", &ctx.auth_token))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn ordinary_user_cannot_read_another_user() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(auth_request(
            "GET",
            "/v1/users/000000000000000000000000",
            &ctx.auth_token,
        ))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ordinary_user_cannot_update_another_user() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/users/profile/000000000000000000000000",
            &ctx.auth_token,
            json!({"firstName": "Hacker"}),
        ))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn ordinary_user_cannot_delete_another_user() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(auth_request(
            "DELETE",
            "/v1/users/000000000000000000000000",
            &ctx.auth_token,
        ))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn current_user_can_read_own_profile() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(auth_request("GET", "/v1/profile", &ctx.auth_token))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn current_user_can_update_own_profile() {
    let ctx = setup().await;
    let res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "PATCH",
            "/v1/profile",
            &ctx.auth_token,
            json!({"firstName": "NewName", "lastName": "NewLast"}),
        ))
        .await
        .expect("request");
    assert_eq!(res.status(), StatusCode::OK);
}
