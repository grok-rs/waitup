#![allow(
    clippy::pub_with_shorthand,
    clippy::pub_without_shorthand,
    reason = "restriction lints have contradictory pub visibility rules"
)]

//! Error types and handling for waitup operations.
//!
//! This module provides comprehensive error handling for the waitup library,
//! including structured error types, error context, and helpful error messages.
//!
//! # Error Types
//!
//! - [`WaitForError`] - Main error enum covering all failure modes
//! - [`ConnectionError`] - Specific TCP connection failures
//! - [`HttpError`] - HTTP request and validation failures
//! - [`ResultExt`] - Extension trait for adding context to errors
//!
//! # Error Context
//!
//! The [`ResultExt`] trait allows adding contextual information to errors:
//!
//! ```rust
//! use waitup::{Target, ResultExt};
//!
//! let result = Target::tcp("invalid-host", 0)
//!     .context("Failed to create database target");
//! ```
//!
//! # Examples
//!
//! ## Handling different error types
//!
//! ```rust
//! use waitup::{WaitForError, ConnectionError, HttpError};
//!
//! fn handle_error(err: WaitForError) {
//!     match err {
//!         WaitForError::Connection(ConnectionError::TcpConnection { host, port, reason }) => {
//!             eprintln!("Failed to connect to {}:{} - {}", host, port, reason);
//!         }
//!         WaitForError::Http(HttpError::UnexpectedStatus { expected, actual }) => {
//!             eprintln!("HTTP error: expected status {}, got {}", expected, actual);
//!         }
//!         WaitForError::Timeout { targets } => {
//!             eprintln!("Timeout waiting for: {}", targets);
//!         }
//!         WaitForError::Cancelled => {
//!             eprintln!("Operation was cancelled");
//!         }
//!         _ => {
//!             eprintln!("Other error: {}", err);
//!         }
//!     }
//! }
//! ```
//!
//! ## Adding context to operations
//!
//! ```rust
//! use waitup::{Target, WaitConfig, wait_for_connection, ResultExt};
//! use std::time::Duration;
//!
//! async fn wait_for_services() -> Result<(), waitup::WaitForError> {
//!     let targets = vec![
//!         Target::tcp("database", 5432)
//!             .context("Database target creation failed")?,
//!         Target::tcp("cache", 6379)
//!             .context("Cache target creation failed")?,
//!     ];
//!
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .build();
//!
//!     wait_for_connection(&targets, &config)
//!         .await
//!         .context("Service readiness check failed")?;
//!
//!     Ok(())
//! }
//! ```

use std::borrow::Cow;
use thiserror::Error;

use crate::types::{ConnectionError, HttpError};

/// Core error source types for proper error chaining without Box
#[derive(Error, Debug)]
pub enum ErrorSource {
    /// Connection-related errors (TCP connection failures, DNS resolution, etc.)
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    /// HTTP-related errors (request failures, unexpected status codes, etc.)
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    /// URL parsing errors when target format is invalid
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    /// Low-level I/O errors from the operating system
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Error types that can occur during wait operations.
#[derive(Error, Debug)]
pub enum WaitForError {
    /// Invalid target format provided (must be host:port or http(s)://host:port/path)
    #[error("Invalid target format '{0}': expected host:port or http(s)://host:port/path")]
    InvalidTarget(Cow<'static, str>),
    /// Port number is out of valid range (1-65535)
    #[error("Invalid port: {0} (must be 1-65535)")]
    InvalidPort(u16),
    /// Hostname format is invalid or contains illegal characters
    #[error("Invalid hostname: {0}")]
    InvalidHostname(Cow<'static, str>),
    /// Timeout format is invalid (expected formats: 30s, 5m, 1h30m, etc.)
    #[error("Invalid timeout format '{0}': {1}")]
    InvalidTimeout(Cow<'static, str>, Cow<'static, str>),
    /// Interval format is invalid (expected formats: 30s, 5m, 1h30m, etc.)
    #[error("Invalid interval format '{0}': {1}")]
    InvalidInterval(Cow<'static, str>, Cow<'static, str>),
    /// Connection-related errors (TCP connection failures, DNS resolution, etc.)
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    /// HTTP-related errors (request failures, unexpected status codes, etc.)
    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),
    /// Timeout occurred while waiting for targets to become available
    #[error("Timeout waiting for {targets}")]
    Timeout {
        /// List of targets that were being waited for
        targets: Cow<'static, str>,
    },
    /// URL parsing errors when target format is invalid
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    /// Maximum number of retry attempts has been exceeded
    #[error("Retry limit exceeded: {limit} attempts")]
    RetryLimitExceeded {
        /// The retry limit that was exceeded
        limit: u32,
    },
    /// Error with additional context information
    #[error("{message}: {source}")]
    WithContext {
        /// Contextual message describing the operation that failed
        message: Cow<'static, str>,
        #[source]
        /// The underlying error that occurred
        source: ErrorSource,
    },
    /// Operation was cancelled (typically by user interrupt)
    #[error("Operation was cancelled")]
    Cancelled,
}

/// Result type alias for waitup operations.
pub type Result<T> = std::result::Result<T, WaitForError>;

// Convenient From implementations for error types
impl From<&'static str> for WaitForError {
    fn from(msg: &'static str) -> Self {
        Self::InvalidTarget(Cow::Borrowed(msg))
    }
}

impl From<String> for WaitForError {
    fn from(msg: String) -> Self {
        Self::InvalidTarget(Cow::Owned(msg))
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    ///
    /// # Errors
    ///
    /// Returns the original error with additional context information
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Add static context to an error
    ///
    /// # Errors
    ///
    /// Returns the original error with additional static context information
    fn context(self, msg: &'static str) -> Result<T>;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<ErrorSource>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| WaitForError::WithContext {
            message: Cow::Owned(f()),
            source: e.into(),
        })
    }

