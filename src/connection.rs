#![allow(
    clippy::pub_with_shorthand,
    clippy::pub_without_shorthand,
    reason = "restriction lints have contradictory pub visibility rules"
)]

//! Connection logic with retry and backoff.

use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpStream, lookup_host};
use tokio::time::{Instant, timeout};
use url::Url;

use crate::types::{ConnectionError, HttpError, Target, TargetResult, WaitConfig, WaitResult};
use crate::{Result, WaitForError};

type HttpHeaders = Option<Vec<(String, String)>>;

const EXPONENTIAL_BACKOFF_MULTIPLIER: f64 = 1.5;

#[inline]
pub(crate) async fn resolve_host(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    // Use tuple to avoid String allocation - ToSocketAddrs is implemented for (&str, u16)
    let addrs: Vec<SocketAddr> = lookup_host((host, port))
        .await
        .map_err(|e| {
            WaitForError::Connection(ConnectionError::DnsResolution {
                host: Cow::Owned(host.to_string()),
                reason: e,
            })
        })?
        .collect();

    if addrs.is_empty() {
        return Err(WaitForError::Connection(ConnectionError::DnsResolution {
            host: Cow::Owned(host.to_string()),
            reason: std::io::Error::new(std::io::ErrorKind::NotFound, "No addresses found"),
        }));
    }

    Ok(addrs)
}

#[inline]
pub(crate) async fn try_http_connect(
    url: &Url,
    expected_status: u16,
    headers: &HttpHeaders,
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
        })?;

    let mut request = client.get(url.clone());

    // Headers are already validated at target creation time
    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }

    let response = request.send().await.map_err(|e| {
        WaitForError::Http(HttpError::RequestFailed {
            url: Cow::Owned(url.to_string()),
            reason: e,
        })
    })?;

    let actual_status = response.status().as_u16();
    if actual_status == expected_status {
        Ok(())
    } else {
        Err(WaitForError::Http(HttpError::UnexpectedStatus {
            expected: expected_status,
            actual: actual_status,
        }))
    }
}

#[inline]
pub(crate) async fn try_connect_target(target: &Target, config: &WaitConfig) -> Result<()> {
    match target {
        Target::Tcp { host, port } => {
            let addrs = resolve_host(host.as_str(), port.get()).await?;

            let mut last_error = None;
            for addr in addrs {
                match timeout(config.connection_timeout, TcpStream::connect(addr)).await {
                    Ok(Ok(_)) => return Ok(()),
                    Ok(Err(e)) => last_error = Some(e),
                    Err(_) => {
                        return Err(WaitForError::Connection(ConnectionError::Timeout {
                            timeout_ms: crate::utils::duration_to_millis_u64(
                                config.connection_timeout,
                            ),
                        }));
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
        }
        Target::Http {
            url,
            expected_status,
            headers,
        } => try_http_connect(url, *expected_status, headers, config.connection_timeout).await,
    }
}

#[inline]
pub(crate) fn calculate_next_interval(current: Duration, max: Duration) -> Duration {
    let current_ms = current.as_millis();

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "f64 calculation needed for exponential backoff"
    )]
    let next_ms = (current_ms as f64 * EXPONENTIAL_BACKOFF_MULTIPLIER) as u64;

    Duration::from_millis(next_ms).min(max)
}

#[inline]
async fn wait_for_any_target(
    targets: &[Target],
    config: &WaitConfig,
    start: Instant,
) -> Result<WaitResult> {
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
}

#[inline]
async fn wait_for_all_targets(
    targets: &[Target],
    config: &WaitConfig,
    start: Instant,
) -> Result<WaitResult> {
    use futures::future::join_all;

    let futures: Vec<_> = targets
        .iter()
        .map(|target| wait_for_single_target(target, config))
        .collect();

    let results = join_all(futures).await;
    let mut target_results = Vec::new();
    let mut all_successful = true;
    let mut total_attempts = 0;
    let mut failed_targets = Vec::new();

    // Single-pass: collect results and track failures simultaneously
    for result in results {
        match result {
            Ok(target_result) => {
                if !target_result.success {
                    all_successful = false;
                    failed_targets.push(target_result.target.to_string());
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

/// Wait for single target with retry.
///
/// # Errors
///
/// Returns error if target is unreachable or cancelled.
#[inline]
pub async fn wait_for_single_target(target: &Target, config: &WaitConfig) -> Result<TargetResult> {
    let start = Instant::now();
    let deadline = start + config.timeout;
    let mut current_interval = config.initial_interval;
    let mut attempt = 0;
    let mut last_error: Option<String> = None;

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
                error: last_error
                    .or_else(|| Some(crate::error_messages::TIMEOUT_EXCEEDED.to_string())),
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
            Err(e) => {
                // Preserve the last error so users can see what went wrong
                last_error = Some(e.to_string());

                if let Some(max_retries) = config.max_retries {
                    if attempt >= max_retries {
                        return Ok(TargetResult {
                            target: target.clone(),
                            success: false,
                            elapsed: now.duration_since(start),
                            attempts: attempt,
                            error: Some(format!(
                                "Max retries ({max_retries}) exceeded. Last error: {}",
                                last_error.as_deref().unwrap_or("unknown")
                            )),
                        });
                    }
                }

                // Sleep for current interval, but never past the deadline
                // Check if deadline has passed to avoid panic in duration_since
                let now = Instant::now();
                let sleep_duration = if now >= deadline {
                    Duration::ZERO
                } else {
                    current_interval.min(deadline.saturating_duration_since(now))
                };

                if sleep_duration > Duration::ZERO {
                    crate::utils::sleep_with_cancellation(
                        sleep_duration,
                        config.cancellation_token.as_ref(),
                    )
                    .await?;
                }

                current_interval = calculate_next_interval(current_interval, config.max_interval);
            }
        }
    }
}

/// Wait for multiple targets (all or any).
///
/// # Errors
///
/// Returns error if targets are unreachable, timeout occurs, or cancelled.
#[inline]
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
        wait_for_any_target(targets, config, start).await
    } else {
        wait_for_all_targets(targets, config, start).await
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code where panics are acceptable")]
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
