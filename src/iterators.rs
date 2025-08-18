//! Iterator utilities and extensions for working with targets and results.
//!
//! This module provides idiomatic Rust iterator patterns for processing
//! collections of targets and results.

use crate::types::{Target, TargetResult, WaitResult};

/// Extension trait for working with iterators of targets
pub trait TargetIterExt: Iterator<Item = Target> {
    /// Collect TCP targets from a mixed iterator
    fn tcp_targets(self) -> impl Iterator<Item = Target>
    where
        Self: Sized,
    {
        self.filter(|target| matches!(target, Target::Tcp { .. }))
    }

    /// Collect HTTP targets from a mixed iterator
    fn http_targets(self) -> impl Iterator<Item = Target>
    where
        Self: Sized,
    {
        self.filter(|target| matches!(target, Target::Http { .. }))
    }

    /// Group targets by hostname
    fn group_by_hostname(self) -> std::collections::HashMap<String, Vec<Target>>
    where
        Self: Sized,
    {
        let mut groups = std::collections::HashMap::new();
        for target in self {
            let hostname = target.hostname().to_string();
            groups.entry(hostname).or_insert_with(Vec::new).push(target);
        }
        groups
    }
}

impl<I> TargetIterExt for I where I: Iterator<Item = Target> {}

/// Extension trait for working with iterators of target results
pub trait TargetResultIterExt: Iterator<Item = TargetResult> {
    /// Filter successful results
    fn successful(self) -> impl Iterator<Item = TargetResult>
    where
        Self: Sized,
    {
        self.filter(|result| result.success)
    }

    /// Filter failed results
    fn failed(self) -> impl Iterator<Item = TargetResult>
    where
        Self: Sized,
    {
        self.filter(|result| !result.success)
    }

    /// Get the slowest result (highest elapsed time)
    fn slowest(self) -> Option<TargetResult>
    where
        Self: Sized,
    {
        self.max_by_key(|result| result.elapsed)
    }

    /// Get the fastest result (lowest elapsed time)
    fn fastest(self) -> Option<TargetResult>
    where
        Self: Sized,
    {
        self.min_by_key(|result| result.elapsed)
    }

    /// Calculate total attempts across all results
    fn total_attempts(self) -> u32
    where
        Self: Sized,
    {
        self.map(|result| result.attempts).sum()
    }
}

impl<I> TargetResultIterExt for I where I: Iterator<Item = TargetResult> {}

/// Extension trait for slices/Vecs of TargetResult to provide summary functionality
pub trait TargetResultSliceExt {
    /// Get summary statistics
    fn summary(&self) -> ResultSummary;
    /// Get successful results
    fn successful_results(&self) -> impl Iterator<Item = &TargetResult>;
    /// Get failed results
    fn failed_results(&self) -> impl Iterator<Item = &TargetResult>;
}

impl TargetResultSliceExt for [TargetResult] {
    fn summary(&self) -> ResultSummary {
        let successful_count = self.iter().filter(|r| r.success).count();
        let failed_count = self.iter().filter(|r| !r.success).count();
        let total_attempts = self.iter().map(|r| r.attempts).sum();
        let total_elapsed: std::time::Duration = self.iter().map(|r| r.elapsed).sum();

        let fastest = self.iter()
            .map(|r| r.elapsed)
            .min();

        let slowest = self.iter()
            .map(|r| r.elapsed)
            .max();

        ResultSummary {
            total_targets: self.len(),
            successful_count,
            failed_count,
            total_attempts,
            total_elapsed,
            fastest_response: fastest,
            slowest_response: slowest,
        }
    }

    fn successful_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.iter().filter(|r| r.success)
    }

    fn failed_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.iter().filter(|r| !r.success)
    }
}

impl<T: AsRef<[TargetResult]>> TargetResultSliceExt for T {
    fn summary(&self) -> ResultSummary {
        self.as_ref().summary()
    }

    fn successful_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.as_ref().successful_results()
    }

    fn failed_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.as_ref().failed_results()
    }
}

impl WaitResult {
    /// Iterate over successful target results
    pub fn successful_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.target_results.iter().filter(|result| result.success)
    }

    /// Iterate over failed target results
    pub fn failed_results(&self) -> impl Iterator<Item = &TargetResult> {
        self.target_results.iter().filter(|result| !result.success)
    }

    /// Get summary statistics
    pub fn summary(&self) -> ResultSummary {
        let successful_count = self.successful_results().count();
        let failed_count = self.failed_results().count();
        let total_attempts = self.target_results.iter().map(|r| r.attempts).sum();

        let fastest = self.target_results
            .iter()
            .filter(|r| r.success)
            .min_by_key(|r| r.elapsed)
            .map(|r| r.elapsed);

        let slowest = self.target_results
            .iter()
            .max_by_key(|r| r.elapsed)
            .map(|r| r.elapsed);

        ResultSummary {
            total_targets: self.target_results.len(),
            successful_count,
            failed_count,
            total_attempts,
            total_elapsed: self.elapsed,
            fastest_response: fastest,
            slowest_response: slowest,
        }
    }
}

/// Summary statistics for wait results
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultSummary {
    pub total_targets: usize,
    pub successful_count: usize,
    pub failed_count: usize,
    pub total_attempts: u32,
    pub total_elapsed: std::time::Duration,
    pub fastest_response: Option<std::time::Duration>,
    pub slowest_response: Option<std::time::Duration>,
}

