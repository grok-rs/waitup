//! Target implementation and builders.
//!
//! This module provides implementations for creating and working with targets,
//! which represent services to wait for. Targets can be either TCP connections
//! or HTTP endpoints.
//!
//! # Features
//!
//! - **TCP Targets**: Direct socket connections to host:port
//! - **HTTP Targets**: HTTP/HTTPS requests with status code validation
//! - **Builder Pattern**: Fluent APIs for complex target configuration
//! - **Input Validation**: Comprehensive validation for all inputs
//! - **Parsing**: String-to-target parsing for CLI and config files
//!
//! # Examples
//!
//! ## Basic target creation
//!
//! ```rust
//! use waitup::Target;
//! use url::Url;
//!
//! // Create TCP targets
//! let db = Target::tcp("database.example.com", 5432)?;
//! let localhost_api = Target::localhost(8080)?;
//!
//! // Create HTTP targets
//! let health_check = Target::http_url("https://api.example.com/health", 200)?;
//! let status_page = Target::http(
//!     Url::parse("https://status.example.com")?,
//!     200
//! )?;
//! # Ok::<(), waitup::WaitForError>(())
//! ```
//!
//! ## Using builders for complex configurations
//!
//! ```rust
//! use waitup::Target;
//! use url::Url;
//!
//! // HTTP target with custom headers
//! let api_target = Target::http_builder(Url::parse("https://api.example.com/protected")?)
//!     .status(201)
//!     .auth_bearer("your-api-token")
//!     .content_type("application/json")
//!     .header("X-Custom-Header", "custom-value")
//!     .build()?;
//!
//! // TCP target with specific port type validation
//! let service = Target::tcp_builder("service.example.com")?
//!     .registered_port(8080)
//!     .build()?;
//! # Ok::<(), waitup::WaitForError>(())
//! ```
//!
//! ## Parsing targets from strings
//!
//! ```rust
//! use waitup::Target;
//!
//! // Parse various target formats
//! let tcp_target = Target::parse("localhost:8080", 200)?;
//! let http_target = Target::parse("https://example.com/health", 200)?;
//! let custom_port = Target::parse("api.example.com:3000", 200)?;
//! # Ok::<(), waitup::WaitForError>(())
//! ```

use std::borrow::Cow;
use url::Url;

use crate::types::{Hostname, Port, Target};
use crate::{Result, ResultExt, WaitForError};

impl Target {
    /// Create multiple TCP targets from a list of host:port pairs
    ///
    /// # Errors
    ///
    /// Returns an error if any hostname or port is invalid
    pub fn tcp_batch<I, S>(targets: I) -> crate::types::TargetVecResult
    where
        I: IntoIterator<Item = (S, u16)>,
        S: AsRef<str>,
    {
        targets
            .into_iter()
            .map(|(host, port)| Self::tcp(host.as_ref(), port))
            .collect()
    }

