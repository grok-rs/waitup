# Developer Guide

This guide is for developers who want to understand, extend, or contribute to wait-for's codebase. It complements the [Contributing Guide](../CONTRIBUTING.md) with deeper technical details.

## Table of Contents

- [Development Environment](#development-environment)
- [Code Architecture Deep Dive](#code-architecture-deep-dive)
- [Testing Strategy](#testing-strategy)
- [Extension Points](#extension-points)
- [Performance Considerations](#performance-considerations)
- [Debugging and Profiling](#debugging-and-profiling)
- [Release Process](#release-process)

## Development Environment

### Prerequisites

```bash
# Required tools
rustup install stable
cargo install cargo-watch cargo-expand cargo-audit
cargo install --locked cargo-deny

# Optional but recommended
cargo install cargo-nextest  # Faster test runner
cargo install cargo-llvm-cov # Coverage reporting
```

### IDE Setup

#### VS Code
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.check.extraArgs": ["--all-targets"],
    "rust-analyzer.rustfmt.extraArgs": ["+nightly"]
}
```

#### IntelliJ IDEA / CLion
- Install Rust plugin
- Enable "Use clippy instead of cargo check"
- Configure formatter to use nightly rustfmt

### Development Workflow

```bash
# 1. Set up development environment
make setup-dev  # or run setup script

# 2. Watch mode for development
cargo watch -x 'check --all-targets' -x 'test --lib'

# 3. Run all checks before committing
make pre-commit  # Runs fmt, clippy, test, doc

# 4. Run integration tests
cargo test --test integration_tests

# 5. Test against real services
docker-compose -f docker-compose.test.yml up -d
cargo test --test integration_tests
```

## Code Architecture Deep Dive

### Module Organization

```
src/
├── lib.rs           # Public API surface and re-exports
├── main.rs          # CLI entry point (minimal)
├── cli.rs           # Command-line argument parsing and handling
├── types.rs         # Core type definitions (Target, WaitConfig, etc.)
├── target.rs        # Target builders and validation
├── config.rs        # Configuration builders
├── connection.rs    # Core connection logic and strategies
├── async_traits.rs  # Async strategy traits and implementations
├── error.rs         # Error types and error handling
├── security.rs      # Security validation and rate limiting
├── iterators.rs     # Result processing and iterator extensions
├── macros.rs        # Convenience macros for target creation
├── presets.rs       # Common configuration presets
└── zero_cost.rs     # Zero-cost abstractions and compile-time optimizations
```

### Key Design Patterns

#### 1. NewType Pattern for Type Safety

```rust
// types.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port(NonZeroU16);

impl Port {
    pub fn new(port: u16) -> Option<Self> {
        NonZeroU16::new(port).map(Self)
    }

    // Specialized constructors with validation
    pub fn well_known(port: u16) -> Option<Self> {
        if (1..=1023).contains(&port) {
            Self::new(port)
        } else {
            None
        }
    }
}
```

**Benefits:**
- Compile-time prevention of invalid ports (0)
- Self-documenting code
- Zero runtime overhead

#### 2. Builder Pattern with Validation

```rust
// target.rs
pub struct TcpTargetBuilder {
    host: Option<Hostname>,
    port: Option<Port>,
}

impl TcpTargetBuilder {
    pub fn port(mut self, port: u16) -> Self {
        self.port = Port::new(port);
        self
    }

    pub fn build(self) -> Result<Target, TargetError> {
        let host = self.host.ok_or(TargetError::MissingHost)?;
        let port = self.port.ok_or(TargetError::MissingPort)?;

        Ok(Target::Tcp { host, port })
    }
}
```

**Benefits:**
- Fluent API
- Deferred validation
- Compile-time correctness where possible

#### 3. Strategy Pattern for Extensibility

```rust
// async_traits.rs
#[async_trait]
pub trait AsyncConnectionStrategy: Send + Sync {
    async fn wait_for_targets(
        &self,
        targets: &[Target],
        config: &WaitConfig,
    ) -> WaitResult;
}

pub struct WaitForAllStrategy;

#[async_trait]
impl AsyncConnectionStrategy for WaitForAllStrategy {
    async fn wait_for_targets(
        &self,
        targets: &[Target],
        config: &WaitConfig,
    ) -> WaitResult {
        // Implementation for "wait for all" logic
    }
}
```

**Benefits:**
- Pluggable behavior
- Easy testing with mock strategies
- Future extensibility

### Error Handling Philosophy

wait-for uses a hierarchical error system:

```rust
// error.rs
#[derive(thiserror::Error, Debug)]
pub enum WaitForError {
    #[error("Invalid target: {0}")]
    InvalidTarget(#[from] TargetError),

    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] ConnectionError),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Operation was cancelled")]
    Cancelled,
}
```

**Principles:**
- **Preserve context**: Use `#[from]` and `#[source]` appropriately
- **User-friendly messages**: Error messages should be actionable
- **Structured data**: Errors carry structured information for programmatic handling
- **No information loss**: Chain errors to preserve the full context

### Memory Management Strategy

```rust
// zero_cost.rs - Examples of zero-cost abstractions

// Small String Optimization
pub struct SmallString {
    data: SmallVec<[u8; 23]>, // Fits in 24 bytes total
}

// Compile-time string building
pub struct StringBuilder<const N: usize> {
    buffer: [u8; N],
    len: usize,
}

// Type-level port validation
pub struct ValidatedPort<const MIN: u16, const MAX: u16> {
    port: u16,
}
```

**Principles:**
- **Stack allocation preferred**: Minimize heap allocations
- **Compile-time computation**: Use const generics and const fn
- **Small String Optimization**: Store short strings inline
- **Lazy evaluation**: Defer expensive operations until needed

## Testing Strategy

### Test Organization

```
tests/
├── integration_tests.rs    # End-to-end CLI tests
├── common/
│   ├── mod.rs             # Shared test utilities
│   ├── fixtures.rs        # Test data and fixtures
│   └── servers.rs         # Test server implementations
└── property_tests/
    ├── mod.rs
    ├── port_validation.rs
    └── hostname_validation.rs
```

### Unit Testing

```rust
// src/types.rs
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_port_validation() {
        assert!(Port::new(80).is_some());
        assert!(Port::new(0).is_none());
    }

    // Property-based testing
    proptest! {
        #[test]
        fn test_port_roundtrip(port in 1u16..=65535) {
            let p = Port::new(port).unwrap();
            assert_eq!(p.get(), port);
        }
    }
}
```

### Integration Testing

```rust
// tests/integration_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_basic_tcp_success() {
    let server = start_test_server(8080);

    let mut cmd = Command::cargo_bin("wait-for").unwrap();
    cmd.arg("localhost:8080")
       .arg("--timeout=5s")
       .assert()
       .success();

    server.shutdown();
}
```

### Property-Based Testing

```rust
// Property tests for robust validation
proptest! {
    #[test]
    fn hostname_validation_properties(
        hostname in "[a-zA-Z0-9][a-zA-Z0-9\\-]{0,61}[a-zA-Z0-9]"
    ) {
        let result = Hostname::new(&hostname);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), hostname);
    }

    #[test]
    fn invalid_hostname_rejection(
        hostname in "[a-zA-Z]{254,300}" // Too long
    ) {
        let result = Hostname::new(&hostname);
        assert!(result.is_err());
    }
}
```

### Async Testing

```rust
#[tokio::test]
async fn test_connection_timeout() {
    let target = Target::tcp("127.0.0.1", 9999).unwrap(); // Unopened port
    let config = WaitConfig::builder()
        .timeout(Duration::from_millis(100))
        .build();

    let result = wait_for_connection(&[target], &config).await;
    assert!(matches!(result.unwrap_err(), WaitForError::Timeout(_)));
}
```

### Test Utilities

```rust
// tests/common/servers.rs
pub struct TestServer {
    handle: tokio::task::JoinHandle<()>,
    shutdown: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    pub async fn start_tcp(port: u16) -> Self {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();

            tokio::select! {
                _ = rx => {}, // Shutdown signal
                result = listener.accept() => {
                    // Handle connections
                }
            }
        });

        Self { handle, shutdown: tx }
    }

    pub fn shutdown(self) {
        let _ = self.shutdown.send(());
        self.handle.abort();
    }
}
```

## Extension Points

### Custom Target Types

To add support for new protocols:

```rust
// 1. Extend the Target enum (requires core changes)
pub enum Target {
    Tcp { host: Hostname, port: Port },
    Http { url: Url, expected_status: u16, headers: HeaderMap },

    // New target type
    Database { connection_string: String },
}

// 2. Implement checking logic
impl AsyncTargetChecker for DefaultTargetChecker {
    async fn check_target(&self, target: &Target) -> Result<(), ConnectionError> {
        match target {
            Target::Database { connection_string } => {
                // Database-specific connection logic
                check_database_connection(connection_string).await
            }
            // ... other target types
        }
    }
}
```

### Custom Retry Strategies

```rust
pub struct ExponentialBackoffWithJitter {
    base_interval: Duration,
    max_interval: Duration,
    jitter_factor: f64,
}

#[async_trait]
impl AsyncRetryStrategy for ExponentialBackoffWithJitter {
    async fn execute_with_retry<F, T, E>(
        &self,
        mut operation: F,
    ) -> Result<T, WaitForError>
    where
        F: FnMut() -> BoxFuture<'static, Result<T, E>> + Send,
        E: Into<WaitForError> + Send,
    {
        let mut interval = self.base_interval;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let jitter = fastrand::f64() * self.jitter_factor;
                    let sleep_duration = interval.mul_f64(1.0 + jitter);

                    tokio::time::sleep(sleep_duration).await;

                    interval = std::cmp::min(interval * 2, self.max_interval);
                }
            }
        }
    }
}
```

### Custom Progress Indicators

```rust
pub struct CustomProgressIndicator {
    pb: ProgressBar,
}

impl ProgressIndicator for CustomProgressIndicator {
    fn on_attempt(&self, target: &Target, attempt: usize) {
        self.pb.set_message(format!("Checking {} (attempt {})", target.display(), attempt));
        self.pb.inc(1);
    }

    fn on_success(&self, target: &Target, elapsed: Duration) {
        self.pb.finish_with_message(format!("✓ {} ready in {:?}", target.display(), elapsed));
    }

    fn on_failure(&self, target: &Target, error: &ConnectionError) {
        self.pb.finish_with_message(format!("✗ {} failed: {}", target.display(), error));
    }
}
```

### Plugin System (Future)

```rust
// Future plugin architecture
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn target_checkers(&self) -> Vec<Box<dyn AsyncTargetChecker>>;
    fn retry_strategies(&self) -> Vec<Box<dyn AsyncRetryStrategy>>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn get_target_checker(&self, name: &str) -> Option<&dyn AsyncTargetChecker> {
        // Find and return appropriate checker
    }
}
```

## Performance Considerations

### Async Performance

```rust
// Connection pooling for HTTP targets
pub struct HttpChecker {
    client: reqwest::Client,
}

impl HttpChecker {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}
```

### Memory Optimization

```rust
// Use SmallVec for targets (most use cases have few targets)
pub struct TargetList {
    targets: SmallVec<[Target; 4]>, // Inline up to 4 targets
}

// Intern common hostnames
pub struct HostnameInterner {
    cache: HashMap<String, Arc<str>>,
}

impl HostnameInterner {
    pub fn intern(&mut self, hostname: &str) -> Arc<str> {
        self.cache.entry(hostname.to_string())
            .or_insert_with(|| Arc::from(hostname))
            .clone()
    }
}
```

### Compile-Time Optimization

```rust
// Cargo.toml optimizations
[profile.release]
lto = true              # Link-time optimization
strip = true           # Strip debug symbols
opt-level = "z"        # Optimize for size
codegen-units = 1      # Better optimization
panic = "abort"        # Smaller binary size

# Feature flags for optional dependencies
[features]
default = ["http-client", "progress-bars"]
http-client = ["reqwest", "url"]
progress-bars = ["indicatif"]
json-output = ["serde_json"]
```

## Debugging and Profiling

### Debug Builds

```bash
# Debug with full logging
RUST_LOG=debug cargo run -- localhost:8080 --verbose

# Memory debugging with valgrind
cargo build
valgrind --tool=memcheck ./target/debug/wait-for localhost:8080

# Performance profiling
cargo build --release
perf record ./target/release/wait-for localhost:8080
perf report
```

### Async Debugging

```rust
// Add tracing for async debugging
use tracing::{info, debug, warn, error, instrument};

#[instrument]
async fn check_tcp_connection(host: &str, port: u16) -> Result<(), ConnectionError> {
    debug!("Attempting TCP connection to {}:{}", host, port);

    let addr = format!("{}:{}", host, port);
    let stream = tokio::time::timeout(
        Duration::from_secs(10),
        TcpStream::connect(&addr),
    ).await;

    match stream {
        Ok(Ok(_)) => {
            info!("TCP connection successful");
            Ok(())
        }
        Ok(Err(e)) => {
            warn!("TCP connection failed: {}", e);
            Err(ConnectionError::TcpError(e))
        }
        Err(_) => {
            error!("TCP connection timed out");
            Err(ConnectionError::Timeout)
        }
    }
}
```

### Benchmarking

```rust
// benches/connection_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_tcp_connection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("tcp_connection", |b| {
        b.to_async(&rt).iter(|| async {
            let target = Target::tcp("127.0.0.1", 80).unwrap();
            let config = WaitConfig::default();

            black_box(check_target(&target, &config).await)
        })
    });
}

criterion_group!(benches, bench_tcp_connection);
criterion_main!(benches);
```

## Release Process

### Version Management

```bash
# 1. Update version in Cargo.toml
cargo update

# 2. Update CHANGELOG.md
# Follow Keep a Changelog format

# 3. Run full test suite
cargo test --all-features
cargo clippy -- -D warnings
cargo fmt --all -- --check

# 4. Test release build
cargo build --release
./target/release/wait-for --version

# 5. Create release commit and tag
git commit -am "chore: bump version to v1.x.y"
git tag v1.x.y

# 6. Push with tags
git push origin main --tags
```

### Cross-Compilation

```bash
# Install cross
cargo install cross

# Build for multiple targets
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-apple-darwin --release
cross build --target x86_64-pc-windows-gnu --release
cross build --target aarch64-unknown-linux-gnu --release

# Test cross-compiled binaries
docker run --rm -v $(pwd):/app alpine /app/target/x86_64-unknown-linux-gnu/release/wait-for --help
```

### Docker Images

```dockerfile
# Multi-stage build for minimal image
FROM rust:slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/wait-for /usr/local/bin/wait-for
USER 1000:1000
ENTRYPOINT ["wait-for"]
```

### Automated Testing

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}

      - name: Run tests
        run: cargo test --all-features

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt --all -- --check
```

## Contributing Guidelines

### Code Review Checklist

- [ ] **Functionality**: Does the code solve the intended problem?
- [ ] **Performance**: Are there any performance regressions?
- [ ] **Security**: Are inputs validated? Any security implications?
- [ ] **Error Handling**: Are errors handled appropriately?
- [ ] **Testing**: Are there sufficient tests?
- [ ] **Documentation**: Is new functionality documented?
- [ ] **API Compatibility**: Any breaking changes properly marked?

### Commit Message Format

```
type(scope): brief description

Longer explanation if needed.

- List any breaking changes
- Reference issues: Fixes #123

Co-authored-by: Name <email>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

This guide should give you a comprehensive understanding of wait-for's internals and how to work with the codebase effectively. For specific questions, check the inline documentation or reach out to the maintainers.
