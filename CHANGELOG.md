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
- 🔧 **Environment Variables**: Configure timeout and interval defaults via `WAIT_FOR_TIMEOUT` and `WAIT_FOR_INTERVAL`
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

## [Unreleased]

### Planned
- JSON output format for CI/CD integration
- Shell completion scripts (bash, zsh, fish, powershell)
- Docker image with multi-arch support
- Library API for programmatic usage
- Additional HTTP methods and headers support
- TLS/SSL certificate validation options
