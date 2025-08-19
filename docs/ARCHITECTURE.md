# Architecture Documentation

This document provides an overview of wait-for's internal architecture, design decisions, and module organization.

## Overview

wait-for is a Rust CLI tool and library designed for high-performance network availability checking. The architecture follows a modular design with clear separation of concerns, emphasizing type safety, async performance, and zero-cost abstractions.

## Design Principles

### 1. Type Safety First
- **NewType Wrappers**: `Port`, `Hostname` types prevent invalid values at compile time
- **Builder Pattern**: Fluent APIs with compile-time validation
- **Strong Typing**: Prevents common network programming errors

### 2. Zero-Cost Abstractions
- **Compile-Time Optimizations**: Generic parameters and const generics where possible
- **Small String Optimization**: Custom string types for minimal allocations
- **Inline Functions**: Critical path functions are inlined

### 3. Async-First Design
- **Tokio Runtime**: Built on Tokio for high-performance async I/O
- **Concurrent Strategies**: Multiple connection strategies (all vs any)
- **Cancellation Support**: Graceful shutdown with cancellation tokens

### 4. Composable Architecture
- **Trait-Based Design**: Extensible through traits
- **Strategy Pattern**: Pluggable retry and connection strategies
- **Modular Components**: Each module has a single responsibility

## Module Architecture

```
wait-for/
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── main.rs             # CLI entry point
│   ├── cli.rs              # Command-line interface
│   ├── types.rs            # Core type definitions
│   ├── target.rs           # Target abstraction (TCP/HTTP)
│   ├── config.rs           # Configuration builders
│   ├── connection.rs       # Core connection logic
│   ├── async_traits.rs     # Async strategy traits
│   ├── error.rs            # Error types and handling
│   ├── security.rs         # Security validation
│   ├── iterators.rs        # Result processing utilities
│   ├── macros.rs           # Convenience macros
│   ├── presets.rs          # Common configurations
│   └── zero_cost.rs        # Zero-cost abstractions
```

## Core Components

### Target System (`target.rs`)

The `Target` enum is the heart of the system, representing different types of network endpoints:

```rust
pub enum Target {
    Tcp { host: Hostname, port: Port },
    Http { url: Url, expected_status: u16, headers: HeaderMap },
}
```

**Key Features:**
- **Type-Safe Construction**: Prevents invalid hostnames/ports at compile time
- **Builder Pattern**: Fluent API for complex configurations
- **Protocol Abstraction**: Unified interface for TCP and HTTP targets

### Configuration System (`config.rs`)

The configuration system uses the builder pattern for flexible and type-safe configuration:

```rust
pub struct WaitConfig {
    pub timeout: Duration,
    pub initial_interval: Duration,
    pub max_interval: Duration,
    pub connection_timeout: Duration,
    pub max_retries: Option<usize>,
    pub wait_for_any: bool,
    pub cancellation_token: Option<CancellationToken>,
}
```

**Design Benefits:**
- **Sensible Defaults**: Works out-of-the-box with reasonable settings
- **Progressive Disclosure**: Simple cases are simple, complex cases are possible
- **Validation**: Invalid configurations caught at build time

### Connection Engine (`connection.rs`)

The connection engine implements the core waiting logic with:

- **Exponential Backoff**: Smart retry strategy that adapts to network conditions
- **Concurrent Execution**: Parallel connection attempts for multiple targets
- **Graceful Cancellation**: Clean shutdown without resource leaks
- **Progress Reporting**: Optional progress indicators for long-running operations

### Async Strategy System (`async_traits.rs`)

The strategy system provides pluggable behavior through traits:

```rust
pub trait AsyncTargetChecker: Send + Sync {
    async fn check_target(&self, target: &Target) -> Result<(), ConnectionError>;
}

pub trait AsyncConnectionStrategy: Send + Sync {
    async fn wait_for_targets(&self, targets: &[Target], config: &WaitConfig) -> WaitResult;
}
```

**Benefits:**
- **Extensibility**: New strategies can be added without core changes
- **Testing**: Easy to mock and test different scenarios
- **Customization**: Users can provide custom strategies

## Data Flow

