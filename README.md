# waitup

[![Crates.io](https://img.shields.io/crates/v/waitup.svg)](https://github.com/grok-rs/waitup)
[![Documentation](https://docs.rs/waitup/badge.svg)](https://docs.rs/waitup)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/grok-rs/waitup/workflows/CI/badge.svg)](https://github.com/grok-rs/waitup/actions)
[![GitHub Container Registry](https://img.shields.io/badge/GHCR-Available-blue)](https://ghcr.io/grok-rs/waitup)

> A modern, feature-rich CLI tool for waiting until TCP ports, HTTP endpoints, and services become available. Perfect for Docker, Kubernetes, CI/CD pipelines, and microservices orchestration.

**Why choose waitup?**

- **DNS Resolution**: Supports both hostnames and IP addresses
- **Multiple Targets**: Wait for multiple services with flexible strategies
- **HTTP Health Checks**: Full HTTP/HTTPS support with custom headers and status codes
- **Rich Output**: JSON output, progress indicators, and verbose logging
- **Container Ready**: Minimal Docker images and Kubernetes examples
- **High Performance**: Written in Rust for speed and reliability
- **Library Support**: Use as a Rust library in your applications

## Features

- **DNS Resolution**: Supports both hostnames and IP addresses
- **Multiple Targets**: Wait for multiple services with `--any` or `--all` strategies
- **Command Execution**: Run commands after successful connections
- **Progress Indicators**: Verbose mode with progress bars and attempt counters
- **HTTP Health Checks**: Support for HTTP/HTTPS endpoints with status code validation
- **Exponential Backoff**: Smart retry strategy with configurable intervals
- **Type Safety**: Built with Rust for reliability and performance
- **Environment Variables**: Configure defaults via environment
- **Optimized**: Small binary size with release optimizations

## Installation

### From Crates.io (Recommended)

```bash
cargo install waitup
```

### Docker

```bash
# Pull from GitHub Container Registry (recommended)
docker pull ghcr.io/grok-rs/waitup:latest
docker run --rm ghcr.io/grok-rs/waitup:latest --help

# Or use the Alpine build (smaller)
docker pull ghcr.io/grok-rs/waitup:alpine
docker run --rm ghcr.io/grok-rs/waitup:alpine --help

# Build from source using Docker
docker build -t waitup .
docker run --rm waitup --help

# Or build the Alpine variant from source
docker build -f Dockerfile.alpine -t waitup:alpine .
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/grok-rs/waitup/releases)

### From Source

```bash
git clone https://github.com/grok-rs/waitup
cd waitup
cargo install --path .
```

### Shell Completions

Generate completion scripts for your shell:

```bash
# Bash
waitup --generate-completion bash > /etc/bash_completion.d/waitup

# Zsh
waitup --generate-completion zsh > ~/.local/share/zsh/completions/_waitup

# Fish
waitup --generate-completion fish > ~/.config/fish/completions/waitup.fish
```

## Usage

### Basic TCP Connection

```bash
# Wait for a service to be ready
waitup localhost:8080

# With timeout and custom interval
waitup db:5432 --timeout 2m --interval 5s
```

### DNS Resolution

```bash
# Works with hostnames
waitup postgres-db:5432
waitup api.example.com:443
```

### Multiple Targets

```bash
# Wait for all services (default)
waitup db:5432 redis:6379 api:8080

# Wait for any service to be ready
waitup primary-db:5432 backup-db:5432 --any
```

### HTTP Health Checks

```bash
# HTTP endpoint health check
waitup https://api.example.com/health

# Custom status code expectation
waitup http://localhost:8080/ready --expect-status 204

# With custom headers (authentication, etc.)
waitup https://api.example.com/private --header "Authorization:Bearer token123" --header "X-API-Key:secret"
```

### Command Execution

```bash
# Run command after successful connection
waitup db:5432 -- npm start

# Multiple services before command
waitup db:5432 redis:6379 --all -- ./start-app.sh
```

### Progress and Verbose Output

```bash
# Verbose mode with progress information
waitup db:5432 --verbose

# Quiet mode (no output)
waitup db:5432 --quiet

# JSON output for CI/CD integration
waitup db:5432 redis:6379 --json
```

### Exponential Backoff

```bash
# Custom backoff configuration
waitup slow-service:8080 --interval 1s --max-interval 30s
```

## Environment Variables

Configure defaults using environment variables:

```bash
export WAITUP_TIMEOUT=60s
export WAITUP_INTERVAL=2s
waitup db:5432  # Uses env defaults
```

## Exit Codes

- `0`: Success - all targets are reachable
- `1`: Timeout - failed to connect within timeout period
- `2`: Invalid arguments or configuration
- `3`: Command execution failed (when using `--` syntax)

## Examples

### Docker Compose

```yaml
services:
  app:
    image: myapp
    depends_on:
      - db
    command: ["waitup", "db:5432", "--", "npm", "start"]

  db:
    image: postgres
```

### Kubernetes Init Container

```yaml
initContainers:
  - name: waitup-db
    image: waitup:latest
    command: ["waitup", "postgres:5432", "--timeout", "5m"]
```

### CI/CD Pipeline

```bash
# Wait for test database before running tests
waitup localhost:5432 --timeout 30s -- npm test
```

### Microservices Health Check

```bash
# Wait for multiple dependencies
waitup \
  auth-service:8001 \
  user-service:8002 \
  https://payment-api/health \
  --all \
  --timeout 2m \
  -- ./start-gateway.sh
```

## Time Format

Supports human-readable durations:

- `30s` - 30 seconds
- `2m` - 2 minutes
- `1h30m` - 1 hour 30 minutes
- `500ms` - 500 milliseconds

## Development

1. Fork the repository
2. Create a feature branch
3. Run tests: `cargo test`
4. Submit a pull request

## Advanced Features

### Retry Limits and Timeouts

```bash
# Limit retry attempts
waitup flaky-service:8080 --retry-limit 5

# Custom connection timeout per attempt
waitup slow-service:8080 --connection-timeout 30s
```

### JSON Output for Automation

```bash
# Perfect for CI/CD pipelines
waitup api:8080 db:5432 --json | jq '.success'

# Example JSON output:
{
  "success": true,
  "elapsed_ms": 2341,
  "total_attempts": 3,
  "targets": [
    {
      "target": "api:8080",
      "success": true,
      "elapsed_ms": 1205,
      "attempts": 2,
      "error": null
    }
  ]
}
```

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
waitup = "1.0"
```

Use in your Rust code:

```rust
use waitup::{Target, WaitConfig, wait_for_connection};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), waitup::WaitForError> {
    let targets = vec![
        Target::tcp("db", 5432)?,
        Target::parse("https://api.example.com/health", 200)?,
    ];

    let config = WaitConfig::builder()
        .timeout(Duration::from_secs(60))
        .interval(Duration::from_secs(2))
        .wait_for_any(false)
        .build();

    wait_for_connection(&targets, &config).await?;
    println!("All services ready!");
    Ok(())
}
```

## Docker Usage

### As a Kubernetes Init Container

```yaml
apiVersion: v1
kind: Pod
spec:
  initContainers:
    - name: waitup-deps
      image: waitup:alpine
      command: ["waitup"]
      args: ["postgres:5432", "redis:6379", "--timeout", "300s"]
  containers:
    - name: app
      image: myapp:latest
```

### In Docker Compose

```yaml
version: "3.8"
services:
  app:
    image: myapp:latest
    depends_on:
      db-ready:
        condition: service_completed_successfully

  db-ready:
    image: waitup:alpine
    command: ["waitup", "postgres:5432", "--timeout", "60s"]
    depends_on:
      - postgres

  postgres:
    image: postgres:15
```

## Comparison with Alternatives

| Feature             | waitup | waitup-it | dockerize | wait-on |
| ------------------- | -------- | ----------- | --------- | ------- |
| Language            | Rust     | Bash        | Go        | Node.js |
| HTTP Support        | ✅       | ❌          | ✅        | ✅      |
| Custom Headers      | ✅       | ❌          | ❌        | ❌      |
| JSON Output         | ✅       | ❌          | ❌        | ❌      |
| Multiple Strategies | ✅       | ❌          | ❌        | ✅      |
| DNS Resolution      | ✅       | ✅          | ✅        | ✅      |
| Binary Size         | ~6MB     | N/A         | ~8MB      | N/A     |
| Shell Completions   | ✅       | ❌          | ❌        | ❌      |
| Library Support     | ✅       | ❌          | ❌        | ❌      |

## Configuration Options

| Option                 | Environment Variable | Description                           |
| ---------------------- | -------------------- | ------------------------------------- |
| `--timeout`            | `WAITUP_TIMEOUT`   | Total timeout (default: 30s)          |
| `--interval`           | `WAITUP_INTERVAL`  | Initial retry interval (default: 1s)  |
| `--max-interval`       | -                    | Maximum retry interval (default: 30s) |
| `--connection-timeout` | -                    | Per-attempt timeout (default: 10s)    |
| `--retry-limit`        | -                    | Maximum retry attempts                |
| `--expect-status`      | -                    | Expected HTTP status (default: 200)   |

## FAQ

**Q: Why not just use `nc` or `telnet`?**
A: waitup provides proper error handling, exponential backoff, multiple targets, HTTP health checks, and structured output that makes it ideal for production deployments.

**Q: Does it work with IPv6?**
A: Yes! waitup supports both IPv4 and IPv6 through Rust's standard networking stack.

**Q: Can I use it to wait for services that require authentication?**
A: Yes, use custom headers: `--header "Authorization:Bearer token"`

**Q: How does it compare to Kubernetes readiness probes?**
A: waitup is perfect for init containers and external dependency checking, while readiness probes are for the service itself.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

- [Report bugs](https://github.com/grok-rs/waitup/issues/new?template=bug_report.md)
- [Request features](https://github.com/grok-rs/waitup/issues/new?template=feature_request.md)
- [Improve docs](https://github.com/grok-rs/waitup/edit/main/README.md)
- [Submit PRs](https://github.com/grok-rs/waitup/pulls)

## Performance

- **Startup time**: ~5ms
- **Memory usage**: ~2MB RSS
- **Binary size**: 6MB (standard), 10MB (Alpine)
- **Concurrent connections**: Efficiently handles 100+ targets

## Security

waitup follows security best practices:

- Runs as non-root user in containers
- No sensitive data logging
- Minimal attack surface
- Regularly updated dependencies

See [SECURITY.md](SECURITY.md) for vulnerability reporting.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [waitup-it](https://github.com/vishnubob/waitup-it) and [dockerize](https://github.com/jwilder/dockerize)
- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Thanks to all [contributors](https://github.com/grok-rs/waitup/graphs/contributors)!
