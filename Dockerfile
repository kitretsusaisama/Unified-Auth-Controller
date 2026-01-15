# Multi-stage Dockerfile for Enterprise SSO Platform
# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
COPY config ./config
COPY migrations ./migrations

# Build for release
RUN cargo build --release --bin auth-platform

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 sso && \
    chown -R sso:sso /app

# Copy binary from builder
COPY --from=builder /app/target/release/auth-platform /app/auth-platform
COPY --from=builder /app/config /app/config

# Switch to non-root user
USER sso

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary
CMD ["/app/auth-platform"]