```
CLI Args → Args Parsing → Target Parsing → Config Building → Connection Strategy → Results
   ↓           ↓             ↓               ↓                   ↓              ↓
main.rs    cli.rs      target.rs       config.rs        async_traits.rs   types.rs
```

### 1. Input Processing
- CLI arguments parsed using `clap`
- Environment variables support for common settings
- Target strings parsed into typed `Target` instances

### 2. Configuration Phase
- `WaitConfig` built using builder pattern
- Validation occurs at construction time
- Cancellation token optionally created

### 3. Execution Phase
- Strategy selected based on configuration (`WaitForAllStrategy` vs `WaitForAnyStrategy`)
- Targets checked concurrently using async tasks
- Progress reported through optional indicators

### 4. Result Processing
- Results aggregated and validated
- Error context preserved through error chain
- Exit codes determined based on success/failure

## Error Handling Strategy

The error system uses a hierarchical approach:

```rust
pub enum WaitForError {
    InvalidTarget(String),
    InvalidConfiguration(String),
    ConnectionFailed(ConnectionError),
    Timeout(Duration),
    Cancelled,
}
```

**Key Features:**
- **Structured Errors**: Each error type carries relevant context
- **Error Chaining**: Root cause preserved through the stack
- **User-Friendly Messages**: Clear error messages for debugging
- **Exit Code Mapping**: Consistent exit codes for scripting

## Performance Considerations

### Memory Efficiency
- **Small String Optimization**: Hostnames stored efficiently
- **Minimal Allocations**: Reuse of buffers and connection pools
- **Stack Allocation**: Prefer stack over heap where possible

### Network Optimization
- **Connection Pooling**: HTTP client reuses connections
- **Concurrent Execution**: Multiple targets checked in parallel
- **Smart Backoff**: Exponential backoff reduces server load

### Compile-Time Optimization
- **Link-Time Optimization (LTO)**: Enabled in release builds
- **Dead Code Elimination**: Unused code removed
- **Inline Expansion**: Hot paths inlined aggressively

## Security Architecture

The security module provides:

- **Input Validation**: All external inputs validated
- **Rate Limiting**: Protection against excessive requests
- **Safe Defaults**: Secure configuration out-of-the-box
- **No Sensitive Logging**: Credentials never logged

## Extension Points

### Custom Target Types
Implement `AsyncTargetChecker` for new protocol support:

```rust
struct CustomChecker;

impl AsyncTargetChecker for CustomChecker {
    async fn check_target(&self, target: &Target) -> Result<(), ConnectionError> {
        // Custom logic here
    }
}
```

### Custom Retry Strategies
Implement `AsyncRetryStrategy` for different backoff algorithms:

```rust
struct CustomRetryStrategy;

impl AsyncRetryStrategy for CustomRetryStrategy {
    async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T, WaitForError> {
        // Custom retry logic
    }
}
```

## Testing Architecture

The codebase uses multiple testing strategies:

- **Unit Tests**: Each module has comprehensive unit tests
- **Property-Based Testing**: Using `proptest` for edge case discovery
- **Integration Tests**: End-to-end testing in `tests/` directory
- **Parameterized Tests**: Using `test-case` for systematic testing

## Build System Integration

### Cargo Features
- **Default Features**: Provide full functionality
- **Optional Features**: Allow minimal builds for embedded use
- **Feature Flags**: Enable/disable functionality at compile time

### Cross-Compilation
- **Multi-Platform Support**: Linux, macOS, Windows
- **Cross.toml Configuration**: Defines cross-compilation targets
- **Static Linking**: Produces portable binaries

## Future Architecture Considerations

### Planned Enhancements
- **Plugin System**: Dynamic loading of custom checkers
- **Configuration Files**: YAML/TOML configuration support
- **Metrics Export**: Prometheus metrics integration
- **Distributed Mode**: Check targets from multiple locations

### Backwards Compatibility
- **Semantic Versioning**: API changes follow semver
- **Deprecation Policy**: Gradual migration for breaking changes
- **Migration Guides**: Documentation for version upgrades

This architecture enables wait-for to be both simple for basic use cases and powerful for complex scenarios, while maintaining high performance and type safety throughout.
