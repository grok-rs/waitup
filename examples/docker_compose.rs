//! Docker Compose integration example.
//!
//! This example shows how to use wait-for in a Docker Compose environment
//! to wait for dependent services before starting your application.
//! Run with: cargo run --example docker_compose

use std::time::Duration;
use wait_for::{Target, WaitConfig, wait_for_connection};

#[tokio::main]
async fn main() -> Result<(), wait_for::WaitForError> {
    println!("🐳 Docker Compose: Waiting for dependent services...");

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

    println!("📋 Waiting for {} services:", targets.len());
    for target in &targets {
        println!("  - {}", target.display());
    }

    let result = wait_for_connection(&targets, &config).await?;

    println!("🎉 All services are ready! Starting application...");
    println!("⏱️  Total wait time: {:?}", result.elapsed);

    // Simulate starting your application
    println!("🚀 Application started successfully!");

    // Example: You could now execute your main application logic
    // or return success to Docker Compose to continue with the next service

    Ok(())
}