    /// Create multiple HTTP targets from a list of URLs
    ///
    /// # Errors
    ///
    /// Returns an error if any URL is invalid or cannot be parsed
    pub fn http_batch<I, S>(urls: I, default_status: u16) -> crate::types::TargetVecResult
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        urls.into_iter()
            .map(|url| Self::http_url(url.as_ref(), default_status))
            .collect()
    }
    /// Create a new TCP target.
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname is invalid or the port is out of range (1-65535)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::tcp("localhost", 8080)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn tcp(host: impl AsRef<str>, port: u16) -> Result<Self> {
        let hostname = Hostname::new(host.as_ref())
            .with_context(|| format!("Invalid hostname '{host}'", host = host.as_ref()))?;
        let port = Port::try_from(port).with_context(|| format!("Invalid port {port}"))?;
        Ok(Self::Tcp {
            host: hostname,
            port,
        })
    }

    /// Create a TCP target for localhost.
    ///
    /// # Errors
    ///
    /// Returns an error if the port is invalid (0 or > 65535)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::localhost(8080)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn localhost(port: u16) -> Result<Self> {
        let port = Port::try_from(port).with_context(|| format!("Invalid port {port}"))?;
        Ok(Self::Tcp {
            host: Hostname::localhost(),
            port,
        })
    }

    /// Create a TCP target for IPv4 loopback (127.0.0.1).
    ///
    /// # Errors
    ///
    /// Returns an error if the port is invalid (0 or > 65535)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::loopback(8080)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn loopback(port: u16) -> Result<Self> {
        let port = Port::try_from(port).with_context(|| format!("Invalid port {port}"))?;
        Ok(Self::Tcp {
            host: Hostname::loopback(),
            port,
        })
    }

    /// Create a TCP target for IPv6 loopback (`::1`).
    ///
    /// # Errors
    ///
    /// Returns an error if the port is invalid (0 or > 65535)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::loopback_v6(8080)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn loopback_v6(port: u16) -> Result<Self> {
        let port = Port::try_from(port).with_context(|| format!("Invalid port {port}"))?;
        Ok(Self::Tcp {
            host: Hostname::loopback_v6(),
            port,
        })
    }

    /// Create a TCP target with validated types (no additional validation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::{Target, Hostname, Port};
    ///
    /// let hostname = Hostname::localhost();
    /// let port = Port::new(8080).unwrap();
    /// let target = Target::from_parts(hostname, port);
    /// ```
    #[must_use]
    pub const fn from_parts(host: Hostname, port: Port) -> Self {
        Self::Tcp { host, port }
    }

    /// Create a new HTTP target with expected status code 200.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL scheme is not HTTP/HTTPS or if the status code is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://api.example.com/health")?;
    /// let target = Target::http(url, 200)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn http(url: Url, expected_status: u16) -> Result<Self> {
        Self::validate_http_config(&url, expected_status, None)?;
        Ok(Self::Http {
            url,
            expected_status,
            headers: None,
        })
    }

    /// Create a new HTTP target from a URL string.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed or if validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::http_url("https://api.example.com/health", 200)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn http_url(url: impl AsRef<str>, expected_status: u16) -> Result<Self> {
        let url = Url::parse(url.as_ref())
            .with_context(|| format!("Invalid URL: {url}", url = url.as_ref()))?;
        Self::http(url, expected_status)
    }

    /// Validate HTTP target configuration
    pub(crate) fn validate_http_config(
        url: &Url,
        expected_status: u16,
        headers: Option<&crate::types::HttpHeaders>,
    ) -> Result<()> {
        // Validate URL scheme
        if !matches!(url.scheme(), "http" | "https") {
            return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                "Unsupported URL scheme: {scheme}",
                scheme = url.scheme()
            ))));
        }

        // Validate status code
        if !(100..=599).contains(&expected_status) {
            return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                "Invalid HTTP status code: {expected_status}"
            ))));
        }

        // Validate headers if present
        if let Some(headers) = headers {
            for (key, value) in headers {
                if key.is_empty() {
                    return Err(WaitForError::InvalidTarget(Cow::Borrowed(
                        "HTTP header key cannot be empty",
                    )));
                }
                if value.is_empty() {
                    return Err(WaitForError::InvalidTarget(Cow::Borrowed(
                        "HTTP header value cannot be empty",
                    )));
                }
                // Basic header name validation (simplified)
                if !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
                {
                    return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                        "Invalid HTTP header name: {key}"
                    ))));
                }
            }
        }

        Ok(())
    }

    /// Create a new HTTP target with custom headers.
    ///
    /// # Errors
    ///
    /// Returns an error if URL validation fails or if headers are invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://api.example.com/health")?;
    /// let headers = vec![("Authorization".to_string(), "Bearer token".to_string())];
    /// let target = Target::http_with_headers(url, 200, headers)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn http_with_headers(
        url: Url,
        expected_status: u16,
        headers: crate::types::HttpHeaders,
    ) -> Result<Self> {
        Self::validate_http_config(&url, expected_status, Some(&headers))?;
        Ok(Self::Http {
            url,
            expected_status,
            headers: Some(headers),
        })
    }

    /// Parse a target from a string.
    ///
    /// Supports formats:
    /// - `host:port` for TCP targets
    /// - `http://host/path` or `https://host/path` for HTTP targets
    ///
    /// # Errors
    ///
    /// Returns an error if the string format is invalid or if parsing fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let tcp_target = Target::parse("localhost:8080", 200)?;
    /// let http_target = Target::parse("https://api.example.com/health", 200)?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn parse(target_str: &str, default_http_status: u16) -> Result<Self> {
        if target_str.starts_with("http://") || target_str.starts_with("https://") {
            let url = Url::parse(target_str)?;
            Ok(Self::Http {
                url,
                expected_status: default_http_status,
                headers: None,
            })
        } else {
            let parts: Vec<&str> = target_str.split(':').collect();
            if parts.len() != 2 {
                return Err(WaitForError::InvalidTarget(Cow::Owned(
                    target_str.to_string(),
                )));
            }
            let hostname = Hostname::try_from(parts[0]).with_context(|| {
                format!(
                    "Invalid hostname '{hostname}' in target '{target_str}'",
                    hostname = parts[0]
                )
            })?;
            let port = parts[1]
                .parse::<u16>()
                .map_err(|_| WaitForError::InvalidTarget(Cow::Owned(target_str.to_string())))
                .with_context(|| {
                    format!(
                        "Invalid port '{port}' in target '{target_str}'",
                        port = parts[1]
                    )
                })?;
            let port = Port::try_from(port)
                .with_context(|| format!("Port {port} out of valid range (1-65535)"))?;
            Ok(Self::Tcp {
                host: hostname,
                port,
            })
        }
    }

    /// Get a string representation of this target for display purposes.
    #[must_use]
    pub fn display(&self) -> String {
        crate::zero_cost::TargetDisplay::new(self).to_string()
    }

    /// Get the hostname for this target (useful for logging and grouping)
    #[must_use]
    pub fn hostname(&self) -> &str {
        match self {
            Self::Tcp { host, .. } => host.as_str(),
            Self::Http { url, .. } => url.host_str().unwrap_or("unknown"),
        }
    }

    /// Get the port for this target
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        match self {
            Self::Tcp { port, .. } => Some(port.get()),
            Self::Http { url, .. } => url.port(),
        }
    }

    /// Create a builder for HTTP targets
    #[must_use]
    pub const fn http_builder(url: Url) -> HttpTargetBuilder {
        HttpTargetBuilder::new(url)
    }

    /// Create a builder for TCP targets
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname is invalid
    pub fn tcp_builder(host: impl AsRef<str>) -> Result<TcpTargetBuilder> {
        let hostname = Hostname::new(host.as_ref())
            .with_context(|| format!("Invalid hostname '{host}'", host = host.as_ref()))?;
        Ok(TcpTargetBuilder::new(hostname))
    }
}

