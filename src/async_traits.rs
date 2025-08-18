//! Async traits for extensible connection strategies and target checking.
//!
//! This module provides async traits that allow for custom implementations
//! of connection checking, retry strategies, and concurrency patterns.

use async_trait::async_trait;
use std::time::Duration;
use crate::types::{Target, TargetResult, WaitConfig, WaitResult};
use crate::{WaitForError, Result};

/// Async trait for checking target availability
///
/// This allows for custom implementations of target checking logic,
/// including mocking for testing, custom protocols, or specialized health checks.
#[async_trait]
pub trait AsyncTargetChecker: Send + Sync {
    /// Check if a target is available
    async fn check_target(&self, target: &Target, config: &WaitConfig) -> Result<()>;

    /// Get a human-readable name for this checker
    fn name(&self) -> &'static str;
}

/// Async trait for retry strategies
///
/// This allows for custom retry logic, exponential backoff algorithms,
/// jitter, and other advanced retry patterns.
#[async_trait]
pub trait AsyncRetryStrategy: Send + Sync {
    /// Calculate the next retry interval
    fn next_interval(&mut self, attempt: u32, last_interval: Duration) -> Duration;

    /// Check if we should continue retrying
    fn should_retry(&self, attempt: u32, elapsed: Duration, max_retries: Option<u32>, timeout: Duration) -> bool;

    /// Reset strategy state for a new target
    fn reset(&mut self);

    /// Get strategy name for debugging
    fn name(&self) -> &'static str;
}

/// Async trait for connection strategies
///
/// This allows for custom concurrency patterns beyond the built-in "all" and "any" strategies.
#[async_trait]
pub trait AsyncConnectionStrategy: Send + Sync {
    /// Execute the connection strategy for multiple targets
    async fn execute(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<WaitResult>;

    /// Get strategy name
    fn name(&self) -> &'static str;

    /// Execute strategy with streaming results for real-time progress
    ///
    /// Note: This is a simpler approach that returns a future of results.
    /// For true streaming behavior, use the execute method with custom logic.
    async fn execute_streaming(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<Vec<TargetResult>> {
        // Default implementation that just executes normally
        match self.execute(targets, checker, config).await {
            Ok(wait_result) => Ok(wait_result.target_results),
            Err(e) => Err(e),
        }
    }
}

/// Default implementation of AsyncTargetChecker using the existing connection logic
pub struct DefaultTargetChecker;

#[async_trait]
impl AsyncTargetChecker for DefaultTargetChecker {
    async fn check_target(&self, target: &Target, config: &WaitConfig) -> Result<()> {
        crate::connection::try_connect_target(target, config).await
    }

    fn name(&self) -> &'static str {
        "default"
    }
}

/// Exponential backoff retry strategy
pub struct ExponentialBackoffStrategy {
    multiplier: f64,
    max_interval: Duration,
}

impl ExponentialBackoffStrategy {
    pub fn new(multiplier: f64, max_interval: Duration) -> Self {
        Self { multiplier, max_interval }
    }
}

impl Default for ExponentialBackoffStrategy {
    fn default() -> Self {
        Self::new(1.5, Duration::from_secs(30))
    }
}

#[async_trait]
impl AsyncRetryStrategy for ExponentialBackoffStrategy {
    fn next_interval(&mut self, _attempt: u32, last_interval: Duration) -> Duration {
        let next = Duration::from_millis((last_interval.as_millis() as f64 * self.multiplier) as u64);
        if next > self.max_interval {
            self.max_interval
        } else {
            next
        }
    }

    fn should_retry(&self, attempt: u32, elapsed: Duration, max_retries: Option<u32>, timeout: Duration) -> bool {
        // Check timeout
        if elapsed >= timeout {
            return false;
        }

        // Check max retries
        if let Some(max) = max_retries {
            if attempt >= max {
                return false;
            }
        }

        true
    }

    fn reset(&mut self) {
        // Nothing to reset for exponential backoff
    }

    fn name(&self) -> &'static str {
        "exponential_backoff"
    }
}

/// Linear backoff retry strategy
pub struct LinearBackoffStrategy {
    increment: Duration,
    max_interval: Duration,
}

impl LinearBackoffStrategy {
    pub fn new(increment: Duration, max_interval: Duration) -> Self {
        Self { increment, max_interval }
    }
}

impl Default for LinearBackoffStrategy {
    fn default() -> Self {
        Self::new(Duration::from_secs(1), Duration::from_secs(30))
    }
}

#[async_trait]
impl AsyncRetryStrategy for LinearBackoffStrategy {
    fn next_interval(&mut self, _attempt: u32, last_interval: Duration) -> Duration {
        let next = last_interval + self.increment;
        if next > self.max_interval {
            self.max_interval
        } else {
            next
        }
    }

    fn should_retry(&self, attempt: u32, elapsed: Duration, max_retries: Option<u32>, timeout: Duration) -> bool {
        if elapsed >= timeout {
            return false;
        }

        if let Some(max) = max_retries {
            if attempt >= max {
                return false;
            }
        }

        true
    }

    fn reset(&mut self) {
        // Nothing to reset
    }

    fn name(&self) -> &'static str {
        "linear_backoff"
    }
}

/// Strategy that waits for all targets to be ready
pub struct WaitForAllStrategy;

