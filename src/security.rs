//! Security and robustness enhancements for network operations.
//!
//! This module provides security features including rate limiting,
//! request validation, and protection against common network security issues.

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::types::{Hostname, Target};
use crate::{Result, WaitForError};

// Type aliases to reduce complexity warnings
type RateLimitMap = HashMap<String, Vec<Instant>>;
type AllowedPorts = Option<Vec<u16>>;

/// Rate limiter to prevent excessive connection attempts
/// Uses `RwLock` for better read performance compared to `Mutex`
#[derive(Debug)]
pub struct RateLimiter {
    limits: RwLock<RateLimitMap>,
    max_requests_per_minute: u32,
    cleanup_interval: Duration,
    last_cleanup: AtomicU64, // Store as milliseconds since epoch
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(60) // 60 requests per minute by default
    }
}

// Clone implementation for RateLimiter
impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        let limits = self.limits.read().map_or_else(
            |_| {
                // If the lock is poisoned, create a new empty HashMap
                HashMap::new()
            },
            |guard| guard.clone(),
        );
        let now_millis = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(u128::from(u64::MAX)),
        )
        .unwrap_or(u64::MAX);

        Self {
            limits: RwLock::new(limits),
            max_requests_per_minute: self.max_requests_per_minute,
            cleanup_interval: self.cleanup_interval,
            last_cleanup: AtomicU64::new(now_millis),
        }
    }
}

impl RateLimiter {
    /// Create a new rate limiter with the specified requests per minute
    #[must_use]
    pub fn new(max_requests_per_minute: u32) -> Self {
        let now_millis = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(u128::from(u64::MAX)),
        )
        .unwrap_or(u64::MAX);

        Self {
            limits: RwLock::new(HashMap::new()),
            max_requests_per_minute,
            cleanup_interval: Duration::from_secs(300), // Clean up every 5 minutes
            last_cleanup: AtomicU64::new(now_millis),
        }
    }

    /// Check if a request to the given target is allowed
    ///
    /// # Errors
    ///
    /// Returns an error if the rate limit is exceeded or if internal lock operations fail
    pub fn check_rate_limit(&self, target: &Target) -> Result<()> {
        let key = Self::get_rate_limit_key(target);
        let now = Instant::now();

        // Clean up old entries periodically
        self.cleanup_if_needed(now);

        // Use write lock for modifying the limits - keep scope tight and drop early
        {
            let mut limits = self.limits.write().map_err(|_| {
                WaitForError::InvalidTarget(Cow::Borrowed("Rate limiter lock error"))
            })?;

            let requests = limits.entry(key).or_insert_with(Vec::new);

            // Remove requests older than 1 minute
            requests.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

            if requests.len() >= self.max_requests_per_minute as usize {
                return Err(WaitForError::RetryLimitExceeded {
                    limit: self.max_requests_per_minute,
                });
            }

            requests.push(now);
            // Explicitly drop the lock guard to satisfy clippy::significant_drop_tightening
            drop(limits);
        }
        Ok(())
    }

    fn get_rate_limit_key(target: &Target) -> String {
        match target {
            Target::Tcp { host, port } => format!(
                "tcp://{host}:{port}",
                host = host.as_str(),
                port = port.get()
            ),
            Target::Http { url, .. } => {
                format!(
                    "http://{host}:{port}",
                    host = url.host_str().unwrap_or("unknown"),
                    port = url.port().unwrap_or_else(|| if url.scheme() == "https" {
                        443
                    } else {
                        80
                    })
                )
            }
        }
    }

    fn cleanup_if_needed(&self, now: Instant) {
        let now_millis = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(u128::from(u64::MAX)),
        )
        .unwrap_or(u64::MAX);

        let last_cleanup_millis = self.last_cleanup.load(Ordering::Relaxed);
        let cleanup_interval_millis =
            u64::try_from(self.cleanup_interval.as_millis().min(u128::from(u64::MAX)))
                .unwrap_or(u64::MAX);

        if now_millis.saturating_sub(last_cleanup_millis) > cleanup_interval_millis {
            // Try to update the cleanup time atomically
            if self
                .last_cleanup
                .compare_exchange_weak(
                    last_cleanup_millis,
                    now_millis,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                // We won the race to do cleanup
                if let Ok(mut limits) = self.limits.write() {
                    limits.retain(|_, requests| {
                        requests.retain(|&time| now.duration_since(time) < Duration::from_secs(60));
                        !requests.is_empty()
                    });
                }
            }
        }
    }
}

/// Security validator for targets and configurations
#[derive(Debug, Clone)]
pub struct SecurityValidator {
    allow_private_ips: bool,
    allow_localhost: bool,
    allowed_ports: AllowedPorts,
    blocked_ports: Vec<u16>,
    max_hostname_length: usize,
    max_url_length: usize,
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self {
            allow_private_ips: true,
            allow_localhost: true,
            allowed_ports: None,
            blocked_ports: vec![
                22,   // SSH
                23,   // Telnet
                135,  // RPC
                445,  // SMB
                1433, // SQL Server
                3389, // RDP
                5432, // PostgreSQL (blocked by default for security)
                6379, // Redis (blocked by default for security)
            ],
            max_hostname_length: 253,
            max_url_length: 2048,
        }
    }
}

impl SecurityValidator {
    /// Create a new security validator with custom settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow or disallow private IP addresses
    #[must_use]
    pub const fn allow_private_ips(mut self, allow: bool) -> Self {
        self.allow_private_ips = allow;
        self
    }

    /// Allow or disallow localhost connections
    #[must_use]
    pub const fn allow_localhost(mut self, allow: bool) -> Self {
        self.allow_localhost = allow;
        self
    }

