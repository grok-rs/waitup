# waitup

A lightweight CLI tool for waiting until TCP ports and HTTP endpoints become available. Perfect for Docker containers, Kubernetes, and CI/CD pipelines.

[![Crates.io](https://img.shields.io/crates/v/waitup.svg)](https://crates.io/crates/waitup)
[![Downloads](https://img.shields.io/crates/d/waitup.svg)](https://crates.io/crates/waitup)
[![Documentation](https://docs.rs/waitup/badge.svg)](https://docs.rs/waitup)
[![CI Status](https://github.com/grok-rs/waitup/workflows/CI/badge.svg)](https://github.com/grok-rs/waitup/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/docker-ghcr.io-blue.svg)](https://github.com/grok-rs/waitup/pkgs/container/waitup)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

## Quick Start

```bash
# Install from source
git clone https://github.com/grok-rs/waitup.git
cd waitup
cargo install --path .

# Wait for a TCP port
waitup localhost:5432

# Wait for HTTP endpoint
waitup https://api.example.com/health --expect-status 200

# Run command after service is ready
waitup postgres:5432 -- npm start
```

## Installation

### From Crates.io (Recommended)

```bash
cargo install waitup
```

### From Source

```bash
git clone https://github.com/grok-rs/waitup.git
cd waitup
cargo install --path .
```

### Docker

```bash
# Pull from GitHub Container Registry
docker pull ghcr.io/grok-rs/waitup:latest
docker pull ghcr.io/grok-rs/waitup:alpine

# Or build locally
# Standard image (92MB)
docker build -t waitup .

# Alpine image (3MB)
docker build -f Dockerfile.alpine -t waitup:alpine .
```

### Pre-built Binaries

Download from [releases](https://github.com/grok-rs/waitup/releases).

## Usage

### Basic Examples

```bash
# Wait for TCP port with timeout
waitup localhost:8080 --timeout 30s

# Wait for multiple services
waitup db:5432 redis:6379 api:8080

# Wait for any service to be ready
waitup primary-db:5432 backup-db:5432 --any

# HTTP health check with custom headers
waitup https://api.example.com/health \
  --header "Authorization:Bearer token" \
  --expect-status 200
```

### Docker Compose

```yaml
services:
  app:
    image: myapp
    depends_on:
      - db
    entrypoint: ["/bin/sh", "-c"]
    command:
      - |
        waitup db:5432 --timeout 60s -- npm start

  db:
    image: postgres:15

  # Or use as separate service
  wait-for-db:
    image: ghcr.io/grok-rs/waitup:alpine
    command: ["db:5432", "--timeout", "60s"]
    depends_on:
      - db
```

### Kubernetes Init Container

```yaml
initContainers:
  - name: wait-for-db
    image: ghcr.io/grok-rs/waitup:alpine
    command: ["waitup", "postgres:5432", "--timeout", "5m"]
```

## Command Line Options

```
waitup [OPTIONS] <TARGETS>... [-- <COMMAND>...]

Options:
  --timeout <DURATION>            Total timeout (default: 30s)
  --interval <DURATION>           Initial retry interval (default: 1s)
  --max-interval <DURATION>       Maximum retry interval (default: 30s)
  --connection-timeout <DURATION> Per-attempt timeout (default: 10s)
  --expect-status <CODE>          Expected HTTP status (default: 200)
  --header <KEY:VALUE>            Custom HTTP headers
  --any                           Wait for any target (default: all)
  --verbose                       Show detailed progress
  --quiet                         No output except errors
  --json                          JSON output format
  -h, --help                      Print help
  -V, --version                   Print version
```

## Time Format

Use human-readable duration formats:
- `30s` - 30 seconds
- `2m` - 2 minutes
- `1h30m` - 1 hour 30 minutes
- `500ms` - 500 milliseconds

## Exit Codes

- `0` - Success, all targets are reachable
- `1` - Timeout, failed to connect within timeout
- `2` - Invalid arguments or configuration
- `3` - Command execution failed (when using `--`)

## Environment Variables

Set defaults to avoid repetitive flags:

```bash
export WAITUP_TIMEOUT=60s
export WAITUP_INTERVAL=2s

waitup db:5432  # Uses environment defaults
```

## Features

- TCP port checking with DNS resolution
- HTTP/HTTPS health checks with custom headers
- Wait for multiple services (all or any)
- Execute commands after services are ready
- Exponential backoff with configurable intervals
- JSON output for automation
- Shell completions (bash, zsh, fish, powershell)
- Docker and Kubernetes ready

## Shell Completions

```bash
# Bash
waitup --generate-completion bash > ~/.local/share/bash-completion/completions/waitup

# Zsh
waitup --generate-completion zsh > ~/.local/share/zsh/site-functions/_waitup

# Fish
waitup --generate-completion fish > ~/.config/fish/completions/waitup.fish
```

## Examples

### Database Startup

```bash
# Wait for PostgreSQL before migrations
waitup postgres:5432 --timeout 60s -- npm run migrate

# Wait for multiple databases
waitup mysql:3306 redis:6379 postgres:5432 -- npm start
```

### Microservices

```bash
# Wait for dependencies with health checks
waitup \
  https://auth-service/health \
  https://user-service/health \
  --timeout 2m \
  -- ./start-gateway.sh
```

### CI/CD Pipelines

```bash
# Wait for test services
waitup localhost:5432 localhost:6379 --timeout 30s -- npm test
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Related Projects

Similar tools for service orchestration:
- [wait-for-it](https://github.com/vishnubob/wait-for-it) - Bash script for TCP waiting
- [dockerize](https://github.com/jwilder/dockerize) - Go-based utility with templating
- [wait-on](https://github.com/jeffbski/wait-on) - Node.js based waiting tool

## Security

Report vulnerabilities at [SECURITY.md](SECURITY.md).
