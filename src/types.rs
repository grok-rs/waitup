//! Core type definitions for waitup library.
//!
//! This module contains the fundamental types used throughout the waitup library:
//! - [`Port`] and [`Hostname`] - `NewType` wrappers for type safety
//! - [`StatusCode`] and [`RetryCount`] - Semantic wrappers for HTTP and retry logic
//! - [`Target`] - Represents services to wait for (TCP or HTTP)
//! - [`WaitConfig`] - Configuration for wait operations
//! - [`WaitResult`] and [`TargetResult`] - Result types for wait operations
//! - Error types for different failure modes
//!
//! # Why NewType Patterns?
//!
//! NewType wrappers provide several benefits that make code more readable and maintainable:
//!
//! ## 1. Type Safety - Prevents Bugs at Compile Time
//!
//! **Without NewTypes (Error-Prone):**
//! ```rust,ignore
//! // Easy to mix up parameters!
//! fn connect(port: u16, timeout: u16, status: u16) { }
//!
//! // Which one is which? Compiler can't help!
//! connect(200, 8080, 30);  // BUG: Wrong order!
//! ```
//!
//! **With NewTypes (Type-Safe):**
//! ```rust,ignore
//! use waitup::{Port, StatusCode};
//!
//! fn connect(port: Port, status: StatusCode) { }
//!
//! // Compiler error - can't mix types!
//! // connect(StatusCode::OK, Port::http());  // Won't compile!
//!
//! // Correct usage is clear and safe
//! connect(Port::http(), StatusCode::OK);  // ✓ Type-safe!
//! ```
//!
//! ## 2. Self-Documenting Code - Improves Readability
//!
//! **Without NewTypes (Unclear):**
//! ```rust,ignore
//! let result = check_service("localhost", 8080, 200, 5);
//! // What do these numbers mean? Need to check docs!
//! ```
//!
//! **With NewTypes (Self-Explanatory):**
//! ```rust,no_run
//! use waitup::{Port, StatusCode, RetryCount, Hostname};
//!
//! let hostname = Hostname::localhost();
//! let port = Port::new(8080).unwrap();
//! let expected = StatusCode::OK;
//! let retries = RetryCount::MODERATE;
//!
//! // Crystal clear what each parameter represents!
//! // (Note: This is illustrative - actual API differs)
//! ```
//!
//! ## 3. Domain Knowledge Built-In
//!
//! **Without NewTypes:**
//! ```rust,ignore
//! if port <= 1023 {
//!     println!("Needs root privileges");
//! }
//! if status >= 200 && status < 300 {
//!     println!("Success!");
//! }
//! ```
//!
//! **With NewTypes (Expressive):**
//! ```rust
//! use waitup::{Port, StatusCode};
//!
//! let port = Port::http();
//! if port.is_system_port() {
//!     println!("Needs root privileges");
//! }
//!
//! let status = StatusCode::OK;
//! if status.is_success() {
//!     println!("Success!");
//! }
//! ```
//!
//! ## 4. Validation at Construction
//!
//! **Without NewTypes (Runtime Errors):**
//! ```rust,ignore
//! let port = 70000;  // Oops! Invalid port, but compiles fine
//! // Error happens later at runtime...
//! ```
//!
//! **With NewTypes (Compile-Time Safety):**
//! ```rust
//! use waitup::Port;
//!
//! // Validation happens at construction
//! let port = Port::new(70000);
//! assert!(port.is_none());  // ✓ Caught immediately!
//!
//! // Constants are compile-time validated
//! let http = Port::http();  // ✓ Always valid!
//! ```
//!
//! # Real-World Example
//!
//! Here's a complete example showing how newtypes make code more readable:
//!
//! ```rust,no_run
//! use waitup::{Port, Hostname, StatusCode, RetryCount, WaitConfig, Target};
//! use std::time::Duration;
//!
//! // Clear, self-documenting configuration
//! let config = WaitConfig::builder()
//!     .timeout(Duration::from_secs(30))
//!     .max_retries(Some(RetryCount::MODERATE.get()))
//!     .build();
//!
//! // Type-safe target construction
//! let database = Target::tcp(
//!     Hostname::localhost().as_str(),
//!     Port::postgres().get()
//! ).unwrap();
//!
//! println!("Connecting to {} on port {}",
//!     "localhost",
//!     if database.port().map(|p| Port::new(p).unwrap().is_system_port()).unwrap_or(false) {
//!         "system port"
//!     } else {
//!         "user port"
//!     }
//! );
//! ```
//!
//! # Examples
//!
//! ## Creating type-safe network identifiers
//!
//! ```rust
//! use waitup::{Port, Hostname};
//!
//! // Create validated ports
//! let port = Port::new(8080).expect("Valid port");
//! assert_eq!(port.get(), 8080);
//! assert!(port.is_user_port());
//!
//! // Use port range validation
//! let http_port = Port::http();
//! assert!(http_port.is_system_port());
//!
//! let app_port = Port::user_port(8080).expect("8080 is user port");
//! let ephemeral = Port::dynamic_port(49152).expect("49152 is dynamic");
//!
//! // Create validated hostnames
//! let hostname = Hostname::new("example.com").expect("Valid hostname");
//! let localhost = Hostname::localhost();
//! assert!(localhost.is_localhost());
//!
//! let ip = Hostname::ipv4("192.168.1.1").expect("Valid IPv4");
//! assert!(ip.is_ipv4());
//! ```
//!
//! ## Working with HTTP status codes
//!
//! ```rust
//! use waitup::StatusCode;
//!
//! // Use common status code constants
//! let ok = StatusCode::OK;
//! let not_found = StatusCode::NOT_FOUND;
//! let server_error = StatusCode::INTERNAL_SERVER_ERROR;
//!
//! // Check status code categories
//! assert!(ok.is_success());
//! assert!(not_found.is_client_error());
//! assert!(server_error.is_server_error());
//!
//! // Custom status codes with validation
//! let created = StatusCode::new(201).expect("Valid status");
//! assert!(created.is_success());
//! ```
//!
//! ## Managing retry logic
//!
//! ```rust
//! use waitup::RetryCount;
//!
//! // Use semantic retry count constants
//! let quick_fail = RetryCount::FEW;        // 3 retries
//! let balanced = RetryCount::MODERATE;     // 5 retries
//! let persistent = RetryCount::MANY;       // 10 retries
//!
//! // Custom retry counts
//! let custom = RetryCount::new(7);
//! assert_eq!(custom.get(), 7);
//!
//! // Unlimited retries
//! let unlimited = RetryCount::unlimited();
//! assert!(unlimited.is_none());
//! ```

