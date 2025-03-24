# Build stage
FROM rust:1.70-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# Build the dependencies (this will be cached unless the dependencies change)
RUN mkdir src && \
    echo "fn main() {println!(\"placeholder\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy the actual source code
COPY src ./src
COPY config ./config
COPY build.rs ./

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary and config from builder
COPY --from=builder /app/target/release/prozchain /app/prozchain
COPY --from=builder /app/config /app/config

# Set the entry point
ENTRYPOINT ["/app/prozchain"]
