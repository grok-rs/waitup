# Integration Patterns

This guide provides proven patterns for integrating waitup into various environments and workflows. These patterns are based on real-world usage and community best practices.

## Table of Contents

- [Docker Patterns](#docker-patterns)
- [Kubernetes Patterns](#kubernetes-patterns)
- [CI/CD Patterns](#cicd-patterns)
- [Microservices Patterns](#microservices-patterns)
- [Development Patterns](#development-patterns)
- [Monitoring Patterns](#monitoring-patterns)

## Docker Patterns

### Pattern 1: Basic Service Dependency

**Use Case:** Ensure database is ready before starting application

```yaml
# docker-compose.yml
version: "3.8"
services:
  app:
    image: myapp:latest
    command: ["waitup", "postgres:5432", "--timeout", "60s", "--", "npm", "start"]
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password
```

**Benefits:**
- Simple and reliable
- App won't start until database is ready
- Prevents connection errors on startup

### Pattern 2: Multi-Stage Service Dependencies

**Use Case:** Complex startup order with migrations

```yaml
version: "3.8"
services:
  # 1. Database starts first
  postgres:
    image: postgres:15
    environment:
      POSTGRES_PASSWORD: password
    healthcheck:
      test: ["CMD-SHELL", "pg_isready"]
      interval: 5s

  # 2. Wait for DB, run migrations
  migrate:
    image: myapp:latest
    command: |
      sh -c "
        waitup postgres:5432 --timeout 60s &&
        npm run migrate
      "
    depends_on:
      postgres:
        condition: service_healthy

  # 3. Wait for DB and migrations, start app
  app:
    image: myapp:latest
    command: |
      sh -c "
        waitup postgres:5432 --timeout 30s &&
        npm start
      "
    depends_on:
      migrate:
        condition: service_completed_successfully
```

### Pattern 3: Health Check Integration

**Use Case:** App readiness checking for load balancer

```yaml
services:
  app:
    image: myapp:latest
    healthcheck:
      test: ["CMD", "waitup", "localhost:8080", "/health", "--timeout", "5s"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
    ports:
      - "8080:8080"
```

### Pattern 4: Sidecar Pattern

**Use Case:** Service mesh or logging sidecar

```yaml
services:
  app:
    image: myapp:latest
    command: |
      sh -c "
        waitup localhost:9901 --timeout 30s && # Envoy admin
        waitup postgres:5432 --timeout 60s &&
        npm start
      "

  envoy:
    image: envoyproxy/envoy:latest
    ports:
      - "9901:9901"  # Admin port
    volumes:
      - ./envoy.yaml:/etc/envoy/envoy.yaml
```

### Pattern 5: Development Override

**Use Case:** Different behavior in development vs production

```yaml
# docker-compose.yml
services:
  app:
    image: myapp:latest
    command: ["waitup", "postgres:5432", "--", "npm", "start"]

# docker-compose.override.yml (for development)
services:
  app:
    command: |
      sh -c "
        waitup postgres:5432 --timeout 120s &&
        npm run dev -- --reload
      "
    volumes:
      - .:/app
    environment:
      NODE_ENV: development
```

## Kubernetes Patterns

### Pattern 1: Init Container Dependency

**Use Case:** Standard microservice with database dependency

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
      - name: waitup-dependencies
        image: ghcr.io/grok-rs/waitup:alpine
        command: ["waitup"]
        args: ["postgres.database.svc.cluster.local:5432", "--timeout", "300s"]

      containers:
      - name: web
        image: myapp:latest
        ports:
        - containerPort: 8080
```

### Pattern 2: Job-Based Migration

**Use Case:** Database migrations before deployment

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-migrate
  annotations:
    "helm.sh/hook": pre-install,pre-upgrade
    "helm.sh/hook-weight": "1"
spec:
  template:
    spec:
      initContainers:
      - name: waitup-db
        image: ghcr.io/grok-rs/waitup:alpine
        command: ["waitup", "postgres:5432", "--timeout", "300s", "--verbose"]

      containers:
      - name: migrate
        image: myapp:migrate
        command: ["./run-migrations.sh"]

      restartPolicy: Never
  backoffLimit: 3
```

### Pattern 3: ConfigMap-Driven Dependencies

**Use Case:** Environment-specific dependencies

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-dependencies
data:
  dependencies.txt: |
    postgres.database.svc.cluster.local:5432
    redis.cache.svc.cluster.local:6379
    elasticsearch.logging.svc.cluster.local:9200

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  template:
    spec:
      initContainers:
      - name: wait-dependencies
        image: ghcr.io/grok-rs/waitup:alpine
        command: |
          sh -c "
            while read dep; do
              echo \"Waiting for $dep\"
              waitup $dep --timeout 60s
            done < /config/dependencies.txt
          "
        volumeMounts:
        - name: deps-config
          mountPath: /config

      volumes:
      - name: deps-config
        configMap:
          name: app-dependencies
```

### Pattern 4: Helm Chart Template

**Use Case:** Reusable dependency waiting across services

```yaml
# templates/deployment.yaml
{{- if .Values.dependencies.enabled }}
initContainers:
{{- range .Values.dependencies.services }}
- name: waitup-{{ .name }}
  image: {{ $.Values.dependencies.waitFor.image }}
  command: ["waitup"]
  args:
    - {{ .target | quote }}
    - "--timeout={{ .timeout | default "60s" }}"
    {{- if .expectStatus }}
    - "--expect-status={{ .expectStatus }}"
    {{- end }}
    {{- if $.Values.dependencies.verbose }}
    - "--verbose"
    {{- end }}
  {{- with .env }}
  env:
    {{- toYaml . | nindent 4 }}
  {{- end }}
{{- end }}
{{- end }}

# values.yaml
dependencies:
  enabled: true
  verbose: true
  waitFor:
    image: "ghcr.io/grok-rs/waitup:alpine"
  services:
    - name: postgres
      target: "postgres.database.svc.cluster.local:5432"
      timeout: "120s"
    - name: api
      target: "http://api-service/health"
      expectStatus: 200
      timeout: "60s"
```

### Pattern 5: CronJob Health Monitoring

**Use Case:** Periodic health checks

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: health-monitor
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: health-check
            image: ghcr.io/grok-rs/waitup:alpine
            command: |
              sh -c "
                waitup api-service:80 --json > /tmp/result.json
                if [ $? -eq 0 ]; then
                  echo 'Health check passed'
                else
                  echo 'Health check failed'
                  cat /tmp/result.json
                fi
              "
          restartPolicy: OnFailure
```

## CI/CD Patterns

### Pattern 1: GitHub Actions Integration

**Use Case:** Integration tests with service dependencies

```yaml
# .github/workflows/test.yml
name: Test
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: password
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Install waitup
        run: |
          wget -O waitup https://github.com/grok-rs/waitup/releases/latest/download/waitup-linux-x86_64
          chmod +x waitup
          sudo mv waitup /usr/local/bin/

      - name: Wait for services
        run: |
          waitup localhost:5432 --timeout 30s

      - name: Run tests
        run: |
          npm test
```

### Pattern 2: GitLab CI Integration

**Use Case:** Multi-stage pipeline with service dependencies

```yaml
# .gitlab-ci.yml
stages:
  - test
  - deploy

variables:
  POSTGRES_PASSWORD: password

services:
  - postgres:15

test:
  stage: test
  image: node:18
  before_script:
    - apt-get update && apt-get install -y wget
    - wget -O waitup https://github.com/grok-rs/waitup/releases/latest/download/waitup-linux-x86_64
    - chmod +x waitup && mv waitup /usr/local/bin/
    - waitup postgres:5432 --timeout 60s
  script:
    - npm ci
    - npm test

integration_test:
  stage: test
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker-compose -f docker-compose.test.yml up -d
    - docker run --rm --network container:$(docker-compose -f docker-compose.test.yml ps -q app)
        ghcr.io/grok-rs/waitup:alpine
        localhost:8080 --timeout 60s
    - # Run integration tests
```

### Pattern 3: Jenkins Pipeline

**Use Case:** Complex deployment pipeline

```groovy
// Jenkinsfile
pipeline {
    agent any

    stages {
        stage('Deploy Services') {
            steps {
                sh 'docker-compose up -d postgres redis'
            }
        }

        stage('Wait for Dependencies') {
            steps {
                sh '''
                    docker run --rm --network myapp_default
                        ghcr.io/grok-rs/waitup:alpine
                        postgres:5432 redis:6379 --timeout 120s --verbose
                '''
            }
        }

        stage('Run Migrations') {
            steps {
                sh 'docker-compose run --rm migrate'
            }
        }

        stage('Start Application') {
            steps {
                sh 'docker-compose up -d app'
                sh '''
                    docker run --rm --network myapp_default
                        ghcr.io/grok-rs/waitup:alpine
                        app:8080 --timeout 60s
                '''
            }
        }

        stage('Integration Tests') {
            steps {
                sh 'npm run test:integration'
            }
        }
    }

    post {
        always {
            sh 'docker-compose down -v'
        }
    }
}
```

### Pattern 4: Azure DevOps Pipeline

**Use Case:** Multi-environment deployment

```yaml
# azure-pipelines.yml
trigger:
  - main

pool:
  vmImage: ubuntu-latest

variables:
  - group: production-secrets

stages:
  - stage: Test
    jobs:
      - job: IntegrationTest
        services:
          postgres:
            image: postgres:15
            env:
              POSTGRES_PASSWORD: $(POSTGRES_PASSWORD)
        steps:
          - script: |
              curl -L https://github.com/grok-rs/waitup/releases/latest/download/waitup-linux-x86_64 -o waitup
              chmod +x waitup
              sudo mv waitup /usr/local/bin/
            displayName: 'Install waitup'

          - script: waitup localhost:5432 --timeout 30s
            displayName: 'Wait for database'

          - script: npm test
            displayName: 'Run tests'

  - stage: Deploy
    dependsOn: Test
    jobs:
      - deployment: Production
        environment: 'production'
        strategy:
          runOnce:
            deploy:
              steps:
                - script: |
                    kubectl apply -f k8s/
                    kubectl wait --for=condition=available deployment/app --timeout=300s
                  displayName: 'Deploy to Kubernetes'
```

## Microservices Patterns

### Pattern 1: Service Mesh Integration

**Use Case:** Istio service mesh with dependency management

```yaml
apiVersion: v1
kind: Service
metadata:
  name: user-service
spec:
  ports:
  - port: 8080
  selector:
    app: user-service

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
spec:
  template:
    metadata:
      labels:
        app: user-service
    spec:
      initContainers:
      # Wait for Istio proxy
      - name: waitup-proxy
        image: ghcr.io/grok-rs/waitup:alpine
        command: ["waitup", "localhost:15000", "--timeout", "30s"]

      # Wait for dependencies
      - name: waitup-deps
        image: ghcr.io/grok-rs/waitup:alpine
        command: ["waitup"]
        args:
          - "postgres.database:5432"
          - "http://auth-service:8080/health"
          - "--timeout=120s"

      containers:
      - name: user-service
        image: user-service:latest
```

### Pattern 2: Event-Driven Architecture

**Use Case:** Services depending on message brokers

```yaml
version: "3.8"
services:
  # Message broker
  kafka:
    image: confluentinc/cp-kafka:latest
    environment:
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
    depends_on:
      - zookeeper

  # Producer service
  producer:
    image: producer-service:latest
    command: |
      sh -c "
        waitup kafka:9092 --timeout 120s &&
        waitup http://schema-registry:8081/subjects --timeout 60s &&
        npm start
      "
    depends_on:
      - kafka
      - schema-registry

  # Consumer service
  consumer:
    image: consumer-service:latest
    command: |
      sh -c "
        waitup kafka:9092 --timeout 120s &&
        waitup postgres:5432 --timeout 60s &&
        npm start
      "
    depends_on:
      - kafka
      - postgres
```

### Pattern 3: API Gateway Pattern

**Use Case:** Gateway waiting for upstream services

```yaml
version: "3.8"
services:
  # Upstream services
  user-service:
    image: user-service:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]

  order-service:
    image: order-service:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]

  # API Gateway
  api-gateway:
    image: api-gateway:latest
    command: |
      sh -c "
        waitup user-service:8080 order-service:8080 --all --timeout 180s &&
        echo 'All upstream services ready' &&
        nginx -g 'daemon off;'
      "
    depends_on:
      user-service:
        condition: service_healthy
      order-service:
        condition: service_healthy
    ports:
      - "80:80"
```

### Pattern 4: Distributed Tracing

**Use Case:** Services with tracing dependencies

```yaml
services:
  # Tracing infrastructure
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "14268:14268"  # Jaeger collector
      - "16686:16686"  # Jaeger UI

  # Microservice with tracing
  payment-service:
    image: payment-service:latest
    command: |
      sh -c "
        waitup jaeger:14268 --timeout 60s &&
        waitup postgres:5432 --timeout 60s &&
        waitup http://external-api:8080/health --timeout 30s &&
        npm start
      "
    environment:
      JAEGER_AGENT_HOST: jaeger
      JAEGER_AGENT_PORT: 6831
```

## Development Patterns

### Pattern 1: Local Development Setup

**Use Case:** Consistent development environment

```bash
#!/bin/bash
# scripts/dev-setup.sh

echo "Starting development environment..."

# Start core services
docker-compose -f docker-compose.dev.yml up -d postgres redis

# Wait for services
waitup localhost:5432 localhost:6379 --timeout 60s --verbose

# Run migrations
npm run migrate

# Start application in development mode
npm run dev
```

### Pattern 2: Test Environment Management

**Use Case:** Reliable test environment setup

```bash
#!/bin/bash
# scripts/test-env.sh

set -e  # Exit on any error

cleanup() {
    echo "Cleaning up test environment..."
    docker-compose -f docker-compose.test.yml down -v
}

trap cleanup EXIT

echo "Setting up test environment..."
docker-compose -f docker-compose.test.yml up -d

echo "Waiting for services to be ready..."
waitup \
  localhost:5432 \
  localhost:6379 \
  localhost:9200 \
  --timeout 120s \
  --verbose

echo "Running database migrations..."
npm run migrate:test

echo "Running tests..."
npm test

echo "Test environment ready!"
```

### Pattern 3: Feature Branch Testing

**Use Case:** Branch-specific environment testing

```yaml
# docker-compose.feature.yml
version: "3.8"
services:
  app:
    image: myapp:${BRANCH_NAME:-main}
    command: |
      sh -c "
        waitup postgres:5432 --timeout 60s &&
        waitup redis:6379 --timeout 30s &&
        npm start
      "
    environment:
      DATABASE_URL: postgresql://user:pass@postgres:5432/db_${BRANCH_NAME}
      REDIS_URL: redis://redis:6379/0
    depends_on:
      - postgres
      - redis

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: db_${BRANCH_NAME:-main}
```

## Monitoring Patterns

### Pattern 1: Health Check Dashboard

**Use Case:** Service health monitoring

```bash
#!/bin/bash
# scripts/health-dashboard.sh

services=(
  "postgres:5432"
  "redis:6379"
  "http://api:8080/health"
  "http://frontend:3000"
)

echo "Service Health Dashboard"
echo "======================="

for service in "${services[@]}"; do
  echo -n "Checking $service... "

  result=$(waitup "$service" --timeout 5s --json 2>/dev/null)

  if [ $? -eq 0 ]; then
    elapsed=$(echo "$result" | jq -r '.elapsed_ms')
    echo "✓ OK (${elapsed}ms)"
  else
    echo "✗ FAILED"
  fi
done
```

### Pattern 2: Prometheus Integration

**Use Case:** Metrics collection for service availability

```bash
#!/bin/bash
# scripts/availability-metrics.sh

# Create metrics file for Prometheus node_exporter textfile collector
metrics_file="/var/lib/node_exporter/textfile_collector/service_availability.prom"

services=(
  "database:postgres:5432"
  "cache:redis:6379"
  "api:api-service:8080"
)

for service_def in "${services[@]}"; do
  IFS=':' read -r name host port <<< "$service_def"

  start_time=$(date +%s%3N)
  waitup "$host:$port" --timeout 10s --json > /tmp/result.json 2>&1
  exit_code=$?
  end_time=$(date +%s%3N)

  # Export metrics
  echo "service_available{service=\"$name\"} $([[ $exit_code -eq 0 ]] && echo 1 || echo 0)" >> "$metrics_file"
  echo "service_check_duration_ms{service=\"$name\"} $((end_time - start_time))" >> "$metrics_file"
done
```

### Pattern 3: Alerting Integration

**Use Case:** Alert on service unavailability

```bash
#!/bin/bash
# scripts/alert-check.sh

SLACK_WEBHOOK="https://hooks.slack.com/services/..."

check_service() {
  local service=$1
  local name=$2

  if ! waitup "$service" --timeout 10s >/dev/null 2>&1; then
    curl -X POST -H 'Content-type: application/json' \
      --data "{\"text\":\"🚨 Service $name is unavailable: $service\"}" \
      "$SLACK_WEBHOOK"
  fi
}

# Critical services
check_service "postgres:5432" "Database"
check_service "redis:6379" "Cache"
check_service "http://api:8080/health" "API Service"
```

## Best Practices Summary

### Timeout Guidelines
- **Database connections**: 60-120 seconds
- **HTTP APIs**: 30-60 seconds
- **Message brokers**: 60-180 seconds
- **Development**: Use longer timeouts
- **Production**: Balance between reliability and fail-fast

### Error Handling
- Always set appropriate timeouts
- Use `--verbose` for debugging
- Implement proper cleanup in scripts
- Monitor and alert on failures
- Use JSON output for automation

### Security Considerations
- Don't log sensitive headers or tokens
- Use secrets management for credentials
- Limit network access with proper firewall rules
- Use least-privilege container users
- Validate all external inputs

### Performance Optimization
- Use appropriate retry intervals
- Parallel check multiple services when possible
- Use connection pooling for HTTP checks
- Monitor resource usage in production
- Cache DNS resolution when appropriate

These patterns provide a solid foundation for integrating waitup into your infrastructure. Adapt them to your specific needs and environment constraints.
