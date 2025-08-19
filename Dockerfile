# Multi-stage Dockerfile for waitup
# Produces a minimal container with just the waitup binary

# Build stage
FROM debian:bookworm-slim AS builder

# Install build dependencies including Rust
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create a new empty shell project
RUN USER=root cargo new --bin waitup
WORKDIR /waitup

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build dependencies - this is the caching Docker layer!
RUN cargo build --release && rm src/*.rs

# Copy source code
COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/waitup*
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates and basic utilities
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN groupadd -r waitup && useradd -r -g waitup waitup

# Copy the binary from the build stage
COPY --from=builder /waitup/target/release/waitup /usr/local/bin/waitup

# Set ownership and permissions
RUN chown waitup:waitup /usr/local/bin/waitup
RUN chmod +x /usr/local/bin/waitup

# Switch to non-root user
USER waitup

# Set the binary as the entrypoint
ENTRYPOINT ["/usr/local/bin/waitup"]

# Default command (can be overridden)
CMD ["--help"]

# Metadata
LABEL org.opencontainers.image.title="waitup"
LABEL org.opencontainers.image.description="A robust CLI tool for waiting until TCP ports, HTTP endpoints, and services become available"
LABEL org.opencontainers.image.version="1.0.0"
LABEL org.opencontainers.image.authors="Serhii Kaliuzhnyi <kalyuzhni.sergei@gmail.com>"
LABEL org.opencontainers.image.url="https://github.com/grok-rs/waitup"
LABEL org.opencontainers.image.source="https://github.com/grok-rs/waitup"
LABEL org.opencontainers.image.vendor="waitup"
LABEL org.opencontainers.image.licenses="MIT"
