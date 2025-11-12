//! Convenience macros for creating targets and configurations.

/// Helper macro to count the number of token trees (for Vec pre-allocation).
#[doc(hidden)]
#[macro_export]
macro_rules! count_tts {
    () => { 0 };
    ($one:tt $($rest:tt)*) => { 1 + $crate::count_tts!($($rest)*) };
}

/// Create TCP targets from a compact syntax.
///
/// Returns a `Result<Vec<Target>, WaitForError>` that contains either all valid targets
/// or the first error encountered.
///
/// # Examples
///
/// ```rust
/// use waitup::tcp_targets;
///
/// let targets = tcp_targets![
///     "localhost" => 8080,
///     "database" => 5432,
///     "cache" => 6379,
/// ]?;
/// assert_eq!(targets.len(), 3);
/// # Ok::<(), waitup::WaitForError>(())
/// ```
#[macro_export]
macro_rules! tcp_targets {
    ($($host:expr => $port:expr),* $(,)?) => {
        {
            #[expect(clippy::vec_init_then_push, reason = "macro expansion pattern with pre-allocated capacity for performance")]
            let result = || -> $crate::Result<Vec<$crate::Target>> {
                // Pre-allocate capacity for better performance
                let mut targets = Vec::with_capacity($crate::count_tts!($($host)*));
                $(
                    targets.push($crate::Target::tcp($host, $port)?);
                )*
                return Ok(targets)
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
/// use waitup::http_targets;
///
/// let targets = http_targets![
///     "https://api.example.com/health" => 200,
///     "http://localhost:8080/status" => 204,
/// ]?;
/// assert_eq!(targets.len(), 2);
/// # Ok::<(), waitup::WaitForError>(())
/// ```
#[macro_export]
macro_rules! http_targets {
    ($($url:expr => $status:expr),* $(,)?) => {
        {
            #[expect(clippy::vec_init_then_push, reason = "macro expansion pattern with pre-allocated capacity for performance")]
            let result = || -> $crate::Result<Vec<$crate::Target>> {
                // Pre-allocate capacity for better performance
                let mut targets = Vec::with_capacity($crate::count_tts!($($url)*));
                $(
                    targets.push($crate::Target::http_url($url, $status)?);
                )*
                return Ok(targets)
            };
            result()
        }
    };
}
