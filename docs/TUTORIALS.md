# Tutorials

This guide provides step-by-step tutorials to help you learn wait-for from basic usage to advanced scenarios. Each tutorial builds on previous knowledge.

## Tutorial 1: Your First Connection Check

Let's start with the simplest possible use case: checking if a service is available.

### Prerequisites
- wait-for installed ([Installation Guide](../README.md#installation))
- A service running on your machine (we'll use a web server on port 8080)

### Step 1: Start a Test Service

First, let's start a simple HTTP server for testing:

```bash
# Option 1: Using Python (if available)
python3 -m http.server 8080

# Option 2: Using Node.js (if available)
npx http-server -p 8080

# Option 3: Using Docker
docker run -p 8080:80 nginx:alpine
```

### Step 2: Basic Connection Check

Now test if the service is reachable:

```bash
wait-for localhost:8080
```

**Expected result:** The command should complete successfully (exit code 0) since the service is running.

### Step 3: Test with Unreachable Service

Stop your test service and try again:

```bash
wait-for localhost:8080
```

**Expected result:** After 30 seconds (default timeout), the command fails with exit code 1.

### Step 4: Customize the Timeout

Let's reduce the wait time:

```bash
wait-for localhost:8080 --timeout 5s
```

**What you learned:**
- Basic `wait-for hostname:port` syntax
- Default timeout is 30 seconds
- Exit codes indicate success (0) or failure (1)
- You can customize timeout with `--timeout`

---

## Tutorial 2: HTTP Health Checks

Many modern applications provide HTTP health check endpoints. Let's learn to use them.

### Prerequisites
- Completed Tutorial 1
- A service with an HTTP endpoint (we'll simulate one)

### Step 1: Create a Mock Health Endpoint

Start a service that responds on a health endpoint:

```bash
# Using Docker with a health endpoint
docker run -d --name nginx-test -p 8080:80 nginx:alpine

# Wait for it to start, then test the endpoint
curl http://localhost:8080
```

### Step 2: HTTP Health Check

Check the HTTP endpoint:

```bash
wait-for http://localhost:8080
```

This expects a 200 status code by default.

### Step 3: Custom Status Codes

Some health endpoints return different status codes:

```bash
# If your endpoint returns 204 No Content
wait-for http://localhost:8080/health --expect-status 204

# Or 202 Accepted
wait-for http://localhost:8080/status --expect-status 202
```

### Step 4: HTTPS Endpoints

For secure endpoints:

```bash
wait-for https://httpbin.org/status/200
```

### Step 5: Custom Headers

Some APIs require authentication:

```bash
wait-for https://httpbin.org/bearer \
  --header "Authorization:Bearer your-token-here"

# Multiple headers
wait-for https://api.example.com/health \
  --header "Authorization:Bearer token" \
  --header "X-API-Key:secret" \
  --header "Content-Type:application/json"
```

### Step 6: Real-world Health Check

Let's test a real API:

```bash
# GitHub API (public, no auth needed)
wait-for https://api.github.com --expect-status 200

# JSONPlaceholder (test API)
wait-for https://jsonplaceholder.typicode.com/posts/1
```

### Cleanup

```bash
docker stop nginx-test
docker rm nginx-test
```

**What you learned:**
- HTTP/HTTPS endpoint checking with `http://` or `https://`
- Custom status codes with `--expect-status`
- Authentication headers with `--header`
- Multiple headers can be specified

---

## Tutorial 3: Managing Timeouts and Retries

Understanding how wait-for handles timing is crucial for production use.

### Prerequisites
- Completed Tutorial 2
- Docker installed (for simulation)

### Step 1: Understanding Default Behavior

```bash
# Default settings
wait-for localhost:9999 --verbose
```

Observe:
- Total timeout: 30 seconds
- Initial interval: 1 second
- Exponential backoff up to 30 seconds maximum

### Step 2: Customizing Timeouts

```bash
# Longer total timeout
wait-for localhost:9999 --timeout 2m

# Faster checking interval
wait-for localhost:9999 --interval 500ms --timeout 10s

# Control maximum interval
wait-for localhost:9999 --interval 1s --max-interval 10s --timeout 30s
```

### Step 3: Simulating Slow Startup

Let's simulate a service with slow startup:

```bash
# Start a container that takes 15 seconds to be ready
docker run -d --name slow-start -p 9999:80 \
  --health-cmd="sleep 15 && curl -f http://localhost/ || exit 1" \
  --health-interval=5s \
  nginx:alpine

# Wait for it with appropriate settings
wait-for localhost:9999 --timeout 30s --interval 2s --verbose
```

### Step 4: Retry Limits

Sometimes you want to limit attempts rather than time:

```bash
# Maximum 10 attempts regardless of time
wait-for unreliable-service:8080 --retry-limit 10

# Combine with other settings
wait-for unreliable-service:8080 \
  --retry-limit 5 \
  --interval 3s \
  --timeout 30s
```

### Step 5: Environment Variables

Set defaults for your environment:

```bash
# Set environment defaults
export WAIT_FOR_TIMEOUT=60s
export WAIT_FOR_INTERVAL=2s

# These now use your defaults
wait-for service1:8080
wait-for service2:8081

# Command line options override environment
wait-for service3:8082 --timeout 10s

# Clean up
unset WAIT_FOR_TIMEOUT WAIT_FOR_INTERVAL
```

### Cleanup

```bash
docker stop slow-start
docker rm slow-start
```

**What you learned:**
- `--timeout` sets total wait time
- `--interval` sets initial retry interval
- `--max-interval` caps exponential backoff
- `--retry-limit` limits number of attempts
- Environment variables set defaults
- `--verbose` shows detailed progress

---

## Tutorial 4: Multiple Services

Real applications often depend on multiple services. Let's learn to handle this.

### Prerequisites
- Completed Tutorial 3
- Docker and Docker Compose installed

### Step 1: Create a Multi-Service Setup

Create a `docker-compose.yml`:

```yaml
version: "3.8"
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"

  web:
    image: nginx:alpine
    ports:
      - "8080:80"
```

Start the services:

```bash
docker-compose up -d
```

### Step 2: Wait for All Services (Default)

```bash
# Wait for all services to be ready
wait-for localhost:5432 localhost:6379 localhost:8080 --timeout 60s
```

This succeeds only when ALL services are ready.

### Step 3: Wait for Any Service

Sometimes you want to proceed when any service is ready:

```bash
# Primary and backup databases
wait-for primary-db:5432 backup-db:5432 --any --timeout 30s

# For our example (both should work, so this will succeed quickly)
wait-for localhost:5432 localhost:6379 --any --timeout 10s
```

### Step 4: Mixed TCP and HTTP Targets

```bash
# Mix TCP and HTTP checks
wait-for \
  localhost:5432 \
  localhost:6379 \
  http://localhost:8080 \
  --timeout 45s \
  --verbose
```

### Step 5: JSON Output for Automation

```bash
# Get detailed results in JSON
wait-for localhost:5432 localhost:6379 localhost:8080 --json
```

Example output:
```json
{
  "success": true,
  "elapsed_ms": 2341,
  "total_attempts": 3,
  "targets": [
    {
      "target": "localhost:5432",
      "success": true,
      "elapsed_ms": 1205,
      "attempts": 2,
      "error": null
    }
  ]
}
```

### Step 6: Staged Dependency Checking

Sometimes services have dependencies on each other:

```bash
# Database first
wait-for localhost:5432 --timeout 30s && \
echo "Database ready" && \

# Then cache
wait-for localhost:6379 --timeout 15s && \
echo "Cache ready" && \

# Finally web service
wait-for http://localhost:8080 --timeout 15s && \
echo "Web service ready"
```

### Step 7: Running Commands After Success

```bash
# Run a command after services are ready
wait-for localhost:5432 localhost:6379 -- echo "Services are ready!"

# More realistic example
wait-for localhost:5432 localhost:6379 -- npm start

# With shell command
wait-for localhost:5432 -- sh -c "echo 'DB ready at $(date)'"
```

### Cleanup

```bash
docker-compose down
```

**What you learned:**
- Multiple targets in one command
- `--all` (default) vs `--any` strategies
- Mixing TCP and HTTP targets
- `--json` output for scripting
- Running commands with `--`
- Staged dependency checking

---

## Tutorial 5: Docker Integration

Learn to use wait-for effectively in containerized environments.

### Prerequisites
- Completed Tutorial 4
- Docker and Docker Compose knowledge

### Step 1: Basic Docker Usage

```bash
# Pull the official image
docker pull ghcr.io/grok-rs/wait-for:latest

# Basic usage
docker run --rm ghcr.io/grok-rs/wait-for:latest --help

# Test connectivity (this will fail, showing the pattern)
docker run --rm ghcr.io/grok-rs/wait-for:latest google.com:80 --timeout 5s
```

### Step 2: Docker Compose with wait-for

Create `tutorial-compose.yml`:

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s
      timeout: 5s
      retries: 5

  app:
    image: alpine:latest
    command: |
      sh -c "
        echo 'App starting...' &&
        echo 'App ready!'
      "
    depends_on:
      postgres-ready:
        condition: service_completed_successfully

  postgres-ready:
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for", "postgres:5432", "--timeout", "60s"]
    depends_on:
      - postgres
```

Run it:

```bash
docker-compose -f tutorial-compose.yml up
```

### Step 3: Init Containers Pattern

Create `init-container.yml`:

```yaml
version: "3.8"
services:
  db:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password

  app:
    image: alpine:latest
    command: ["sh", "-c", "echo 'App running with ready dependencies!'"]
    init: true
    depends_on:
      - init-wait

  init-wait:
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for", "db:5432", "--timeout", "30s", "--verbose"]
    depends_on:
      - db
```

### Step 4: Multi-Stage Waiting

For complex dependency chains:

```yaml
version: "3.8"
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password

  redis:
    image: redis:alpine

  migrate:
    image: ghcr.io/grok-rs/wait-for:alpine
    command: |
      sh -c "
        wait-for postgres:5432 --timeout 60s &&
        echo 'Running migrations...' &&
        sleep 2 &&
        echo 'Migrations complete'
      "
    depends_on:
      - postgres

  app:
    image: ghcr.io/grok-rs/wait-for:alpine
    command: |
      sh -c "
        wait-for postgres:5432 redis:6379 --timeout 30s &&
        echo 'All dependencies ready, starting app...'
      "
    depends_on:
      - postgres
      - redis
      - migrate
```

### Step 5: Custom Application Integration

Example `Dockerfile` for your app:

```dockerfile
FROM rust:slim as builder
WORKDIR /app
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

# Install wait-for
COPY --from=ghcr.io/grok-rs/wait-for:latest /usr/local/bin/wait-for /usr/local/bin/wait-for

COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp

# Your app waits for dependencies, then starts
CMD ["wait-for", "postgres:5432", "redis:6379", "--timeout", "60s", "--", "myapp"]
```

### Step 6: Health Checks in Docker

Combining with Docker health checks:

```yaml
version: "3.8"
services:
  api:
    image: my-api:latest
    healthcheck:
      test: ["CMD", "wait-for", "localhost:8080", "--timeout", "3s"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 30s
```

### Cleanup

```bash
docker-compose -f tutorial-compose.yml down
docker-compose -f init-container.yml down
```

**What you learned:**
- Using official wait-for Docker images
- Docker Compose integration patterns
- Init container pattern for dependencies
- Multi-stage dependency waiting
- Health check integration
- Custom Dockerfile integration

---

## Tutorial 6: Kubernetes Integration

Learn to use wait-for in Kubernetes environments.

### Prerequisites
- Completed Tutorial 5
- Kubernetes cluster access (minikube, kind, or real cluster)
- kubectl installed

### Step 1: Basic Init Container

Create `k8s-basic.yaml`:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-with-wait
spec:
  initContainers:
  - name: wait-for-db
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for"]
    args: ["postgres.default.svc.cluster.local:5432", "--timeout", "120s"]

  containers:
  - name: app
    image: alpine:latest
    command: ["sh", "-c", "echo 'App started!' && sleep 3600"]

---
apiVersion: v1
kind: Service
metadata:
  name: postgres
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres
spec:
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:15
        env:
        - name: POSTGRES_PASSWORD
          value: "password"
        ports:
        - containerPort: 5432
```

Deploy and test:

```bash
kubectl apply -f k8s-basic.yaml
kubectl get pods -w  # Watch the init container run
kubectl logs app-with-wait -c wait-for-db
```

### Step 2: ConfigMap Configuration

Create `k8s-config.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: wait-config
data:
  WAIT_FOR_TIMEOUT: "300s"
  WAIT_FOR_INTERVAL: "5s"

---
apiVersion: v1
kind: Pod
metadata:
  name: app-with-config
spec:
  initContainers:
  - name: wait-for-services
    image: ghcr.io/grok-rs/wait-for:alpine
    envFrom:
    - configMapRef:
        name: wait-config
    command: ["wait-for"]
    args: ["postgres:5432", "redis:6379", "--verbose"]

  containers:
  - name: app
    image: alpine:latest
    command: ["sleep", "3600"]
```

### Step 3: Multiple Dependencies

Create `k8s-multi.yaml`:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: multi-dep-app
spec:
  initContainers:
  # Wait for database first
  - name: wait-for-db
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for", "postgres:5432", "--timeout", "60s"]

  # Then wait for cache
  - name: wait-for-cache
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for", "redis:6379", "--timeout", "30s"]

  # Finally wait for API
  - name: wait-for-api
    image: ghcr.io/grok-rs/wait-for:alpine
    command: ["wait-for", "http://api-service/health", "--timeout", "45s"]

  containers:
  - name: app
    image: alpine:latest
    command: ["sh", "-c", "echo 'All dependencies ready!' && sleep 3600"]
```

### Step 4: Deployment with Init Containers

Create `k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-app
  template:
    metadata:
      labels:
        app: web-app
    spec:
      initContainers:
      - name: wait-dependencies
        image: ghcr.io/grok-rs/wait-for:alpine
        command: ["wait-for"]
        args:
        - "postgres:5432"
        - "redis:6379"
        - "--timeout=180s"
        - "--verbose"

      containers:
      - name: web
        image: nginx:alpine
        ports:
        - containerPort: 80
```

### Step 5: Job Pattern

For one-time tasks:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: database-migration
spec:
  template:
    spec:
      initContainers:
      - name: wait-for-db
        image: ghcr.io/grok-rs/wait-for:alpine
        command: ["wait-for", "postgres:5432", "--timeout", "300s"]

      containers:
      - name: migrate
        image: my-migration:latest
        command: ["./run-migrations.sh"]

      restartPolicy: Never
  backoffLimit: 3
```

### Step 6: Helm Chart Integration

Create `helm-template.yaml`:

```yaml
{{- if .Values.waitFor.enabled }}
initContainers:
{{- range .Values.waitFor.services }}
- name: wait-for-{{ .name }}
  image: {{ $.Values.waitFor.image }}
  command: ["wait-for"]
  args:
  - "{{ .target }}"
  - "--timeout={{ .timeout | default "60s" }}"
  {{- if .verbose }}
  - "--verbose"
  {{- end }}
{{- end }}
{{- end }}

containers:
# Your app containers...
```

With `values.yaml`:

```yaml
waitFor:
  enabled: true
  image: "ghcr.io/grok-rs/wait-for:alpine"
  services:
  - name: database
    target: "postgres:5432"
    timeout: "120s"
    verbose: true
  - name: cache
    target: "redis:6379"
    timeout: "30s"
```

### Step 7: Monitoring and Debugging

```bash
# Check init container status
kubectl describe pod app-with-wait

# View init container logs
kubectl logs app-with-wait -c wait-for-db

# Debug failed init containers
kubectl get events --sort-by=.metadata.creationTimestamp

# Test connectivity from within cluster
kubectl run debug --image=alpine --rm -it -- sh
# Then inside the container:
apk add curl
curl postgres:5432
```

### Cleanup

```bash
kubectl delete -f k8s-basic.yaml
kubectl delete -f k8s-config.yaml
kubectl delete -f k8s-multi.yaml
kubectl delete -f k8s-deployment.yaml
```

**What you learned:**
- Init container pattern in Kubernetes
- Service discovery with DNS names
- ConfigMap for environment configuration
- Multi-stage dependency waiting
- Deployment integration
- Job pattern for migrations
- Helm chart templating
- Debugging techniques

---

## Next Steps

Congratulations! You've completed all the wait-for tutorials. You now know:

✅ Basic connection checking
✅ HTTP health checks with authentication
✅ Timeout and retry management
✅ Multiple service coordination
✅ Docker integration patterns
✅ Kubernetes deployment strategies

### Recommended Reading

- [Integration Patterns](INTEGRATION_PATTERNS.md) - Common real-world patterns
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Solve common issues
- [Developer Guide](DEVELOPER_GUIDE.md) - Extend and customize wait-for
- [Performance Guide](PERFORMANCE.md) - Optimize for your use case

### Practice Projects

Try these to cement your knowledge:

1. **Microservices Stack**: Create a Docker Compose setup with 5+ services using wait-for
2. **CI/CD Pipeline**: Integrate wait-for into GitHub Actions or GitLab CI
3. **Kubernetes App**: Deploy a real application with proper dependency waiting
4. **Health Check Dashboard**: Use JSON output to create a service status dashboard
5. **Custom Integration**: Write a script that uses wait-for for your specific use case

### Community and Support

- 📖 [Documentation](../README.md)
- 🐛 [Report Issues](https://github.com/grok-rs/wait-for/issues)
- 💡 [Feature Requests](https://github.com/grok-rs/wait-for/issues/new)
- 🤝 [Contribute](../CONTRIBUTING.md)

Happy waiting! 🚀
