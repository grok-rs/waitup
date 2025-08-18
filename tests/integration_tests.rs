use std::process::Command;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_successful_tcp_connection() {
    // Start a test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server task
    tokio::spawn(async move {
        let (_stream, _addr) = listener.accept().await.unwrap();
    });

    // Test wait-for
    let output = Command::new("./target/debug/wait-for")
        .args(&[
            &format!("127.0.0.1:{}", addr.port()),
            "--timeout",
            "5s",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute wait-for");

    assert!(output.status.success());
}

#[tokio::test]
async fn test_timeout_failure() {
    let output = Command::new("./target/debug/wait-for")
        .args(&["127.0.0.1:65534", "--timeout", "1s", "--quiet"])
        .output()
        .expect("Failed to execute wait-for");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[tokio::test]
async fn test_dns_resolution() {
    let output = Command::new("./target/debug/wait-for")
        .args(&["google.com:80", "--timeout", "10s", "--quiet"])
        .output()
        .expect("Failed to execute wait-for");

    assert!(output.status.success());
}

#[tokio::test]
async fn test_multiple_targets_any() {
    // Start one test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (_stream, _addr) = listener.accept().await.unwrap();
    });

    let output = Command::new("./target/debug/wait-for")
        .args(&[
            &format!("127.0.0.1:{}", addr.port()),
            "127.0.0.1:65534", // This will fail
            "--any",
            "--timeout",
            "5s",
            "--quiet",
        ])
        .output()
        .expect("Failed to execute wait-for");

    assert!(output.status.success());
}

#[tokio::test]
async fn test_command_execution() {
    // Start a test server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (_stream, _addr) = listener.accept().await.unwrap();
    });

    let output = Command::new("./target/debug/wait-for")
        .args(&[
            &format!("127.0.0.1:{}", addr.port()),
            "--timeout",
            "5s",
            "--quiet",
            "--",
            "echo",
            "command executed",
        ])
        .output()
        .expect("Failed to execute wait-for");

    assert!(output.status.success());
}

#[test]
fn test_invalid_target_format() {
    let output = Command::new("./target/debug/wait-for")
        .args(&["invalid-target", "--quiet"])
        .output()
        .expect("Failed to execute wait-for");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_invalid_timeout_format() {
    let output = Command::new("./target/debug/wait-for")
        .args(&["localhost:8080", "--timeout", "invalid", "--quiet"])
        .output()
        .expect("Failed to execute wait-for");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(2));
}
