//! A robust library for waiting until TCP ports, HTTP endpoints, and services become available.
//!
//! This library provides functionality for testing network connectivity and service availability,
//! with support for DNS resolution, exponential backoff, and multiple connection strategies.
//! Perfect for Docker, Kubernetes, CI/CD pipelines, and microservices orchestration.
//!
//! # Features
//!
//! - **Type Safety**: `NewType` wrappers for ports and hostnames with validation
//! - **Multiple Protocols**: TCP socket connections and HTTP/HTTPS requests
//! - **Flexible Configuration**: Timeouts, retry limits, exponential backoff
//! - **Concurrency Strategies**: Wait for all targets or any target
//! - **Graceful Cancellation**: Cancellation token support for clean shutdown
//! - **Rich Error Context**: Detailed error information with contextual messages
//! - **High Performance**: Optimized for minimal allocations and fast execution
//! - **Comprehensive Testing**: Property-based and parameterized test coverage
//!
//! # Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! waitup = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! # Examples
//!
//! ## Basic TCP Connection Check
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::tcp("localhost", 8080)?;
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .interval(Duration::from_secs(1))
//!         .build();
//!
//!     wait_for_connection(&[target], &config).await?;
//!     println!("Service is ready!");
//!     Ok(())
//! }
//! ```
//!
//! ## HTTP Health Check with Custom Headers
//!
//! ```rust,no_run
//! use waitup::Target;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::http_builder(Url::parse("https://api.example.com/health")?)
//!         .status(200)
//!         .auth_bearer("your-api-token")
//!         .content_type("application/json")
//!         .build()?;
//!
//!     let config = waitup::WaitConfig::builder()
//!         .timeout(std::time::Duration::from_secs(60))
//!         .build();
//!
//!     waitup::wait_for_connection(&[target], &config).await?;
//!     println!("API is healthy!");
//!     Ok(())
//! }
//! ```
//!
//! ## Multiple Services with Different Strategies
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let targets = vec![
//!         Target::tcp("database", 5432)?,
//!         Target::tcp("cache", 6379)?,
//!         Target::http_url("https://api.example.com/health", 200)?,
//!     ];
//!
//!     // Wait for ALL services to be ready
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(120))
//!         .wait_for_any(false)
//!         .max_retries(Some(20))
//!         .build();
//!
//!     wait_for_connection(&targets, &config).await?;
//!     println!("All services are ready!");
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Configuration with Cancellation
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let target = Target::tcp("slow-service", 8080)?;
//!
//!     let (builder, cancel_token) = WaitConfig::builder()
//!         .timeout(Duration::from_secs(60))
//!         .interval(Duration::from_millis(500))
//!         .with_cancellation();
//!
//!     let config = builder.build();
//!
//!     // Cancel after 10 seconds
//!     let cancel_handle = {
//!         let token = cancel_token.clone();
//!         tokio::spawn(async move {
//!             sleep(Duration::from_secs(10)).await;
//!             token.cancel();
//!         })
//!     };
//!
//!     match wait_for_connection(&[target], &config).await {
//!         Ok(_) => println!("Service is ready!"),
//!         Err(waitup::WaitForError::Cancelled) => println!("Operation was cancelled"),
//!         Err(e) => println!("Error: {}", e),
//!     }
//!
//!     cancel_handle.abort(); // Clean up the cancel task
//!     Ok(())
//! }
//! ```
//!
//! ## Docker Compose Integration
//!
//! ```rust,no_run
//! use waitup::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! /// Wait for services defined in docker-compose.yml
//! #[tokio::main]
//! async fn main() -> Result<(), waitup::WaitForError> {
//!     let services = vec![
//!         Target::tcp("database", 5432)?,     // Database server
//!         Target::tcp("cache", 6379)?,        // Cache server
//!         Target::tcp("search", 9200)?,       // Search server
//!         Target::http_url("http://web:8000/health", 200)?, // Web app
//!     ];
//!
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(300))  // 5 minutes for Docker startup
//!         .interval(Duration::from_secs(2))   // Check every 2 seconds
//!         .max_interval(Duration::from_secs(10)) // Max 10 seconds between retries
//!         .connection_timeout(Duration::from_secs(5)) // 5 second connection timeout
//!         .wait_for_any(false)               // Wait for ALL services
//!         .build();
//!
//!     println!("Waiting for services to be ready...");
//!     wait_for_connection(&services, &config).await?;
//!     println!("All services are ready! Starting application...");
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod config;
pub mod connection;
pub mod error;
pub mod macros;
pub mod target;
pub mod types;
pub(crate) mod utils;

