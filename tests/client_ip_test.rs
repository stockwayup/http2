use axum::http::{HeaderMap, HeaderValue};
use http2::client_ip::extract_client_ip;

/// Integration test for client IP extraction from various headers
#[tokio::test]
async fn test_client_ip_extraction_integration() {
    // Test 1: Cloudflare CF-Connecting-IP (highest priority)
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", HeaderValue::from_static("203.0.113.1"));
    headers.insert("x-forwarded-for", HeaderValue::from_static("10.0.0.1, 203.0.113.2"));
    headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.3"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.1", "Should prioritize CF-Connecting-IP");

    // Test 2: X-Forwarded-For when no CF header
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.4, 192.168.1.1, 10.0.0.1"));
    headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.5"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.4", "Should use first public IP from X-Forwarded-For");

    // Test 3: X-Real-IP when no CF and XFF headers
    let mut headers = HeaderMap::new();
    headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.6"));
    headers.insert("x-client-ip", HeaderValue::from_static("203.0.113.7"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.6", "Should use X-Real-IP when higher priority headers absent");

    // Test 4: Only private IP addresses
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));
    headers.insert("x-real-ip", HeaderValue::from_static("172.16.0.1"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "", "Should return empty string when only private IPs available");

    // Test 5: IPv6 addresses
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", HeaderValue::from_static("2001:db8::1"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "2001:db8::1", "Should handle IPv6 addresses correctly");

    // Test 6: No IP headers at all
    let headers = HeaderMap::new();
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "", "Should return empty string when no IP headers present");
}

/// Test real-world scenarios with typical configurations
#[tokio::test]
async fn test_real_world_scenarios() {
    // Scenario 1: Cloudflare + nginx ingress
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", HeaderValue::from_static("198.51.100.1")); // Real client
    headers.insert("x-forwarded-for", HeaderValue::from_static("198.51.100.1, 10.244.0.5, 192.168.1.1")); // client, pod, nginx
    headers.insert("x-real-ip", HeaderValue::from_static("10.244.0.5")); // kubernetes pod IP
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "198.51.100.1", "Cloudflare scenario should extract real client IP");

    // Scenario 2: Only nginx (without Cloudflare)  
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.195, 10.0.0.10")); // client, internal LB
    headers.insert("x-real-ip", HeaderValue::from_static("10.0.0.10")); // internal load balancer
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.195", "nginx scenario should extract first public IP");

    // Scenario 3: Proxy chain with private addresses at the beginning (attack)
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.100, 203.0.113.50, 10.0.0.1"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.50", "Should skip private IPs and find first public IP");

    // Scenario 4: Mobile carriers (often use private IPs)
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("10.128.0.5, 203.0.113.89"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.89", "Should handle mobile carrier scenarios");
}

/// Security test - protection against IP spoofing
#[tokio::test]
async fn test_security_scenarios() {
    // Attempt to spoof Cloudflare header
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", HeaderValue::from_static("127.0.0.1")); // Spoofing attempt
    headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.100"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.100", "Should reject loopback IPs even from trusted headers");

    // Attempt to inject invalid IPs
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", HeaderValue::from_static("invalid-ip, 203.0.113.101"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "203.0.113.101", "Should skip invalid IPs and find valid ones");

    // Only internal addresses (possible attack)
    let mut headers = HeaderMap::new();
    headers.insert("cf-connecting-ip", HeaderValue::from_static("192.168.1.1"));
    headers.insert("x-forwarded-for", HeaderValue::from_static("10.0.0.1, 172.16.0.1"));
    headers.insert("x-real-ip", HeaderValue::from_static("192.168.0.1"));
    
    let ip = extract_client_ip(&headers);
    assert_eq!(ip, "", "Should return empty when all IPs are private/internal");
}