# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-08-17

### Added

- 🌐 **DNS Resolution Support**: Wait for hostnames, not just IP addresses
- 🔗 **Multiple Targets**: Support waiting for multiple services with `--any` or `--all` strategies
- 🚀 **Command Execution**: Run commands after successful connections using `-- command` syntax
- 📊 **Progress Indicators**: Verbose mode with progress bars and attempt counters using indicatif
- 🏥 **HTTP/HTTPS Health Checks**: Support for HTTP endpoints with customizable status code validation
- 📈 **Exponential Backoff**: Smart retry strategy with configurable maximum intervals
- 🛡️ **Better Error Handling**: Structured error types using thiserror for clear error messages
- 🔧 **Environment Variables**: Configure timeout and interval defaults via `WAITUP_TIMEOUT` and `WAITUP_INTERVAL`
- ⚡ **Release Optimizations**: LTO, strip symbols, size optimization for minimal binary
- ⏱️ **Human-readable Time**: Support for time formats like "30s", "2m", "1h30m" using humantime
- 🎯 **Type Safety**: Full Rust type safety with proper error propagation
- 📦 **Comprehensive Testing**: Integration tests and CI/CD pipeline
- 📖 **Rich Documentation**: Extensive examples and use cases

### Features

- TCP port connectivity testing with DNS resolution
- HTTP/HTTPS endpoint health checking
- Multiple target support with flexible strategies
- Command execution after successful connections
- Exponential backoff retry strategy
- Progress indicators and verbose logging
- Environment variable configuration
- Human-readable time format support
- Optimized binary size and performance

### Exit Codes

- `0`: Success - all targets are reachable
- `1`: Timeout - failed to connect within timeout period
- `2`: Invalid arguments or configuration
- `3`: Command execution failed

### Examples

- Docker Compose integration
- Kubernetes init containers
- CI/CD pipeline usage
- Microservices orchestration
- Health check automation

## [1.1.0] - 2025-10-17

### Added

- **Type-safe Port Classification**: New `PortCategory` enum implementing RFC 6335 port ranges
  - System Ports (0-1023): `is_system_port()`, `Port::system_port()`
  - User Ports (1024-49151): `is_user_port()`, `Port::user_port()`
  - Dynamic Ports (49152-65535): `is_dynamic_port()`, `Port::dynamic_port()`
  - `Port::category()` method for type-safe port classification
- **Semantic Newtype Wrappers**: Self-documenting types with built-in validation
  - `StatusCode`: HTTP status code validation with semantic methods (`is_success()`, `is_client_error()`, etc.)
  - `RetryCount`: Semantic wrapper for retry logic with constants (FEW, MODERATE, MANY, AGGRESSIVE)
  - `Hostname`: Helper methods (`is_localhost()`, `is_loopback()`, `is_ipv4()`, `is_ipv6()`)
- **API Extensibility**: Added `#[non_exhaustive]` attribute to public enums (`Target`, `ConnectionError`, `HttpError`, `ErrorSource`, `WaitForError`) for future-proof API evolution
- **Rust 2024 Edition**: Upgraded to Rust 2024 edition with minimum rust-version 1.85.0

### Changed

- **Performance Optimizations**: Strategic `#[inline]` hints on hot-path functions
- **Code Quality**: Refactored `wait_with_progress` function, extracted helper functions to improve maintainability
- Port method naming updated to RFC 6335 official terminology for better standards compliance

### Fixed

- Verbose progress now streams per-target completion updates in real-time
- Clippy lints: `uninlined_format_args`, `too_many_lines`, unnecessary semicolons
- Simplified `Port::new_unchecked` to remove unsafe code patterns

### Dependencies

- Updated `clap` to 4.5.48
- Updated `clap_complete` to 4.5.58
- Updated `thiserror` to 2.0.17
- Updated `humantime` to 2.3.0
- Updated `serde` to 1.0.228
- Updated `proptest` to 1.8.0
- Multiple minor dependency updates for improved security and performance

## [1.1.1] - 2025-10-22

### Changed

#### Dependencies

- Updated `clap` to 4.5.50
- Updated `clap_complete` to 4.5.59
- Updated `tokio` to 1.48.0
- Updated `reqwest` to 0.12.24
- Updated `rustls` to 0.23.34
- Updated `indicatif` to 0.18.1
- Updated 51 other transitive dependencies for improved security and performance

## [Unreleased]

### Planned

- JSON output format for CI/CD integration
- Shell completion scripts (bash, zsh, fish, powershell)
- Docker image with multi-arch support
- Library API for programmatic usage
- Additional HTTP methods and headers support
- TLS/SSL certificate validation options
