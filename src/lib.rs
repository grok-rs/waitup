//! A robust library for waiting until TCP ports, HTTP endpoints, and services become available.
//!
//! This library provides functionality for testing network connectivity and service availability,
//! with support for DNS resolution, exponential backoff, and multiple connection strategies.
//!
//! # Examples
//!
//! ## Basic TCP Connection Check
//!
//! ```rust
//! use wait_for::{Target, WaitConfig, WaitForError, wait_for_connection};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WaitForError> {
//!     let target = Target::tcp("localhost", 8080)?;
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(30))
//!         .interval(Duration::from_secs(1))
//!         .build();
//!
//!     wait_for_connection(&[target], &config).await?;
//!     println!("Service is ready!");
//!     Ok(())
//! }
//! ```
//!
//! ## HTTP Health Check
//!
//! ```rust
//! use wait_for::{Target, WaitConfig, WaitForError, wait_for_connection};
//! use std::time::Duration;
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WaitForError> {
//!     let url = Url::parse("https://api.example.com/health")?;
//!     let target = Target::http(url, 200)?;
//!     let config = WaitConfig::builder()
//!         .timeout(Duration::from_secs(60))
//!         .interval(Duration::from_secs(2))
//!         .wait_for_any(false)
//!         .build();
//!
//!     wait_for_connection(&[target], &config).await?;
//!     println!("API is healthy!");
//!     Ok(())
//! }
//! ```

use std::net::SocketAddr;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::net::{lookup_host, TcpStream};
use tokio::time::{sleep, timeout};
use url::Url;

/// Error types that can occur during wait operations.
#[derive(Error, Debug)]
pub enum WaitForError {
    #[error("Invalid target format '{0}': expected host:port or http(s)://host:port/path")]
    InvalidTarget(String),
    #[error("Invalid timeout format '{0}': {1}")]
    InvalidTimeout(String, String),
    #[error("Invalid interval format '{0}': {1}")]
    InvalidInterval(String, String),
    #[error("DNS resolution failed for '{0}': {1}")]
    DnsResolution(String, String),
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("Timeout waiting for {0}")]
    Timeout(String),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Result type alias for wait-for operations.
pub type Result<T> = std::result::Result<T, WaitForError>;

/// A target service to wait for.
#[derive(Debug, Clone)]
pub enum Target {
    /// TCP connection target with host and port.
    Tcp {
        /// The hostname or IP address
        host: String,
        /// The port number
        port: u16,
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

impl Target {
    /// Create a new TCP target.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wait_for::Target;
    ///
    /// let target = Target::tcp("localhost", 8080)?;
    /// # Ok::<(), wait_for::WaitForError>(())
    /// ```
    pub fn tcp(host: impl Into<String>, port: u16) -> Result<Self> {
        Ok(Target::Tcp {
            host: host.into(),
            port,
        })
    }

    /// Create a new HTTP target with expected status code 200.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wait_for::Target;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://api.example.com/health")?;
    /// let target = Target::http(url, 200)?;
    /// # Ok::<(), wait_for::WaitForError>(())
    /// ```
    pub fn http(url: Url, expected_status: u16) -> Result<Self> {
        Ok(Target::Http {
            url,
            expected_status,
            headers: None,
        })
    }

    /// Create a new HTTP target with custom headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wait_for::Target;
    /// use url::Url;
    ///
    /// let url = Url::parse("https://api.example.com/health")?;
    /// let headers = vec![("Authorization".to_string(), "Bearer token".to_string())];
    /// let target = Target::http_with_headers(url, 200, headers)?;
    /// # Ok::<(), wait_for::WaitForError>(())
    /// ```
    pub fn http_with_headers(
        url: Url,
        expected_status: u16,
        headers: Vec<(String, String)>,
    ) -> Result<Self> {
        Ok(Target::Http {
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
    /// # Examples
    ///
    /// ```rust
    /// use wait_for::Target;
    ///
    /// let tcp_target = Target::parse("localhost:8080", 200)?;
    /// let http_target = Target::parse("https://api.example.com/health", 200)?;
    /// # Ok::<(), wait_for::WaitForError>(())
    /// ```
    pub fn parse(target_str: &str, default_http_status: u16) -> Result<Self> {
        if target_str.starts_with("http://") || target_str.starts_with("https://") {
            let url = Url::parse(target_str)?;
            Ok(Target::Http {
                url,
                expected_status: default_http_status,
                headers: None,
            })
        } else {
            let parts: Vec<&str> = target_str.split(':').collect();
            if parts.len() != 2 {
                return Err(WaitForError::InvalidTarget(target_str.to_string()));
            }
            let host = parts[0].to_string();
            let port = parts[1].parse::<u16>()
                .map_err(|_| WaitForError::InvalidTarget(target_str.to_string()))?;
            Ok(Target::Tcp { host, port })
        }
    }

    /// Get a string representation of this target for display purposes.
    pub fn display(&self) -> String {
        match self {
            Target::Tcp { host, port } => format!("{}:{}", host, port),
            Target::Http { url, .. } => url.to_string(),
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
        }
    }
}

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

    /// Build the WaitConfig.
    pub fn build(self) -> WaitConfig {
        self.config
    }
}

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

/// Resolve a hostname and port to socket addresses.
async fn resolve_host(host: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let host_port = format!("{}:{}", host, port);
    let addrs: Vec<SocketAddr> = lookup_host(&host_port)
        .await
        .map_err(|e| WaitForError::DnsResolution(host.to_string(), e.to_string()))?
        .collect();

    if addrs.is_empty() {
        return Err(WaitForError::DnsResolution(
            host.to_string(),
            "No addresses found".to_string(),
        ));
    }

    Ok(addrs)
}

/// Try to establish a TCP connection.
async fn try_tcp_connect(host: &str, port: u16, timeout_duration: Duration) -> Result<()> {
    let addrs = resolve_host(host, port).await?;

    for addr in addrs {
        if let Ok(_) = timeout(timeout_duration, TcpStream::connect(addr)).await {
            return Ok(());
        }
    }

    Err(WaitForError::Connection(format!("Failed to connect to {}:{}", host, port)))
}

/// Try to make an HTTP request and check the response.
async fn try_http_connect(
    url: &Url,
    expected_status: u16,
    headers: &Option<Vec<(String, String)>>,
    timeout_duration: Duration,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(timeout_duration)
        .build()
        .map_err(|e| WaitForError::Http(e.to_string()))?;

    let mut request = client.get(url.clone());

    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }

    let response = request
        .send()
        .await
        .map_err(|e| WaitForError::Http(e.to_string()))?;

    if response.status().as_u16() == expected_status {
        Ok(())
    } else {
        Err(WaitForError::Http(format!(
            "Expected status {}, got {}",
            expected_status,
            response.status()
        )))
    }
}

/// Try to connect to a target.
async fn try_connect_target(target: &Target, config: &WaitConfig) -> Result<()> {
    match target {
        Target::Tcp { host, port } => {
            try_tcp_connect(host, *port, config.connection_timeout).await
        }
        Target::Http { url, expected_status, headers } => {
            try_http_connect(url, *expected_status, headers, config.connection_timeout).await
        }
    }
}

/// Calculate the next retry interval using exponential backoff.
fn calculate_next_interval(current: Duration, max: Duration) -> Duration {
    let next = Duration::from_millis((current.as_millis() as f64 * 1.5) as u64);
    if next > max {
        max
    } else {
        next
    }
}

/// Wait for a single target to become available.
pub async fn wait_for_single_target(target: &Target, config: &WaitConfig) -> Result<TargetResult> {
    let start = Instant::now();
    let mut current_interval = config.initial_interval;
    let mut attempt = 0;

    loop {
        attempt += 1;

        match try_connect_target(target, config).await {
            Ok(()) => {
                return Ok(TargetResult {
                    target: target.clone(),
                    success: true,
                    elapsed: start.elapsed(),
                    attempts: attempt,
                    error: None,
                });
            }
            Err(e) => {
                if start.elapsed() >= config.timeout {
                    return Ok(TargetResult {
                        target: target.clone(),
                        success: false,
                        elapsed: start.elapsed(),
                        attempts: attempt,
                        error: Some(e.to_string()),
                    });
                }

                if let Some(max_retries) = config.max_retries {
                    if attempt >= max_retries {
                        return Ok(TargetResult {
                            target: target.clone(),
                            success: false,
                            elapsed: start.elapsed(),
                            attempts: attempt,
                            error: Some(format!("Max retries ({}) exceeded", max_retries)),
                        });
                    }
                }

                sleep(current_interval).await;
                current_interval = calculate_next_interval(current_interval, config.max_interval);
            }
        }
    }
}

/// Wait for connections to multiple targets.
///
/// This is the main function for waiting on multiple targets with different strategies.
///
/// # Examples
///
/// ```rust
/// use wait_for::{Target, WaitConfig, wait_for_connection};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), wait_for::WaitForError> {
///     let targets = vec![
///         Target::tcp("localhost", 8080)?,
///         Target::tcp("localhost", 5432)?,
///     ];
///
///     let config = WaitConfig::builder()
///         .timeout(Duration::from_secs(60))
///         .wait_for_any(false) // Wait for all targets
///         .build();
///
///     wait_for_connection(&targets, &config).await?;
///     println!("All services are ready!");
///     Ok(())
/// }
/// ```
pub async fn wait_for_connection(targets: &[Target], config: &WaitConfig) -> Result<WaitResult> {
    let start = Instant::now();

    if targets.is_empty() {
        return Ok(WaitResult {
            success: true,
            elapsed: start.elapsed(),
            attempts: 0,
            target_results: vec![],
        });
    }

    if config.wait_for_any {
        // Wait for any target to be ready
        use futures::future::select_ok;

        let futures: Vec<_> = targets.iter()
            .map(|target| Box::pin(wait_for_single_target(target, config)))
            .collect();

        match select_ok(futures).await {
            Ok((result, _)) => {
                Ok(WaitResult {
                    success: result.success,
                    elapsed: start.elapsed(),
                    attempts: result.attempts,
                    target_results: vec![result],
                })
            }
            Err(e) => Err(e),
        }
    } else {
        // Wait for all targets to be ready
        use futures::future::join_all;

        let futures: Vec<_> = targets.iter()
            .map(|target| wait_for_single_target(target, config))
            .collect();

        let results = join_all(futures).await;
        let mut target_results = Vec::new();
        let mut all_successful = true;
        let mut total_attempts = 0;

        for result in results {
            match result {
                Ok(target_result) => {
                    if !target_result.success {
                        all_successful = false;
                    }
                    total_attempts += target_result.attempts;
                    target_results.push(target_result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if !all_successful {
            let failed_targets: Vec<_> = target_results.iter()
                .filter(|r| !r.success)
                .map(|r| r.target.display())
                .collect();
            return Err(WaitForError::Timeout(format!(
                "Failed targets: {}",
                failed_targets.join(", ")
            )));
        }

        Ok(WaitResult {
            success: all_successful,
            elapsed: start.elapsed(),
            attempts: total_attempts,
            target_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_target_parse_tcp() {
        let target = Target::parse("localhost:8080", 200).unwrap();
        match target {
            Target::Tcp { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected TCP target"),
        }
    }

    #[test]
    fn test_target_parse_http() {
        let target = Target::parse("https://example.com/health", 200).unwrap();
        match target {
            Target::Http { url, expected_status, .. } => {
                assert_eq!(url.to_string(), "https://example.com/health");
                assert_eq!(expected_status, 200);
            }
            _ => panic!("Expected HTTP target"),
        }
    }

    #[test]
    fn test_target_display() {
        let tcp_target = Target::tcp("localhost", 8080).unwrap();
        assert_eq!(tcp_target.display(), "localhost:8080");

        let url = Url::parse("https://example.com/health").unwrap();
        let http_target = Target::http(url, 200).unwrap();
        assert_eq!(http_target.display(), "https://example.com/health");
    }

    #[test]
    fn test_wait_config_builder() {
        let config = WaitConfig::builder()
            .timeout(Duration::from_secs(60))
            .interval(Duration::from_secs(2))
            .max_interval(Duration::from_secs(30))
            .wait_for_any(true)
            .max_retries(Some(10))
            .build();

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(30));
        assert!(config.wait_for_any);
        assert_eq!(config.max_retries, Some(10));
    }

    #[test]
    fn test_calculate_next_interval() {
        let current = Duration::from_secs(1);
        let max = Duration::from_secs(30);

        let next = calculate_next_interval(current, max);
        assert_eq!(next, Duration::from_millis(1500));

        let large_current = Duration::from_secs(25);
        let next = calculate_next_interval(large_current, max);
        assert_eq!(next, max);
    }
}
