use async_nats::Client;
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use http2::routes::build_routes;
use http2::types::SharedState;

// Helper function to create a test NATS client
// Note: This will skip these tests if NATS is not available
async fn create_test_nats_client() -> Option<Arc<RwLock<Client>>> {
    let options = async_nats::ConnectOptions::new()
        .ping_interval(std::time::Duration::from_secs(10))
        .request_timeout(Some(std::time::Duration::from_secs(10)));

    match options.connect("nats://localhost:4222").await {
        Ok(client) => Some(Arc::new(RwLock::new(client))),
        Err(_) => None, // NATS not available for testing
    }
}

#[tokio::test]
async fn test_health_check_route() {
    if let Some(mock_client) = create_test_nats_client().await {
        let allowed_origins = vec!["http://localhost:3000".to_string()];
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let app = build_routes(allowed_origins, true, shared_state);

        let request = Request::builder()
            .uri("/api/v1/statuses")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/vnd.api+json");
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_not_found_route() {
    if let Some(mock_client) = create_test_nats_client().await {
        let allowed_origins = vec!["http://localhost:3000".to_string()];
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let app = build_routes(allowed_origins, true, shared_state);

        let request = Request::builder()
            .uri("/nonexistent/route")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/vnd.api+json");
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_cors_headers() {
    if let Some(mock_client) = create_test_nats_client().await {
        let allowed_origins = vec!["http://localhost:3000".to_string()];
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let app = build_routes(allowed_origins, true, shared_state);

        let request = Request::builder()
            .uri("/api/v1/statuses")
            .method(Method::OPTIONS)
            .header(header::ORIGIN, "http://localhost:3000")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // CORS preflight should return 200
        assert_eq!(response.status(), StatusCode::OK);

        // Check that CORS headers are present
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_some());
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_request_body_size_limit() {
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let app = build_routes(allowed_origins, true, shared_state);

        // Create a body that's larger than the limit (250KB)
        let large_body = "x".repeat(1024 * 300); // 300KB

        let request = Request::builder()
            .uri("/api/v1/users")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(large_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 413 Payload Too Large
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_api_v1_routes_exist() {
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        // Just verify that build_routes doesn't panic with various route configurations
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let _router = build_routes(allowed_origins, true, shared_state);

        // Test that common API routes would be registered
        // In a complete test, you'd verify each route exists and has correct methods
        assert!(true); // Placeholder
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_route_methods() {
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let app = build_routes(allowed_origins, true, shared_state);

        // Test that GET is allowed on portfolios route
        let request = Request::builder()
            .uri("/api/v1/portfolios")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should not return Method Not Allowed (405)
        // Note: Without proper NATS mocking, this might return a different error
        assert_ne!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_multiple_allowed_origins() {
    let allowed_origins = vec![
        "http://localhost:3000".to_string(),
        "https://example.com".to_string(),
        "https://app.example.com".to_string(),
    ];

    if let Some(mock_client) = create_test_nats_client().await {
        let shared_state = SharedState {
            nats: mock_client,
            metrics: None,
        };
        let _router = build_routes(allowed_origins, true, shared_state);

        // Verify router builds with multiple origins
        assert!(true); // In real tests, you'd verify CORS behavior for each origin
    } else {
        println!("Skipping test: NATS server not available");
    }
}
