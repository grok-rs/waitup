//! Comprehensive library usage example.
//!
//! This example showcases all the features available when using wait-for
//! as a library in your Rust applications.
//! Run with: cargo run --example library_usage

use std::time::Duration;
use wait_for::{Target, WaitConfig, WaitResult, wait_for_connection, wait_for_single_target};

/// Example of a custom service health checker
struct ServiceHealthChecker {
    name: String,
    targets: Vec<Target>,
    config: WaitConfig,
}

impl ServiceHealthChecker {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            targets: Vec::new(),
            config: WaitConfig::default(),
        }
    }

    fn add_tcp_target(mut self, host: &str, port: u16) -> Result<Self, wait_for::WaitForError> {
        self.targets.push(Target::tcp(host, port)?);
        Ok(self)
    }

    fn add_http_target(mut self, url: &str, status: u16) -> Result<Self, wait_for::WaitForError> {
        self.targets.push(Target::parse(url, status)?);
        Ok(self)
    }

    fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config = WaitConfig::builder()
            .timeout(timeout)
            .interval(self.config.initial_interval)
            .max_interval(self.config.max_interval)
            .wait_for_any(self.config.wait_for_any)
            .max_retries(self.config.max_retries)
            .connection_timeout(self.config.connection_timeout)
            .build();
        self
    }

    fn with_strategy(mut self, wait_for_any: bool) -> Self {
        self.config = WaitConfig::builder()
            .timeout(self.config.timeout)
            .interval(self.config.initial_interval)
            .max_interval(self.config.max_interval)
            .wait_for_any(wait_for_any)
            .max_retries(self.config.max_retries)
            .connection_timeout(self.config.connection_timeout)
            .build();
        self
    }

    async fn check_health(&self) -> Result<WaitResult, wait_for::WaitForError> {
        println!("🔍 Checking health for service: {}", self.name);

        let result = wait_for_connection(&self.targets, &self.config).await?;

        if result.success {
            println!("✅ Service '{}' is healthy! ({}ms, {} attempts)",
                     self.name,
                     result.elapsed.as_millis(),
                     result.attempts);
        } else {
            println!("❌ Service '{}' is unhealthy after {}ms",
                     self.name,
                     result.elapsed.as_millis());
        }

        Ok(result)
    }
}

async fn example_basic_usage() -> Result<(), wait_for::WaitForError> {
    println!("\n📚 Example 1: Basic Library Usage");
    println!("================================");

    // Simple TCP check
    let target = Target::tcp("httpbin.org", 80)?;
    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(10))
        .build();

    let result = wait_for_single_target(&target, &config).await?;

    println!("Target: {}", result.target.display());
    println!("Success: {}", result.success);
    println!("Elapsed: {:?}", result.elapsed);
    println!("Attempts: {}", result.attempts);

    Ok(())
}

async fn example_advanced_configuration() -> Result<(), wait_for::WaitForError> {
    println!("\n⚙️  Example 2: Advanced Configuration");
    println!("===================================");

    let targets = vec![
        Target::parse("https://httpbin.org/status/200", 200)?,
        Target::tcp("httpbin.org", 80)?,
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(30))
        .interval(Duration::from_millis(500))
        .max_interval(Duration::from_secs(5))
        .connection_timeout(Duration::from_secs(3))
        .max_retries(Some(10))
        .wait_for_any(false)
        .build();

    let result = wait_for_connection(&targets, &config).await?;

    println!("Overall success: {}", result.success);
    println!("Total elapsed: {:?}", result.elapsed);
    println!("Total attempts: {}", result.attempts);

    for (i, target_result) in result.target_results.iter().enumerate() {
        println!("Target {}: {} - {} in {:?} ({} attempts)",
                 i + 1,
                 target_result.target.display(),
                 if target_result.success { "✅" } else { "❌" },
                 target_result.elapsed,
                 target_result.attempts);
    }

    Ok(())
}

async fn example_custom_service_checker() -> Result<(), wait_for::WaitForError> {
    println!("\n🏗️  Example 3: Custom Service Health Checker");
    println!("===========================================");

    // Database service
    let db_checker = ServiceHealthChecker::new("Database")
        .add_tcp_target("httpbin.org", 80)?  // Using httpbin as example
        .with_timeout(Duration::from_secs(30))
        .with_strategy(false); // All targets must be ready

    let db_result = db_checker.check_health().await?;

    // API service with multiple endpoints
    let api_checker = ServiceHealthChecker::new("API Service")
        .add_http_target("https://httpbin.org/status/200", 200)?
        .add_http_target("https://httpbin.org/json", 200)?
        .with_timeout(Duration::from_secs(20))
        .with_strategy(true); // Any endpoint working is fine

    let api_result = api_checker.check_health().await?;

    println!("\n📊 Summary:");
    println!("Database health: {}", if db_result.success { "✅ Healthy" } else { "❌ Unhealthy" });
    println!("API health: {}", if api_result.success { "✅ Healthy" } else { "❌ Unhealthy" });

    Ok(())
}

async fn example_error_handling() -> Result<(), wait_for::WaitForError> {
    println!("\n🚨 Example 4: Error Handling");
    println!("============================");

    // This will likely timeout/fail
    let failing_target = vec![
        Target::tcp("definitely-not-a-real-host.invalid", 12345)?,
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(5))
        .max_retries(Some(3))
        .build();

    match wait_for_connection(&failing_target, &config).await {
        Ok(_) => println!("🤔 Unexpected success!"),
        Err(e) => {
            println!("❌ Expected failure: {}", e);
            println!("🧠 This demonstrates proper error handling in your application");
        }
    }

    Ok(())
}

async fn example_performance_monitoring() -> Result<(), wait_for::WaitForError> {
    println!("\n⚡ Example 5: Performance Monitoring");
    println!("===================================");

    let targets = vec![
        Target::parse("https://httpbin.org/delay/1", 200)?,  // 1 second delay
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(10))
        .interval(Duration::from_millis(100))
        .build();

    let start = std::time::Instant::now();
    let result = wait_for_connection(&targets, &config).await?;
    let total_time = start.elapsed();

    println!("⏱️  Performance Metrics:");
    println!("  - Library overhead: {:?}", total_time - result.elapsed);
    println!("  - Actual wait time: {:?}", result.elapsed);
    println!("  - Total execution time: {:?}", total_time);
    println!("  - Attempts made: {}", result.attempts);
    println!("  - Average time per attempt: {:?}",
             result.elapsed.div_f32(result.attempts as f32));

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), wait_for::WaitForError> {
    println!("📖 Wait-For Library Usage Examples");
    println!("==================================");

    // Run all examples
    example_basic_usage().await?;
    example_advanced_configuration().await?;
    example_custom_service_checker().await?;
    example_error_handling().await?;
    example_performance_monitoring().await?;

    println!("\n🎉 All library examples completed successfully!");
    println!("💡 You can now integrate wait-for into your Rust applications!");

    Ok(())
}
