#![allow(
    clippy::print_stdout,
    clippy::uninlined_format_args,
    reason = "example code that demonstrates library usage"
)]

//! HTTP health check example with custom headers and status codes.
//!
//! This example demonstrates advanced HTTP health checking capabilities
//! including custom headers, different status codes, and authentication.
//! Run with: cargo run --example `http_health`

use std::time::Duration;
use url::Url;
use waitup::{wait_for_connection, Target, WaitConfig, WaitResult};

async fn basic_health_check() -> Result<(), waitup::WaitForError> {
    println!("\n📊 Example 1: Basic health check");
    let basic_target = vec![Target::parse("https://httpbin.org/status/200", 200)?];

    let basic_config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_secs(2))
        .build();

    match wait_for_connection(&basic_target, &basic_config).await {
        Ok(result) => println!("✅ Basic health check passed in {:?}", result.elapsed),
        Err(e) => println!("❌ Basic health check failed: {e}"),
    }
    Ok(())
}

async fn custom_status_check() -> Result<(), waitup::WaitForError> {
    println!("\n📊 Example 2: Custom status code health check");
    let custom_status_target = vec![Target::parse("https://httpbin.org/status/204", 204)?];

    let basic_config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_secs(2))
        .build();

    match wait_for_connection(&custom_status_target, &basic_config).await {
        Ok(result) => println!(
            "✅ Custom status check (204) passed in {:?}",
            result.elapsed
        ),
        Err(e) => println!("❌ Custom status check failed: {e}"),
    }
    Ok(())
}

async fn auth_header_check() -> Result<(), waitup::WaitForError> {
    println!("\n📊 Example 3: Health check with authentication header");
    let auth_url = Url::parse("https://httpbin.org/bearer")?;
    let auth_headers = vec![
        (
            "Authorization".to_string(),
            "Bearer your-token-here".to_string(),
        ),
        ("User-Agent".to_string(), "waitup/1.0".to_string()),
    ];

    let auth_target = vec![
        Target::http_with_headers(auth_url, 401, auth_headers)?, // Will get 401 because token is fake
    ];

    let basic_config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_secs(2))
        .build();

    match wait_for_connection(&auth_target, &basic_config).await {
        Ok(result) => println!("✅ Auth health check passed in {:?}", result.elapsed),
        Err(e) => println!("❌ Auth health check failed (expected): {e}"),
    }
    Ok(())
}

async fn multiple_endpoints_check() -> Result<(), waitup::WaitForError> {
    println!("\n📊 Example 4: Multiple health endpoints - ANY strategy");
    let multiple_targets = vec![
        Target::parse("https://httpbin.org/status/200", 200)?,
        Target::parse("https://httpbin.org/delay/10", 200)?, // This will be slow
        Target::parse("https://httpbin.org/status/503", 503)?, // This will "fail" but we expect 503
    ];

    let any_config = WaitConfig::builder()
        .timeout(Duration::from_secs(15))
        .wait_for_any(true) // Return as soon as ANY endpoint is ready
        .build();

    match wait_for_connection(&multiple_targets, &any_config).await {
        Ok(result) => {
            println!(
                "✅ First available endpoint ready in {:?}: {}",
                result.elapsed,
                result
                    .target_results
                    .first()
                    .map_or_else(|| "Unknown".to_string(), |r| r.target.display())
            );
        }
        Err(e) => println!("❌ No endpoints available: {e}"),
    }
    Ok(())
}

async fn production_config_check() -> Result<WaitResult, waitup::WaitForError> {
    println!("\n📊 Example 5: Production-ready health check configuration");
    let production_targets = vec![Target::parse("https://httpbin.org/status/200", 200)?];

    let production_config = WaitConfig::builder()
        .timeout(Duration::from_secs(120)) // 2 minute total timeout
        .interval(Duration::from_secs(5)) // Check every 5 seconds
        .max_interval(Duration::from_secs(30)) // Max 30 second intervals
        .connection_timeout(Duration::from_secs(10)) // 10 second request timeout
        .max_retries(Some(20)) // Maximum 20 attempts
        .build();

    wait_for_connection(&production_targets, &production_config).await
}

fn print_results(result: &WaitResult) {
    println!("✅ Production health check completed!");
    println!("📊 Results:");
    println!("  - Total time: {:?}", result.elapsed);
    println!("  - Total attempts: {}", result.attempts);
    println!("  - Success: {}", result.success);

    // Detailed per-target results
    for target_result in &result.target_results {
        println!(
            "  - {}: {} ({} attempts, {:?})",
            target_result.target.display(),
            if target_result.success {
                "✅ Ready"
            } else {
                "❌ Failed"
            },
            target_result.attempts,
            target_result.elapsed
        );

        if let Some(error) = &target_result.error {
            println!("    Error: {error}");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), waitup::WaitForError> {
    println!("🏥 HTTP Health Check Example");
    println!("============================");

    basic_health_check().await?;
    custom_status_check().await?;
    auth_header_check().await?;
    multiple_endpoints_check().await?;

    let result = production_config_check().await?;
    print_results(&result);

    println!("\n🎉 HTTP health check examples completed!");
    Ok(())
}