use core::fmt;
use core::num::NonZeroU16;
use core::time::Duration;
use std::borrow::Cow;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::error_messages;

// Type aliases for complex types to improve readability
/// HTTP headers represented as a vector of key-value pairs
pub type HttpHeaders = Vec<(String, String)>;

/// Result type alias for functions returning a vector of targets
pub type TargetVecResult = crate::Result<Vec<Target>>;

/// `NewType` wrapper for ports to provide type safety
/// Uses `NonZeroU16` internally to guarantee valid port numbers (1-65535)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Port(NonZeroU16);

impl Port {
    /// Create a new port, validating it's not zero
    #[must_use]
    #[inline]
    pub const fn new(port: u16) -> Option<Self> {
        match NonZeroU16::new(port) {
            Some(nz) => Some(Self(nz)),
            None => None,
        }
    }

    /// Create a port from a system port number (0-1023)
    ///
    /// System Ports (per RFC 6335) are reserved for system services and require elevated privileges.
    #[must_use]
    #[inline]
    pub const fn system_port(port: u16) -> Option<Self> {
        if port == 0 || port > 1023 {
            None
        } else {
            Self::new(port)
        }
    }

    /// Create a port from a user port number range (1024-49151)
    ///
    /// User Ports (per RFC 6335) are assigned by IANA for user applications.
    #[must_use]
    #[inline]
    pub const fn user_port(port: u16) -> Option<Self> {
        if port < 1024 || port > 49151 {
            None
        } else {
            Self::new(port)
        }
    }

