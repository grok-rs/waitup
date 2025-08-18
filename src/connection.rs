//! Connection handling and network operations.
//!
//! This module contains the core connection logic for testing TCP and HTTP targets.
//! It provides functions for DNS resolution, connection attempts, and coordinating
//! multiple target checks with different strategies.
//!
//! # Features
//!
//! - **TCP Connection Testing**: Direct socket connections with timeout handling
//! - **HTTP Request Testing**: HTTP/HTTPS requests with status code validation
//! - **DNS Resolution**: Hostname-to-IP resolution with error handling
//! - **Exponential Backoff**: Smart retry intervals that increase over time
//! - **Concurrency**: Parallel or sequential target checking
//! - **Cancellation**: Graceful shutdown with immediate response to cancel signals
//! - **Deadline Management**: Precise timeout handling across all operations
//!
//! # Connection Strategies
//!
//! The module supports two main strategies:
//!
//! - **Wait for All**: All targets must be ready before returning success
//! - **Wait for Any**: Return success as soon as any target becomes ready
//!
//! # Examples
//!
//! ## Single target check
//!
//! ```rust,no_run
//! use wait_for::{Target, WaitConfig, wait_for_single_target};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), wait_for::WaitForError> {
//!     let target = Target::tcp("localhost", 8080)?;
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .build();
//!
//!     let result = wait_for_single_target(&target, &config).await?;
//!     if result.success {
//!         println!("Target is ready after {} attempts in {:?}",
//!             result.attempts, result.elapsed);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Multiple targets with different strategies
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
//!     // Wait for ALL targets to be ready
//!     let all_config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(60))
//!         .wait_for_any(false)
//!         .build();
//!
//!     wait_for_connection(&targets, &all_config).await?;
//!     println!("All services are ready!");
//!
//!     // Or wait for ANY target to be ready
//!     let any_config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .wait_for_any(true)
//!         .build();
//!
//!     wait_for_connection(&targets, &any_config).await?;
//!     println!("At least one service is ready!");
//!     Ok(())
//! }
//! ```

use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, lookup_host};
use tokio::time::{Instant, sleep, timeout};
use url::Url;

use crate::types::{
    ConnectionError, Hostname, HttpError, Port, Target, TargetResult, WaitConfig, WaitResult,
};
use crate::{Result, ResultExt, WaitForError};

/// Resolve a hostname and port to socket addresses.
pub(crate) async fn resolve_host(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let mut host_port_builder = crate::zero_cost::StringBuilder::<64>::new();
    host_port_builder.push_str(host).unwrap();
    host_port_builder.push_char(':').unwrap();
    host_port_builder.push_str(&port.to_string()).unwrap();
    let host_port = host_port_builder.as_str();
    let addrs: Vec<SocketAddr> = lookup_host(&host_port)
        .await
        .map_err(|e| {
            WaitForError::Connection(ConnectionError::DnsResolution {
                host: Cow::Owned(host.to_string()),
                reason: e,
            })
        })
        .with_context(|| format!("Failed to resolve hostname '{}'", host))?
        .collect();

    if addrs.is_empty() {
        let dns_error = WaitForError::Connection(ConnectionError::DnsResolution {
            host: Cow::Owned(host.to_string()),
            reason: std::io::Error::new(std::io::ErrorKind::NotFound, "No addresses found"),
        });
        return Err(dns_error)
            .with_context(|| format!("No IP addresses found for hostname '{}'", host));
    }

    Ok(addrs)
}

/// Try to establish a TCP connection.
pub(crate) async fn try_tcp_connect(
    host: &Hostname,
    port: Port,
    timeout_duration: Duration,
) -> Result<()> {
    let addrs = resolve_host(host.as_str(), port.get())
        .await
        .with_context(|| format!("Failed to resolve {}:{}", host, port))?;

    let mut last_error = None;
    for addr in addrs {
        match timeout(timeout_duration, TcpStream::connect(addr)).await {
            Ok(Ok(_)) => return Ok(()),
            Ok(Err(e)) => last_error = Some(e),
            Err(_) => {
                return Err(WaitForError::Connection(ConnectionError::Timeout {
                    timeout_ms: timeout_duration.as_millis() as u64,
                }))
                .with_context(|| {
                    format!(
                        "Connection timeout after {}ms to {}:{}",
                        timeout_duration.as_millis(),
                        host,
                        port
                    )
                });
            }
        }
    }

    Err(WaitForError::Connection(ConnectionError::TcpConnection {
        host: Cow::Owned(host.to_string()),
        port: port.get(),
        reason: last_error.unwrap_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "No addresses available",
            )
        }),
    }))
    .with_context(|| format!("Failed to establish TCP connection to {}:{}", host, port))
}

