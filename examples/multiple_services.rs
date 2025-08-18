//! Multiple services orchestration example.
//!
//! This example demonstrates complex service orchestration with different
//! wait strategies for different groups of services.
//! Run with: cargo run --example multiple_services

use std::time::Duration;
use wait_for::{Target, WaitConfig, wait_for_connection};

async fn wait_for_core_services() -> Result<(), wait_for::WaitForError> {
    println!("🏗️  Phase 1: Waiting for core infrastructure services...");

    let core_services = vec![
        Target::tcp("postgres", 5432)?,      // Primary database
        Target::tcp("redis", 6379)?,         // Cache
        Target::tcp("elasticsearch", 9200)?, // Search engine
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(180)) // 3 minutes for core services
        .interval(Duration::from_secs(2))
        .wait_for_any(false) // ALL core services must be ready
        .build();

    let result = wait_for_connection(&core_services, &config).await?;
    println!("✅ Core services ready in {:?}", result.elapsed);

    Ok(())
}

async fn wait_for_application_services() -> Result<(), wait_for::WaitForError> {
    println!("🚀 Phase 2: Waiting for application services...");

    let app_services = vec![
        Target::parse("http://auth-service:8001/health", 200)?,
        Target::parse("http://user-service:8002/health", 200)?,
        Target::parse("http://notification-service:8003/health", 200)?,
        Target::parse("http://payment-service:8004/health", 200)?,
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(120)) // 2 minutes for app services
        .interval(Duration::from_secs(3))
        .wait_for_any(false)
        .build();

    let result = wait_for_connection(&app_services, &config).await?;
    println!("✅ Application services ready in {:?}", result.elapsed);

    Ok(())
}

async fn wait_for_external_dependencies() -> Result<(), wait_for::WaitForError> {
    println!("🌐 Phase 3: Checking external dependencies (any one available)...");

    // Multiple external APIs - we only need one to be available
    let external_apis = vec![
        Target::parse("https://api.stripe.com/v1", 401)?, // Expected auth error
        Target::parse("https://api.sendgrid.com/v3", 401)?, // Expected auth error
        Target::parse("https://hooks.slack.com/services", 404)?, // Expected not found
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(60)) // 1 minute for external APIs
        .interval(Duration::from_secs(5))
        .wait_for_any(true) // ANY external API working is fine
        .build();

    let result = wait_for_connection(&external_apis, &config).await?;
    println!("✅ External dependencies available in {:?}", result.elapsed);

    Ok(())
}

async fn start_load_balancer() -> Result<(), wait_for::WaitForError> {
    println!("⚖️  Phase 4: Starting load balancer...");

    // Check that our load balancer is ready
    let lb_target = vec![Target::parse("http://load-balancer:80/health", 200)?];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_secs(2))
        .build();

    let result = wait_for_connection(&lb_target, &config).await?;
    println!("✅ Load balancer ready in {:?}", result.elapsed);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), wait_for::WaitForError> {
    println!("🎭 Complex Multi-Service Orchestration Example");
    println!("============================================");

    let start_time = std::time::Instant::now();

    // Phase 1: Core infrastructure must be ready first
    wait_for_core_services().await?;

    // Phase 2: Application services depend on core infrastructure
    wait_for_application_services().await?;

    // Phase 3: External dependencies (at least one must work)
    wait_for_external_dependencies().await?;

    // Phase 4: Load balancer brings everything together
    start_load_balancer().await?;

    let total_time = start_time.elapsed();

    println!();
    println!("🎉 All services are orchestrated and ready!");
    println!("⏱️  Total orchestration time: {:?}", total_time);
    println!("🌟 Your distributed system is now fully operational!");

    // At this point, you could start accepting traffic or run your main application

    Ok(())
}
