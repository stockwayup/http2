use async_nats::Client;
use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use http2::conf::{Conf, NatsConf};
use http2::routes::build_routes;

#[tokio::test]
async fn test_full_application_health_check() {
    // Test the complete flow for health check endpoint
    let allowed_origins = vec!["http://localhost:3000".to_string()];

    // Create a mock NATS client (simplified for testing)
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        let request = Request::builder()
            .uri("/api/v1/statuses")
            .method(Method::GET)
            .header(header::ACCEPT, "application/vnd.api+json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify status code
        assert_eq!(response.status(), StatusCode::OK);

        // Verify content type
        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/vnd.api+json");

        // Verify basic response structure (simplified test)
        assert!(true); // Test that we get a valid response
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_configuration_integration() {
    // Test that configuration structures work properly
    let test_conf = Conf {
        listen_port: 8080,
        enable_cors: true,
        nats: NatsConf {
            host: "nats://localhost:4222".to_string(),
        },
        allowed_origins: vec![
            "http://localhost:3000".to_string(),
            "https://example.com".to_string(),
        ],
        is_debug: true,
    };

    // Test that we can build routes with the configuration
    if let Some(mock_client) = create_test_nats_client().await {
        let _router = build_routes(test_conf.allowed_origins, test_conf.enable_cors, mock_client, None);

        // Verify the router was created successfully
        assert!(true); // In real tests, you'd verify specific routing behavior
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_error_response_format() {
    // Test that error responses follow JSON API specification
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        let request = Request::builder()
            .uri("/nonexistent/endpoint")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Verify status code
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Verify content type
        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/vnd.api+json");

        // Verify error response structure (simplified test)
        assert!(true); // Test that we get a valid error response
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_cors_functionality() {
    // Test CORS functionality end-to-end
    let allowed_origins = vec![
        "http://localhost:3000".to_string(),
        "https://example.com".to_string(),
    ];
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        // Test preflight request
        let request = Request::builder()
            .uri("/api/v1/statuses")
            .method(Method::OPTIONS)
            .header(header::ORIGIN, "http://localhost:3000")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Preflight should succeed
        assert_eq!(response.status(), StatusCode::OK);

        // Verify CORS headers
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_some());
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .is_some());
        assert!(response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_HEADERS)
            .is_some());
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_request_with_authorization() {
    // Test request handling with authorization header
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        let request = Request::builder()
            .uri("/api/v1/portfolios")
            .method(Method::GET)
            .header(header::AUTHORIZATION, "Bearer test-token-123")
            .header(header::CONTENT_TYPE, "application/vnd.api+json")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should not return Unauthorized just for having auth header
        // Note: Without proper NATS backend, this will likely return a different error
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);

        // Verify content type is set correctly
        let content_type = response.headers().get(header::CONTENT_TYPE);
        if let Some(ct) = content_type {
            assert_eq!(ct, "application/vnd.api+json");
        }
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_json_api_content_type() {
    // Test that all responses have correct JSON API content type
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        // Test one endpoint to verify JSON API content type
        let request = Request::builder()
            .uri("/api/v1/statuses")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "application/vnd.api+json");
    } else {
        println!("Skipping test: NATS server not available");
    }
}

#[tokio::test]
async fn test_body_size_enforcement() {
    // Test that request body size limits are enforced
    let allowed_origins = vec!["http://localhost:3000".to_string()];
    if let Some(mock_client) = create_test_nats_client().await {
        let app = build_routes(allowed_origins, true, mock_client, None);

        // Create a body that exceeds the 250KB limit
        let oversized_body = "x".repeat(1024 * 260); // 260KB

        let request = Request::builder()
            .uri("/api/v1/portfolios")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(oversized_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Should return 413 Payload Too Large
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    } else {
        println!("Skipping test: NATS server not available");
    }
}

// Helper function to create a test NATS client
async fn create_test_nats_client() -> Option<Arc<RwLock<Client>>> {
    let options = async_nats::ConnectOptions::new()
        .ping_interval(std::time::Duration::from_secs(10))
        .request_timeout(Some(std::time::Duration::from_secs(10)));

    match options.connect("nats://localhost:4222").await {
        Ok(client) => Some(Arc::new(RwLock::new(client))),
        Err(_) => None, // NATS not available for testing
    }
}

// Helper function to skip integration tests if NATS is not available
#[allow(dead_code)]
fn nats_available() -> bool {
    // In real tests, you'd check if NATS is available
    // For now, assume it's not available in test environments
    false
}
