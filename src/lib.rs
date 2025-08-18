//! A robust library for waiting until TCP ports, HTTP endpoints, and services become available.
//!
//! This library provides functionality for testing network connectivity and service availability,
//! with support for DNS resolution, exponential backoff, and multiple connection strategies.
//! Perfect for Docker, Kubernetes, CI/CD pipelines, and microservices orchestration.
//!
//! # Features
//!
//! - **Type Safety**: NewType wrappers for ports and hostnames with validation
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
//! wait-for = "1.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! # Examples
//!
//! ## Basic TCP Connection Check
//!
//! ```rust,no_run
//! use wait_for::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
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
//! use wait_for::Target;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
//!     let target = Target::http_builder(Url::parse("https://api.example.com/health")?)
//!         .status(200)
//!         .auth_bearer("your-api-token")
//!         .content_type("application/json")
//!         .build()?;
//!
//!     let config = wait_for::WaitConfig::builder()
//!         .timeout(std::time::Duration::from_secs(60))
//!         .build();
//!
//!     wait_for::wait_for_connection(&[target], &config).await?;
//!     println!("API is healthy!");
//!     Ok(())
//! }
//! ```
//!
//! ## Multiple Services with Different Strategies
//!
//! ```rust,no_run
//! use wait_for::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
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
//! use wait_for::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//! use tokio::time::sleep;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
//!     let target = Target::tcp("slow-service", 8080)?;
//!
//!     let (config, cancel_token) = WaitConfig::builder()
//!         .timeout(Duration::from_secs(60))
//!         .interval(Duration::from_millis(500))
//!         .with_cancellation();
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
//!         Err(wait_for::WaitForError::Cancelled) => println!("Operation was cancelled"),
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
//! use wait_for::{Target, WaitConfig, wait_for_connection};
//! use std::time::Duration;
//!
//! /// Wait for services defined in docker-compose.yml
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
//!     let services = vec![
//!         Target::tcp("postgres", 5432)?,     // Database
//!         Target::tcp("redis", 6379)?,        // Cache
//!         Target::tcp("elasticsearch", 9200)?, // Search
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
pub mod types;
pub mod target;
pub mod config;
pub mod connection;
pub mod error;
pub mod iterators;
pub mod presets;
pub mod security;
pub mod async_traits;
pub mod zero_cost;

#[macro_use]
pub mod macros;

// Re-export commonly used types for convenient public API
pub use error::{WaitForError, Result, ResultExt};
pub use types::{
    Port, Hostname, Target, WaitConfig, WaitResult, TargetResult,
    ConnectionError, HttpError
};
pub use target::{HttpTargetBuilder, TcpTargetBuilder};
pub use config::WaitConfigBuilder;
pub use connection::{wait_for_connection, wait_for_single_target};
pub use iterators::{TargetIterExt, TargetResultIterExt, ResultSummary};
pub use security::{RateLimiter, SecurityValidator};
pub use async_traits::{
    AsyncTargetChecker, AsyncRetryStrategy, AsyncConnectionStrategy,
    DefaultTargetChecker, ExponentialBackoffStrategy, LinearBackoffStrategy,
    WaitForAllStrategy, WaitForAnyStrategy, ConcurrentProgressStrategy
};
pub use zero_cost::{
    LazyFormat, StringBuilder, TargetDisplay, SmallString,
    ValidatedPort, WellKnownPort, RegisteredPort, DynamicPort,
    ConstRetryStrategy
};

// Re-export error_messages for internal use
pub(crate) use error::error_messages;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use url::Url;
    use proptest::prelude::*;
    use test_case::test_case;

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
            Target::Http { url, expected_status, .. } => {
                assert_eq!(url.to_string(), "https://example.com/health");
                assert_eq!(expected_status, 200);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_target_display() {
        let tcp_target = Target::tcp("localhost", 8080).unwrap();
        assert_eq!(tcp_target.display(), "localhost:8080");

        let url = Url::parse("https://example.com/health").unwrap();
        let http_target = Target::http(url, 200).unwrap();
        assert_eq!(http_target.display(), "https://example.com/health");
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
        fn test_port_well_known_valid_range(port in 1u16..=1023) {
            let result = Port::well_known(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_well_known_invalid_range(port in 1024u16..=65535) {
            let result = Port::well_known(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_registered_valid_range(port in 1024u16..=49151) {
            let result = Port::registered(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_registered_invalid_low_range(port in 1u16..=1023) {
            let result = Port::registered(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_registered_invalid_high_range(port in 49152u16..=65535) {
            let result = Port::registered(port);
            assert!(result.is_none());
        }

        #[test]
        fn test_port_dynamic_valid_range(port in 49152u16..=65535) {
            let result = Port::dynamic(port);
            assert!(result.is_some());
            assert_eq!(result.unwrap().get(), port);
        }

        #[test]
        fn test_port_dynamic_invalid_range(port in 1u16..=49151) {
            let result = Port::dynamic(port);
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
            max_ms in 60000u64..=300000
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

    #[test_case(80, Port::http(); "http port constant")]
    #[test_case(443, Port::https(); "https port constant")]
    #[test_case(22, Port::ssh(); "ssh port constant")]
    #[test_case(5432, Port::postgres(); "postgres port constant")]
    #[test_case(3306, Port::mysql(); "mysql port constant")]
    #[test_case(6379, Port::redis(); "redis port constant")]
    fn test_port_constants(expected: u16, port: Port) {
        assert_eq!(port.get(), expected);
    }

    #[test_case("http://example.com/", 200; "http url")]
    #[test_case("https://api.example.com/health", 200; "https health endpoint")]
    #[test_case("https://example.com:8080/status", 204; "custom port and status")]
    fn test_http_target_parsing(url_str: &str, status: u16) {
        let target = Target::parse(url_str, status).unwrap();
        match target {
            Target::Http { url, expected_status, .. } => {
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
    fn test_tcp_builder_fluent_interface() {
        // Test that builder methods return Self for fluent chaining
        let target = Target::tcp_builder("example.com").unwrap()
            .registered_port(8080)
            .build().unwrap();

        assert_eq!(target.hostname(), "example.com");
        assert_eq!(target.port(), Some(8080));
    }

    #[test]
    fn test_tcp_builder_error_deferred() {
        // Test that validation errors are deferred until build()
        let result = Target::tcp_builder("example.com").unwrap()
            .well_known_port(8080) // Invalid for well-known range
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_safe_tcp_targets_macro() {
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
    fn test_safe_tcp_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = tcp_targets![
            "localhost" => 8080,
            "example.com" => 0, // Invalid port
        ];

        assert!(result.is_err());
    }

    #[test]
    fn test_safe_http_targets_macro() {
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
    fn test_safe_http_targets_macro_error() {
        // Test that the macro properly propagates errors
        let result = http_targets![
            "https://example.com" => 200,
            "invalid-url" => 200, // Invalid URL
        ];

        assert!(result.is_err());
    }
}
