//! Common presets and configurations for typical use cases.

use std::time::Duration;
use crate::{WaitConfig, Target};

/// Preset configurations for common scenarios
impl WaitConfig {
    /// Fast configuration for local development (short timeouts, quick polling)
    pub fn local_dev() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(1),
            connection_timeout: Duration::from_secs(2),
            wait_for_any: false,
            max_retries: Some(50),
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::development()),
            rate_limiter: Some(crate::security::RateLimiter::new(120)),
        }
    }

    /// Configuration for CI/CD environments (moderate timeouts)
    pub fn ci_cd() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(5),
            connection_timeout: Duration::from_secs(10),
            wait_for_any: false,
            max_retries: Some(30),
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::development()),
            rate_limiter: Some(crate::security::RateLimiter::new(60)),
        }
    }

    /// Configuration for Docker container startup (longer timeouts)
    pub fn docker() -> Self {
        Self {
            timeout: Duration::from_secs(300), // 5 minutes
            initial_interval: Duration::from_secs(2),
            max_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(15),
            wait_for_any: false,
            max_retries: None, // No limit for Docker startup
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::development()),
            rate_limiter: Some(crate::security::RateLimiter::new(60)),
        }
    }

    /// Configuration for production health checks (conservative timeouts with security)
    pub fn production() -> Self {
        Self {
            timeout: Duration::from_secs(120),
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(30),
            wait_for_any: false,
            max_retries: Some(20),
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::production()),
            rate_limiter: Some(crate::security::RateLimiter::new(30)),
        }
    }

    /// Configuration for microservices readiness checks
    pub fn microservices() -> Self {
        Self {
            timeout: Duration::from_secs(90),
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(10),
            connection_timeout: Duration::from_secs(5),
            wait_for_any: false,
            max_retries: Some(40),
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::development()),
            rate_limiter: Some(crate::security::RateLimiter::new(60)),
        }
    }

    /// Configuration for external service dependency checks
    pub fn external_services() -> Self {
        Self {
            timeout: Duration::from_secs(180), // 3 minutes
            initial_interval: Duration::from_secs(5),
            max_interval: Duration::from_secs(60),
            connection_timeout: Duration::from_secs(30),
            wait_for_any: false,
            max_retries: Some(15),
            cancellation_token: None,
            security_validator: Some(crate::security::SecurityValidator::production()),
            rate_limiter: Some(crate::security::RateLimiter::new(20)),
        }
    }
}

/// Common target patterns
impl Target {
    /// Common database targets
    pub fn database_targets() -> crate::Result<Vec<Target>> {
        Ok(vec![
            Target::tcp("postgres", 5432)?,
            Target::tcp("mysql", 3306)?,
            Target::tcp("mongodb", 27017)?,
            Target::tcp("redis", 6379)?,
        ])
    }

    /// Common web service targets
    pub fn web_service_targets() -> crate::Result<Vec<Target>> {
        Ok(vec![
            Target::tcp("web", 80)?,
            Target::tcp("api", 8080)?,
            Target::http_url("http://web/health", 200)?,
            Target::http_url("http://api:8080/health", 200)?,
        ])
    }

    /// Elasticsearch cluster targets
    pub fn elasticsearch_targets() -> crate::Result<Vec<Target>> {
        Ok(vec![
            Target::tcp("elasticsearch", 9200)?,
            Target::tcp("kibana", 5601)?,
            Target::http_url("http://elasticsearch:9200/_cluster/health", 200)?,
        ])
    }

    /// Message queue targets
    pub fn message_queue_targets() -> crate::Result<Vec<Target>> {
        Ok(vec![
            Target::tcp("rabbitmq", 5672)?,
            Target::tcp("kafka", 9092)?,
            Target::tcp("nats", 4222)?,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_configs() {
        let local = WaitConfig::local_dev();
        assert_eq!(local.timeout, Duration::from_secs(10));
        assert_eq!(local.initial_interval, Duration::from_millis(100));

        let docker = WaitConfig::docker();
        assert_eq!(docker.timeout, Duration::from_secs(300));

        let production = WaitConfig::production();
        assert_eq!(production.timeout, Duration::from_secs(120));
    }

    #[test]
    fn test_common_targets() {
        let db_targets = Target::database_targets().unwrap();
        assert_eq!(db_targets.len(), 4);

        let web_targets = Target::web_service_targets().unwrap();
        assert_eq!(web_targets.len(), 4);
    }
}
