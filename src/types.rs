//! Core type definitions for wait-for library.
//!
//! This module contains the fundamental types used throughout the wait-for library:
//! - [`Port`] and [`Hostname`] - NewType wrappers for type safety
//! - [`Target`] - Represents services to wait for (TCP or HTTP)
//! - [`WaitConfig`] - Configuration for wait operations
//! - [`WaitResult`] and [`TargetResult`] - Result types for wait operations
//! - Error types for different failure modes
//!
//! # Examples
//!
//! ## Creating type-safe network identifiers
//!
//! ```rust
//! use wait_for::{Port, Hostname};
//!
//! // Create a validated port
//! let port = Port::new(8080).expect("Valid port");
//! assert_eq!(port.get(), 8080);
//!
//! // Use port range validation
//! let http_port = Port::well_known(80).expect("HTTP is well-known");
//! let app_port = Port::registered(8080).expect("8080 is registered");
//! let ephemeral = Port::dynamic(49152).expect("49152 is dynamic");
//!
//! // Create validated hostnames
//! let hostname = Hostname::new("example.com").expect("Valid hostname");
//! let localhost = Hostname::localhost();
//! let ip = Hostname::ipv4("192.168.1.1").expect("Valid IPv4");
//! ```
//!
//! ## Defining targets
//!
//! ```rust
//! use wait_for::Target;
//! use url::Url;
//!
//! // TCP target
//! let tcp_target = Target::Tcp {
//!     host: wait_for::Hostname::new("database.example.com").unwrap(),
//!     port: wait_for::Port::new(5432).unwrap(),
//! };
//!
//! // HTTP target
//! let http_target = Target::Http {
//!     url: Url::parse("https://api.example.com/health").unwrap(),
//!     expected_status: 200,
//!     headers: Some(vec![("Authorization".to_string(), "Bearer token".to_string())]),
//! };
//! ```

use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroU16;
use std::time::Duration;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::error_messages;

/// NewType wrapper for ports to provide type safety
/// Uses NonZeroU16 internally to guarantee valid port numbers (1-65535)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Port(NonZeroU16);

impl Port {
    /// Create a new port, validating it's not zero
    pub const fn new(port: u16) -> Option<Self> {
        match NonZeroU16::new(port) {
            Some(nz) => Some(Self(nz)),
            None => None,
        }
    }

    /// Create a port from a standard well-known port number
    pub const fn well_known(port: u16) -> Option<Self> {
        if port == 0 || port > 1023 {
            None
        } else {
            Self::new(port)
        }
    }

    /// Create a port from a registered port number range
    pub const fn registered(port: u16) -> Option<Self> {
        if port < 1024 || port > 49151 {
            None
        } else {
            Self::new(port)
        }
    }

    /// Create a port from a dynamic/private port number range
    pub const fn dynamic(port: u16) -> Option<Self> {
        if port < 49152 { None } else { Self::new(port) }
    }

    /// Create a new port without validation (for known valid values)
    /// Only use this when you know the port is valid (not zero)
    pub const fn new_unchecked(port: u16) -> Self {
        match NonZeroU16::new(port) {
            Some(nz) => Self(nz),
            None => panic!("Port cannot be zero"),
        }
    }

    /// Common HTTP port (80)
    pub const fn http() -> Self {
        Self::new_unchecked(80)
    }

    /// Common HTTPS port (443)
    pub const fn https() -> Self {
        Self::new_unchecked(443)
    }

    /// Common SSH port (22)
    pub const fn ssh() -> Self {
        Self::new_unchecked(22)
    }

    /// Common PostgreSQL port (5432)
    pub const fn postgres() -> Self {
        Self::new_unchecked(5432)
    }

    /// Common MySQL port (3306)
    pub const fn mysql() -> Self {
        Self::new_unchecked(3306)
    }

    /// Common Redis port (6379)
    pub const fn redis() -> Self {
        Self::new_unchecked(6379)
    }

