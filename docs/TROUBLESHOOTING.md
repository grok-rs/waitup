# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when using waitup.

## Quick Diagnostics

Before diving into specific issues, try these general diagnostic steps:

1. **Test basic connectivity**: `waitup localhost:80 --timeout 5s`
2. **Enable verbose output**: Add `--verbose` to see detailed progress
3. **Check network basics**: `ping hostname` or `telnet hostname port`
4. **Verify target syntax**: Ensure proper `host:port` or URL format

## Common Issues and Solutions

### Connection Issues

#### "Connection refused" Error

**Symptoms:**
```
Error: Connection failed: Connection refused (os error 111)
```

**Causes & Solutions:**

1. **Service not running**
   ```bash
   # Check if service is listening
   ss -tulpn | grep :PORT
   netstat -tulpn | grep :PORT
   ```

2. **Wrong hostname/IP**
   ```bash
   # Test DNS resolution
   nslookup hostname
   dig hostname

   # Try IP directly
   waitup 192.168.1.100:8080 --timeout 10s
   ```

3. **Firewall blocking connection**
   ```bash
   # Test from same machine
   waitup localhost:8080

   # Check firewall rules
   sudo iptables -L
   sudo ufw status
   ```

4. **Service binding to localhost only**
   ```bash
   # Service might only bind to 127.0.0.1
   waitup 127.0.0.1:8080 instead of hostname:8080
   ```

#### "Connection timed out" Error

**Symptoms:**
```
Error: Connection failed: Connection timed out (os error 110)
```

**Solutions:**

1. **Increase connection timeout**
   ```bash
   waitup slow-service:8080 --connection-timeout 30s
   ```

2. **Check network latency**
   ```bash
   ping hostname
   traceroute hostname
   ```

3. **Verify service is responsive**
   ```bash
   # Use verbose mode to see retry attempts
   waitup hostname:8080 --verbose
   ```

#### "Name resolution failed" Error

**Symptoms:**
```
Error: Invalid target: Name resolution failed
```

**Solutions:**

1. **Check DNS configuration**
   ```bash
   # Test DNS resolution
   nslookup hostname
   dig hostname

   # Try with IP address instead
   waitup 192.168.1.100:8080
   ```

2. **Use localhost for local services**
   ```bash
   waitup localhost:8080
   waitup 127.0.0.1:8080
   ```

3. **Check /etc/hosts for local entries**
   ```bash
   cat /etc/hosts | grep hostname
   ```

### HTTP/HTTPS Issues

#### "HTTP request failed" Error

**Symptoms:**
```
Error: HTTP request failed: status code 404
```

**Solutions:**

1. **Check expected status code**
   ```bash
   # Default expects 200, adjust as needed
   waitup https://api.example.com/health --expect-status 204
   ```

2. **Verify URL path**
   ```bash
   # Test with curl first
   curl -I https://api.example.com/health

   # Then use waitup
   waitup https://api.example.com/health
   ```

3. **Add required headers**
   ```bash
   waitup https://api.example.com/private \
     --header "Authorization:Bearer token123" \
     --header "X-API-Key:secret"
   ```

#### "SSL/TLS Certificate Error"

**Symptoms:**
```
Error: HTTP request failed: certificate verify failed
```

**Solutions:**

1. **For development/testing** (⚠️ Not recommended for production)
   ```bash
   # Use HTTP instead of HTTPS
   waitup http://api.example.com/health
   ```

2. **Check certificate validity**
   ```bash
   curl -I https://api.example.com/health
   openssl s_client -connect api.example.com:443
   ```

3. **Use proper CA certificates**
   ```bash
   # Update ca-certificates package
   sudo apt update && sudo apt install ca-certificates
   ```

### Timeout Issues

#### "Operation timed out" Error

**Symptoms:**
```
Error: Timeout after 30s waiting for targets to become available
```

**Solutions:**

1. **Increase overall timeout**
   ```bash
   waitup slow-service:8080 --timeout 5m
   ```

2. **Adjust retry intervals**
   ```bash
   waitup slow-service:8080 \
     --interval 5s \
     --max-interval 30s
   ```

3. **Limit retry attempts**
   ```bash
   waitup unreliable-service:8080 --retry-limit 10
   ```

4. **Use environment variables for defaults**
   ```bash
   export WAITUP_TIMEOUT=120s
   export WAITUP_INTERVAL=5s
   waitup slow-service:8080
   ```

### Multiple Target Issues

#### Some targets fail in "wait for all" mode

**Symptoms:**
```
Error: Not all targets became available within timeout
```

**Solutions:**

1. **Check which target is failing**
   ```bash
   # Use JSON output to see individual results
   waitup db:5432 redis:6379 api:8080 --json
   ```

2. **Test targets individually**
   ```bash
   waitup db:5432 --timeout 30s
   waitup redis:6379 --timeout 30s
   waitup api:8080 --timeout 30s
   ```

3. **Use "wait for any" strategy**
   ```bash
   # Succeed when any target is ready
   waitup primary-db:5432 backup-db:5432 --any
   ```

4. **Stagger target checking**
   ```bash
   # Check dependencies in order
   waitup db:5432 --timeout 60s && \
   waitup redis:6379 --timeout 30s && \
   waitup api:8080 --timeout 30s
   ```

## Docker-Specific Issues

### Container Networking

**Problem:** waitup works on host but fails in Docker container

**Solutions:**

1. **Use correct hostname in Docker**
   ```yaml
   # docker-compose.yml
   services:
     app:
       command: ["waitup", "postgres:5432", "--", "npm", "start"]
       depends_on:
         - postgres
     postgres:
       image: postgres:15
   ```

