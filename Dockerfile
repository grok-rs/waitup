# Multi-stage Dockerfile for wait-for
# Produces a minimal container with just the wait-for binary

# Build stage
FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
RUN USER=root cargo new --bin wait-for
WORKDIR /wait-for

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build dependencies - this is the caching Docker layer!
RUN cargo build --release && rm src/*.rs

# Copy source code
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/wait_for*
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates and basic utilities
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r wait-for && useradd -r -g wait-for wait-for

# Copy the binary from the build stage
COPY --from=builder /wait-for/target/release/wait-for /usr/local/bin/wait-for

# Set ownership and permissions
RUN chown wait-for:wait-for /usr/local/bin/wait-for
RUN chmod +x /usr/local/bin/wait-for

# Switch to non-root user
USER wait-for

# Set the binary as the entrypoint
ENTRYPOINT ["/usr/local/bin/wait-for"]

# Default command (can be overridden)
CMD ["--help"]

# Metadata
LABEL org.opencontainers.image.title="wait-for"
LABEL org.opencontainers.image.description="A robust CLI tool for waiting until TCP ports, HTTP endpoints, and services become available"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.authors="Serhii Kaliuzhnyi <kalyuzhni.sergei@gmail.com>"
LABEL org.opencontainers.image.url="https://github.com/grok-rs/wait-for"
LABEL org.opencontainers.image.source="https://github.com/grok-rs/wait-for"
LABEL org.opencontainers.image.vendor="wait-for"
LABEL org.opencontainers.image.licenses="MIT"