    /// Get the inner port value
    pub const fn get(&self) -> u16 {
        self.0.get()
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u16> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: u16) -> crate::Result<Self> {
        Self::new(port).ok_or_else(|| crate::WaitForError::InvalidPort(port))
    }
}

impl TryFrom<u32> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: u32) -> crate::Result<Self> {
        if port > u16::MAX as u32 {
            return Err(crate::WaitForError::InvalidPort(port as u16)); // Will be 0 due to overflow, which is caught
        }
        Self::try_from(port as u16)
    }
}

impl TryFrom<i32> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: i32) -> crate::Result<Self> {
        if port < 0 || port > u16::MAX as i32 {
            return Err(crate::WaitForError::InvalidPort(0)); // Use 0 to represent invalid
        }
        Self::try_from(port as u16)
    }
}

impl TryFrom<usize> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: usize) -> crate::Result<Self> {
        if port > u16::MAX as usize {
            return Err(crate::WaitForError::InvalidPort(0)); // Use 0 to represent invalid
        }
        Self::try_from(port as u16)
    }
}

impl TryFrom<NonZeroU16> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: NonZeroU16) -> crate::Result<Self> {
        Ok(Self(port))
    }
}

/// Parse port from string representations
impl std::str::FromStr for Port {
    type Err = crate::WaitForError;

    fn from_str(s: &str) -> crate::Result<Self> {
        let port: u16 = s.parse().map_err(|_| crate::WaitForError::InvalidPort(0))?;
        Self::try_from(port)
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> Self {
        port.0.get()
    }
}

impl From<Port> for NonZeroU16 {
    fn from(port: Port) -> Self {
        port.0
    }
}

/// NewType wrapper for hostnames to provide type safety
/// Uses Cow<'static, str> to avoid allocations for common static hostnames
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hostname(Cow<'static, str>);

impl Hostname {
    /// Create a new hostname with validation
    pub fn new(hostname: impl Into<String>) -> crate::Result<Self> {
        let hostname = hostname.into();
        Self::validate(&hostname)?;
        Ok(Self(Cow::Owned(hostname)))
    }

    /// Create a hostname from a static string (zero allocation)
    pub const fn from_static(hostname: &'static str) -> Self {
        Self(Cow::Borrowed(hostname))
    }

    /// Validate a hostname according to RFC standards
    fn validate(hostname: &str) -> crate::Result<()> {
        if hostname.is_empty() {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::EMPTY_HOSTNAME,
            )));
        }

