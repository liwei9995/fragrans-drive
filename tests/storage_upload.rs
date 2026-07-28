mod common;
use common::*;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use serial_test::serial;
use tower::util::ServiceExt;

fn multipart_upload_request(
    uri: &str,
    token: &str,
    parent_id: &str,
    file_name: &str,
    file_content: &[u8],
    hash: Option<&str>,
    index: Option<i32>,
    total: Option<i32>,
    size: Option<i64>,
) -> Request<Body> {
    let boundary = "------------------------Boundary1234567890";
    let mut body = Vec::new();

    // parentId field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"parentId\"\r\n\r\n");
    body.extend_from_slice(parent_id.as_bytes());
    body.extend_from_slice(b"\r\n");

    // hash field if provided
    if let Some(h) = hash {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"hash\"\r\n\r\n");
        body.extend_from_slice(h.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // index field if provided
    if let Some(i) = index {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"index\"\r\n\r\n");
        body.extend_from_slice(i.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // total field if provided
    if let Some(t) = total {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"total\"\r\n\r\n");
        body.extend_from_slice(t.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // size field if provided
    if let Some(s) = size {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"size\"\r\n\r\n");
        body.extend_from_slice(s.to_string().as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    // file field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            file_name
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(file_content);
    body.extend_from_slice(b"\r\n");

    // End boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .expect("multipart request")
}

#[tokio::test]
#[serial]
async fn test_single_chunk_upload() {
    let ctx = setup().await;

    let file_content = b"Hello from integration test!";
    let req = multipart_upload_request(
        "/v1/storage/upload",
        &ctx.auth_token,
        "root",
        "test_file.txt",
        file_content,
        None, // hash
        None, // index
        None, // total
        None, // size
    );

    let res = ctx
        .app
        .clone()
        .oneshot(req)
        .await
        .expect("upload request");

    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<String> =
        serde_json::from_slice(&response_bytes(res).await).expect("parse upload response");
    assert_eq!(data.len(), 1);
    let file_id = &data[0];

    // Download it back to verify
    let download_res = ctx
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/v1/storage/{}?token={}", file_id, ctx.download_token))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("download request");

    assert_eq!(download_res.status(), StatusCode::OK);
    let downloaded_bytes = response_bytes(download_res).await;
    assert_eq!(downloaded_bytes.as_ref(), file_content);

    ctx.teardown().await;
}

#[tokio::test]
#[serial]
async fn test_multiple_files_upload() {
    let ctx = setup().await;

    let boundary = "------------------------Boundary1234567890";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"parentId\"\r\n\r\n");
    body.extend_from_slice(b"root\r\n");

    // File 1
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"file1.txt\"\r\n");
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(b"File 1 content");
    body.extend_from_slice(b"\r\n");

    // File 2
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"file2.txt\"\r\n");
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(b"File 2 content");
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/upload")
        .header(header::AUTHORIZATION, format!("Bearer {}", ctx.auth_token))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .expect("multipart request");

    let res = ctx
        .app
        .clone()
        .oneshot(req)
        .await
        .expect("upload request");

    assert_eq!(res.status(), StatusCode::OK);
    let data: Vec<String> =
        serde_json::from_slice(&response_bytes(res).await).expect("parse upload response");
    assert_eq!(data.len(), 2);

    ctx.teardown().await;
}
