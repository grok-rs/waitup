#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::uninlined_format_args,
    reason = "example code that demonstrates library usage"
)]

//! Kubernetes init container example.
//!
//! This example demonstrates using wait-for as a Kubernetes init container
//! to ensure dependencies are ready before the main application starts.
//! Run with: cargo run --example `kubernetes_init`

use std::time::Duration;
use wait_for::{wait_for_connection, Target, WaitConfig};

#[tokio::main]
async fn main() -> Result<(), wait_for::WaitForError> {
    println!("☸️  Kubernetes Init Container: Checking dependencies...");

    // Environment variables typically set in Kubernetes
    let db_host = std::env::var("DATABASE_HOST").unwrap_or_else(|_| "postgres-service".to_string());
    let db_port = std::env::var("DATABASE_PORT")
        .unwrap_or_else(|_| "5432".to_string())
        .parse::<u16>()
        .unwrap_or(5432);

    let cache_host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "redis-service".to_string());
    let cache_port = std::env::var("REDIS_PORT")
        .unwrap_or_else(|_| "6379".to_string())
        .parse::<u16>()
        .unwrap_or(6379);

    // External API dependency
    let api_url = std::env::var("EXTERNAL_API_URL")
        .unwrap_or_else(|_| "https://api.external-service.com/health".to_string());

    let targets = vec![
        Target::tcp(&db_host, db_port)?,
        Target::tcp(&cache_host, cache_port)?,
        Target::parse(&api_url, 200)?,
    ];

    // Kubernetes-appropriate configuration
    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(300)) // 5 minutes max
        .interval(Duration::from_secs(5)) // Check every 5 seconds
        .max_interval(Duration::from_secs(30))
        .max_retries(Some(60)) // Limit retries
        .wait_for_any(false) // All dependencies must be ready
        .build();

    println!("🔍 Checking dependencies:");
    for target in &targets {
        println!("  ⏳ {}", target.display());
    }

    match wait_for_connection(&targets, &config).await {
        Ok(result) => {
            println!("✅ All dependencies are ready!");
            println!(
                "📊 Completed in {:?} with {} total attempts",
                result.elapsed, result.attempts
            );

            // Log detailed results for troubleshooting
            for target_result in &result.target_results {
                println!(
                    "  ✅ {}: Ready in {:?} ({} attempts)",
                    target_result.target.display(),
                    target_result.elapsed,
                    target_result.attempts
                );
            }

            println!("🎯 Init container completed successfully. Main container can now start.");
        }
        Err(e) => {
            eprintln!("❌ Dependencies not ready: {e}");
            eprintln!("🔧 Check your service configurations and try again.");
            std::process::exit(1);
        }
    }

    Ok(())
}