        if hostname.len() > 253 {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_TOO_LONG,
            )));
        }

        if hostname.starts_with('-') || hostname.ends_with('-') {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_INVALID_HYPHEN,
            )));
        }

        for label in hostname.split('.') {
            if label.is_empty() {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::HOSTNAME_EMPTY_LABEL,
                )));
            }
            if label.len() > 63 {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::HOSTNAME_LABEL_TOO_LONG,
                )));
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::HOSTNAME_LABEL_INVALID_HYPHEN,
                )));
            }
            if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::HOSTNAME_INVALID_CHARS,
                )));
            }
        }

        Ok(())
    }

    /// Create hostname for localhost (zero allocation)
    pub const fn localhost() -> Self {
        Self::from_static("localhost")
    }

    /// Create hostname for IPv4 loopback (zero allocation)
    pub const fn loopback() -> Self {
        Self::from_static("127.0.0.1")
    }

    /// Create hostname for IPv6 loopback (zero allocation)
    pub const fn loopback_v6() -> Self {
        Self::from_static("::1")
    }

    /// Create hostname for wildcard/any address (zero allocation)
    pub const fn any() -> Self {
        Self::from_static("0.0.0.0")
    }

    /// Create hostname for an IPv4 address (validates format)
    pub fn ipv4(ip: impl AsRef<str>) -> crate::Result<Self> {
        let ip = ip.as_ref();
        // Basic IPv4 validation without allocating a vector
        let mut parts_count = 0;
        for part in ip.split('.') {
            parts_count += 1;
            if parts_count > 4 {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::INVALID_IPV4_FORMAT,
                )));
            }
            let _num: u8 = part.parse().map_err(|_| {
                crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::INVALID_IPV4_OCTET,
                ))
            })?;
            // _num is automatically validated to be 0-255 by u8 parsing
        }
        if parts_count != 4 {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::INVALID_IPV4_FORMAT,
            )));
        }
        Ok(Self(Cow::Owned(ip.to_string())))
    }

    /// Get the hostname as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// IPv4 loopback address (zero allocation)
    pub const fn ipv4_loopback() -> Self {
        Self::from_static("127.0.0.1")
    }

    /// IPv6 loopback address (zero allocation)
    pub const fn ipv6_loopback() -> Self {
        Self::from_static("::1")
    }

    /// Any IPv4 address (zero allocation)
    pub const fn ipv4_any() -> Self {
        Self::from_static("0.0.0.0")
    }

    /// Any IPv6 address (zero allocation)
    pub const fn ipv6_any() -> Self {
        Self::from_static("::")
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Hostname {
    type Error = crate::WaitForError;

    fn try_from(hostname: String) -> crate::Result<Self> {
        Self::new(hostname)
    }
}

impl TryFrom<&str> for Hostname {
    type Error = crate::WaitForError;

    fn try_from(hostname: &str) -> crate::Result<Self> {
        Self::new(hostname)
    }
}

/// Parse hostname from string (same as TryFrom<&str> but explicit)
impl std::str::FromStr for Hostname {
    type Err = crate::WaitForError;

    fn from_str(s: &str) -> crate::Result<Self> {
        Self::try_from(s)
    }
}

/// Additional conversions for Hostname
impl TryFrom<std::net::IpAddr> for Hostname {
    type Error = crate::WaitForError;

    fn try_from(ip: std::net::IpAddr) -> crate::Result<Self> {
        match ip {
            std::net::IpAddr::V4(ipv4) => Self::ipv4(ipv4.to_string()),
            std::net::IpAddr::V6(ipv6) => Self::new(ipv6.to_string()),
        }
    }
}

impl TryFrom<std::net::Ipv4Addr> for Hostname {
    type Error = crate::WaitForError;

    fn try_from(ip: std::net::Ipv4Addr) -> crate::Result<Self> {
        Self::ipv4(ip.to_string())
    }
}

impl TryFrom<std::net::Ipv6Addr> for Hostname {
    type Error = crate::WaitForError;

    fn try_from(ip: std::net::Ipv6Addr) -> crate::Result<Self> {
        Self::new(ip.to_string())
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Hostname {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Specific error types for different connection failure modes
#[derive(Error, Debug)]
pub enum ConnectionError {
    /// Failed to establish TCP connection to the target host and port
    #[error("Failed to connect to {host}:{port} - {reason}")]
    TcpConnection {
        /// The hostname or IP address that connection failed to
        host: Cow<'static, str>,
        /// The port number that connection failed to
        port: u16,
        #[source]
        /// The underlying I/O error that caused the connection failure
        reason: std::io::Error,
    },
    /// Connection attempt timed out before establishing a connection
    #[error("Connection timeout after {timeout_ms}ms")]
    Timeout {
        /// The timeout duration in milliseconds that was exceeded
        timeout_ms: u64,
    },
    /// Failed to resolve hostname to IP address via DNS
    #[error("DNS resolution failed for {host}: {reason}")]
    DnsResolution {
        /// The hostname that failed to resolve
        host: Cow<'static, str>,
        #[source]
        /// The underlying I/O error from DNS resolution
        reason: std::io::Error,
    },
}

/// Specific error types for HTTP operations
#[derive(Error, Debug)]
pub enum HttpError {
    /// HTTP request failed due to network or server error
    #[error("HTTP request failed for {url}: {reason}")]
    RequestFailed {
        /// The URL that the request failed to reach
        url: Cow<'static, str>,
        #[source]
        /// The underlying HTTP client error
        reason: reqwest::Error,
    },
    /// HTTP response returned unexpected status code
    #[error("Unexpected status code: expected {expected}, got {actual}")]
    UnexpectedStatus {
        /// The HTTP status code that was expected
        expected: u16,
        /// The actual HTTP status code received
        actual: u16,
    },
    /// Invalid HTTP header format or value
    #[error("Invalid header: {header}")]
    InvalidHeader {
        /// The header that was invalid
        header: Cow<'static, str>,
    },
}

/// A target service to wait for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// TCP connection target with host and port.
    Tcp {
        /// The hostname or IP address
        host: Hostname,
        /// The port number
        port: Port,
    },
    /// HTTP/HTTPS endpoint target.
    Http {
        /// The URL to check
        url: Url,
        /// Expected HTTP status code
        expected_status: u16,
        /// Optional custom headers
        headers: Option<Vec<(String, String)>>,
    },
}

impl TryFrom<&str> for Target {
    type Error = crate::WaitForError;

    fn try_from(target_str: &str) -> crate::Result<Self> {
        // Use the existing parse method with default status 200
        // Note: This will be implemented in target.rs as Target::parse
        Self::parse(target_str, 200)
    }
}

impl TryFrom<String> for Target {
    type Error = crate::WaitForError;

    fn try_from(target_str: String) -> crate::Result<Self> {
        Self::try_from(target_str.as_str())
    }
}

impl std::str::FromStr for Target {
    type Err = crate::WaitForError;

    fn from_str(s: &str) -> crate::Result<Self> {
        Self::try_from(s)
    }
}

/// Additional Target construction methods
impl Target {
    /// Try to create a TCP target from host and port
    pub fn try_tcp<H, P>(host: H, port: P) -> crate::Result<Self>
    where
        H: TryInto<Hostname>,
        P: TryInto<Port>,
        H::Error: Into<crate::WaitForError>,
        P::Error: Into<crate::WaitForError>,
    {
        let hostname = host.try_into().map_err(Into::into)?;
        let port = port.try_into().map_err(Into::into)?;
        Ok(Self::Tcp {
            host: hostname,
            port,
        })
    }

    /// Try to create an HTTP target from URL string
    pub fn try_http(url: impl AsRef<str>, expected_status: u16) -> crate::Result<Self> {
        let url = Url::parse(url.as_ref())?;
        Ok(Self::Http {
            url,
            expected_status,
            headers: None,
        })
    }
}

/// Validated Duration wrapper with string parsing support
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatedDuration(Duration);

impl ValidatedDuration {
    /// Create a new validated duration
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Get the inner Duration
    pub const fn get(&self) -> Duration {
        self.0
    }

    /// Create from seconds
    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Create from milliseconds
    pub const fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }
}

impl From<ValidatedDuration> for Duration {
    fn from(vd: ValidatedDuration) -> Self {
        vd.0
    }
}

impl TryFrom<Duration> for ValidatedDuration {
    type Error = crate::WaitForError;

    fn try_from(duration: Duration) -> crate::Result<Self> {
        // Could add validation here (e.g., max duration limits)
        Ok(Self(duration))
    }
}

/// Parse duration from string with support for common suffixes
/// Supports: "30s", "5m", "2h", "1000ms", etc.
impl std::str::FromStr for ValidatedDuration {
    type Err = crate::WaitForError;

    fn from_str(s: &str) -> crate::Result<Self> {
        let s = s.trim();

        if let Ok(secs) = s.parse::<u64>() {
            // Pure number interpreted as seconds
            return Ok(Self::from_secs(secs));
        }

        let (number_part, unit_part) =
            if let Some(pos) = s.find(|c: char| !c.is_ascii_digit() && c != '.') {
                s.split_at(pos)
            } else {
                return Err(crate::WaitForError::InvalidTimeout(
                    Cow::Owned(s.to_string()),
                    Cow::Borrowed("Invalid duration format"),
                ));
            };

        let number: f64 = number_part.parse().map_err(|_| {
            crate::WaitForError::InvalidTimeout(
                Cow::Owned(s.to_string()),
                Cow::Borrowed("Invalid number"),
            )
        })?;

        let duration = match unit_part {
            "ms" => Duration::from_millis((number * 1.0) as u64),
            "s" => Duration::from_millis((number * 1000.0) as u64),
            "m" => Duration::from_millis((number * 60_000.0) as u64),
            "h" => Duration::from_millis((number * 3_600_000.0) as u64),
            _ => {
                return Err(crate::WaitForError::InvalidTimeout(
                    Cow::Owned(s.to_string()),
                    Cow::Borrowed("Unknown time unit (use: ms, s, m, h)"),
                ));
            }
        };

        Ok(Self(duration))
    }
}

impl TryFrom<&str> for ValidatedDuration {
    type Error = crate::WaitForError;

    fn try_from(s: &str) -> crate::Result<Self> {
        s.parse()
    }
}

impl TryFrom<String> for ValidatedDuration {
    type Error = crate::WaitForError;

    fn try_from(s: String) -> crate::Result<Self> {
        s.as_str().parse()
    }
}

impl fmt::Display for ValidatedDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display in a human readable format
        let total_ms = self.0.as_millis() as u64;

        if total_ms >= 3_600_000 {
            write!(f, "{}h", total_ms / 3_600_000)
        } else if total_ms >= 60_000 {
            write!(f, "{}m", total_ms / 60_000)
        } else if total_ms >= 1_000 {
            write!(f, "{}s", total_ms / 1_000)
        } else {
            write!(f, "{}ms", total_ms)
        }
    }
}