    /// Create a port from a dynamic port number range (49152-65535)
    ///
    /// Dynamic Ports (per RFC 6335) are used for temporary or private connections.
    #[must_use]
    #[inline]
    pub const fn dynamic_port(port: u16) -> Option<Self> {
        if port < 49152 { None } else { Self::new(port) }
    }

    /// Create a new port without validation (for known valid values)
    /// Only use this when you know the port is valid (not zero)
    ///
    /// This method is safe because it panics at compile-time if called
    /// with an invalid port number (like 0), preventing runtime errors.
    /// It's optimized for known valid port constants.
    #[must_use]
    #[inline]
    pub const fn new_unchecked(port: u16) -> Self {
        // SAFETY: This function is intended for compile-time known valid ports.
        // If called with 0, it will panic at compile time in const contexts,
        // or unwrap at runtime with a clear error message for debugging.
        match NonZeroU16::new(port) {
            Some(nz) => Self(nz),
            None => {
                // This will cause a compile-time error if used in const context with port = 0
                panic!("Port::new_unchecked called with invalid port 0");
            }
        }
    }

    /// Common HTTP port (80)
    #[must_use]
    pub const fn http() -> Self {
        Self::new_unchecked(80)
    }

    /// Common HTTPS port (443)
    #[must_use]
    pub const fn https() -> Self {
        Self::new_unchecked(443)
    }

    /// Common SSH port (22)
    #[must_use]
    pub const fn ssh() -> Self {
        Self::new_unchecked(22)
    }

    /// Common `PostgreSQL` port (5432)
    #[must_use]
    pub const fn postgres() -> Self {
        Self::new_unchecked(5432)
    }

    /// Common `MySQL` port (3306)
    #[must_use]
    pub const fn mysql() -> Self {
        Self::new_unchecked(3306)
    }

    /// Common Redis port (6379)
    #[must_use]
    pub const fn redis() -> Self {
        Self::new_unchecked(6379)
    }

    /// Get the inner port value
    #[must_use]
    #[inline(always)]
    pub const fn get(&self) -> u16 {
        self.0.get()
    }

    // Predicate methods for better readability

    /// Check if this port is a System Port (0-1023)
    ///
    /// System Ports (per RFC 6335) are reserved for system services and require elevated privileges.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Port;
    ///
    /// let http = Port::http();
    /// assert!(http.is_system_port());
    ///
    /// let app_port = Port::new(8080).unwrap();
    /// assert!(!app_port.is_system_port());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_system_port(&self) -> bool {
        self.0.get() <= 1023
    }

    /// Check if this port is a User Port (1024-49151)
    ///
    /// User Ports (per RFC 6335) are assigned by IANA for user applications.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Port;
    ///
    /// let app_port = Port::new(8080).unwrap();
    /// assert!(app_port.is_user_port());
    ///
    /// let http = Port::http();
    /// assert!(!http.is_user_port());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_user_port(&self) -> bool {
        let port = self.0.get();
        port >= 1024 && port <= 49151
    }

    /// Check if this port is a Dynamic Port (49152-65535)
    ///
    /// Dynamic Ports (per RFC 6335) are used for temporary or private connections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Port;
    ///
    /// let ephemeral = Port::new(50000).unwrap();
    /// assert!(ephemeral.is_dynamic_port());
    ///
    /// let http = Port::http();
    /// assert!(!http.is_dynamic_port());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_dynamic_port(&self) -> bool {
        self.0.get() >= 49152
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
        if port > u32::from(u16::MAX) {
            return Err(crate::WaitForError::InvalidPort(0)); // Use 0 to represent invalid port
        }
        Self::try_from(u16::try_from(port).unwrap_or(0))
    }
}

impl TryFrom<i32> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: i32) -> crate::Result<Self> {
        if port < 0 || port > i32::from(u16::MAX) {
            return Err(crate::WaitForError::InvalidPort(0)); // Use 0 to represent invalid
        }
        Self::try_from(u16::try_from(port).unwrap_or(0))
    }
}

impl TryFrom<usize> for Port {
    type Error = crate::WaitForError;

