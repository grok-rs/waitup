//! Target builders for TCP and HTTP endpoints.

use std::borrow::Cow;
use url::Url;

use crate::types::{Hostname, Port, Target};
use crate::{Result, ResultExt, WaitForError};

// Constants for HTTP status code validation
/// Minimum valid HTTP status code
const MIN_HTTP_STATUS_CODE: u16 = 100;
/// Maximum valid HTTP status code
const MAX_HTTP_STATUS_CODE: u16 = 599;

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

    /// Create multiple TCP targets for different ports on the same host.
    ///
    /// This is a convenience method for the common pattern of checking multiple ports
    /// on a single host (e.g., a service cluster with multiple instances).
    ///
    /// # Errors
    ///
    /// Returns an error if the hostname is invalid or any port is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// // Check multiple ports on localhost
    /// let targets = Target::tcp_ports("localhost", &[8080, 8081, 8082])?;
    /// assert_eq!(targets.len(), 3);
    ///
    /// // Check multiple ports on a server
    /// let app_targets = Target::tcp_ports("api-server", &[8080, 8081, 8082])?;
    /// assert_eq!(app_targets.len(), 3);
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn tcp_ports(host: impl AsRef<str>, ports: &[u16]) -> crate::types::TargetVecResult {
        let hostname = Hostname::new(host.as_ref())
            .with_context(|| format!("Invalid hostname '{host}'", host = host.as_ref()))?;

        ports
            .iter()
            .map(|&port| {
                Port::try_from(port)
                    .map(|p| Self::Tcp {
                        host: hostname.clone(),
                        port: p,
                    })
                    .with_context(|| format!("Invalid port {port}"))
            })
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

    /// Internal helper to create a TCP target with a validated port.
    ///
    /// Reduces code duplication for localhost/loopback constructors.
    #[inline]
    fn tcp_with_hostname(hostname: Hostname, port: u16) -> Result<Self> {
        let port = Port::try_from(port)?;
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
    #[inline]
    pub fn localhost(port: u16) -> Result<Self> {
        Self::tcp_with_hostname(Hostname::localhost(), port)
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
    #[inline]
    pub fn loopback(port: u16) -> Result<Self> {
        Self::tcp_with_hostname(Hostname::loopback(), port)
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
    #[inline]
    pub fn loopback_v6(port: u16) -> Result<Self> {
        Self::tcp_with_hostname(Hostname::loopback_v6(), port)
    }

    /// Create an HTTP target for localhost with a custom port.
    ///
    /// # Errors
    ///
    /// Returns an error if the port is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    ///
    /// let target = Target::http_localhost(8080)?;
    /// // Equivalent to: Target::http_url("http://localhost:8080", 200)?
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    pub fn http_localhost(port: u16) -> Result<Self> {
        Self::http_url(format!("http://localhost:{port}"), 200)
    }

    /// Create a TCP target from already-validated hostname and port.
    ///
    /// This constructor performs no additional validation, assuming the provided
    /// `Hostname` and `Port` are already valid. This is safe because both types
    /// guarantee validity through their constructors.
    ///
    /// Use this when you have already-validated components and want to avoid
    /// redundant validation overhead. For parsing strings, use [`Target::tcp()`]
    /// or [`Target::parse()`] instead.
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

    /// Validate a single HTTP header key-value pair
    fn validate_header(key: &str, value: &str) -> Result<()> {
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

        // Basic header name validation (RFC 7230: field-name = token)
        if !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))
        {
            return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                "Invalid HTTP header name: {key}"
            ))));
        }
        Ok(())
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
        if !(MIN_HTTP_STATUS_CODE..=MAX_HTTP_STATUS_CODE).contains(&expected_status) {
            return Err(WaitForError::InvalidTarget(Cow::Owned(format!(
                "Invalid HTTP status code: {expected_status}"
            ))));
        }

        // Validate headers if present
        if let Some(headers) = headers {
            for (key, value) in headers {
                Self::validate_header(key, value)?;
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
        // Handle HTTP/HTTPS URLs early
        if target_str.starts_with("http://") || target_str.starts_with("https://") {
            let url = Url::parse(target_str)?;
            return Ok(Self::Http {
                url,
                expected_status: default_http_status,
                headers: None,
            });
        }

        // Parse TCP target (host:port)
        let (host_str, port_str) = target_str
            .split_once(':')
            .ok_or_else(|| WaitForError::InvalidTarget(Cow::Owned(target_str.to_string())))?;

        let hostname = Hostname::try_from(host_str)
            .with_context(|| format!("Invalid hostname '{host_str}' in target '{target_str}'"))?;

        let port_num = port_str
            .parse::<u16>()
            .map_err(|_| WaitForError::InvalidTarget(Cow::Owned(target_str.to_string())))
            .with_context(|| format!("Invalid port '{port_str}' in target '{target_str}'"))?;

        let port = Port::try_from(port_num)
            .with_context(|| format!("Port {port_num} out of valid range (1-65535)"))?;

        Ok(Self::Tcp {
            host: hostname,
            port,
        })
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
        // Optimize string concatenation to avoid format! macro overhead
        let auth_value = ["Bearer ", token.as_ref()].concat();
        self.header("Authorization", auth_value)
    }

    /// Set authorization header with Basic authentication
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    /// use url::Url;
    ///
    /// let target = Target::http_builder(Url::parse("https://api.example.com/data")?)
    ///     .basic_auth("user", "password")
    ///     .build()?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    #[must_use]
    pub fn basic_auth(self, username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        use base64::Engine;
        let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        let auth_value = ["Basic ", &encoded].concat();
        self.header("Authorization", auth_value)
    }

    /// Set Content-Type header to application/json
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    /// use url::Url;
    ///
    /// let target = Target::http_builder(Url::parse("https://api.example.com/data")?)
    ///     .json()
    ///     .build()?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    #[must_use]
    pub fn json(self) -> Self {
        self.header("Content-Type", "application/json")
    }

    /// Set Accept header to application/json
    ///
    /// # Examples
    ///
    /// ```rust
    /// use waitup::Target;
    /// use url::Url;
    ///
    /// let target = Target::http_builder(Url::parse("https://api.example.com/data")?)
    ///     .accept_json()
    ///     .build()?;
    /// # Ok::<(), waitup::WaitForError>(())
    /// ```
    #[must_use]
    pub fn accept_json(self) -> Self {
        self.header("Accept", "application/json")
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