/// Configuration for wait operations.
#[derive(Debug, Clone)]
pub struct WaitConfig {
    /// Total timeout for all wait operations.
    pub timeout: Duration,
    /// Initial retry interval.
    pub initial_interval: Duration,
    /// Maximum retry interval for exponential backoff.
    pub max_interval: Duration,
    /// If true, wait for any target to be ready. If false, wait for all targets.
    pub wait_for_any: bool,
    /// Maximum number of retry attempts (None for unlimited).
    pub max_retries: Option<u32>,
    /// Individual connection timeout.
    pub connection_timeout: Duration,
    /// Cancellation token for graceful shutdown.
    pub cancellation_token: Option<CancellationToken>,
    /// Security validator for targets (None to skip validation).
    pub security_validator: Option<crate::security::SecurityValidator>,
    /// Rate limiter for connection attempts (None to disable rate limiting).
    pub rate_limiter: Option<crate::security::RateLimiter>,
}

impl Default for WaitConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(30),
            wait_for_any: false,
            max_retries: None,
            connection_timeout: Duration::from_secs(10),
            cancellation_token: None,
            security_validator: None,
            rate_limiter: None,
        }
    }
}

// Implement common duration conversions for convenience
impl From<Duration> for WaitConfig {
    fn from(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Default::default()
        }
    }
}