    fn try_from(port: usize) -> crate::Result<Self> {
        if port > usize::from(u16::MAX) {
            return Err(crate::WaitForError::InvalidPort(0)); // Use 0 to represent invalid
        }
        Self::try_from(u16::try_from(port).unwrap_or(0))
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
    #[inline(always)]
    fn from(port: Port) -> Self {
        port.0.get()
    }
}

impl From<Port> for NonZeroU16 {
    #[inline(always)]
    fn from(port: Port) -> Self {
        port.0
    }
}

/// `NewType` wrapper for HTTP status codes to provide type safety and validation
///
/// This type ensures that HTTP status codes are always in the valid range (100-599)
/// and provides convenient constructors for common status codes.
///
/// # Examples
///
/// ```rust
/// use waitup::StatusCode;
///
/// // Common status codes
/// let ok = StatusCode::OK;
/// let not_found = StatusCode::NOT_FOUND;
/// let server_error = StatusCode::INTERNAL_SERVER_ERROR;
///
/// // Custom status code with validation
/// let custom = StatusCode::new(201).expect("Valid status code");
/// assert_eq!(custom.get(), 201);
///
/// // Invalid status codes are rejected
/// assert!(StatusCode::new(999).is_none());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StatusCode(u16);

impl StatusCode {
    /// Create a new status code with validation (must be 100-599)
    #[must_use]
    #[inline]
    pub const fn new(code: u16) -> Option<Self> {
        if code >= 100 && code <= 599 {
            Some(Self(code))
        } else {
            None
        }
    }

    /// Get the inner status code value
    #[must_use]
    #[inline(always)]
    pub const fn get(&self) -> u16 {
        self.0
    }

    /// Check if this is a successful status code (200-299)
    #[must_use]
    #[inline]
    pub const fn is_success(&self) -> bool {
        self.0 >= 200 && self.0 <= 299
    }

    /// Check if this is a redirection status code (300-399)
    #[must_use]
    #[inline]
    pub const fn is_redirection(&self) -> bool {
        self.0 >= 300 && self.0 <= 399
    }

    /// Check if this is a client error status code (400-499)
    #[must_use]
    #[inline]
    pub const fn is_client_error(&self) -> bool {
        self.0 >= 400 && self.0 <= 499
    }

    /// Check if this is a server error status code (500-599)
    #[must_use]
    #[inline]
    pub const fn is_server_error(&self) -> bool {
        self.0 >= 500 && self.0 <= 599
    }

    // Common HTTP status codes as constants

    /// HTTP 200 OK
    pub const OK: Self = Self(200);

    /// HTTP 201 Created
    pub const CREATED: Self = Self(201);

    /// HTTP 202 Accepted
    pub const ACCEPTED: Self = Self(202);

    /// HTTP 204 No Content
    pub const NO_CONTENT: Self = Self(204);

    /// HTTP 301 Moved Permanently
    pub const MOVED_PERMANENTLY: Self = Self(301);

    /// HTTP 302 Found
    pub const FOUND: Self = Self(302);

    /// HTTP 304 Not Modified
    pub const NOT_MODIFIED: Self = Self(304);

    /// HTTP 400 Bad Request
    pub const BAD_REQUEST: Self = Self(400);

    /// HTTP 401 Unauthorized
    pub const UNAUTHORIZED: Self = Self(401);

    /// HTTP 403 Forbidden
    pub const FORBIDDEN: Self = Self(403);

    /// HTTP 404 Not Found
    pub const NOT_FOUND: Self = Self(404);

    /// HTTP 500 Internal Server Error
    pub const INTERNAL_SERVER_ERROR: Self = Self(500);

    /// HTTP 502 Bad Gateway
    pub const BAD_GATEWAY: Self = Self(502);

    /// HTTP 503 Service Unavailable
    pub const SERVICE_UNAVAILABLE: Self = Self(503);
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = crate::WaitForError;