impl std::fmt::Display for ResultSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Targets: {}/{} successful, {} attempts, elapsed: {:?}",
            self.successful_count,
            self.total_targets,
            self.total_attempts,
            self.total_elapsed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Target, TargetResult, WaitResult};
    use std::time::Duration;

    fn create_test_target_result(target: Target, success: bool, elapsed: Duration, attempts: u32) -> TargetResult {
        TargetResult {
            target,
            success,
            elapsed,
            attempts,
            error: if success { None } else { Some("Test error".to_string()) },
        }
    }

    #[test]
    fn test_target_iter_ext_successful_results() {
        let target1 = Target::tcp("localhost", 8080).unwrap();
        let target2 = Target::tcp("localhost", 8081).unwrap();
        let target3 = Target::tcp("localhost", 8082).unwrap();

        let results = vec![
            create_test_target_result(target1, true, Duration::from_millis(100), 1),
            create_test_target_result(target2, false, Duration::from_millis(200), 2),
            create_test_target_result(target3, true, Duration::from_millis(150), 1),
        ];

        let successful: Vec<_> = results.successful_results().collect();
        assert_eq!(successful.len(), 2);
        assert!(successful[0].success);
        assert!(successful[1].success);
    }

    #[test]
    fn test_target_iter_ext_failed_results() {
        let target1 = Target::tcp("localhost", 8080).unwrap();
        let target2 = Target::tcp("localhost", 8081).unwrap();
        let target3 = Target::tcp("localhost", 8082).unwrap();

        let results = vec![
            create_test_target_result(target1, true, Duration::from_millis(100), 1),
            create_test_target_result(target2, false, Duration::from_millis(200), 2),
            create_test_target_result(target3, false, Duration::from_millis(150), 3),
        ];

        let failed: Vec<_> = results.failed_results().collect();
        assert_eq!(failed.len(), 2);
        assert!(!failed[0].success);
        assert!(!failed[1].success);
    }

    #[test]
    fn test_target_result_iter_ext_summary() {
        let target1 = Target::tcp("localhost", 8080).unwrap();
        let target2 = Target::tcp("localhost", 8081).unwrap();
        let target3 = Target::tcp("localhost", 8082).unwrap();

        let results = vec![
            create_test_target_result(target1, true, Duration::from_millis(100), 1),
            create_test_target_result(target2, false, Duration::from_millis(200), 2),
            create_test_target_result(target3, true, Duration::from_millis(150), 1),
        ];

        let summary = results.summary();
        assert_eq!(summary.total_targets, 3);
        assert_eq!(summary.successful_count, 2);
        assert_eq!(summary.failed_count, 1);
        assert_eq!(summary.total_attempts, 4);
        assert_eq!(summary.fastest_response, Some(Duration::from_millis(100)));
        assert_eq!(summary.slowest_response, Some(Duration::from_millis(200)));
        assert_eq!(summary.total_elapsed, Duration::from_millis(450)); // Sum of all elapsed times
    }

    #[test]
    fn test_wait_result_summary() {
        let target1 = Target::tcp("localhost", 8080).unwrap();
        let target2 = Target::tcp("localhost", 8081).unwrap();

        let wait_result = WaitResult {
            success: true,
            elapsed: Duration::from_millis(300),
            attempts: 3,
            target_results: vec![
                create_test_target_result(target1, true, Duration::from_millis(100), 1),
                create_test_target_result(target2, true, Duration::from_millis(200), 2),
            ],
        };

        let summary = wait_result.summary();
        assert_eq!(summary.total_targets, 2);
        assert_eq!(summary.successful_count, 2);
        assert_eq!(summary.failed_count, 0);
        assert_eq!(summary.total_attempts, 3);
        assert_eq!(summary.total_elapsed, Duration::from_millis(300)); // Uses WaitResult elapsed
        assert_eq!(summary.fastest_response, Some(Duration::from_millis(100)));
        assert_eq!(summary.slowest_response, Some(Duration::from_millis(200)));
    }

    #[test]
    fn test_result_summary_empty() {
        let results: Vec<TargetResult> = vec![];
        let summary = results.summary();

        assert_eq!(summary.total_targets, 0);
        assert_eq!(summary.successful_count, 0);
        assert_eq!(summary.failed_count, 0);
        assert_eq!(summary.total_attempts, 0);
        assert_eq!(summary.total_elapsed, Duration::ZERO);
        assert_eq!(summary.fastest_response, None);
        assert_eq!(summary.slowest_response, None);
    }

    #[test]
    fn test_result_summary_display() {
        let target = Target::tcp("localhost", 8080).unwrap();
        let results = vec![
            create_test_target_result(target, true, Duration::from_millis(100), 2),
        ];

        let summary = results.summary();
        let display = format!("{}", summary);
        assert!(display.contains("1/1 successful"));
        assert!(display.contains("2 attempts"));
        assert!(display.contains("100ms"));
    }

    #[test]
    fn test_result_summary_all_failed() {
        let target1 = Target::tcp("localhost", 8080).unwrap();
        let target2 = Target::tcp("localhost", 8081).unwrap();

        let results = vec![
            create_test_target_result(target1, false, Duration::from_millis(100), 1),
            create_test_target_result(target2, false, Duration::from_millis(200), 2),
        ];

        let summary = results.summary();
        assert_eq!(summary.total_targets, 2);
        assert_eq!(summary.successful_count, 0);
        assert_eq!(summary.failed_count, 2);
        assert_eq!(summary.total_attempts, 3);
    }
}
