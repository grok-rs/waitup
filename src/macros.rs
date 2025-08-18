//! Convenience macros for creating targets and configurations.

/// Create TCP targets from a compact syntax.
///
/// Returns a `Result<Vec<Target>, WaitForError>` that contains either all valid targets
/// or the first error encountered.
///
/// # Examples
///
/// ```rust
/// use wait_for::tcp_targets;
///
/// let targets = tcp_targets![
///     "localhost" => 8080,
///     "database" => 5432,
///     "cache" => 6379,
/// ]?;
/// assert_eq!(targets.len(), 3);
/// # Ok::<(), wait_for::WaitForError>(())
/// ```
#[macro_export]
macro_rules! tcp_targets {
    ($($host:expr => $port:expr),* $(,)?) => {
        {
            let result = || -> $crate::Result<Vec<$crate::Target>> {
                let mut targets = Vec::new();
                $(
                    targets.push($crate::Target::tcp($host, $port)?);
                )*
                Ok(targets)
            };
            result()
        }
    };
}

/// Create HTTP targets from a compact syntax.
///
/// Returns a `Result<Vec<Target>, WaitForError>` that contains either all valid targets
/// or the first error encountered.
///
/// # Examples
///
/// ```rust
/// use wait_for::http_targets;
///
/// let targets = http_targets![
///     "https://api.example.com/health" => 200,
///     "http://localhost:8080/status" => 204,
/// ]?;
/// assert_eq!(targets.len(), 2);
/// # Ok::<(), wait_for::WaitForError>(())
/// ```
#[macro_export]
macro_rules! http_targets {
    ($($url:expr => $status:expr),* $(,)?) => {
        {
            let result = || -> $crate::Result<Vec<$crate::Target>> {
                let mut targets = Vec::new();
                $(
                    targets.push($crate::Target::http_url($url, $status)?);
                )*
                Ok(targets)
            };
            result()
        }
    };
}

/// Create a wait configuration with a compact syntax.
///
/// # Examples
///
/// ```rust
/// use wait_for::wait_config;
/// use std::time::Duration;
///
/// let config = wait_config! {
///     timeout: Duration::from_secs(30),
///     interval: Duration::from_millis(500),
///     wait_for_any: false,
/// };
/// ```
#[macro_export]
macro_rules! wait_config {
    (
        $(timeout: $timeout:expr,)?
        $(interval: $interval:expr,)?
        $(max_interval: $max_interval:expr,)?
        $(connection_timeout: $connection_timeout:expr,)?
        $(max_retries: $max_retries:expr,)?
        $(wait_for_any: $wait_for_any:expr,)?
    ) => {
        {
            let mut builder = $crate::WaitConfig::builder();
            $(builder = builder.timeout($timeout);)?
            $(builder = builder.interval($interval);)?
            $(builder = builder.max_interval($max_interval);)?
            $(builder = builder.connection_timeout($connection_timeout);)?
            $(builder = builder.max_retries($max_retries);)?
            $(builder = builder.wait_for_any($wait_for_any);)?
            builder.build()
        }
    };
}

/// Create common port configurations.
///
/// # Examples
///
/// ```rust
/// use wait_for::common_ports;
///
/// let ports = common_ports![http, https, ssh, postgres];
/// assert_eq!(ports.len(), 4);
/// ```
#[macro_export]
macro_rules! common_ports {
    ($($port_name:ident),* $(,)?) => {
        vec![
            $(
                $crate::Port::$port_name()
            ),*
        ]
    };
}

/// Check that all targets in a collection are ready within a timeout.
///
/// Returns a `Result<WaitResult, WaitForError>` instead of panicking.
///
/// # Examples
///
/// ```rust,no_run
/// use wait_for::{check_ready, Target, WaitConfig};
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), wait_for::WaitForError> {
/// let targets = vec![
///     Target::tcp("localhost", 8080)?,
/// ];
///
/// // Create config first to avoid temporary value issues
/// let config = WaitConfig::builder()
///     .timeout(Duration::from_secs(30))
///     .build();
/// let result = wait_for::wait_for_connection(&targets, &config).await?;
/// println!("All targets ready in {:?}", result.elapsed);
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! check_ready {
    ($targets:expr, timeout: $timeout:expr) => {
        {
            let config = $crate::WaitConfig::builder()
                .timeout($timeout)
                .build();
            $crate::wait_for_connection(&$targets, &config)
        }
    };
    ($targets:expr, $($config_field:ident: $config_value:expr),+ $(,)?) => {
        {
            let config = $crate::wait_config! {
                $($config_field: $config_value,)+
            };
            $crate::wait_for_connection(&$targets, &config)
        }
    };
}

/// Assert that all targets in a collection are ready within a timeout (panics on failure).
///
/// **Note:** This macro panics on failure. Consider using `check_ready!` for non-test code.
///
/// # Examples
///
/// ```rust,no_run
/// use wait_for::{assert_ready, Target};
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), wait_for::WaitForError> {
/// let targets = vec![
///     Target::tcp("localhost", 8080)?,
/// ];
///
/// assert_ready!(targets, timeout: Duration::from_secs(30));
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! assert_ready {
    ($targets:expr, timeout: $timeout:expr) => {
        {
            let config = $crate::WaitConfig::builder()
                .timeout($timeout)
                .build();
            $crate::wait_for_connection(&$targets, &config)
                .await
                .expect("Targets should be ready")
        }
    };
    ($targets:expr, $($config_field:ident: $config_value:expr),+ $(,)?) => {
        {
            let config = $crate::wait_config! {
                $($config_field: $config_value,)+
            };
            $crate::wait_for_connection(&$targets, &config)
                .await
                .expect("Targets should be ready")
        }
    };
}
