#![allow(
    clippy::print_stdout,
    clippy::uninlined_format_args,
    clippy::std_instead_of_core,
    reason = "example code that demonstrates library usage"
)]

//! Docker Compose integration example.
//!
//! This example shows how to use waitup in a Docker Compose environment
//! to wait for dependent services before starting your application.
//! Run with: cargo run --example `docker_compose`

use std::time::Duration;
use waitup::{Target, WaitConfig, wait_for_connection};

#[tokio::main]
async fn main() -> Result<(), waitup::WaitForError> {
    println!("\u{1F433} Docker Compose: Waiting for dependent services...");

    // Typical services in a Docker Compose setup
    let targets = vec![
        Target::tcp("postgres", 5432)?, // Database
        Target::tcp("redis", 6379)?,    // Cache
        Target::tcp("rabbitmq", 5672)?, // Message queue
        // Health check endpoint for API gateway
        Target::parse("http://api-gateway:8080/health", 200)?,
    ];

    // Configuration suitable for Docker Compose startup
    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(120)) // 2 minutes for all services
        .interval(Duration::from_secs(2)) // Check every 2 seconds
        .max_interval(Duration::from_secs(10))
        .wait_for_any(false) // Wait for ALL services
        .build();

    println!("\u{1F4CB} Waiting for {} services:", targets.len());
    for target in &targets {
        println!("  - {}", target.display());
    }

    let result = wait_for_connection(&targets, &config).await?;

    println!("\u{1F389} All services are ready! Starting application...");
    println!("\u{23F1}\u{FE0F} Total wait time: {:?}", result.elapsed);

    // Simulate starting your application
    println!("\u{1F680} Application started successfully!");

    // Example: You could now execute your main application logic
    // or return success to Docker Compose to continue with the next service

    Ok(())
}