    fn try_from(code: u16) -> crate::Result<Self> {
        Self::new(code).ok_or_else(|| {
            crate::WaitForError::InvalidTarget(Cow::Owned(format!(
                "Invalid HTTP status code: {code} (must be 100-599)"
            )))
        })
    }
}

impl From<StatusCode> for u16 {
    #[inline(always)]
    fn from(code: StatusCode) -> Self {
        code.0
    }
}

/// `NewType` wrapper for retry counts to provide type safety
///
/// This type provides semantic meaning to retry counts and prevents
/// accidental mixing with other numeric types.
///
/// # Examples
///
/// ```rust
/// use waitup::RetryCount;
///
/// // Common retry patterns
/// let few_retries = RetryCount::FEW;      // 3 retries
/// let moderate = RetryCount::MODERATE;    // 5 retries
/// let many = RetryCount::MANY;            // 10 retries
///
/// // Custom retry count
/// let custom = RetryCount::new(7);
/// assert_eq!(custom.get(), 7);
///
/// // Unlimited retries (None represents unlimited)
/// let unlimited = RetryCount::unlimited();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RetryCount(u32);

impl RetryCount {
    /// Create a new retry count
    #[must_use]
    #[inline]
    pub const fn new(count: u32) -> Self {
        Self(count)
    }

    /// Get the inner retry count value
    #[must_use]
    #[inline(always)]
    pub const fn get(&self) -> u32 {
        self.0
    }

    /// Create an Option<RetryCount> representing unlimited retries
    #[must_use]
    #[inline]
    pub const fn unlimited() -> Option<Self> {
        None
    }

    // Common retry count patterns

    /// Very few retries (3) - for fast-failing operations
    pub const FEW: Self = Self(3);

    /// Moderate retries (5) - balanced approach
    pub const MODERATE: Self = Self(5);

    /// Many retries (10) - for critical operations
    pub const MANY: Self = Self(10);

    /// Aggressive retries (20) - for long-running services
    pub const AGGRESSIVE: Self = Self(20);
}

impl fmt::Display for RetryCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for RetryCount {
    #[inline(always)]
    fn from(count: u32) -> Self {
        Self(count)
    }
}

impl From<RetryCount> for u32 {
    #[inline(always)]
    fn from(count: RetryCount) -> Self {
        count.0
    }
}

/// `NewType` wrapper for hostnames to provide type safety
/// Uses Cow<'static, str> to avoid allocations for common static hostnames
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hostname(Cow<'static, str>);

impl Hostname {
    /// Create a new hostname with validation
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname is invalid or too long
    pub fn new(hostname: impl Into<String>) -> crate::Result<Self> {
        let hostname = hostname.into();
        Self::validate(&hostname)?;
        Ok(Self(Cow::Owned(hostname)))
    }

    /// Create a hostname from a static string (zero allocation)
    #[must_use]
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
    #[must_use]
    pub const fn localhost() -> Self {
        Self::from_static("localhost")
    }

    /// Create hostname for IPv4 loopback (zero allocation)
    #[must_use]
    pub const fn loopback() -> Self {
        Self::from_static("127.0.0.1")
    }

    /// Create hostname for IPv6 loopback (zero allocation)
    #[must_use]
    pub const fn loopback_v6() -> Self {
        Self::from_static("::1")
    }

    /// Create hostname for wildcard/any address (zero allocation)
    #[must_use]
    pub const fn any() -> Self {
        Self::from_static("0.0.0.0")
    }

    /// Create hostname for an IPv4 address (validates format)
    ///
    /// # Errors
    ///
    /// Returns an error if the IPv4 format is invalid
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
    #[must_use]
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Predicate methods for better readability

    /// Check if this hostname is "localhost"
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Hostname;
    ///
    /// let host = Hostname::localhost();
    /// assert!(host.is_localhost());
    ///
    /// let other = Hostname::new("example.com").unwrap();
    /// assert!(!other.is_localhost());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_localhost(&self) -> bool {
        self.0 == "localhost"
    }

    /// Check if this hostname is a loopback address (127.0.0.1 or ::1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Hostname;
    ///
    /// let ipv4 = Hostname::loopback();
    /// assert!(ipv4.is_loopback());
    ///
    /// let ipv6 = Hostname::loopback_v6();
    /// assert!(ipv6.is_loopback());
    ///
    /// let other = Hostname::new("example.com").unwrap();
    /// assert!(!other.is_loopback());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_loopback(&self) -> bool {
        self.0 == "127.0.0.1" || self.0 == "::1"
    }