#[async_trait]
impl AsyncConnectionStrategy for WaitForAllStrategy {
    async fn execute(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<WaitResult> {
        use futures::future::join_all;
        use tokio::time::Instant;

        let start = Instant::now();

        if targets.is_empty() {
            return Ok(WaitResult {
                success: true,
                elapsed: start.elapsed(),
                attempts: 0,
                target_results: vec![],
            });
        }

        // Create futures for each target using the async target checker
        let futures: Vec<_> = targets.iter()
            .map(|target| wait_for_single_target_with_checker(target, checker, config))
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
            let failed_targets: Vec<_> = target_results.iter()
                .filter(|r| !r.success)
                .map(|r| r.target.display())
                .collect();
            return Err(WaitForError::Timeout {
                targets: std::borrow::Cow::Owned(failed_targets.join(", ")),
            });
        }

        Ok(WaitResult {
            success: all_successful,
            elapsed: start.elapsed(),
            attempts: total_attempts,
            target_results,
        })
    }

    fn name(&self) -> &'static str {
        "wait_for_all"
    }
}

/// Strategy that waits for any target to be ready
pub struct WaitForAnyStrategy;

#[async_trait]
impl AsyncConnectionStrategy for WaitForAnyStrategy {
    async fn execute(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<WaitResult> {
        use futures::future::select_ok;
        use tokio::time::Instant;

        let start = Instant::now();

        if targets.is_empty() {
            return Ok(WaitResult {
                success: true,
                elapsed: start.elapsed(),
                attempts: 0,
                target_results: vec![],
            });
        }

        let futures: Vec<_> = targets.iter()
            .map(|target| Box::pin(wait_for_single_target_with_checker(target, checker, config)))
            .collect();

        match select_ok(futures).await {
            Ok((result, _)) => {
                Ok(WaitResult {
                    success: result.success,
                    elapsed: start.elapsed(),
                    attempts: result.attempts,
                    target_results: vec![result],
                })
            }
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &'static str {
        "wait_for_any"
    }
}

/// Concurrent strategy that provides real-time feedback as targets become ready
pub struct ConcurrentProgressStrategy {
    concurrency_limit: usize,
}

impl ConcurrentProgressStrategy {
    pub fn new(concurrency_limit: usize) -> Self {
        Self { concurrency_limit }
    }
}

impl Default for ConcurrentProgressStrategy {
    fn default() -> Self {
        Self::new(10) // Default to 10 concurrent checks
    }
}

#[async_trait]
impl AsyncConnectionStrategy for ConcurrentProgressStrategy {
    async fn execute(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<WaitResult> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use tokio::time::Instant;

        let start = Instant::now();

        if targets.is_empty() {
            return Ok(WaitResult {
                success: true,
                elapsed: start.elapsed(),
                attempts: 0,
                target_results: vec![],
            });
        }

        let mut futures = FuturesUnordered::new();
        let mut target_results = Vec::new();
        let mut total_attempts = 0;

        // Add futures with concurrency limit
        for chunk in targets.chunks(self.concurrency_limit) {
            for target in chunk {
                futures.push(wait_for_single_target_with_checker(target, checker, config));
            }

            // Process this batch
            while let Some(result) = futures.next().await {
                match result {
                    Ok(target_result) => {
                        total_attempts += target_result.attempts;
                        target_results.push(target_result);
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        // Check if all were successful
        let all_successful = target_results.iter().all(|r| r.success);

        if !all_successful {
            let failed_targets: Vec<_> = target_results.iter()
                .filter(|r| !r.success)
                .map(|r| r.target.display())
                .collect();
            return Err(WaitForError::Timeout {
                targets: std::borrow::Cow::Owned(failed_targets.join(", ")),
            });
        }

        Ok(WaitResult {
            success: all_successful,
            elapsed: start.elapsed(),
            attempts: total_attempts,
            target_results,
        })
    }

    fn name(&self) -> &'static str {
        "concurrent_progress"
    }

    /// Streaming implementation that yields results as they complete
    async fn execute_streaming(
        &self,
        targets: &[Target],
        checker: &dyn AsyncTargetChecker,
        config: &WaitConfig,
    ) -> Result<Vec<TargetResult>> {
        // For this strategy, just use the normal execute and return all results
        // In a real implementation, this could provide progress callbacks
        match self.execute(targets, checker, config).await {
            Ok(wait_result) => Ok(wait_result.target_results),
            Err(e) => Err(e),
        }
    }
}

/// Helper function to wait for a single target using a custom checker
async fn wait_for_single_target_with_checker(
    target: &Target,
    checker: &dyn AsyncTargetChecker,
    config: &WaitConfig,
) -> Result<TargetResult> {
    use tokio::time::{sleep, Instant};

    let start = Instant::now();
    let deadline = start + config.timeout;
    let mut current_interval = config.initial_interval;
    let mut attempt = 0;
    let mut retry_strategy = ExponentialBackoffStrategy::default();

    loop {
        // Check for cancellation
        if let Some(token) = &config.cancellation_token {
            if token.is_cancelled() {
                return Err(WaitForError::Cancelled);
            }
        }

        // Check if we've exceeded the deadline
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

        // Try connection with remaining time constraint
        let remaining_time = deadline.duration_since(now);
        let connection_timeout = config.connection_timeout.min(remaining_time);

        let mut connection_config = config.clone();
        connection_config.connection_timeout = connection_timeout;

        match checker.check_target(target, &connection_config).await {
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
                // Check if we should retry
                if !retry_strategy.should_retry(
                    attempt,
                    now.duration_since(start),
                    config.max_retries,
                    config.timeout
                ) {
                    return Ok(TargetResult {
                        target: target.clone(),
                        success: false,
                        elapsed: now.duration_since(start),
                        attempts: attempt,
                        error: Some(format!("Max retries ({:?}) exceeded", config.max_retries)),
                    });
                }

                // Calculate sleep duration
                current_interval = retry_strategy.next_interval(attempt, current_interval);
                let sleep_duration = current_interval.min(deadline.duration_since(Instant::now()));

                // Sleep with cancellation support
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
            }
        }
    }
}
