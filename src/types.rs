//! Type-safe wrappers for network targets and configuration.

use core::fmt;
use core::num::NonZeroU16;
use core::time::Duration;
use std::borrow::Cow;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::error_messages;

const MS_PER_MS: f64 = 1.0;
const MS_PER_SECOND: f64 = 1000.0;
const MS_PER_MINUTE: f64 = 60_000.0;
const MS_PER_HOUR: f64 = 3_600_000.0;

const MAX_HOSTNAME_LENGTH: usize = 253; // RFC 1035
const MAX_LABEL_LENGTH: usize = 63;

const LOCALHOST_HOSTNAME: &str = "localhost";
const LOOPBACK_V4: &str = "127.0.0.1";
const LOOPBACK_V6: &str = "::1";

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_INITIAL_INTERVAL_SECS: u64 = 1;
const DEFAULT_MAX_INTERVAL_SECS: u64 = 30;
const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;

/// HTTP headers as key-value pairs.
pub type HttpHeaders = Vec<(String, String)>;

/// Result type for batch target operations.
pub type TargetVecResult = crate::Result<Vec<Target>>;

/// Type-safe port number (1-65535).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Port(NonZeroU16);

impl Port {
    /// Create port if non-zero.
    #[must_use]
    #[inline]
    pub const fn new(port: u16) -> Option<Self> {
        match NonZeroU16::new(port) {
            Some(nz) => Some(Self(nz)),
            None => None,
        }
    }

    /// Create port from `NonZeroU16` (zero-cost).
    #[must_use]
    #[inline]
    pub const fn from_nonzero(port: NonZeroU16) -> Self {
        Self(port)
    }

    /// Create port without validation.
    ///
    /// # Panics
    ///
    /// Panics if port is zero.
    #[must_use]
    #[inline]
    #[allow(clippy::panic)]
    pub const fn new_unchecked(port: u16) -> Self {
        match NonZeroU16::new(port) {
            Some(nz) => Self(nz),
            None => panic!("Port::new_unchecked called with port 0"),
        }
    }

    /// Get port value.
    #[must_use]
    #[inline]
    pub const fn get(self) -> u16 {
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

impl TryFrom<NonZeroU16> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: NonZeroU16) -> crate::Result<Self> {
        Ok(Self(port))
    }
}

impl std::str::FromStr for Port {
    type Err = crate::WaitForError;

    fn from_str(s: &str) -> crate::Result<Self> {
        let port: u16 = s.parse().map_err(|_| crate::WaitForError::InvalidPort(0))?;
        Self::try_from(port)
    }
}

impl From<Port> for u16 {
    #[inline]
    fn from(port: Port) -> Self {
        port.0.get()
    }
}

impl From<Port> for NonZeroU16 {
    #[inline]
    fn from(port: Port) -> Self {
        port.0
    }
}

/// Validated hostname (RFC 1035).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hostname(Cow<'static, str>);

impl Hostname {
    /// Create and validate hostname.
    ///
    /// # Errors
    ///
    /// Returns error if hostname is invalid per RFC 1035.
    pub fn new(hostname: impl Into<String>) -> crate::Result<Self> {
        let hostname = hostname.into();
        Self::validate(&hostname)?;

        let cow = match hostname.as_str() {
            "localhost" => Cow::Borrowed(LOCALHOST_HOSTNAME),
            "127.0.0.1" => Cow::Borrowed(LOOPBACK_V4),
            "::1" => Cow::Borrowed(LOOPBACK_V6),
            _ => Cow::Owned(hostname),
        };

        Ok(Self(cow))
    }

    /// Create from static string.
    #[must_use]
    pub const fn from_static(hostname: &'static str) -> Self {
        Self(Cow::Borrowed(hostname))
    }

    fn validate_label(label: &str) -> crate::Result<()> {
        if label.is_empty() {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_EMPTY_LABEL,
            )));
        }
        if label.len() > MAX_LABEL_LENGTH {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_LABEL_TOO_LONG,
            )));
        }

        // Single-pass validation: check all constraints in one iteration
        let mut chars = label.chars();

        // Check first character
        let Some(first) = chars.next() else {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_EMPTY_LABEL,
            )));
        };

        if first == '-' {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_LABEL_INVALID_HYPHEN,
            )));
        }
        if !first.is_ascii_alphanumeric() {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_INVALID_CHARS,
            )));
        }

        // Check middle and last characters
        let mut last = first;
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '-' {
                return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                    error_messages::HOSTNAME_INVALID_CHARS,
                )));
            }
            last = c;
        }

        // Check last character isn't a hyphen
        if last == '-' {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::HOSTNAME_LABEL_INVALID_HYPHEN,
            )));
        }

        Ok(())
    }

    /// Validate a hostname according to RFC standards
    fn validate(hostname: &str) -> crate::Result<()> {
        if hostname.is_empty() {
            return Err(crate::WaitForError::InvalidHostname(Cow::Borrowed(
                error_messages::EMPTY_HOSTNAME,
            )));
        }

        if hostname.len() > MAX_HOSTNAME_LENGTH {
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
            Self::validate_label(label)?;
        }

        Ok(())
    }

    /// Localhost hostname.
    #[must_use]
    pub const fn localhost() -> Self {
        Self::from_static(LOCALHOST_HOSTNAME)
    }

    /// IPv4 loopback (127.0.0.1).
    #[must_use]
    pub const fn loopback() -> Self {
        Self::from_static(LOOPBACK_V4)
    }

    /// IPv6 loopback (`::1`).
    #[must_use]
    pub const fn loopback_v6() -> Self {
        Self::from_static(LOOPBACK_V6)
    }

    /// Wildcard address (0.0.0.0).
    #[must_use]
    pub const fn any() -> Self {
        Self::from_static("0.0.0.0")
    }

    /// Create from IPv4 address string.
    ///
    /// # Errors
    ///
    /// Returns error if IP address is invalid.
    pub fn ipv4(ip: impl AsRef<str>) -> crate::Result<Self> {
        let ip_str = ip.as_ref();
        ip_str.parse::<std::net::Ipv4Addr>().map_err(|_| {
            crate::WaitForError::InvalidHostname(Cow::Borrowed(error_messages::INVALID_IPV4_FORMAT))
        })?;
        Ok(Self(Cow::Owned(ip_str.to_string())))
    }

    /// Get as string slice.
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
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