// Re-export commonly used types for convenient public API
pub use config::WaitConfigBuilder;
pub use connection::{wait_for_connection, wait_for_single_target};
pub use error::{Result, ResultExt, WaitForError};
pub use target::HttpTargetBuilder;
pub use types::{
    ConnectionError, Hostname, HttpError, Port, Target, TargetKind, TargetResult, WaitConfig,
    WaitResult,
};

// Re-export error_messages for internal use
pub(crate) use error::error_messages;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test code where panics are acceptable"
)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::time::Duration;
    use test_case::test_case;
    use url::Url;

    #[test]
    fn test_target_parse_tcp() {
        let target = Target::parse("localhost:8080", 200).unwrap();
        match target {
            Target::Tcp { host, port } => {
                assert_eq!(host.as_str(), "localhost");
                assert_eq!(port.get(), 8080);
            }
            _ => panic!("Expected TCP target"),
        }
    }

    #[test]
    fn test_target_parse_http() {
        let target = Target::parse("https://example.com/health", 200).unwrap();
        match target {
            Target::Http {
                url,
                expected_status,
                ..
            } => {
                assert_eq!(url.to_string(), "https://example.com/health");
                assert_eq!(expected_status, 200);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_target_display() {
        let tcp_target = Target::tcp("localhost", 8080).unwrap();
        assert_eq!(tcp_target.to_string(), "localhost:8080");

        let url = Url::parse("https://example.com/health").unwrap();
        let http_target = Target::http(url, 200).unwrap();
        assert_eq!(http_target.to_string(), "https://example.com/health");
    }

    #[test]
    fn test_wait_config_builder() {
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(60))
            .interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(30))
            .wait_for_any(true)
            .max_retries(Some(10))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(30));
        assert!(config.wait_for_any);
        assert_eq!(config.max_retries, Some(10));
    }

    #[test]
    fn test_calculate_next_interval() {
        use connection::calculate_next_interval;

        let current = Duration::from_secs(1);
        let max = Duration::from_secs(30);

        let next = calculate_next_interval(current, max);
        assert_eq!(next, Duration::from_millis(1500));

        let large_current = Duration::from_secs(25);
        let next = calculate_next_interval(large_current, max);
        assert_eq!(next, max);
    }

    // Property-based tests for Port validation
    proptest! {
        #[test]
        fn test_port_new_valid_range(port in 1u16..=65535) {
            let result = Port::new(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_new_zero_invalid(port in 0u16..=0) {
            let result = Port::new(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_hostname_validation_alphanumeric(
            hostname in "[a-zA-Z0-9][a-zA-Z0-9\\-]{0,61}[a-zA-Z0-9]"
        ) {
            let result = Hostname::new(hostname);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hostname_validation_too_long(
            hostname in "[a-zA-Z]{254,300}"
        ) {
            let result = Hostname::new(hostname);
            assert!(result.is_err());
        }

        #[test]
        fn test_target_tcp_creation(
            hostname in "[a-zA-Z0-9][a-zA-Z0-9\\-]{0,30}[a-zA-Z0-9]",
            port in 1u16..=65535
        ) {
            let result = Target::tcp(hostname, port);
            assert!(result.is_ok());
        }

        #[test]
        fn test_calculate_next_interval_property(
            current_ms in 1u64..=60000,
            max_ms in 60_000u64..=300_000
        ) {
            let current = Duration::from_millis(current_ms);
            let max = Duration::from_millis(max_ms);

            let next = connection::calculate_next_interval(current, max);

            // Next interval should be greater than current (due to exponential backoff)
            assert!(next >= current);
            // Next interval should not exceed max
            assert!(next <= max);
        }
    }

    // Parameterized tests using test-case
    #[test_case("localhost", 80; "http port")]
    #[test_case("example.com", 443; "https port")]
    #[test_case("127.0.0.1", 22; "ssh port")]
    #[test_case("db.example.com", 5432; "postgres port")]
    fn test_tcp_target_creation(hostname: &str, port: u16) {
        let target = Target::tcp(hostname, port).unwrap();
        match target {
            Target::Tcp { host, port: p } => {
                assert_eq!(host.as_str(), hostname);
                assert_eq!(p.get(), port);
            }
            _ => panic!("Expected TCP target"),
        }
    }

    #[test_case("http://example.com/", 200; "http url")]
    #[test_case("https://api.example.com/health", 200; "https health endpoint")]
    #[test_case("https://example.com:8080/status", 204; "custom port and status")]
    fn test_http_target_parsing(url_str: &str, status: u16) {
        let target = Target::parse(url_str, status).unwrap();
        match target {
            Target::Http {
                url,
                expected_status,
                ..
            } => {
                assert_eq!(url.to_string(), url_str);
                assert_eq!(expected_status, status);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test_case(""; "empty string")]
    #[test_case("invalid-target"; "missing port")]
    #[test_case("host:"; "empty port")]
    #[test_case("host:abc"; "non-numeric port")]
    #[test_case("host:0"; "zero port")]
    #[test_case("host:65536"; "port too high")]
    fn test_invalid_target_parsing(target_str: &str) {
        let result = Target::parse(target_str, 200);
        assert!(result.is_err());
    }

    #[test_case(""; "empty hostname")]
    #[test_case("-example.com"; "starts with hyphen")]
    #[test_case("example.com-"; "ends with hyphen")]
    #[test_case("ex..ample.com"; "empty label")]
    #[test_case(&"a".repeat(254); "too long")]
    fn test_invalid_hostname_validation(hostname: &str) {
        let result = Hostname::new(hostname);
        assert!(result.is_err());
    }

    #[test_case("192.168.1.1"; "valid ipv4")]
    #[test_case("10.0.0.1"; "valid private ip")]
    #[test_case("255.255.255.255"; "max ipv4")]
    fn test_valid_ipv4_hostname(ip: &str) {
        let result = Hostname::ipv4(ip);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), ip);
    }

    #[test_case("192.168.1"; "incomplete ipv4")]
    #[test_case("192.168.1.1.1"; "too many parts")]
    #[test_case("192.168.256.1"; "octet too high")]
    #[test_case("192.168.abc.1"; "invalid octet")]
    fn test_invalid_ipv4_hostname(ip: &str) {
        let result = Hostname::ipv4(ip);
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_const_constructors() {
        let localhost = Hostname::localhost();
        assert_eq!(localhost.as_str(), "localhost");

        let loopback = Hostname::loopback();
        assert_eq!(loopback.as_str(), "127.0.0.1");

        let loopback_v6 = Hostname::loopback_v6();
        assert_eq!(loopback_v6.as_str(), "::1");

        let any = Hostname::any();
        assert_eq!(any.as_str(), "0.0.0.0");
    }

    #[test]
    fn test_target_convenience_constructors() {
        let localhost_target = Target::localhost(8080).unwrap();
        assert_eq!(localhost_target.hostname(), "localhost");
        assert_eq!(localhost_target.port(), Some(8080));

        let loopback_target = Target::loopback(3000).unwrap();
        assert_eq!(loopback_target.hostname(), "127.0.0.1");
        assert_eq!(loopback_target.port(), Some(3000));

        let loopback_v6_target = Target::loopback_v6(9090).unwrap();
        assert_eq!(loopback_v6_target.hostname(), "::1");
        assert_eq!(loopback_v6_target.port(), Some(9090));
    }

    #[test]
    fn safe_tcp_targets_macro() {
        // Test the new safe tcp_targets! macro
        let result = tcp_targets![
            "localhost" => 8080,
            "example.com" => 443,
        ];

        assert!(result.is_ok());
        let targets = result.unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].hostname(), "localhost");
        assert_eq!(targets[0].port(), Some(8080));
    }

    #[test]
    fn safe_tcp_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = tcp_targets![
            "localhost" => 8080,
            "example.com" => 0, // Invalid port
        ];

        assert!(result.is_err());
    }

    #[test]
    fn safe_http_targets_macro() {
        // Test the new safe http_targets! macro
        let result = http_targets![
            "https://example.com" => 200,
            "http://localhost:8080" => 204,
        ];

        assert!(result.is_ok());
        let targets = result.unwrap();
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn safe_http_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = http_targets![
            "https://example.com" => 200,
            "invalid-url" => 200, // Invalid URL
        ];

        result.unwrap_err();
    }

    // Test Target pattern matching with #[non_exhaustive]
    #[test]
    fn test_target_pattern_matching_tcp() {
        let tcp_target = Target::tcp("localhost", 8080).unwrap();

        let result = match tcp_target {
            Target::Tcp { host, port } => {
                format!("TCP target: {}:{}", host.as_str(), port.get())
            }
            Target::Http { .. } => String::from("HTTP target"),
            #[allow(unreachable_patterns)]
            _ => String::from("Unknown target type"),
        };

        assert_eq!(result, "TCP target: localhost:8080");
    }

    #[test]
    fn test_target_pattern_matching_http() {
        let http_target = Target::http_url("https://example.com/health", 200).unwrap();

        let result = match http_target {
            Target::Tcp { .. } => String::from("TCP target"),
            Target::Http {
                url,
                expected_status,
                ..
            } => {
                format!("HTTP target: {url} (expecting {expected_status})")
            }
            #[allow(unreachable_patterns)]
            _ => String::from("Unknown target type"),
        };

        assert_eq!(
            result,
            "HTTP target: https://example.com/health (expecting 200)"
        );
    }

    // Test error pattern matching with #[non_exhaustive]
    #[test]
    fn test_error_pattern_matching() {
        let invalid_port_error = WaitForError::InvalidPort(0);

        let message = match invalid_port_error {
            WaitForError::InvalidPort(port) => format!("Invalid port: {port}"),
            WaitForError::InvalidTarget(msg) => format!("Invalid target: {msg}"),
            WaitForError::Timeout { targets } => format!("Timeout: {targets}"),
            WaitForError::Cancelled => String::from("Cancelled"),
            #[allow(unreachable_patterns)]
            _ => String::from("Other error"),
        };

        assert_eq!(message, "Invalid port: 0");
    }

    // ========== Hostname Validation Tests ==========

    #[test]
    fn test_hostname_empty() {
        let result = Hostname::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_too_long() {
        // RFC 1035: Max 253 characters
        let long_hostname = "a".repeat(254);
        let result = Hostname::new(long_hostname);
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_max_length() {
        // Exactly 253 characters should work with valid labels
        // RFC 1035: max 253 chars total, max 63 per label
        // Build: 63 + dot + 63 + dot + 63 + dot + 61 = 253
        let label63 = "a".repeat(63);
        let label61 = "a".repeat(61);
        let max_hostname = format!("{label63}.{label63}.{label63}.{label61}");
        assert_eq!(max_hostname.len(), 253);
        let result = Hostname::new(max_hostname);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hostname_label_too_long() {
        // RFC 1035: Max 63 characters per label
        let label = "a".repeat(64);
        let hostname = format!("{label}.com");
        let result = Hostname::new(hostname);
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_label_max_length() {
        // Exactly 63 characters per label should work
        let label = "a".repeat(63);
        let hostname = format!("{label}.com");
        let result = Hostname::new(hostname);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hostname_starts_with_hyphen() {
        let result = Hostname::new("-example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_ends_with_hyphen() {
        let result = Hostname::new("example.com-");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_label_starts_with_hyphen() {
        let result = Hostname::new("example.-invalid.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_label_ends_with_hyphen() {
        let result = Hostname::new("example.invalid-.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_empty_label() {
        let result = Hostname::new("example..com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_invalid_chars() {
        let result = Hostname::new("example!.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_with_underscore() {
        // Underscores are commonly used but technically invalid in hostnames
        let result = Hostname::new("test_host.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_valid_hyphen() {
        // Hyphens in the middle are valid
        let result = Hostname::new("my-host.example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hostname_ipv4_valid() {
        let result = Hostname::ipv4("192.168.1.1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_hostname_ipv4_invalid() {
        let result = Hostname::ipv4("256.1.1.1");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_ipv4_invalid_format() {
        let result = Hostname::ipv4("192.168.1");
        assert!(result.is_err());
    }

    #[test]
    fn test_hostname_ipv4_too_many_octets() {
        let result = Hostname::ipv4("192.168.1.1.1");
        assert!(result.is_err());
    }

    // ========== Port Validation Tests ==========

    #[test]
    fn test_port_zero_invalid() {
        let result = Port::try_from(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_port_one_valid() {
        let result = Port::try_from(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), 1);
    }

    #[test]
    fn test_port_max_valid() {
        let result = Port::try_from(65535);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), 65535);
    }

    #[test]
    fn test_port_from_str_valid() {
        let result = "8080".parse::<Port>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), 8080);
    }

    #[test]
    fn test_port_from_str_invalid() {
        let result = "0".parse::<Port>();
        assert!(result.is_err());
    }

    #[test]
    fn test_port_from_str_not_a_number() {
        let result = "abc".parse::<Port>();
        assert!(result.is_err());
    }

    #[test]
    fn test_port_display() {
        let port = Port::try_from(8080).unwrap();
        assert_eq!(port.to_string(), "8080");
    }

    // ========== HTTP Target Validation Tests ==========

    #[test]
    fn test_http_target_invalid_status_too_low() {
        let url = Url::parse("http://example.com").unwrap();
        let result = Target::http(url, 99);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_invalid_status_too_high() {
        let url = Url::parse("http://example.com").unwrap();
        let result = Target::http(url, 600);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_status_100_valid() {
        let url = Url::parse("http://example.com").unwrap();
        let result = Target::http(url, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_target_status_599_valid() {
        let url = Url::parse("http://example.com").unwrap();
        let result = Target::http(url, 599);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_target_unsupported_scheme() {
        let result = Target::http_url("ftp://example.com", 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_file_scheme() {
        let result = Target::http_url("file:///etc/passwd", 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_empty_header_key() {
        let url = Url::parse("http://example.com").unwrap();
        let headers = vec![("".to_string(), "value".to_string())];
        let result = Target::http_with_headers(url, 200, headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_empty_header_value() {
        let url = Url::parse("http://example.com").unwrap();
        let headers = vec![("X-Custom".to_string(), "".to_string())];
        let result = Target::http_with_headers(url, 200, headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_invalid_header_name_space() {
        let url = Url::parse("http://example.com").unwrap();
        let headers = vec![("X Custom".to_string(), "value".to_string())];
        let result = Target::http_with_headers(url, 200, headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_target_valid_header() {
        let url = Url::parse("http://example.com").unwrap();
        let headers = vec![("X-Custom-Header".to_string(), "value".to_string())];
        let result = Target::http_with_headers(url, 200, headers);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_builder_auth_bearer() {
        let url = Url::parse("http://example.com").unwrap();
        let target = Target::http_builder(url)
            .auth_bearer("token123")
            .build()
            .unwrap();

        match target {
            Target::Http { headers, .. } => {
                assert!(headers.is_some());
                let headers = headers.unwrap();
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].0, "Authorization");
                assert_eq!(headers[0].1, "Bearer token123");
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_http_builder_basic_auth() {
        let url = Url::parse("http://example.com").unwrap();
        let target = Target::http_builder(url)
            .basic_auth("user", "pass")
            .build()
            .unwrap();

        match target {
            Target::Http { headers, .. } => {
                assert!(headers.is_some());
                let headers = headers.unwrap();
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].0, "Authorization");
                assert!(headers[0].1.starts_with("Basic "));
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    // ========== Target Creation Tests ==========

    #[test]
    fn test_target_tcp_invalid_hostname() {
        let result = Target::tcp("", 8080);
        assert!(result.is_err());
    }

    #[test]
    fn test_target_tcp_invalid_port() {
        let result = Target::tcp("localhost", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_target_localhost_valid() {
        let result = Target::localhost(8080);
        assert!(result.is_ok());
    }

    #[test]
    fn test_target_loopback_valid() {
        let result = Target::loopback(8080);
        assert!(result.is_ok());
    }

    #[test]
    fn test_target_loopback_v6_valid() {
        let result = Target::loopback_v6(8080);
        assert!(result.is_ok());
    }

    #[test]
    fn test_target_http_localhost() {
        let result = Target::http_localhost(3000);
        assert!(result.is_ok());
        match result.unwrap() {
            Target::Http { url, .. } => {
                assert_eq!(url.to_string(), "http://localhost:3000/");
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_target_from_parts() {
        let hostname = Hostname::localhost();
        let port = Port::try_from(8080).unwrap();
        let target = Target::from_parts(hostname, port);

        assert_eq!(target.hostname(), "localhost");
        assert_eq!(target.port(), Some(8080));
    }

    #[test]
    fn test_target_kind() {
        let tcp = Target::tcp("localhost", 8080).unwrap();
        assert_eq!(tcp.kind(), TargetKind::Tcp);

        let http = Target::http_url("http://example.com", 200).unwrap();
        assert_eq!(http.kind(), TargetKind::Http);
    }

    #[test]
    fn test_target_tcp_batch_valid() {
        let targets = vec![("localhost", 8080), ("127.0.0.1", 9090)];
        let result = Target::tcp_batch(targets);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_target_tcp_batch_one_invalid() {
        let targets = vec![
            ("localhost", 8080),
            ("", 9090), // Invalid hostname
        ];
        let result = Target::tcp_batch(targets);
        assert!(result.is_err());
    }

    #[test]
    fn test_target_tcp_ports_single_host() {
        let result = Target::tcp_ports("localhost", &[8080, 8081, 8082]);
        assert!(result.is_ok());
        let targets = result.unwrap();
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].port(), Some(8080));
        assert_eq!(targets[1].port(), Some(8081));
        assert_eq!(targets[2].port(), Some(8082));
    }

    #[test]
    fn test_target_tcp_ports_invalid_port() {
        let result = Target::tcp_ports("localhost", &[8080, 0, 8082]);
        assert!(result.is_err());
    }

    #[test]
    fn test_target_http_batch() {
        let urls = vec!["http://example.com", "https://example.org"];
        let result = Target::http_batch(urls, 200);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    // ========== Duration Validation Tests ==========

    #[test]
    fn test_validated_duration_from_str_seconds() {
        let result = "30s".parse::<types::ValidatedDuration>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), Duration::from_secs(30));
    }

    #[test]
    fn test_validated_duration_from_str_minutes() {
        let result = "5m".parse::<types::ValidatedDuration>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), Duration::from_secs(300));
    }

    #[test]
    fn test_validated_duration_from_str_hours() {
        let result = "2h".parse::<types::ValidatedDuration>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), Duration::from_secs(7200));
    }

    #[test]
    fn test_validated_duration_from_str_milliseconds() {
        let result = "500ms".parse::<types::ValidatedDuration>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), Duration::from_millis(500));
    }

    #[test]
    fn test_validated_duration_from_str_plain_number() {
        let result = "60".parse::<types::ValidatedDuration>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), Duration::from_secs(60));
    }

    #[test]
    fn test_validated_duration_from_str_invalid_unit() {
        let result = "30d".parse::<types::ValidatedDuration>();
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_duration_from_str_invalid_number() {
        let result = "abc".parse::<types::ValidatedDuration>();
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_duration_display() {
        let duration = types::ValidatedDuration::from_secs(90);
        assert_eq!(duration.to_string(), "1m");

        let duration = types::ValidatedDuration::from_millis(500);
        assert_eq!(duration.to_string(), "500ms");
    }

    // ========== WaitConfig Tests ==========

    #[test]
    fn test_wait_config_default() {
        let config = WaitConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.initial_interval, Duration::from_secs(1));
        assert!(!config.wait_for_any);
        assert!(config.max_retries.is_none());
    }

    #[test]
    fn test_wait_config_builder_custom() {
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(60))
            .interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(10))
            .wait_for_any(true)
            .max_retries(Some(5))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(10));
        assert!(config.wait_for_any);
        assert_eq!(config.max_retries, Some(5));
    }

    #[test]
    fn test_wait_config_from_duration() {
        let config = WaitConfig::from(Duration::from_secs(120));
        assert_eq!(config.timeout, Duration::from_secs(120));
    }

    // ========== Error Context Tests ==========

    #[test]
    fn test_result_ext_context() {
        let hostname_result: Result<Hostname> = Hostname::new("");
        let result_with_context = hostname_result.context("Failed to create hostname");

        assert!(result_with_context.is_err());
        let err = result_with_context.unwrap_err();
        assert!(err.to_string().contains("Failed to create hostname"));
    }

    #[test]
    fn test_result_ext_with_context() {
        let hostname_result: Result<Hostname> = Hostname::new("");
        let result_with_context =
            hostname_result.with_context(|| "Dynamic error message".to_string());

        assert!(result_with_context.is_err());
    }
}