    /// Check if this hostname looks like an IPv4 address
    ///
    /// This is a simple heuristic check, not a full validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Hostname;
    ///
    /// let ip = Hostname::ipv4("192.168.1.1").unwrap();
    /// assert!(ip.is_ipv4());
    ///
    /// let hostname = Hostname::new("example.com").unwrap();
    /// assert!(!hostname.is_ipv4());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_ipv4(&self) -> bool {
        // Simple heuristic: contains dots and all parts are digits
        let parts: Vec<&str> = self.0.split('.').collect();
        parts.len() == 4 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
    }

    /// Check if this hostname looks like an IPv6 address
    ///
    /// This is a simple heuristic check, not a full validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Hostname;
    ///
    /// let ipv6 = Hostname::loopback_v6();
    /// assert!(ipv6.is_ipv6());
    ///
    /// let hostname = Hostname::new("example.com").unwrap();
    /// assert!(!hostname.is_ipv6());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_ipv6(&self) -> bool {
        // Simple heuristic: contains colons
        self.0.contains(':')
    }

    /// IPv4 loopback address (zero allocation)
    #[must_use]
    pub const fn ipv4_loopback() -> Self {
        Self::from_static("127.0.0.1")
    }

    /// IPv6 loopback address (zero allocation)
    #[must_use]
    pub const fn ipv6_loopback() -> Self {
        Self::from_static("::1")
    }

    /// Any IPv4 address (zero allocation)
    #[must_use]
    pub const fn ipv4_any() -> Self {
        Self::from_static("0.0.0.0")
    }

    /// Any IPv6 address (zero allocation)
    #[must_use]
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
    #[inline(always)]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Hostname {
    #[inline(always)]
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
        headers: Option<HttpHeaders>,
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
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname or port conversion fails
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
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or is invalid
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

        let number: f64 = number_part.parse().map_err(|_| {
            crate::WaitForError::InvalidTimeout(
                Cow::Owned(s.to_string()),
                Cow::Borrowed("Invalid number"),
            )
        })?;

        let duration = match unit_part {
            "ms" => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "duration calculation requires f64"
                )]
                let millis = (number * 1.0).min(u64::MAX as f64);
                if millis < 0.0 {
                    return Err(crate::WaitForError::InvalidTimeout(
                        Cow::Owned(s.to_string()),
                        Cow::Borrowed("Duration cannot be negative"),
                    ));
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "safe cast after bounds check"
                )]
                Duration::from_millis(millis as u64)
            }
            "s" => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "duration calculation requires f64"
                )]
                let millis = (number * 1000.0).min(u64::MAX as f64);
                if millis < 0.0 {
                    return Err(crate::WaitForError::InvalidTimeout(
                        Cow::Owned(s.to_string()),
                        Cow::Borrowed("Duration cannot be negative"),
                    ));
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "safe cast after bounds check"
                )]
                Duration::from_millis(millis as u64)
            }
            "m" => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "duration calculation requires f64"
                )]
                let millis = (number * 60_000.0).min(u64::MAX as f64);
                if millis < 0.0 {
                    return Err(crate::WaitForError::InvalidTimeout(
                        Cow::Owned(s.to_string()),
                        Cow::Borrowed("Duration cannot be negative"),
                    ));
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "safe cast after bounds check"
                )]
                Duration::from_millis(millis as u64)
            }
            "h" => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "duration calculation requires f64"
                )]
                let millis = (number * 3_600_000.0).min(u64::MAX as f64);
                if millis < 0.0 {
                    return Err(crate::WaitForError::InvalidTimeout(
                        Cow::Owned(s.to_string()),
                        Cow::Borrowed("Duration cannot be negative"),
                    ));
                }
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "safe cast after bounds check"
                )]
                Duration::from_millis(millis as u64)
            }
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