/// Builder for HTTP targets
#[derive(Debug)]
pub struct HttpTargetBuilder {
    url: Url,
    expected_status: u16,
    headers: crate::types::HttpHeaders,
}

impl HttpTargetBuilder {
    pub(crate) const fn new(url: Url) -> Self {
        Self {
            url,
            expected_status: 200,
            headers: Vec::new(),
        }
    }

    /// Set the expected HTTP status code
    #[must_use]
    pub const fn status(mut self, status: u16) -> Self {
        self.expected_status = status;
        self
    }

    /// Add a header
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Add multiple headers
    #[must_use]
    pub fn headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Set authorization header with Bearer token
    #[must_use]
    pub fn auth_bearer(self, token: impl AsRef<str>) -> Self {
        self.header(
            "Authorization",
            crate::lazy_format!("Bearer {}", token.as_ref()).to_string(),
        )
    }

    /// Set content type header
    #[must_use]
    pub fn content_type(self, content_type: impl Into<String>) -> Self {
        self.header("Content-Type", content_type)
    }

    /// Set user agent header
    #[must_use]
    pub fn user_agent(self, user_agent: impl Into<String>) -> Self {
        self.header("User-Agent", user_agent)
    }

    /// Build the HTTP target
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails
    pub fn build(self) -> Result<Target> {
        let headers = if self.headers.is_empty() {
            None
        } else {
            Some(self.headers)
        };
        Target::validate_http_config(&self.url, self.expected_status, headers.as_ref())?;
        Ok(Target::Http {
            url: self.url,
            expected_status: self.expected_status,
            headers,
        })
    }
}

/// Builder for TCP targets
#[derive(Debug)]
pub struct TcpTargetBuilder {
    host: Hostname,
    port: Option<Port>,
    port_validation_error: Option<WaitForError>,
}

impl TcpTargetBuilder {
    pub(crate) const fn new(host: Hostname) -> Self {
        Self {
            host,
            port: None,
            port_validation_error: None,
        }
    }

    /// Set the port
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        match Port::try_from(port) {
            Ok(p) => {
                self.port = Some(p);
                self.port_validation_error = None;
            }
            Err(e) => {
                self.port_validation_error = Some(e);
            }
        }
        self
    }

    /// Set a well-known port (0-1023)
    #[must_use]
    pub fn well_known_port(mut self, port: u16) -> Self {
        match Port::system_port(port) {
            Some(p) => {
                self.port = Some(p);
                self.port_validation_error = None;
            }
            None => {
                self.port_validation_error = Some(WaitForError::InvalidPort(port));
            }
        }
        self
    }

    /// Set a registered port (1024-49151)
    #[must_use]
    pub fn registered_port(mut self, port: u16) -> Self {
        match Port::user_port(port) {
            Some(p) => {
                self.port = Some(p);
                self.port_validation_error = None;
            }
            None => {
                self.port_validation_error = Some(WaitForError::InvalidPort(port));
            }
        }
        self
    }

    /// Set a dynamic port (49152-65535)
    #[must_use]
    pub fn dynamic_port(mut self, port: u16) -> Self {
        match Port::dynamic_port(port) {
            Some(p) => {
                self.port = Some(p);
                self.port_validation_error = None;
            }
            None => {
                self.port_validation_error = Some(WaitForError::InvalidPort(port));
            }
        }
        self
    }

    /// Build the TCP target
    ///
    /// # Errors
    ///
    /// Returns an error if no port was specified or if validation fails
    pub fn build(self) -> Result<Target> {
        // Check for validation errors first
        if let Some(error) = self.port_validation_error {
            return Err(error);
        }

        let port = self
            .port
            .ok_or_else(|| WaitForError::InvalidTarget(Cow::Borrowed("Port must be specified")))?;
        Ok(Target::Tcp {
            host: self.host,
            port,
        })
    }
}
