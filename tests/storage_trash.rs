use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::Utc;
use fragrans::{
    api::{self, middleware::Claims},
    config::Config,
    domain::{
        storage::{
            Storage, StorageListPaginatedResponse, TrashCleanupResponse, TrashRestoreResponse, StorageType,
        },
        user::User,
    },
    infrastructure::{db::storage_repo::StorageRepository, storage::local::LocalStorage},
    utils::{crypto::hash_password, encryption::get_iv},
};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use mongodb::{Client, Database, bson::doc, options::ClientOptions};
use serial_test::serial;
use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tempfile::TempDir;
use tower::util::ServiceExt;

mod common;
use common::*;

#[tokio::test]
#[serial]
async fn delete_restore_roundtrip_cascades_to_children_and_thumbnail() {
    let Some(ctx) = setup().await else {
        return;
    };

    let repo = StorageRepository::new(&ctx.db);
    let now = Utc::now();
    let folder_id = repo
        .create(Storage {
            id: None,
            name: "folder".to_string(),
            base_name: None,
            ext_name: None,
            mime_type: None,
            encoding: None,
            size: None,
            md5_hash: None,
            iv: None,
            parent_id: "root".to_string(),
            r#type: StorageType::Folder,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create folder");

    let thumb_hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string();
    let thumb_iv = get_iv();
    let thumb_id = repo
        .create(Storage {
            id: None,
            name: "image_thumbnail".to_string(),
            base_name: None,
            ext_name: None,
            mime_type: Some("image/jpeg".to_string()),
            encoding: None,
            size: Some(3),
            md5_hash: Some(thumb_hash.clone()),
            iv: Some(thumb_iv.clone()),
            parent_id: folder_id.to_hex(),
            r#type: StorageType::Thumbnail,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create thumbnail");

    let file_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    let file_iv = get_iv();
    let file_id = repo
        .create(Storage {
            id: None,
            name: "image.jpg".to_string(),
            base_name: Some("image".to_string()),
            ext_name: Some("jpg".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            encoding: None,
            size: Some(4),
            md5_hash: Some(file_hash.clone()),
            iv: Some(file_iv.clone()),
            parent_id: folder_id.to_hex(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: Some(thumb_id.to_hex()),
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create file");

    let storage = LocalStorage::new();
    storage
        .store(&file_hash, b"file".to_vec(), Some(&file_iv))
        .await
        .expect("store file");
    storage
        .store(&thumb_hash, b"thm".to_vec(), Some(&thumb_iv))
        .await
        .expect("store thumbnail");

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/v1/storage/{}?token={}",
                    file_id.to_hex(),
                    ctx.download_token
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("download before delete");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_bytes(response).await.as_ref(), b"file");

    let response = ctx
        .app
        .clone()
        .oneshot(auth_request(
            "DELETE",
            &format!("/v1/storage/{}", folder_id.to_hex()),
            &ctx.auth_token,
        ))
        .await
        .expect("delete folder");
    assert_eq!(response.status(), StatusCode::OK);

    for id in [folder_id, file_id, thumb_id] {
        let item = repo
            .find_by_id(id)
            .await
            .expect("load item")
            .expect("item exists");
        assert!(item.trashed, "item {} should be trashed", id.to_hex());
    }

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/v1/storage/{}?token={}",
                    file_id.to_hex(),
                    ctx.download_token
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("download after delete");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = ctx
        .app
        .clone()
        .oneshot(auth_request(
            "POST",
            &format!("/v1/storage/{}/restore", folder_id.to_hex()),
            &ctx.auth_token,
        ))
        .await
        .expect("restore folder");
    assert_eq!(response.status(), StatusCode::OK);

    for id in [folder_id, file_id, thumb_id] {
        let item = repo
            .find_by_id(id)
            .await
            .expect("load restored item")
            .expect("restored item exists");
        assert!(!item.trashed, "item {} should be restored", id.to_hex());
    }

    let response = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/v1/storage/{}?token={}",
                    thumb_id.to_hex(),
                    ctx.download_token
                ))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("thumbnail download after restore");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_bytes(response).await.as_ref(), b"thm");

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn empty_trash_deletes_docs_and_gc_only_orphaned_files() {
    let Some(ctx) = setup().await else {
        return;
    };

    let repo = StorageRepository::new(&ctx.db);
    let now = Utc::now();

    let orphan_hash = "cccccccccccccccccccccccccccccccc".to_string();
    let shared_hash = "dddddddddddddddddddddddddddddddd".to_string();
    let orphan_iv = get_iv();
    let shared_iv = get_iv();

    let trashed_one = repo
        .create(Storage {
            id: None,
            name: "old.txt".to_string(),
            base_name: Some("old".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(3),
            md5_hash: Some(orphan_hash.clone()),
            iv: Some(orphan_iv.clone()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create trashed file 1");

    let trashed_two = repo
        .create(Storage {
            id: None,
            name: "stale.txt".to_string(),
            base_name: Some("stale".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(3),
            md5_hash: Some(shared_hash.clone()),
            iv: Some(shared_iv.clone()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create trashed file 2");

    let active_file = repo
        .create(Storage {
            id: None,
            name: "live.txt".to_string(),
            base_name: Some("live".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(4),
            md5_hash: Some(shared_hash.clone()),
            iv: Some(shared_iv.clone()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: false,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create active file");

    let storage = LocalStorage::new();
    storage
        .store(&orphan_hash, b"old".to_vec(), Some(&orphan_iv))
        .await
        .expect("store orphan hash");
    storage
        .store(&shared_hash, b"live".to_vec(), Some(&shared_iv))
        .await
        .expect("store shared hash");

    let response = ctx
        .app
        .clone()
        .oneshot(auth_request("DELETE", "/v1/storage/trash", &ctx.auth_token))
        .await
        .expect("empty trash");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: TrashCleanupResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse cleanup response");
    assert_eq!(payload.deleted_docs, 2);
    assert_eq!(payload.deleted_files, 1);

    assert!(
        repo.find_by_id(trashed_one)
            .await
            .expect("query trashed one")
            .is_none()
    );
    assert!(
        repo.find_by_id(trashed_two)
            .await
            .expect("query trashed two")
            .is_none()
    );
    assert!(
        repo.find_by_id(active_file)
            .await
            .expect("query active file")
            .is_some()
    );

    assert!(!LocalStorage::new().exists(&orphan_hash).await);
    assert!(LocalStorage::new().exists(&shared_hash).await);
    assert!(ctx.storage_dir.path().exists());

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn trash_list_is_paginated_and_excludes_thumbnail_rows() {
    let Some(ctx) = setup().await else {
        return;
    };

    let repo = StorageRepository::new(&ctx.db);
    let now = Utc::now();

    let thumb_id = repo
        .create(Storage {
            id: None,
            name: "thumb".to_string(),
            base_name: None,
            ext_name: None,
            mime_type: Some("image/jpeg".to_string()),
            encoding: None,
            size: Some(3),
            md5_hash: Some("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::Thumbnail,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create thumbnail");

    for name in ["a.txt", "b.txt", "folder"] {
        let is_folder = name == "folder";
        repo.create(Storage {
            id: None,
            name: name.to_string(),
            base_name: Some(name.trim_end_matches(".txt").to_string()),
            ext_name: (!is_folder).then(|| "txt".to_string()),
            mime_type: (!is_folder).then(|| "text/plain".to_string()),
            encoding: None,
            size: (!is_folder).then_some(1),
            md5_hash: (!is_folder).then(|| match name {
                "a.txt" => "ffffffffffffffffffffffffffffffff".to_string(),
                _ => "11111111111111111111111111111111".to_string(),
            }),
            iv: (!is_folder).then(get_iv),
            parent_id: "root".to_string(),
            r#type: if is_folder { StorageType::Folder } else { StorageType::File },
            user_id: ctx.user_id.clone(),
            thumbnail: (name == "a.txt").then(|| thumb_id.to_hex()),
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create trashed item");
    }

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/list",
            &ctx.auth_token,
            serde_json::json!({
                "page": 1,
                "limit": 5,
                "query": {},
                "keyword": ".txt",
                "types": ["file"],
                "sortBy": "name",
                "sortOrder": -1
            }),
        ))
        .await
        .expect("trash list");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: StorageListPaginatedResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse trash list");
    assert_eq!(payload.total, 2);
    assert_eq!(payload.page, 1);
    assert_eq!(payload.limit, 5);
    assert_eq!(payload.pages, 1);
    assert_eq!(payload.docs.len(), 2);
    assert!(payload.docs.iter().all(|doc| doc.r#type != StorageType::Thumbnail));
    assert_eq!(payload.docs[0].name, "b.txt");
    assert_eq!(payload.docs[1].name, "a.txt");

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn trash_list_defaults_to_top_level_and_can_include_children() {
    let Some(ctx) = setup().await else {
        return;
    };

    let repo = StorageRepository::new(&ctx.db);
    let now = Utc::now();

    let folder_id = repo
        .create(Storage {
            id: None,
            name: "deleted-folder".to_string(),
            base_name: None,
            ext_name: None,
            mime_type: None,
            encoding: None,
            size: None,
            md5_hash: None,
            iv: None,
            parent_id: "root".to_string(),
            r#type: StorageType::Folder,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create folder");

    repo.create(Storage {
        id: None,
        name: "nested.txt".to_string(),
        base_name: Some("nested".to_string()),
        ext_name: Some("txt".to_string()),
        mime_type: Some("text/plain".to_string()),
        encoding: None,
        size: Some(1),
        md5_hash: Some("77777777777777777777777777777777".to_string()),
        iv: Some(get_iv()),
        parent_id: folder_id.to_hex(),
        r#type: StorageType::File,
        user_id: ctx.user_id.clone(),
        thumbnail: None,
        trashed: true,
        created_at: Some(now),
        updated_at: Some(now),
    })
    .await
    .expect("create nested file");

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/list",
            &ctx.auth_token,
            serde_json::json!({ "query": {} }),
        ))
        .await
        .expect("top level trash list");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: StorageListPaginatedResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse top level list");
    assert_eq!(payload.total, 1);
    assert_eq!(payload.docs.len(), 1);
    assert_eq!(payload.docs[0].name, "deleted-folder");

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/list",
            &ctx.auth_token,
            serde_json::json!({ "query": {}, "viewMode": "all", "sortBy": "name", "sortOrder": 1 }),
        ))
        .await
        .expect("all trash list");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: StorageListPaginatedResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse all list");
    assert_eq!(payload.total, 2);
    assert_eq!(payload.docs.len(), 2);
    assert_eq!(payload.docs[0].name, "deleted-folder");
    assert_eq!(payload.docs[1].name, "nested.txt");

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn trash_restore_supports_single_batch_and_all_modes() {
    let Some(ctx) = setup().await else {
        return;
    };

    let repo = StorageRepository::new(&ctx.db);
    let now = Utc::now();

    let single_id = repo
        .create(Storage {
            id: None,
            name: "single.txt".to_string(),
            base_name: Some("single".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(1),
            md5_hash: Some("22222222222222222222222222222222".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create single");

    let batch_a = repo
        .create(Storage {
            id: None,
            name: "batch-a.txt".to_string(),
            base_name: Some("batch-a".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(1),
            md5_hash: Some("33333333333333333333333333333333".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create batch a");
    let batch_b = repo
        .create(Storage {
            id: None,
            name: "batch-b.txt".to_string(),
            base_name: Some("batch-b".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(1),
            md5_hash: Some("44444444444444444444444444444444".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create batch b");

    let all_a = repo
        .create(Storage {
            id: None,
            name: "all-a.txt".to_string(),
            base_name: Some("all-a".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(1),
            md5_hash: Some("55555555555555555555555555555555".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create all a");
    let all_b = repo
        .create(Storage {
            id: None,
            name: "all-b.txt".to_string(),
            base_name: Some("all-b".to_string()),
            ext_name: Some("txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            encoding: None,
            size: Some(1),
            md5_hash: Some("66666666666666666666666666666666".to_string()),
            iv: Some(get_iv()),
            parent_id: "root".to_string(),
            r#type: StorageType::File,
            user_id: ctx.user_id.clone(),
            thumbnail: None,
            trashed: true,
            created_at: Some(now),
            updated_at: Some(now),
        })
        .await
        .expect("create all b");

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/restore",
            &ctx.auth_token,
            serde_json::json!({ "fileIds": [single_id.to_hex()] }),
        ))
        .await
        .expect("restore single");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: TrashRestoreResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse single restore");
    assert_eq!(payload.requested_items, 1);
    assert_eq!(payload.restored_docs, 1);
    assert!(
        !repo
            .find_by_id(single_id)
            .await
            .expect("load single")
            .expect("single exists")
            .trashed
    );

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/restore",
            &ctx.auth_token,
            serde_json::json!({ "fileIds": [batch_a.to_hex(), batch_b.to_hex()] }),
        ))
        .await
        .expect("restore batch");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: TrashRestoreResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse batch restore");
    assert_eq!(payload.requested_items, 2);
    assert_eq!(payload.restored_docs, 2);
    for id in [batch_a, batch_b] {
        assert!(
            !repo
                .find_by_id(id)
                .await
                .expect("load batch item")
                .expect("batch item exists")
                .trashed
        );
    }

    let response = ctx
        .app
        .clone()
        .oneshot(json_auth_request(
            "POST",
            "/v1/storage/trash/restore",
            &ctx.auth_token,
            serde_json::json!({ "restoreAll": true }),
        ))
        .await
        .expect("restore all");
    assert_eq!(response.status(), StatusCode::OK);
    let payload: TrashRestoreResponse =
        serde_json::from_slice(&response_bytes(response).await).expect("parse restore all");
    assert_eq!(payload.requested_items, 2);
    assert_eq!(payload.restored_docs, 2);
    for id in [all_a, all_b] {
        assert!(
            !repo
                .find_by_id(id)
                .await
                .expect("load all item")
                .expect("all item exists")
                .trashed
        );
    }

    ctx.teardown().await;
}
