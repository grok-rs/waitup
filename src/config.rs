//! Configuration builders and related functionality.
//!
//! This module provides the [`WaitConfigBuilder`] for creating [`WaitConfig`] instances
//! with various timeout, retry, and concurrency settings.
//!
//! # Features
//!
//! - **Flexible Timeouts**: Total timeout, connection timeout, and retry intervals
//! - **Exponential Backoff**: Configurable initial and maximum retry intervals
//! - **Retry Limits**: Optional maximum retry attempts
//! - **Concurrency**: Wait for any vs all targets
//! - **Cancellation**: Graceful shutdown with cancellation tokens
//!
//! # Examples
//!
//! ## Basic configuration
//!
//! ```rust
//! use wait_for::WaitConfig;
//! use std::time::Duration;
//!
//! let config = WaitConfig::builder()
//!     .timeout(Duration::from_secs(30))
//!     .interval(Duration::from_secs(1))
//!     .build();
//! ```
//!
//! ## Advanced configuration with cancellation
//!
//! ```rust
//! use wait_for::WaitConfig;
//! use std::time::Duration;
//!
//! let (config, cancel_token) = WaitConfig::builder()
//!     .timeout(Duration::from_secs(60))
//!     .interval(Duration::from_millis(500))
//!     .max_interval(Duration::from_secs(10))
//!     .connection_timeout(Duration::from_secs(5))
//!     .max_retries(Some(20))
//!     .wait_for_any(false)
//!     .with_cancellation();
//!
//! // Later, cancel the operation
//! // cancel_token.cancel();
//! ```
//!
//! ## Microservice readiness configuration
//!
//! ```rust
//! use wait_for::WaitConfig;
//! use std::time::Duration;
//!
//! // Fast polling for local services
//! let local_config = WaitConfig::builder()
//!     .timeout(Duration::from_secs(10))
//!     .interval(Duration::from_millis(100))
//!     .max_interval(Duration::from_secs(1))
//!     .connection_timeout(Duration::from_secs(2))
//!     .build();
//!
//! // Conservative settings for external services
//! let external_config = WaitConfig::builder()
//!     .timeout(Duration::from_secs(120))
//!     .interval(Duration::from_secs(2))
//!     .max_interval(Duration::from_secs(30))
//!     .connection_timeout(Duration::from_secs(15))
//!     .max_retries(Some(10))
//!     .build();
//! ```

use std::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::types::WaitConfig;

impl WaitConfig {
    /// Create a new builder for WaitConfig.
    pub fn builder() -> WaitConfigBuilder {
        WaitConfigBuilder::default()
    }
}

/// Builder for WaitConfig.
#[derive(Debug, Clone)]
pub struct WaitConfigBuilder {
    config: WaitConfig,
}

impl Default for WaitConfigBuilder {
    fn default() -> Self {
        Self {
            config: WaitConfig::default(),
        }
    }
}

impl WaitConfigBuilder {
    /// Set the total timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the initial retry interval.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.config.initial_interval = interval;
        self
    }

    /// Set the maximum retry interval for exponential backoff.
    pub fn max_interval(mut self, max_interval: Duration) -> Self {
        self.config.max_interval = max_interval;
        self
    }

    /// Set whether to wait for any target (true) or all targets (false).
    pub fn wait_for_any(mut self, wait_for_any: bool) -> Self {
        self.config.wait_for_any = wait_for_any;
        self
    }

    /// Set the maximum number of retry attempts.
    pub fn max_retries(mut self, max_retries: Option<u32>) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set the individual connection timeout.
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.config.connection_timeout = timeout;
        self
    }

    /// Set the cancellation token for graceful shutdown.
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.config.cancellation_token = Some(token);
        self
    }

    /// Enable cancellation with a new token.
    pub fn with_cancellation(mut self) -> (Self, CancellationToken) {
        let token = CancellationToken::new();
        self.config.cancellation_token = Some(token.clone());
        (self, token)
    }

    /// Set the security validator.
    pub fn security_validator(mut self, validator: crate::security::SecurityValidator) -> Self {
        self.config.security_validator = Some(validator);
        self
    }

    /// Set the rate limiter.
    pub fn rate_limiter(mut self, limiter: crate::security::RateLimiter) -> Self {
        self.config.rate_limiter = Some(limiter);
        self
    }

    /// Enable production security (strict validation and rate limiting).
    pub fn production_security(mut self) -> Self {
        self.config.security_validator = Some(crate::security::SecurityValidator::production());
        self.config.rate_limiter = Some(crate::security::RateLimiter::new(30)); // Conservative rate limit
        self
    }

    /// Enable development security (permissive validation, basic rate limiting).
    pub fn development_security(mut self) -> Self {
        self.config.security_validator = Some(crate::security::SecurityValidator::development());
        self.config.rate_limiter = Some(crate::security::RateLimiter::new(120)); // Relaxed rate limit
        self
    }

    /// Build the WaitConfig.
    pub fn build(self) -> WaitConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_wait_config_builder_defaults() {
        let config = WaitConfig::builder().build();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.initial_interval, Duration::from_secs(1));
        assert_eq!(config.max_interval, Duration::from_secs(30));
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert!(!config.wait_for_any);
        assert_eq!(config.max_retries, None);
        assert!(config.cancellation_token.is_none());
    }

    #[test]
    fn test_wait_config_builder_custom_values() {
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(120))
            .interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(60))
            .connection_timeout(Duration::from_secs(20))
            .wait_for_any(true)
            .max_retries(Some(10))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(60));
        assert_eq!(config.connection_timeout, Duration::from_secs(20));
        assert!(config.wait_for_any);
        assert_eq!(config.max_retries, Some(10));
    }

    #[test]
    fn test_wait_config_with_cancellation() {
        let (builder, token) = WaitConfig::builder()
            .timeout(Duration::from_secs(30))
            .with_cancellation();

        let config = builder.build();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.cancellation_token.is_some());
        assert!(!token.is_cancelled());

        // Test that the token works
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_wait_config_security_presets() {
        let config = WaitConfig::builder()
            .production_security()
            .build();

        // Should have security settings applied
        assert!(config.security_validator.is_some());
        assert!(config.rate_limiter.is_some());

        let config = WaitConfig::builder()
            .development_security()
            .build();

        // Should have development security settings applied
        assert!(config.security_validator.is_some());
        assert!(config.rate_limiter.is_some());
    }

    #[test]
    fn test_wait_config_custom_security() {
        use crate::security::{SecurityValidator, RateLimiter};

        let validator = SecurityValidator::new();
        let limiter = RateLimiter::new(100);

        let config = WaitConfig::builder()
            .security_validator(validator)
            .rate_limiter(limiter)
            .build();

        assert!(config.security_validator.is_some());
        assert!(config.rate_limiter.is_some());
    }

    #[test]
    fn test_wait_config_builder_chaining() {
        // Test that all methods return Self for fluent chaining
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(30))
            .interval(Duration::from_millis(100))
            .max_interval(Duration::from_secs(10))
            .connection_timeout(Duration::from_secs(5))
            .wait_for_any(false)
            .max_retries(Some(5))
            .production_security()
            .build();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.initial_interval, Duration::from_millis(100));
        assert_eq!(config.max_interval, Duration::from_secs(10));
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert!(!config.wait_for_any);
        assert_eq!(config.max_retries, Some(5));
    }
}