// Custom PartialEq implementation that ignores runtime fields
impl PartialEq for WaitConfig {
    fn eq(&self, other: &Self) -> bool {
        self.timeout == other.timeout
            && self.initial_interval == other.initial_interval
            && self.max_interval == other.max_interval
            && self.wait_for_any == other.wait_for_any
            && self.max_retries == other.max_retries
            && self.connection_timeout == other.connection_timeout
        // Intentionally ignore cancellation_token, security_validator, and rate_limiter
        // as they don't implement PartialEq or are runtime-specific
    }
}

impl Eq for WaitConfig {}

/// Information about a wait operation result.
#[derive(Debug, Clone)]
pub struct WaitResult {
    /// Whether the operation was successful.
    pub success: bool,
    /// Time elapsed during the operation.
    pub elapsed: Duration,
    /// Number of attempts made.
    pub attempts: u32,
    /// Results for each target.
    pub target_results: Vec<TargetResult>,
}

/// Result for an individual target.
#[derive(Debug, Clone)]
pub struct TargetResult {
    /// The target that was tested.
    pub target: Target,
    /// Whether this target was successful.
    pub success: bool,
    /// Time elapsed for this target.
    pub elapsed: Duration,
    /// Number of attempts for this target.
    pub attempts: u32,
    /// Error message if unsuccessful.
    pub error: Option<String>,
}
