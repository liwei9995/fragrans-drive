use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use common::setup;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn upload_exceeds_limit_returns_413() {
    let ctx = setup().await;

    // We can simulate a large payload by constructing a multipart body that exceeds the allowed size.
    // However, it's easier to just pass a very large chunk if we write a custom stream,
    // or just let the API reject it by setting a small limit for the test, but the app limit is probably fixed.
    // The instructions say: Return 413 Payload Too Large for files exceeding limits.
    // Let's create a body with size > 1GB, or just mock it.

    // Actually, sending 1GB in a test is slow.
    // But let's assume the limit is 100MB or something we can configure, or we just hardcode
    // a rejection if size > 1_000_000_000 (1GB).
    // Let's just create a large string for 1MB and see if we can trigger a limit if we assume the limit is small?
    // Wait, the limit might be set to 1GB. We can just test that large files return 413.
    // The easiest way is to mock a stream or set the limit explicitly.
    // For now, I'll write a test that fails if we don't return 413. We'll send a file that is "too large"
    // based on whatever we choose as the limit. Let's pick 1GB as limit.
    // Writing 1GB in a test is too slow. How about we just test if the limit enforcement logic exists?

    // Let's just send 100 bytes and the server can accept it, but we can't easily test 413 without sending huge data.
    // Let's configure the app limit or check the handler logic manually.
    // I will write a test that sends 11MB if the limit is 10MB.

    let boundary = "------------------------14737809831466499882746641449";
    let body_data = format!(
        "--{boundary}\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"large.txt\"\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        {}\r\n\
        --{boundary}--\r\n",
        "A".repeat(11 * 1024 * 1024)
    );

    let req = Request::builder()
        .method("POST")
        .uri("/v1/storage/upload")
        .header("Authorization", format!("Bearer {}", ctx.auth_token))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .header("Content-Length", body_data.len())
        .body(Body::from(body_data))
        .unwrap();

    let res = ctx.app.oneshot(req).await.unwrap();
    let status = res.status();
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    println!("Response: {} {:?}", status, body_bytes);
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}
