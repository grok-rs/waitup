#![allow(
    clippy::print_stdout,
    clippy::uninlined_format_args,
    reason = "example code that demonstrates library usage"
)]

//! Basic TCP connection example using wait-for as a library.
//!
//! This example demonstrates waiting for a single TCP service to become available.
//! Run with: cargo run --example `basic_tcp`

use std::time::Duration;
use wait_for::{wait_for_connection, Target, WaitConfig};

#[tokio::main]
async fn main() -> Result<(), wait_for::WaitForError> {
    println!("🔍 Waiting for TCP service to become available...");

    // Create a TCP target
    let target = Target::tcp("localhost", 8080)?;

    // Configure the wait parameters
    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_secs(1))
        .max_interval(Duration::from_secs(5))
        .build();

    // Wait for the service
    let result = wait_for_connection(&[target], &config).await?;

    println!("✅ Service is ready!");
    println!(
        "📊 Connection successful in {:?} with {} attempts",
        result.elapsed, result.attempts
    );

    // Print details for each target
    for target_result in &result.target_results {
        println!(
            "  - {}: {} in {:?} ({} attempts)",
            target_result.target.display(),
            if target_result.success {
                "✅ Success"
            } else {
                "❌ Failed"
            },
            target_result.elapsed,
            target_result.attempts
        );
    }

    Ok(())
}