/// Try to make an HTTP request and check the response.
pub(crate) async fn try_http_connect(
    url: &Url,
    expected_status: u16,
    headers: &Option<Vec<(String, String)>>,
    timeout_duration: Duration,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(timeout_duration)
        .build()
        .map_err(|e| {
            WaitForError::Http(HttpError::RequestFailed {
                url: Cow::Owned(url.to_string()),
                reason: e,
            })
        })
        .with_context(|| format!("Failed to create HTTP client for {}", url))?;

    let mut request = client.get(url.clone());

    if let Some(headers) = headers {
        for (key, value) in headers {
            if key.is_empty() || value.is_empty() {
                return Err(WaitForError::Http(HttpError::InvalidHeader {
                    header: Cow::Owned(format!("{}:{}", key, value)),
                }))
                .with_context(|| format!("Invalid header for request to {}", url));
            }
            request = request.header(key, value);
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| {
            WaitForError::Http(HttpError::RequestFailed {
                url: Cow::Owned(url.to_string()),
                reason: e,
            })
        })
        .with_context(|| format!("HTTP request failed to {}", url))?;

    let actual_status = response.status().as_u16();
    if actual_status == expected_status {
        Ok(())
    } else {
        Err(WaitForError::Http(HttpError::UnexpectedStatus {
            expected: expected_status,
            actual: actual_status,
        }))
        .with_context(|| {
            format!(
                "Unexpected HTTP status from {}: expected {}, got {}",
                url, expected_status, actual_status
            )
        })
    }
}

/// Try to connect to a target with security validation.
pub(crate) async fn try_connect_target(target: &Target, config: &WaitConfig) -> Result<()> {
    if let Some(ref validator) = config.security_validator {
        validator.validate_target(target)?;
    }

    if let Some(ref rate_limiter) = config.rate_limiter {
        rate_limiter.check_rate_limit(target)?;
    }

    match target {
        Target::Tcp { host, port } => try_tcp_connect(host, *port, config.connection_timeout).await,
        Target::Http {
            url,
            expected_status,
            headers,
        } => try_http_connect(url, *expected_status, headers, config.connection_timeout).await,
    }
}

/// Calculate the next retry interval using exponential backoff.
pub(crate) fn calculate_next_interval(current: Duration, max: Duration) -> Duration {
    let next = Duration::from_millis((current.as_millis() as f64 * 1.5) as u64);
    if next > max { max } else { next }
}

/// Wait for a single target to become available.
pub async fn wait_for_single_target(target: &Target, config: &WaitConfig) -> Result<TargetResult> {
    let start = Instant::now();
    let deadline = start + config.timeout;
    let mut current_interval = config.initial_interval;
    let mut attempt = 0;

    loop {
        if let Some(token) = &config.cancellation_token {
            if token.is_cancelled() {
                return Err(WaitForError::Cancelled);
            }
        }

        let now = Instant::now();
        if now >= deadline {
            return Ok(TargetResult {
                target: target.clone(),
                success: false,
                elapsed: now.duration_since(start),
                attempts: attempt,
                error: Some("Overall timeout exceeded".to_string()),
            });
        }

        attempt += 1;

        let remaining_time = deadline.duration_since(now);
        let connection_timeout = config.connection_timeout.min(remaining_time);

        let mut connection_config = config.clone();
        connection_config.connection_timeout = connection_timeout;

        match try_connect_target(target, &connection_config).await {
            Ok(()) => {
                return Ok(TargetResult {
                    target: target.clone(),
                    success: true,
                    elapsed: now.duration_since(start),
                    attempts: attempt,
                    error: None,
                });
            }
            Err(_e) => {
                if let Some(max_retries) = config.max_retries {
                    if attempt >= max_retries {
                        return Ok(TargetResult {
                            target: target.clone(),
                            success: false,
                            elapsed: now.duration_since(start),
                            attempts: attempt,
                            error: Some(format!("Max retries ({}) exceeded", max_retries)),
                        });
                    }
                }

                let sleep_duration = current_interval.min(deadline.duration_since(Instant::now()));

                if let Some(token) = &config.cancellation_token {
                    tokio::select! {
                        _ = sleep(sleep_duration) => {},
                        _ = token.cancelled() => {
                            return Err(WaitForError::Cancelled);
                        }
                    }
                } else {
                    sleep(sleep_duration).await;
                }

                current_interval = calculate_next_interval(current_interval, config.max_interval);
            }
        }
    }
}

/// Wait for connections to multiple targets.
///
/// This is the main function for waiting on multiple targets with different strategies.
///
/// # Examples
///
/// ```rust,no_run
/// use wait_for::{Target, WaitConfig, wait_for_connection};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), wait_for::WaitForError> {
///     let targets = vec![
///         Target::tcp("localhost", 8080)?,
///         Target::tcp("localhost", 5432)?,
///     ];
///
///     let config = WaitConfig::builder()
///         .timeout(Duration::from_secs(60))
///         .wait_for_any(false) // Wait for all targets
///         .build();
///
///     wait_for_connection(&targets, &config).await?;
///     println!("All services are ready!");
///     Ok(())
/// }
/// ```
pub async fn wait_for_connection(targets: &[Target], config: &WaitConfig) -> Result<WaitResult> {
    let start = Instant::now();

    if targets.is_empty() {
        return Ok(WaitResult {
            success: true,
            elapsed: start.elapsed(),
            attempts: 0,
            target_results: vec![],
        });
    }

    if config.wait_for_any {
        // Wait for any target to be ready
        use futures::future::select_ok;

        let futures: Vec<_> = targets
            .iter()
            .map(|target| Box::pin(wait_for_single_target(target, config)))
            .collect();

        match select_ok(futures).await {
            Ok((result, _)) => Ok(WaitResult {
                success: result.success,
                elapsed: start.elapsed(),
                attempts: result.attempts,
                target_results: vec![result],
            }),
            Err(e) => Err(e),
        }
    } else {
        // Wait for all targets to be ready
        use futures::future::join_all;

        let futures: Vec<_> = targets
            .iter()
            .map(|target| wait_for_single_target(target, config))
            .collect();

        let results = join_all(futures).await;
        let mut target_results = Vec::new();
        let mut all_successful = true;
        let mut total_attempts = 0;

        for result in results {
            match result {
                Ok(target_result) => {
                    if !target_result.success {
                        all_successful = false;
                    }
                    total_attempts += target_result.attempts;
                    target_results.push(target_result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if !all_successful {
            let failed_targets: Vec<_> = target_results
                .iter()
                .filter(|r| !r.success)
                .map(|r| r.target.display())
                .collect();
            return Err(WaitForError::Timeout {
                targets: Cow::Owned(failed_targets.join(", ")),
            });
        }

        Ok(WaitResult {
            success: all_successful,
            elapsed: start.elapsed(),
            attempts: total_attempts,
            target_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_calculate_next_interval() {
        // Test exponential backoff calculation
        let current = Duration::from_millis(100);
        let max = Duration::from_secs(10);

        let next = calculate_next_interval(current, max);
        // Should be current * 1.5 = 150ms
        assert_eq!(next, Duration::from_millis(150));

        // Test max interval limiting
        let large_current = Duration::from_secs(8);
        let next = calculate_next_interval(large_current, max);
        assert_eq!(next, max);
    }

    #[test]
    fn test_calculate_next_interval_edge_cases() {
        // Test minimum interval that will actually increase
        let current = Duration::from_millis(10);
        let max = Duration::from_secs(1);
        let next = calculate_next_interval(current, max);
        assert!(next >= current); // 10 * 1.5 = 15, so it will be greater
        assert!(next <= max);

        // Test zero interval - the function should return zero (0 * 1.5 = 0)
        let current = Duration::ZERO;
        let max = Duration::from_millis(100);
        let next = calculate_next_interval(current, max);
        assert_eq!(next, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_resolve_host_localhost() {
        // Test localhost resolution
        let result = resolve_host("localhost", 8080).await;
        assert!(result.is_ok());
        let addrs = result.unwrap();
        assert!(!addrs.is_empty());
        assert!(addrs.iter().all(|addr| addr.port() == 8080));
    }

    #[tokio::test]
    async fn test_resolve_host_invalid() {
        // Test invalid hostname
        let result = resolve_host("invalid.nonexistent.domain.test", 8080).await;
        assert!(result.is_err());
        // Just verify it's an error - the specific error type may vary by system
    }

    #[tokio::test]
    async fn test_try_connect_target_invalid_host() {
        use crate::types::WaitConfig;

        let target = Target::tcp("invalid.nonexistent.domain.test", 8080).unwrap();
        let config = WaitConfig::builder()
            .timeout(Duration::from_millis(100))
            .connection_timeout(Duration::from_millis(50))
            .build();

        let result = try_connect_target(&target, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_try_connect_target_http_invalid_url() {
        use crate::types::WaitConfig;

        let target = Target::http_url("http://invalid.nonexistent.domain.test/", 200).unwrap();
        let config = WaitConfig::builder()
            .timeout(Duration::from_millis(100))
            .connection_timeout(Duration::from_millis(50))
            .build();

        let result = try_connect_target(&target, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wait_for_single_target_timeout() {
        use crate::types::WaitConfig;

        let target = Target::tcp("127.0.0.1", 65535).unwrap(); // Unlikely to be used
        let config = WaitConfig::builder()
            .timeout(Duration::from_millis(50))
            .interval(Duration::from_millis(10))
            .connection_timeout(Duration::from_millis(5))
            .max_retries(Some(2))
            .build();

        let result = wait_for_single_target(&target, &config).await;
        assert!(result.is_ok());
        let target_result = result.unwrap();
        assert!(!target_result.success);
        assert!(target_result.attempts >= 1);
    }

    #[tokio::test]
    async fn test_wait_for_connection_empty_targets() {
        use crate::types::WaitConfig;

        let targets: Vec<Target> = vec![];
        let config = WaitConfig::builder().build();

        let result = wait_for_connection(&targets, &config).await;
        assert!(result.is_ok());
        let wait_result = result.unwrap();
        assert!(wait_result.success);
        assert_eq!(wait_result.target_results.len(), 0);
    }
}
