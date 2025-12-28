FROM rust:1.75 as builder
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crate sources
COPY crates/ crates/

# Build release binaries
RUN cargo build --release --bin meridian-cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        libssl3 \
        ca-certificates \
        libpq5 \
        libproj25 \
        && rm -rf /var/lib/apt/lists/*

# Copy binaries from builder
COPY --from=builder /app/target/release/meridian-cli /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1000 meridian && \
    mkdir -p /data && \
    chown -R meridian:meridian /data

USER meridian
WORKDIR /data

# Default to running the CLI in server mode
# Can be overridden with docker run command
ENTRYPOINT ["meridian-cli"]
CMD ["--help"]
