#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code where panics are acceptable"
)]

use std::process::Command;
use tokio::net::TcpListener;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn successful_tcp_connection() {
        // Start a test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            let (_stream, _addr) = listener.accept().await.unwrap();
        });

        // Test waitup
        let output = Command::new("./target/debug/waitup")
            .args([
                &format!("127.0.0.1:{}", addr.port()),
                "--timeout",
                "5s",
                "--quiet",
            ])
            .output()
            .expect("Failed to execute waitup");

        assert!(output.status.success());
    }

    #[tokio::test]
    async fn timeout_failure() {
        let output = Command::new("./target/debug/waitup")
            .args(["127.0.0.1:65534", "--timeout", "1s", "--quiet"])
            .output()
            .expect("Failed to execute waitup");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
    }

    #[tokio::test]
    async fn dns_resolution() {
        let output = Command::new("./target/debug/waitup")
            .args(["google.com:80", "--timeout", "10s", "--quiet"])
            .output()
            .expect("Failed to execute waitup");

        assert!(output.status.success());
    }

    #[tokio::test]
    async fn multiple_targets_any() {
        // Start one test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_stream, _addr) = listener.accept().await.unwrap();
        });

        let output = Command::new("./target/debug/waitup")
            .args([
                &format!("127.0.0.1:{}", addr.port()),
                "127.0.0.1:65534", // This will fail
                "--any",
                "--timeout",
                "5s",
                "--quiet",
            ])
            .output()
            .expect("Failed to execute waitup");

        assert!(output.status.success());
    }

    #[tokio::test]
    async fn command_execution() {
        // Start a test server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_stream, _addr) = listener.accept().await.unwrap();
        });

        let output = Command::new("./target/debug/waitup")
            .args([
                &format!("127.0.0.1:{}", addr.port()),
                "--timeout",
                "5s",
                "--quiet",
                "--",
                "echo",
                "command executed",
            ])
            .output()
            .expect("Failed to execute waitup");

        assert!(output.status.success());
    }

    #[tokio::test]
    async fn verbose_streaming_updates_per_target() {
        // Start one test server that will accept a single connection
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (_stream, _addr) = listener.accept().await.unwrap();
        });

        // Run waitup against one reachable and one unreachable target with --verbose
        let output = Command::new("./target/debug/waitup")
            .args([
                &format!("127.0.0.1:{}", addr.port()),
                "127.0.0.1:65534", // likely unreachable
                "--timeout",
                "1s",
                "--verbose",
            ])
            .output()
            .expect("Failed to execute waitup");

        // Combine stdout and stderr since progress bars may write to stderr
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        // On non-tty environments progress symbols may be stripped; ensure the
        // invocation with --verbose doesn't crash and reports the unreachable target.
        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        assert!(
            combined.contains("127.0.0.1:65534"),
            "expected unreachable target in output: {}",
            combined
        );
    }

    #[test]
    fn invalid_target_format() {
        let output = Command::new("./target/debug/waitup")
            .args(["invalid-target", "--quiet"])
            .output()
            .expect("Failed to execute waitup");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(2));
    }

    #[test]
    fn invalid_timeout_format() {
        let output = Command::new("./target/debug/waitup")
            .args(["localhost:8080", "--timeout", "invalid", "--quiet"])
            .output()
            .expect("Failed to execute waitup");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(2));
    }
}
