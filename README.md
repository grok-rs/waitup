# waitup - Wait for TCP Ports & HTTP Services | Docker/Kubernetes Ready

## The Modern Port Checker and Service Health Monitor for DevOps

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT "MIT License")
[![CI Status](https://github.com/grok-rs/waitup/workflows/CI/badge.svg?style=flat-square)](https://github.com/grok-rs/waitup/actions "Continuous Integration")
[![Rust](https://img.shields.io/badge/language-Rust-orange.svg?style=flat-square)](https://www.rust-lang.org/ "Built with Rust")
[![Docker Ready](https://img.shields.io/badge/docker-ready-blue.svg?style=flat-square)](https://github.com/grok-rs/waitup#installation "Docker Compatible")

**waitup** is a powerful **TCP port checker** and **service health monitor** that waits for services to become available. Essential for **Docker**, **Kubernetes**, **CI/CD pipelines**, and **microservices orchestration**. Replace `wait-for-it`, `dockerize`, and other dependency management tools with a single, fast, reliable solution.

---

## 📖 Table of Contents

- [🚀 Why waitup?](#-why-waitup-the-best-port-checker-for-modern-devops)
- [📊 vs Other Tools](#-waitup-vs-other-tools)
- [🎯 Common Use Cases](#-common-use-cases---when-to-use-waitup)
- [⚙️ Installation](#installation)
- [📘 Usage Examples](#usage)
- [❓ FAQ](#-frequently-asked-questions)
- [🔧 Quick Solutions](#-quick-solutions-for-common-problems)
- [📚 Resources](#-related-tools-and-resources)

---

## 🚀 Why waitup? The Best Port Checker for Modern DevOps

Perfect for developers who need to:

- **Wait for database startup** in Docker containers
- **Check if port is open** before starting services
- **Monitor service health** in Kubernetes clusters
- **Ensure service dependencies** are ready in CI/CD
- **Test HTTP endpoints** with custom headers and status codes

### ⚡ Key Advantages Over Alternatives

- ✅ **TCP & HTTP Support**: Check ports and HTTP endpoints with custom headers
- ✅ **Multiple Service Monitoring**: Wait for any or all services simultaneously
- ✅ **Smart Retry Logic**: Exponential backoff with configurable intervals
- ✅ **Rich Output Options**: JSON, verbose, and quiet modes for any workflow
- ✅ **Container Optimized**: Alpine Docker images under 10MB
- ✅ **Production Ready**: Built with Rust for speed, safety, and reliability

## 📊 waitup vs Other Tools

Choose the best port checker and service health monitor:

| Feature             | **waitup** | dockerize | wait-on | wait-for-it |
| ------------------- | ---------- | --------- | ------- | ----------- |
| **Language**        | Rust 🦀    | Go        | Node.js | Bash        |
| **HTTP Support**    | ✅ Full    | ✅ Basic  | ✅ Basic| ❌ None     |
| **Custom Headers**  | ✅ Yes     | ❌ No     | ❌ No   | ❌ No       |
| **JSON Output**     | ✅ Yes     | ❌ No     | ❌ No   | ❌ No       |
| **Multiple Targets**| ✅ Any/All | ❌ No     | ✅ Basic| ❌ No       |
| **DNS Resolution**  | ✅ Yes     | ✅ Yes    | ✅ Yes  | ✅ Yes      |
| **Binary Size**     | ~6MB       | ~8MB      | N/A     | N/A         |
| **Shell Completions** | ✅ Yes   | ❌ No     | ❌ No   | ❌ No       |
| **Library Support** | ✅ Rust    | ❌ No     | ❌ No   | ❌ No       |
| **Docker Ready**    | ✅ Alpine  | ✅ Yes    | ✅ Yes  | ✅ Yes      |

## 🎯 Common Use Cases - When to Use waitup

### Database Startup in Docker

```bash
# Wait for PostgreSQL before running migrations
waitup postgres:5432 --timeout 60s -- npm run migrate

# Wait for multiple databases
waitup mysql:3306 redis:6379 postgres:5432 --all -- npm start
```

### Kubernetes Init Containers

```bash
# Wait for external services before pod starts
waitup external-api:443 database:5432 --timeout 300s
```

### CI/CD Pipeline Dependencies

```bash
# Ensure test services are ready
waitup localhost:5432 localhost:6379 --timeout 30s -- npm test
```

### Microservices Health Checks

```bash
# Wait for API dependencies with custom headers
waitup https://auth-service/health --header "X-Health-Check:true" --expect-status 200
```

### Docker Compose Service Dependencies

```yaml
services:
  app:
    depends_on:
      - db-ready
    command: ["./app"]

  db-ready:
    image: waitup:alpine
    command: ["waitup", "postgres:5432", "--timeout", "60s"]
```

## Installation

### Prerequisites

- **Rust toolchain** (1.70.0 or later)
- **Git** for cloning the repository
- **Docker** (optional, for containerized usage)

### Quick Start

The fastest way to get started with waitup:

```bash
# Clone and install in one step
git clone https://github.com/grok-rs/waitup.git && cd waitup && cargo install --path .

# Verify installation
waitup --version
```

### Installation Methods

#### 📦 From Source (Recommended)

```bash
# 1. Clone the repository
git clone https://github.com/grok-rs/waitup.git
cd waitup

# 2. Build and install
cargo install --path .

# 3. Verify installation
waitup --help
```

#### 🐳 Docker

##### Option 1: Use pre-built image from source

```bash
# Build the standard image
docker build -t waitup .

# Test it works
docker run --rm waitup --version
```

##### Option 2: Use Alpine variant (smaller size)

```bash
# Build Alpine image (recommended for production)
docker build -f Dockerfile.alpine -t waitup:alpine .

# Test it works
docker run --rm waitup:alpine --version
```

##### Option 3: Use in your Dockerfile

```dockerfile
# Multi-stage build example
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/waitup /usr/local/bin/waitup
ENTRYPOINT ["waitup"]
```

#### 🔧 Development Setup

For contributing or development:

```bash
# Clone with full history
git clone https://github.com/grok-rs/waitup.git
cd waitup

# Install development dependencies
cargo build

# Run tests
cargo test

# Install for development
cargo install --path . --force
```

### Post-Installation Setup

#### Shell Completions

Enhance your command-line experience with auto-completions:

```bash
# Bash (system-wide)
sudo waitup --generate-completion bash > /usr/share/bash-completion/completions/waitup

# Bash (user-specific)
waitup --generate-completion bash > ~/.local/share/bash-completion/completions/waitup

# Zsh
waitup --generate-completion zsh > ~/.local/share/zsh/site-functions/_waitup

# Fish
waitup --generate-completion fish > ~/.config/fish/completions/waitup.fish

# PowerShell (Windows)
waitup --generate-completion powershell > waitup.ps1
```

#### Environment Configuration

Set default values to avoid repetitive flags:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export WAITUP_TIMEOUT=60s
export WAITUP_INTERVAL=2s

# Now you can use shorter commands
waitup db:5432  # Uses your defaults
```

### Verification

Confirm everything is working:

```bash
# Check version
waitup --version

# Test basic functionality
waitup google.com:80 --timeout 5s

# Verify completions (if installed)
waitup <TAB><TAB>  # Should show available options
```

### Troubleshooting

**Command not found after installation:**

```bash
# Ensure Cargo's bin directory is in your PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

**Permission denied for shell completions:**

```bash
# Use user-specific directories instead of system-wide
mkdir -p ~/.local/share/bash-completion/completions
waitup --generate-completion bash > ~/.local/share/bash-completion/completions/waitup
```

**Docker build fails:**

```bash
# Ensure you have the latest base images
docker pull rust:1.70
docker pull debian:bookworm-slim
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

You can use waitup as a Rust library in your applications. Check the source code and documentation in the repository for API details.

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

## Configuration Options

| Option                 | Environment Variable | Description                           |
| ---------------------- | -------------------- | ------------------------------------- |
| `--timeout`            | `WAITUP_TIMEOUT`   | Total timeout (default: 30s)          |
| `--interval`           | `WAITUP_INTERVAL`  | Initial retry interval (default: 1s)  |
| `--max-interval`       | -                    | Maximum retry interval (default: 30s) |
| `--connection-timeout` | -                    | Per-attempt timeout (default: 10s)    |
| `--retry-limit`        | -                    | Maximum retry attempts                |
| `--expect-status`      | -                    | Expected HTTP status (default: 200)   |

## ❓ Frequently Asked Questions

### How do I wait for a database to start in Docker?

Use `waitup database:5432 --timeout 60s -- your-app-command` to wait for PostgreSQL, MySQL, or any database before starting your application.

### What's the best way to check if a port is open?

```bash
waitup hostname:port --timeout 10s
```

Returns exit code 0 if successful, 1 if timeout, 2 for invalid arguments.

### How do I wait for multiple services before starting my app?

```bash
# Wait for ALL services (default)
waitup db:5432 redis:6379 api:8080 -- npm start

# Wait for ANY service to be ready
waitup primary-db:5432 backup-db:5432 --any -- npm start
```

### Can waitup replace wait-for-it in Docker containers?

**Yes!** waitup is a modern replacement with better features:

- ✅ HTTP health checks (wait-for-it only does TCP)
- ✅ JSON output for automation
- ✅ Multiple target strategies
- ✅ Custom headers for authenticated endpoints
- ✅ Smaller binary size (~6MB vs bash dependency)

### How do I wait for HTTP services with authentication?

```bash
waitup https://api.example.com/health \
  --header "Authorization:Bearer $TOKEN" \
  --header "X-API-Key:$API_KEY" \
  --expect-status 200
```

### Does waitup work with IPv6 addresses?

**Yes!** waitup supports both IPv4 and IPv6 through Rust's standard networking stack. Use IPv6 addresses normally: `waitup [::1]:8080`

### What's the difference between waitup and dockerize?

| Feature | waitup | dockerize |
|---------|--------|-----------|
| HTTP Headers | ✅ Yes | ❌ No |
| JSON Output | ✅ Yes | ❌ No |
| Multiple Strategies | ✅ Any/All | ❌ Sequential |
| Language | Rust | Go |
| Binary Size | ~6MB | ~8MB |

### How do I troubleshoot "connection refused" errors?

1. **Check the service is running**: `docker ps` or `kubectl get pods`
2. **Verify the port**: Use `--verbose` flag to see connection attempts
3. **Increase timeout**: Services might take longer to start
4. **Check networking**: Ensure containers can reach each other

### Can I use waitup in Kubernetes init containers?

**Absolutely!** Perfect for waiting for external dependencies:

```yaml
initContainers:
  - name: wait-for-db
    image: waitup:alpine
    command: ["waitup", "postgres:5432", "--timeout", "300s"]
```

### How do I wait for a service to be completely ready, not just accepting connections?

Use HTTP health checks instead of TCP:

```bash
# Instead of: waitup api:8080
# Use health endpoint:
waitup https://api:8080/health --expect-status 200
```

## 🔧 Quick Solutions for Common Problems

### "Connection Refused" Error

```bash
# Problem: Service not accepting connections yet
# Solution: Increase timeout and add verbose logging
waitup database:5432 --timeout 120s --verbose

# For Docker: Ensure containers are on same network
docker network ls
```

### Service Takes Too Long to Start

```bash
# Problem: Default timeout too short
# Solution: Increase timeout and interval
waitup slow-service:8080 --timeout 5m --interval 10s --max-interval 30s
```

### Docker Compose Dependencies Not Working

```yaml
# Problem: App starts before database ready
# Solution: Use waitup in entrypoint
services:
  app:
    entrypoint: ["waitup", "postgres:5432", "--", "npm", "start"]
    depends_on:
      - postgres
```

### Kubernetes Pod Won't Start

```yaml
# Problem: External dependencies not ready
# Solution: Add waitup init container
initContainers:
  - name: wait-deps
    image: waitup:alpine
    command: ["waitup", "external-api:443", "database:5432"]
```

### HTTP Health Check Failing

```bash
# Problem: API returns 503 during startup
# Solution: Wait for specific status or use different endpoint
waitup https://api/ready --expect-status 204
# OR
waitup https://api/health --header "Accept:application/json"
```

### CI/CD Pipeline Timeouts

```bash
# Problem: Tests failing because services not ready
# Solution: Add explicit wait with JSON output
waitup localhost:5432 localhost:6379 --json --timeout 60s
if [ $? -eq 0 ]; then npm test; fi
```

## 📚 Related Tools and Resources

- **[Docker Compose Wait Strategies](https://docs.docker.com/compose/startup-order/)** - Official Docker guidance
- **[Kubernetes Init Containers](https://kubernetes.io/docs/concepts/workloads/pods/init-containers/)** - Pod initialization patterns
- **[Health Check Patterns](https://microservices.io/patterns/observability/health-check-api.html)** - Microservices health monitoring
- **[12-Factor App Dependencies](https://12factor.net/dependencies)** - Dependency management best practices

### Migration Guides

- **From wait-for-it**: Replace `./wait-for-it.sh host:port` with `waitup host:port`
- **From dockerize**: Replace `dockerize -wait tcp://host:port` with `waitup host:port`
- **From wait-on**: Replace `wait-on tcp:host:port` with `waitup host:port`
- **[Request Migration Guide](https://github.com/grok-rs/waitup/issues/new?template=feature_request.md)** - Need help migrating?

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

- [Report bugs](https://github.com/grok-rs/waitup/issues/new?template=bug_report.md)
- [Request features](https://github.com/grok-rs/waitup/issues/new?template=feature_request.md)
- [Improve docs](https://github.com/grok-rs/waitup/edit/main/README.md)
- [Submit PRs](https://github.com/grok-rs/waitup/pulls)

## Performance

Built with Rust for high performance, low memory usage, and fast startup times.

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

- Inspired by [dockerize](https://github.com/jwilder/dockerize)
- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Thanks to all [contributors](https://github.com/grok-rs/waitup/graphs/contributors)!