    /// Set allowed ports (if None, all ports except blocked are allowed)
    #[must_use]
    pub fn allowed_ports(mut self, ports: AllowedPorts) -> Self {
        self.allowed_ports = ports;
        self
    }

    /// Set blocked ports
    #[must_use]
    pub fn blocked_ports(mut self, ports: Vec<u16>) -> Self {
        self.blocked_ports = ports;
        self
    }

    /// Set maximum hostname length
    #[must_use]
    pub const fn max_hostname_length(mut self, length: usize) -> Self {
        self.max_hostname_length = length;
        self
    }

    /// Set maximum URL length
    #[must_use]
    pub const fn max_url_length(mut self, length: usize) -> Self {
        self.max_url_length = length;
        self
    }

    /// Validate a target against security rules
    ///
    /// # Errors
    ///
    /// Returns an error if the target fails any security validation checks
    pub fn validate_target(&self, target: &Target) -> Result<()> {
        match target {
            Target::Tcp { host, port } => {
                self.validate_hostname(host)?;
                self.validate_port(port.get())?;
            }
            Target::Http { url, .. } => {
                self.validate_url(url)?;
                if let Some(host) = url.host_str() {
                    let hostname = Hostname::new(host)?;
                    self.validate_hostname(&hostname)?;
                }
                if let Some(port) = url.port() {
                    self.validate_port(port)?;
                }
            }
        }
        Ok(())
    }

    fn validate_hostname(&self, hostname: &Hostname) -> Result<()> {
        let host_str = hostname.as_str();

        if host_str.len() > self.max_hostname_length {
            return Err(WaitForError::InvalidHostname(Cow::Owned(format!(
                "Hostname too long: {} > {}",
                host_str.len(),
                self.max_hostname_length
            ))));
        }

        if !self.allow_localhost && (host_str == "localhost" || host_str == "127.0.0.1") {
            return Err(WaitForError::InvalidHostname(Cow::Borrowed(
                "Localhost connections are not allowed",
            )));
        }

        if !self.allow_private_ips {
            if let Ok(ip) = host_str.parse::<IpAddr>() {
                if Self::is_private_ip(&ip) {
                    return Err(WaitForError::InvalidHostname(Cow::Borrowed(
                        "Private IP addresses are not allowed",
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_port(&self, port: u16) -> Result<()> {
        if self.blocked_ports.contains(&port) {
            return Err(WaitForError::InvalidPort(port));
        }

        if let Some(allowed) = &self.allowed_ports {
            if !allowed.contains(&port) {
                return Err(WaitForError::InvalidPort(port));
            }
        }

        Ok(())
    }

    fn validate_url(&self, url: &url::Url) -> Result<()> {
        let url_str = url.as_str();

        // Check URL length
        if url_str.len() > self.max_url_length {
            return Err(WaitForError::UrlParse(url::ParseError::IdnaError));
        }

        // Only allow HTTP and HTTPS
        if !matches!(url.scheme(), "http" | "https") {
            return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                "Unsupported URL scheme: {}",
                url.scheme()
            ))));
        }

        Ok(())
    }

    const fn is_private_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
                octets[0] == 10
                    || (octets[0] == 172 && (octets[1] & 0xf0) == 16)
                    || (octets[0] == 192 && octets[1] == 168)
                    || octets[0] == 127 // Loopback
            }
            IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified(),
        }
    }
}

/// Production-ready security configuration
impl SecurityValidator {
    /// Strict security configuration for production environments
    #[must_use]
    pub fn production() -> Self {
        Self {
            allow_private_ips: false,
            allow_localhost: false,
            allowed_ports: Some(vec![80, 443, 8080, 8443]), // Only web ports
            blocked_ports: vec![
                22, 23, 25, 53, 135, 139, 445, 993, 995, 1433, 1521, 3306, 3389, 5432, 6379,
            ],
            max_hostname_length: 100,
            max_url_length: 1024,
        }
    }

    /// Development-friendly security configuration
    #[must_use]
    pub fn development() -> Self {
        Self {
            allow_private_ips: true,
            allow_localhost: true,
            allowed_ports: None,
            blocked_ports: vec![22, 23, 135, 445, 3389], // Only block dangerous ports
            max_hostname_length: 253,
            max_url_length: 2048,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_normal_requests() {
        let limiter = RateLimiter::new(5);
        let target = Target::tcp("localhost", 8080).unwrap();

        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(&target).is_ok());
        }

        // Should block the 6th request
        assert!(limiter.check_rate_limit(&target).is_err());
    }

    #[test]
    fn test_security_validator_blocks_dangerous_ports() {
        let validator = SecurityValidator::production();
        let ssh_target = Target::tcp("example.com", 22).unwrap();

        assert!(validator.validate_target(&ssh_target).is_err());
    }

    #[test]
    fn test_security_validator_allows_web_ports() {
        let validator = SecurityValidator::production();
        let web_target = Target::tcp("example.com", 443).unwrap();

        assert!(validator.validate_target(&web_target).is_ok());
    }

    #[test]
    fn test_security_validator_blocks_private_ips_in_production() {
        let validator = SecurityValidator::production();
        let private_target = Target::tcp("192.168.1.1", 80).unwrap();

        assert!(validator.validate_target(&private_target).is_err());
    }

    #[test]
    fn test_security_validator_allows_private_ips_in_development() {
        let validator = SecurityValidator::development();
        let private_target = Target::tcp("192.168.1.1", 80).unwrap();

        assert!(validator.validate_target(&private_target).is_ok());
    }
}
