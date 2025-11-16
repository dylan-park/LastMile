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

# Create minimal lib.rs for cargo fetch
RUN mkdir -p src && echo "// dummy lib for caching" > src/lib.rs

# Fetch dependencies only (no compilation yet)
RUN cargo fetch

# Remove dummy lib.rs
RUN rm -rf src

# Copy the full source code
COPY . .

# Build the project with exact locked dependencies
RUN cargo build --release --locked

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
COPY --from=builder /usr/src/app/target/release/lastmile /app/lastmile

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
CMD ["./lastmile"]