/// Parse hostname from string (same as `TryFrom`<&str> but explicit)
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
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Hostname {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Connection failure modes.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ConnectionError {
    #[error("Failed to connect to {host}:{port} - {reason}")]
    TcpConnection {
        host: Cow<'static, str>,
        port: u16,
        #[source]
        reason: std::io::Error,
    },
    #[error("Connection timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("DNS resolution failed for {host}: {reason}")]
    DnsResolution {
        host: Cow<'static, str>,
        #[source]
        reason: std::io::Error,
    },
}

/// HTTP operation errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum HttpError {
    #[error("HTTP request failed for {url}: {reason}")]
    RequestFailed {
        url: Cow<'static, str>,
        #[source]
        reason: reqwest::Error,
    },
    #[error("Unexpected status code: expected {expected}, got {actual}")]
    UnexpectedStatus { expected: u16, actual: u16 },
    #[error("Invalid header: {header}")]
    InvalidHeader { header: Cow<'static, str> },
}

/// Network target to wait for (TCP or HTTP).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Target {
    Tcp {
        host: Hostname,
        port: Port,
    },
    Http {
        url: Url,
        expected_status: u16,
        headers: Option<HttpHeaders>,
    },
}

/// Target type discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    Tcp,
    Http,
}

impl Target {
    /// Get target type.
    #[must_use]
    pub const fn kind(&self) -> TargetKind {
        match self {
            Self::Tcp { .. } => TargetKind::Tcp,
            Self::Http { .. } => TargetKind::Http,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tcp { host, port } => write!(f, "{}:{}", host.as_str(), port.get()),
            Self::Http { url, .. } => write!(f, "{url}"),
        }
    }
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

/// Validated Duration wrapper with string parsing support
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatedDuration(Duration);

impl ValidatedDuration {
    /// Create a new validated duration
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Get the inner Duration
    #[must_use]
    pub const fn get(&self) -> Duration {
        self.0
    }

    /// Create from seconds
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// Create from milliseconds
    #[must_use]
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

        // Validate unit first (fail fast before parsing number)
        let multiplier = match unit_part {
            "ms" => MS_PER_MS,
            "s" => MS_PER_SECOND,
            "m" => MS_PER_MINUTE,
            "h" => MS_PER_HOUR,
            _ => {
                return Err(crate::WaitForError::InvalidTimeout(
                    Cow::Owned(s.to_string()),
                    Cow::Borrowed("Unknown time unit (use: ms, s, m, h)"),
                ));
            }
        };

        let number: f64 = number_part.parse().map_err(|_| {
            crate::WaitForError::InvalidTimeout(
                Cow::Owned(s.to_string()),
                Cow::Borrowed("Invalid number"),
            )
        })?;

        let duration = crate::utils::parse_duration_unit(number, multiplier, s)?;

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
        let total_ms =
            u64::try_from(self.0.as_millis().min(u128::from(u64::MAX))).unwrap_or(u64::MAX);

        if total_ms >= 3_600_000 {
            write!(f, "{hours}h", hours = total_ms / 3_600_000)
        } else if total_ms >= 60_000 {
            write!(f, "{minutes}m", minutes = total_ms / 60_000)
        } else if total_ms >= 1_000 {
            write!(f, "{seconds}s", seconds = total_ms / 1_000)
        } else {
            write!(f, "{total_ms}ms")
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
}

impl Default for WaitConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            initial_interval: Duration::from_secs(DEFAULT_INITIAL_INTERVAL_SECS),
            max_interval: Duration::from_secs(DEFAULT_MAX_INTERVAL_SECS),
            wait_for_any: false,
            max_retries: None,
            connection_timeout: Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS),
            cancellation_token: None,
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
        // Intentionally ignore cancellation_token
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
