# Meridian GIS Platform v0.3.0 - Production Dockerfile
# Multi-stage build for optimized production image

# ==============================================================================
# Stage 1: Node.js Builder - Build TypeScript packages
# ==============================================================================
FROM node:20-alpine AS node-builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache python3 make g++

# Copy package files
COPY package.json package-lock.json ./
COPY turbo.json ./
COPY tsconfig.base.json ./

# Copy all workspace package.json files
COPY web/package.json ./web/
COPY crates/meridian-ui-components/ts/package.json ./crates/meridian-ui-components/ts/
COPY crates/meridian-dashboard/ts/package.json ./crates/meridian-dashboard/ts/
COPY crates/accessibility-scanner/ts/package.json ./crates/accessibility-scanner/ts/
COPY crates/accessibility-dashboard/ts/package.json ./crates/accessibility-dashboard/ts/
COPY crates/accessibility-realtime/ts/package.json ./crates/accessibility-realtime/ts/
COPY crates/accessibility-reports/ts/package.json ./crates/accessibility-reports/ts/
COPY crates/accessibility-contrast/ts/package.json ./crates/accessibility-contrast/ts/
COPY crates/accessibility-screenreader/ts/package.json ./crates/accessibility-screenreader/ts/
COPY crates/accessibility-keyboard/ts/package.json ./crates/accessibility-keyboard/ts/
COPY crates/accessibility-aria/ts/package.json ./crates/accessibility-aria/ts/
COPY crates/accessibility-documents/ts/package.json ./crates/accessibility-documents/ts/
COPY crates/accessibility-tenant/ts/package.json ./crates/accessibility-tenant/ts/

# Install dependencies
RUN npm ci --no-audit --prefer-offline

# Copy source code
COPY . .

# Build all TypeScript packages
RUN npm run build:ts

# ==============================================================================
# Stage 2: Rust Builder - Build Rust workspace
# ==============================================================================
FROM rust:1.75-alpine AS rust-builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev \
    sqlite-dev

# Copy Cargo files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy all crate manifests
COPY crates ./crates

# Build dependencies first (cached layer)
RUN mkdir -p /tmp/dummy/src && \
    echo "fn main() {}" > /tmp/dummy/src/main.rs && \
    cargo build --release --workspace || true

# Build the actual application
RUN cargo build --release --workspace

# Strip debug symbols for smaller binaries
RUN find target/release -maxdepth 1 -type f -executable -exec strip {} \;

# ==============================================================================
# Stage 3: Runtime - Minimal production image
# ==============================================================================
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    libssl3 \
    libcrypto3 \
    postgresql-libs \
    tzdata

# Create non-root user
RUN addgroup -g 1000 meridian && \
    adduser -D -u 1000 -G meridian meridian

WORKDIR /app

# Copy Rust binaries from rust-builder
COPY --from=rust-builder --chown=meridian:meridian /app/target/release/meridian-server* ./bin/
COPY --from=rust-builder --chown=meridian:meridian /app/target/release/meridian-cli* ./bin/

# Copy TypeScript build artifacts from node-builder
COPY --from=node-builder --chown=meridian:meridian /app/web/dist ./web/dist
COPY --from=node-builder --chown=meridian:meridian /app/crates/*/ts/dist ./packages/

# Copy configuration files
COPY --chown=meridian:meridian docker/entrypoint.sh ./
RUN chmod +x ./entrypoint.sh

# Set environment variables
ENV RUST_LOG=info \
    RUST_BACKTRACE=1 \
    NODE_ENV=production \
    PORT=8000 \
    WEB_PORT=3000

# Expose ports
EXPOSE 8000 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8000/health || exit 1

# Switch to non-root user
USER meridian

# Set entrypoint
ENTRYPOINT ["./entrypoint.sh"]
CMD ["./bin/meridian-server"]

# ==============================================================================
# Stage 4: Development - Development image with hot reload
# ==============================================================================
FROM node:20-alpine AS development

WORKDIR /app

# Install development dependencies
RUN apk add --no-cache \
    bash \
    git \
    curl \
    wget \
    ca-certificates \
    python3 \
    make \
    g++ \
    postgresql-client \
    redis

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy package files
COPY package.json package-lock.json ./
COPY turbo.json ./
COPY tsconfig.base.json ./

# Install all dependencies (including dev)
RUN npm install

# Copy source code
COPY . .

# Set development environment
ENV NODE_ENV=development \
    RUST_LOG=debug \
    RUST_BACKTRACE=full

# Expose development ports
EXPOSE 3000 8000 5173

CMD ["npm", "run", "dev"]
