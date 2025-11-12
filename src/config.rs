//! Builder for WaitConfig with timeout and retry settings.

use core::time::Duration;
use tokio_util::sync::CancellationToken;

use crate::types::WaitConfig;

impl WaitConfig {
    /// Create a new builder for `WaitConfig`.
    #[must_use]
    #[inline]
    pub fn builder() -> WaitConfigBuilder {
        WaitConfigBuilder::default()
    }
}

/// Builder for `WaitConfig`.
#[derive(Debug, Clone, Default)]
pub struct WaitConfigBuilder {
    config: WaitConfig,
}

impl WaitConfigBuilder {
    /// Set the total timeout.
    #[must_use]
    #[inline]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Set the initial retry interval.
    #[must_use]
    #[inline]
    pub const fn interval(mut self, interval: Duration) -> Self {
        self.config.initial_interval = interval;
        self
    }

    /// Set the maximum retry interval for exponential backoff.
    #[must_use]
    #[inline]
    pub const fn max_interval(mut self, max_interval: Duration) -> Self {
        self.config.max_interval = max_interval;
        self
    }

    /// Set whether to wait for any target (true) or all targets (false).
    #[must_use]
    #[inline]
    pub const fn wait_for_any(mut self, wait_for_any: bool) -> Self {
        self.config.wait_for_any = wait_for_any;
        self
    }

    /// Set the maximum number of retry attempts.
    #[must_use]
    #[inline]
    pub const fn max_retries(mut self, max_retries: Option<u32>) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set the individual connection timeout.
    #[must_use]
    #[inline]
    pub const fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.config.connection_timeout = timeout;
        self
    }

    /// Set the cancellation token for graceful shutdown.
    #[must_use]
    #[inline]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.config.cancellation_token = Some(token);
        self
    }

    /// Enable cancellation with a new token.
    #[inline]
    pub fn with_cancellation(mut self) -> (Self, CancellationToken) {
        let token = CancellationToken::new();
        self.config.cancellation_token = Some(token.clone());
        (self, token)
    }

    /// Build the `WaitConfig`.
    #[inline]
    pub fn build(self) -> WaitConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn wait_config_builder_defaults() {
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
    fn wait_config_builder_custom_values() {
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
    fn wait_config_with_cancellation() {
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
    fn wait_config_builder_chaining() {
        // Test that all methods return Self for fluent chaining
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(30))
            .interval(Duration::from_millis(100))
            .max_interval(Duration::from_secs(10))
            .connection_timeout(Duration::from_secs(5))
            .wait_for_any(false)
            .max_retries(Some(5))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.initial_interval, Duration::from_millis(100));
        assert_eq!(config.max_interval, Duration::from_secs(10));
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert!(!config.wait_for_any);
        assert_eq!(config.max_retries, Some(5));
    }
}
