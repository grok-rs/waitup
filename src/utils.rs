//! Shared utility functions for the waitup library.

use std::borrow::Cow;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{Result, WaitForError};

/// Safe conversion from Duration to u64 milliseconds.
///
/// Handles overflow by clamping to `u64::MAX`.
#[inline]
pub fn duration_to_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis().min(u128::from(u64::MAX))).unwrap_or(u64::MAX)
}

/// Parse a duration string with a unit multiplier.
///
/// Used for parsing duration strings like "30s", "5m", "2h".
///
/// # Arguments
///
/// * `number` - The numeric value
/// * `unit_multiplier` - Milliseconds per unit (e.g., 1000.0 for seconds)
/// * `input` - Original input string for error messages
///
/// # Errors
///
/// Returns an error if the duration is negative.
#[inline]
pub fn parse_duration_unit(number: f64, unit_multiplier: f64, input: &str) -> Result<Duration> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "duration calculation requires f64"
    )]
    let millis = (number * unit_multiplier).min(u64::MAX as f64);

    if millis < 0.0 {
        return Err(WaitForError::InvalidTimeout(
            Cow::Owned(input.to_string()),
            Cow::Borrowed("Duration cannot be negative"),
        ));
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "safe cast after bounds check"
    )]
    Ok(Duration::from_millis(millis as u64))
}

/// Sleep for a duration with optional cancellation support.
///
/// If a cancellation token is provided, the sleep will be interrupted
/// if the token is cancelled, returning an error.
///
/// # Errors
///
/// Returns `WaitForError::Cancelled` if the cancellation token is triggered.
#[inline]
pub async fn sleep_with_cancellation(
    duration: Duration,
    token: Option<&CancellationToken>,
) -> Result<()> {
    if let Some(t) = token {
        tokio::select! {
            () = sleep(duration) => Ok(()),
            () = t.cancelled() => Err(WaitForError::Cancelled),
        }
    } else {
        sleep(duration).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_to_millis_u64_normal() {
        let duration = Duration::from_secs(5);
        assert_eq!(duration_to_millis_u64(duration), 5000);
    }

    #[test]
    fn test_duration_to_millis_u64_max() {
        // Test with a very large duration
        let duration = Duration::from_secs(u64::MAX / 1000);
        let result = duration_to_millis_u64(duration);
        assert!(result > 0);
    }

    #[test]
    fn test_parse_duration_unit_seconds() {
        let result = parse_duration_unit(5.0, 1000.0, "5s").expect("valid duration");
        assert_eq!(result, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_duration_unit_minutes() {
        let result = parse_duration_unit(2.0, 60_000.0, "2m").expect("valid duration");
        assert_eq!(result, Duration::from_secs(120));
    }

    #[test]
    fn test_parse_duration_unit_negative() {
        let result = parse_duration_unit(-5.0, 1000.0, "-5s");
        assert!(result.is_err());
    }
}