2. **Check Docker network**
   ```bash
   # List Docker networks
   docker network ls

   # Inspect specific network
   docker network inspect myapp_default
   ```

3. **Use Docker internal IPs**
   ```bash
   # Find container IP
   docker inspect postgres | grep IPAddress
   ```

### Permission Issues in Containers

**Problem:** Permission denied errors in Docker

**Solutions:**

1. **Run as non-root user**
   ```dockerfile
   FROM rust:slim
   RUN adduser --disabled-password --gecos '' waituser
   USER waituser
   COPY --chown=waituser:waituser waitup /usr/local/bin/
   ```

2. **Use official Docker image**
   ```bash
   docker run --rm ghcr.io/grok-rs/waitup:alpine hostname:port
   ```

## Kubernetes-Specific Issues

### Init Container Problems

**Problem:** Init container keeps restarting

**Solution:**

1. **Check init container logs**
   ```bash
   kubectl logs pod-name -c init-container-name
   ```

2. **Use appropriate timeout**
   ```yaml
   initContainers:
   - name: waitup-deps
     image: waitup:alpine
     command: ["waitup"]
     args: ["postgres:5432", "--timeout", "300s"]
   ```

3. **Verify service names**
   ```bash
   # List services in namespace
   kubectl get services
   ```

### DNS Resolution in Kubernetes

**Problem:** Service names not resolving

**Solutions:**

1. **Use fully qualified service names**
   ```bash
   waitup postgres.default.svc.cluster.local:5432
   ```

2. **Check service discovery**
   ```bash
   kubectl exec -it pod-name -- nslookup postgres
   ```

## CI/CD Pipeline Issues

### Pipeline Timeouts

**Problem:** CI/CD pipelines failing due to timeouts

**Solutions:**

1. **Set appropriate timeouts for CI**
   ```bash
   # GitHub Actions
   waitup db:5432 --timeout 120s -- npm test

   # GitLab CI
   waitup db:5432 --timeout 2m -- ./run-tests.sh
   ```

2. **Use background services**
   ```yaml
   # .github/workflows/test.yml
   services:
     postgres:
       image: postgres:15
       options: >-
         --health-cmd pg_isready
         --health-interval 10s
         --health-timeout 5s
         --health-retries 5
   ```

### Environment Variable Conflicts

**Problem:** CI environment overriding settings

**Solution:**

1. **Explicit parameters override env vars**
   ```bash
   # This works even if WAITUP_TIMEOUT is set
   waitup db:5432 --timeout 60s
   ```

2. **Clear environment variables**
   ```bash
   unset WAITUP_TIMEOUT WAITUP_INTERVAL
   waitup db:5432 --timeout 30s
   ```

## Performance Issues

### Slow Connection Checking

**Problem:** waitup seems slow

**Solutions:**

1. **Reduce retry interval**
   ```bash
   waitup fast-service:8080 --interval 100ms
   ```

2. **Use parallel checking for multiple targets**
   ```bash
   # waitup automatically parallelizes multiple targets
   waitup db:5432 redis:6379 api:8080
   ```

3. **Set connection timeout**
   ```bash
   waitup slow-service:8080 --connection-timeout 5s
   ```

### Memory Usage

**Problem:** High memory usage with many targets

**Solution:**

1. **Check targets in batches**
   ```bash
   waitup db:5432 cache:6379 && \
   waitup api:8080 worker:8081
   ```

2. **Use lightweight Docker image**
   ```dockerfile
   FROM ghcr.io/grok-rs/waitup:alpine
   ```

## Debugging Tips

### Enable Verbose Output

```bash
waitup hostname:port --verbose
```

This shows:
- Connection attempts
- Retry intervals
- Progress information
- Detailed error messages

### Use JSON Output for Scripting

```bash
waitup db:5432 redis:6379 --json | jq '.targets[] | select(.success == false)'
```

### Test with Minimal Configuration

```bash
# Simplest possible test
waitup localhost:80 --timeout 5s --interval 1s
```

### Check Exit Codes

```bash
waitup hostname:port
echo "Exit code: $?"
```

Exit codes:
- `0`: Success
- `1`: Timeout
- `2`: Invalid arguments
- `3`: Command execution failed

### Network Debugging Commands

```bash
# Check if port is open
nc -zv hostname port

# Test HTTP endpoint
curl -I http://hostname:port/path

# Check DNS resolution
dig hostname
nslookup hostname

# Check routing
traceroute hostname

# Check local listening ports
ss -tulpn
netstat -tulpn
```

## Getting Help

If you're still experiencing issues:

1. **Check the GitHub issues**: [github.com/grok-rs/waitup/issues](https://github.com/grok-rs/waitup/issues)
2. **Enable verbose output** and include it in your issue report
3. **Provide minimal reproduction case**
4. **Include environment information**:
   ```bash
   waitup --version
   uname -a
   docker --version  # if using Docker
   ```

## Common Environment-Specific Solutions

### macOS

```bash
# Install via Homebrew if available
brew install waitup

# Or use Docker
docker run --rm ghcr.io/grok-rs/waitup:alpine hostname:port
```

### Windows

```bash
# Use WSL2
wsl
waitup hostname:port

# Or use Docker Desktop
docker run --rm ghcr.io/grok-rs/waitup:alpine hostname:port
```

### Alpine Linux

```bash
# Install gcompat if needed
apk add gcompat

# Or use Alpine-specific build
docker run --rm ghcr.io/grok-rs/waitup:alpine hostname:port
```

This troubleshooting guide covers the most common issues. For more complex scenarios, refer to the [Architecture Documentation](ARCHITECTURE.md) for deeper understanding of waitup's internals.
