# Multi-stage Dockerfile for Aureon Blockchain Node
# Stage 1: Builder
FROM rust:latest as builder

WORKDIR /build

# Copy source code
COPY . .

# Build in release mode with optimizations
RUN cargo build --release -p aureon-node

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/aureon-node /app/aureon-node

# Copy default configuration
COPY config.toml /app/config.toml

# Create data directory for RocksDB
RUN mkdir -p /app/aureon_db

# Expose ports
# P2P Network
EXPOSE 6000
# REST API
EXPOSE 8080
# WebSocket (future)
EXPOSE 8081

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/chain/head || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV AUREON_API_HOST=0.0.0.0

# Run the node
ENTRYPOINT ["/app/aureon-node"]
