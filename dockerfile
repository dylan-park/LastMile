# Use the official Rust image
FROM rust:slim-bookworm AS builder

# Install build dependencies for RocksDB
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang && \
    rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies for cache
RUN cargo build --release

# Remove the dummy source and target artifacts
RUN rm -rf src target/release/deps/uber_eats_tracker* target/release/uber-eats-tracker*

# Copy the actual source code
COPY . .

# Build the project
RUN cargo build --release

# Use a minimal image for running
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Create app user and directories
RUN useradd -m -u 1000 appuser && \
    mkdir -p /app/data /app/static && \
    chown -R appuser:appuser /app

# Copy the binary from the builder
COPY --from=builder /usr/src/app/target/release/uber-eats-tracker /app/uber-eats-tracker

# Copy static files
COPY --chown=appuser:appuser static /app/static

# Set working directory
WORKDIR /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Set default environment variables
ENV DATABASE_PATH=/app/data \
    RUST_LOG=info

# Run the binary
CMD ["./uber-eats-tracker"]
