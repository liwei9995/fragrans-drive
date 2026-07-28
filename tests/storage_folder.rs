mod common;
use common::*;

use axum::http::StatusCode;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn test_folder_creation_and_listing() {
    let ctx = setup().await;

    // 1. Create a folder in root
    let folder_payload = serde_json::json!({
        "name": "Docs",
        "parentId": "root",
        "type": "folder"
    });

    let create_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            folder_payload,
        ))
        .await
        .expect("create folder");

    assert_eq!(create_res.status(), StatusCode::OK);
    let create_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(create_res).await).expect("parse create folder");
    let root_folder_id = create_data["id"].as_str().unwrap().to_string();
    assert_eq!(create_data["name"].as_str().unwrap(), "Docs");
    assert_eq!(create_data["parentId"].as_str().unwrap(), "root");

    // 2. Create nested folder
    let nested_payload = serde_json::json!({
        "name": "Work",
        "parentId": root_folder_id,
        "type": "folder"
    });

    let nested_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            nested_payload,
        ))
        .await
        .expect("create nested folder");

    assert_eq!(nested_res.status(), StatusCode::OK);
    let nested_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(nested_res).await).expect("parse nested folder");
    let nested_folder_id = nested_data["id"].as_str().unwrap().to_string();
    assert_eq!(nested_data["name"].as_str().unwrap(), "Work");
    assert_eq!(nested_data["parentId"].as_str().unwrap(), root_folder_id);

    // 3. List folders in root
    let list_payload = serde_json::json!({
        "query": { "parentId": "root" }
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .expect("list folders");

    assert_eq!(list_res.status(), StatusCode::OK);
    let list_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(list_res).await).expect("parse list response");
    assert_eq!(list_data["total"].as_u64().unwrap(), 1);
    assert_eq!(list_data["docs"][0]["id"].as_str().unwrap(), root_folder_id);
    assert_eq!(list_data["docs"][0]["type"].as_str().unwrap(), "folder");

    // 4. Rename folder
    let rename_payload = serde_json::json!({
        "name": "Documents"
    });
    let rename_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "PUT",
            &format!("/v1/storage/{}", root_folder_id),
            &ctx.auth_token,
            rename_payload,
        ))
        .await
        .expect("rename folder");

    assert_eq!(rename_res.status(), StatusCode::OK);

    // 5. Get Path to nested folder
    let path_payload = serde_json::json!({
        "id": nested_folder_id
    });
    let path_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/path",
            &ctx.auth_token,
            path_payload,
        ))
        .await
        .expect("get path");

    assert_eq!(path_res.status(), StatusCode::OK);
    let path_data: serde_json::Value =
        serde_json::from_slice(&response_bytes(path_res).await).expect("parse path");
    // path should be [root_folder, nested_folder]
    let path_arr = path_data.as_array().unwrap();
    assert_eq!(path_arr.len(), 2);
    assert_eq!(path_arr[0]["id"].as_str().unwrap(), root_folder_id);
    assert_eq!(path_arr[0]["name"].as_str().unwrap(), "Documents"); // it should be renamed
    assert_eq!(path_arr[1]["id"].as_str().unwrap(), nested_folder_id);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn types_file_returns_only_files() {
    let ctx = setup().await;
    let folder_payload = serde_json::json!({ "name": "F1", "parentId": "root", "type": "folder" });
    let _ = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            folder_payload,
        ))
        .await;

    let repo = fragrans::infrastructure::db::storage_repo::StorageRepository::new(&ctx.db);
    repo.create(fragrans::domain::storage::Storage {
        id: None,
        name: "test.txt".to_string(),
        base_name: Some("test".to_string()),
        ext_name: Some("txt".to_string()),
        mime_type: Some("text/plain".to_string()),
        encoding: None,
        size: Some(1),
        md5_hash: Some("abc".to_string()),
        content_hash: None,
        hash_algorithm: None,
        encryption_format: None,
        iv: Some("000000000000000000000000".to_string()),
        parent_id: "root".to_string(),
        r#type: fragrans::domain::storage::StorageType::File,
        user_id: ctx.user_id.clone(),
        thumbnail: None,
        trashed: false,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    })
    .await
    .unwrap();

    let list_payload = serde_json::json!({
        "types": ["file"]
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&response_bytes(list_res).await).unwrap();
    assert!(data["total"].as_u64().unwrap() > 0);
    for doc in data["docs"].as_array().unwrap() {
        assert_eq!(doc["type"].as_str().unwrap(), "file");
    }
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn types_folder_returns_only_folders() {
    let ctx = setup().await;
    let folder_payload = serde_json::json!({ "name": "F1", "parentId": "root", "type": "folder" });
    let _ = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            folder_payload,
        ))
        .await;

    let repo = fragrans::infrastructure::db::storage_repo::StorageRepository::new(&ctx.db);
    repo.create(fragrans::domain::storage::Storage {
        id: None,
        name: "test.txt".to_string(),
        base_name: Some("test".to_string()),
        ext_name: Some("txt".to_string()),
        mime_type: Some("text/plain".to_string()),
        encoding: None,
        size: Some(1),
        md5_hash: Some("abc".to_string()),
        content_hash: None,
        hash_algorithm: None,
        encryption_format: None,
        iv: Some("000000000000000000000000".to_string()),
        parent_id: "root".to_string(),
        r#type: fragrans::domain::storage::StorageType::File,
        user_id: ctx.user_id.clone(),
        thumbnail: None,
        trashed: false,
        created_at: Some(chrono::Utc::now()),
        updated_at: Some(chrono::Utc::now()),
    })
    .await
    .unwrap();

    let list_payload = serde_json::json!({
        "types": ["folder"]
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&response_bytes(list_res).await).unwrap();
    assert!(data["total"].as_u64().unwrap() > 0);
    for doc in data["docs"].as_array().unwrap() {
        assert_eq!(doc["type"].as_str().unwrap(), "folder");
    }
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn keyword_is_literal_not_regex() {
    let ctx = setup().await;
    let folder_payload =
        serde_json::json!({ "name": "test.*", "parentId": "root", "type": "folder" });
    let _ = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            folder_payload,
        ))
        .await;

    let folder_payload =
        serde_json::json!({ "name": "test12", "parentId": "root", "type": "folder" });
    let _ = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/folder",
            &ctx.auth_token,
            folder_payload,
        ))
        .await;

    let list_payload = serde_json::json!({
        "keyword": "test.*"
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&response_bytes(list_res).await).unwrap();
    assert_eq!(data["total"].as_u64().unwrap(), 1);
    assert_eq!(data["docs"][0]["name"].as_str().unwrap(), "test.*");
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn long_unicode_keyword_does_not_panic() {
    let ctx = setup().await;
    let list_payload = serde_json::json!({
        "keyword": "香".repeat(101)
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn unknown_type_returns_empty_or_400_consistently() {
    let ctx = setup().await;
    let list_payload = serde_json::json!({
        "types": ["magic"]
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::BAD_REQUEST);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn sort_uses_id_as_stable_tiebreaker() {
    let ctx = setup().await;
    for _i in 0..5 {
        let folder_payload =
            serde_json::json!({ "name": "SameName", "parentId": "root", "type": "folder" });
        let _ = ctx
            .app
            .clone()
            .oneshot(json_auth_request(
                "POST",
                "/v1/storage/folder",
                &ctx.auth_token,
                folder_payload,
            ))
            .await;
    }
    let list_payload = serde_json::json!({
        "sortBy": "name",
        "sortOrder": 1
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload.clone(),
        ))
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_slice(&response_bytes(list_res).await).unwrap();

    // Fetch again and ensure order is identical
    let list_res2 = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    let data2: serde_json::Value =
        serde_json::from_slice(&response_bytes(list_res2).await).unwrap();

    assert_eq!(data, data2);
    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn raw_mongo_operators_are_rejected_by_deserialization() {
    let ctx = setup().await;
    let list_payload = serde_json::json!({
        "query": { "parentId": { "$gt": "" } }
    });
    let list_res = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/list",
            &ctx.auth_token,
            list_payload,
        ))
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    ctx.teardown().await;
}
