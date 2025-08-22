use axum::http::HeaderMap;
use std::net::IpAddr;
use std::str::FromStr;
use tracing::{debug, warn};

/// Extracts the real client IP address from HTTP headers
/// Considers header priority order:
/// 1. CF-Connecting-IP (Cloudflare)
/// 2. X-Forwarded-For (nginx, other proxies)
/// 3. X-Real-IP (nginx)
/// 4. X-Client-IP (other proxies)
/// 5. Fallback to empty string if cannot determine
pub fn extract_client_ip(headers: &HeaderMap) -> String {
    // List of headers in priority order
    let header_names = [
        "cf-connecting-ip",    // Cloudflare - most reliable
        "x-forwarded-for",     // Standard proxy header
        "x-real-ip",          // nginx
        "x-client-ip",        // Other proxies
    ];

    for header_name in &header_names {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(header_str) = header_value.to_str() {
                debug!(header = header_name, value = header_str, "checking IP header");
                
                // X-Forwarded-For can contain a comma-separated list of IPs
                // Iterate through all IPs and take the first public one
                if *header_name == "x-forwarded-for" {
                    for ip_str in header_str.split(',') {
                        let cleaned_ip = ip_str.trim();
                        if let Some(valid_ip) = validate_and_clean_ip(cleaned_ip) {
                            debug!(extracted_ip = valid_ip, source = header_name, "extracted client IP from list");
                            return valid_ip;
                        }
                    }
                } else {
                    // For other headers, take the value as is
                    let cleaned_ip = header_str.trim();
                    if let Some(valid_ip) = validate_and_clean_ip(cleaned_ip) {
                        debug!(extracted_ip = valid_ip, source = header_name, "extracted client IP");
                        return valid_ip;
                    }
                }
            }
        }
    }

    warn!("no valid client IP found in headers, using empty string");
    String::new()
}

/// Validates and cleans IP address
/// Returns Some(ip) if IP is valid, otherwise None
fn validate_and_clean_ip(ip_str: &str) -> Option<String> {
    let cleaned = ip_str.trim();
    
    // Check that this is not a private proxy IP
    if is_internal_ip(cleaned) {
        debug!(ip = cleaned, "skipping internal/private IP");
        return None;
    }

    // Validate that this is a proper IP address
    match IpAddr::from_str(cleaned) {
        Ok(ip_addr) => {
            // Additional check for loopback and other special addresses
            match ip_addr {
                IpAddr::V4(ipv4) => {
                    if ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local() {
                        debug!(ip = cleaned, "skipping special IPv4 address");
                        None
                    } else {
                        Some(cleaned.to_string())
                    }
                },
                IpAddr::V6(ipv6) => {
                    if ipv6.is_loopback() || ipv6.is_unspecified() {
                        debug!(ip = cleaned, "skipping special IPv6 address");
                        None
                    } else {
                        Some(cleaned.to_string())
                    }
                }
            }
        },
        Err(e) => {
            debug!(ip = cleaned, error = %e, "invalid IP address format");
            None
        }
    }
}

/// Checks if IP is an internal/private address
/// Excludes addresses that might be proxies/load balancers
fn is_internal_ip(ip_str: &str) -> bool {
    // List of known internal subnets and proxy addresses
    let internal_patterns = [
        "127.", "10.", "172.", "192.168.", "169.254.",
        "::1", "fc00:", "fd00:", "fe80:",
        "localhost"
    ];
    
    for pattern in &internal_patterns {
        if ip_str.starts_with(pattern) {
            return true;
        }
    }
    
    // Check for range 172.16.0.0 - 172.31.255.255
    if ip_str.starts_with("172.") {
        if let Some(second_octet) = ip_str.split('.').nth(1) {
            if let Ok(octet) = second_octet.parse::<u8>() {
                if (16..=31).contains(&octet) {
                    return true;
                }
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_extract_cloudflare_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", HeaderValue::from_static("203.0.113.1"));
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1, 203.0.113.1"));

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "203.0.113.1"); // Cloudflare has priority
    }

    #[test]
    fn test_extract_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.1, 10.0.0.1, 172.16.0.1"));

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "203.0.113.1"); // First public IP
    }

    #[test]
    fn test_extract_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.1"));

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "203.0.113.1");
    }

    #[test]
    fn test_no_valid_ip_headers() {
        let headers = HeaderMap::new();
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "");
    }

    #[test]
    fn test_only_private_ips() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));
        headers.insert("x-real-ip", HeaderValue::from_static("172.16.0.1"));

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, ""); // All IPs are private
    }

    #[test]
    fn test_validate_and_clean_ip() {
        assert_eq!(validate_and_clean_ip("203.0.113.1"), Some("203.0.113.1".to_string()));
        assert_eq!(validate_and_clean_ip("  203.0.113.1  "), Some("203.0.113.1".to_string()));
        assert_eq!(validate_and_clean_ip("192.168.1.1"), None); // Private
        assert_eq!(validate_and_clean_ip("invalid-ip"), None);
        assert_eq!(validate_and_clean_ip("127.0.0.1"), None); // Loopback
    }

    #[test]
    fn test_is_internal_ip() {
        assert!(is_internal_ip("192.168.1.1"));
        assert!(is_internal_ip("10.0.0.1"));
        assert!(is_internal_ip("172.16.0.1"));
        assert!(is_internal_ip("127.0.0.1"));
        assert!(is_internal_ip("localhost"));
        
        assert!(!is_internal_ip("203.0.113.1"));
        assert!(!is_internal_ip("8.8.8.8"));
        assert!(!is_internal_ip("1.1.1.1"));
    }

    #[test]
    fn test_ipv6_support() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", HeaderValue::from_static("2001:db8::1"));
        
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "2001:db8::1");
    }

    #[test]
    fn test_header_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.2"));
        headers.insert("cf-connecting-ip", HeaderValue::from_static("203.0.113.1"));
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.3, 10.0.0.1"));

        let ip = extract_client_ip(&headers);
        assert_eq!(ip, "203.0.113.1"); // CF has highest priority
    }
}