    fn context(self, msg: &'static str) -> Result<T> {
        self.map_err(|e| WaitForError::WithContext {
            message: Cow::Borrowed(msg),
            source: e.into(),
        })
    }
}

/// Special `ResultExt` implementation for errors that are already `WaitForError`
/// This handles the case where we want to add context to a `WaitForError`
impl<T> ResultExt<T> for std::result::Result<T, WaitForError> {
    fn with_context<F>(self, f: F) -> Self
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            // Convert WaitForError to ErrorSource where possible, or keep as-is
            match e {
                WaitForError::Connection(conn_err) => WaitForError::WithContext {
                    message: Cow::Owned(f()),
                    source: ErrorSource::Connection(conn_err),
                },
                WaitForError::Http(http_err) => WaitForError::WithContext {
                    message: Cow::Owned(f()),
                    source: ErrorSource::Http(http_err),
                },
                WaitForError::UrlParse(url_err) => WaitForError::WithContext {
                    message: Cow::Owned(f()),
                    source: ErrorSource::UrlParse(url_err),
                },
                // For other error types, we can't easily add context without Box
                // so we return the original error with a modified message
                other => {
                    let context_msg = f();
                    match other {
                        WaitForError::InvalidTarget(msg) => {
                            WaitForError::InvalidTarget(Cow::Owned(format!("{context_msg}: {msg}")))
                        }
                        WaitForError::InvalidHostname(msg) => WaitForError::InvalidHostname(
                            Cow::Owned(format!("{context_msg}: {msg}")),
                        ),
                        _ => other, // For complex cases, return as-is
                    }
                }
            }
        })
    }

    fn context(self, msg: &'static str) -> Self {
        self.map_err(|e| {
            // Convert WaitForError to ErrorSource where possible
            match e {
                WaitForError::Connection(conn_err) => WaitForError::WithContext {
                    message: Cow::Borrowed(msg),
                    source: ErrorSource::Connection(conn_err),
                },
                WaitForError::Http(http_err) => WaitForError::WithContext {
                    message: Cow::Borrowed(msg),
                    source: ErrorSource::Http(http_err),
                },
                WaitForError::UrlParse(url_err) => WaitForError::WithContext {
                    message: Cow::Borrowed(msg),
                    source: ErrorSource::UrlParse(url_err),
                },
                // For other error types, prepend the context message
                other => {
                    match other {
                        WaitForError::InvalidTarget(orig_msg) => {
                            WaitForError::InvalidTarget(Cow::Owned(format!("{msg}: {orig_msg}")))
                        }
                        WaitForError::InvalidHostname(orig_msg) => {
                            WaitForError::InvalidHostname(Cow::Owned(format!("{msg}: {orig_msg}")))
                        }
                        _ => other, // For complex cases, return as-is
                    }
                }
            }
        })
    }
}

/// Common error messages as constants to avoid allocations
pub(crate) mod error_messages {
    pub const EMPTY_HOSTNAME: &str = "Hostname cannot be empty";
    pub const HOSTNAME_TOO_LONG: &str = "Hostname too long (max 253 characters)";
    pub const HOSTNAME_INVALID_HYPHEN: &str = "Hostname cannot start or end with hyphen";
    pub const HOSTNAME_EMPTY_LABEL: &str = "Hostname labels cannot be empty";
    pub const HOSTNAME_LABEL_TOO_LONG: &str = "Hostname labels cannot exceed 63 characters";
    pub const HOSTNAME_LABEL_INVALID_HYPHEN: &str =
        "Hostname labels cannot start or end with hyphen";
    pub const HOSTNAME_INVALID_CHARS: &str = "Hostname contains invalid characters";
    pub const INVALID_IPV4_FORMAT: &str = "Invalid IPv4 format";
    pub const INVALID_IPV4_OCTET: &str = "Invalid IPv4 octet";
}
