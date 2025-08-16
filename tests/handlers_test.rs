use axum::http::StatusCode;
use axum::response::IntoResponse;

// Import the modules we need to test
use http2::handlers::{health_check, not_found};

#[tokio::test]
async fn test_health_check() {
    let response = health_check().await.into_response();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "application/vnd.api+json");
}

#[tokio::test]
async fn test_not_found() {
    let response = not_found().await.into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "application/vnd.api+json");
}

#[tokio::test]
async fn test_response_headers() {
    let health_response = health_check().await.into_response();
    let not_found_response = not_found().await.into_response();

    // Both responses should have correct content type
    assert_eq!(
        health_response.headers().get("content-type").unwrap(),
        "application/vnd.api+json"
    );
    assert_eq!(
        not_found_response.headers().get("content-type").unwrap(),
        "application/vnd.api+json"
    );

    // Health check should be 200, not found should be 404
    assert_eq!(health_response.status(), StatusCode::OK);
    assert_eq!(not_found_response.status(), StatusCode::NOT_FOUND);
}
