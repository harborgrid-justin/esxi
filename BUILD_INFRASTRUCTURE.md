# Build Infrastructure Documentation

## Enterprise Web-Accessibility SaaS v0.3.0

This document describes the complete build infrastructure setup for the Meridian GIS Platform with Enterprise Web-Accessibility SaaS.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Project Structure](#project-structure)
- [Build Configuration](#build-configuration)
- [Development Workflow](#development-workflow)
- [Docker Setup](#docker-setup)
- [CI/CD Pipeline](#cicd-pipeline)
- [Scripts](#scripts)
- [Troubleshooting](#troubleshooting)

---

## 🎯 Overview

The Meridian GIS Platform is a monorepo containing:
- **30+ Rust crates** for backend services
- **12+ TypeScript packages** for frontend components
- **Web application** built with React and Vite
- **Enterprise accessibility features** (WCAG 2.1 AA compliant)

### Technology Stack

#### Frontend
- **TypeScript 5.3+** - Type-safe JavaScript
- **React 18** - UI framework
- **Vite** - Build tool and dev server
- **Turbo** - Monorepo build system
- **ESLint** - Code linting with accessibility rules
- **Prettier** - Code formatting

#### Backend
- **Rust 1.75+** - Systems programming language
- **Tokio** - Async runtime
- **PostgreSQL 16** - Primary database with PostGIS
- **Redis 7** - Caching layer

#### DevOps
- **Docker** - Containerization
- **Docker Compose** - Local development
- **GitHub Actions** - CI/CD pipeline

---

## 📁 Project Structure

```
esxi/
├── .github/
│   └── workflows/
│       └── ci.yml              # CI/CD pipeline
├── crates/                      # Rust workspace
│   ├── accessibility-*/         # 10 accessibility crates
│   ├── meridian-*/              # 30+ platform crates
│   └── */ts/                    # TypeScript bindings
├── docker/
│   ├── entrypoint.sh            # Container entrypoint
│   └── postgres/
│       └── init.sql             # Database initialization
├── scripts/
│   └── build-all.sh             # Comprehensive build script
├── web/                         # Main web application
├── .editorconfig                # Editor configuration
├── .eslintrc.js                 # ESLint configuration
├── .prettierrc                  # Prettier configuration
├── .dockerignore                # Docker ignore patterns
├── .prettierignore              # Prettier ignore patterns
├── .nvmrc                       # Node.js version
├── Cargo.toml                   # Rust workspace config
├── docker-compose.yml           # Development services
├── Dockerfile                   # Production container
├── package.json                 # Root package config
├── tsconfig.base.json           # Base TypeScript config
└── turbo.json                   # Turbo build config
```

---

## 🔧 Build Configuration

### Root Package Configuration

**File:** `package.json`

The root package.json manages all TypeScript workspaces:

```json
{
  "workspaces": [
    "web",
    "crates/*/ts"
  ]
}
```

### TypeScript Configuration

**File:** `tsconfig.base.json`

Features:
- ✅ Strict type checking enabled
- ✅ Path aliases for all packages
- ✅ ES2022 target with DOM libraries
- ✅ Source maps and declarations

### Turbo Configuration

**File:** `turbo.json`

Build pipeline with:
- **Dependency tracking** - Builds run in correct order
- **Caching** - Speeds up repeated builds
- **Parallel execution** - Utilizes all CPU cores
- **Remote caching** - Optional team-wide cache

### Linting & Formatting

#### ESLint Configuration

**File:** `.eslintrc.js`

Includes:
- TypeScript best practices
- React and React Hooks rules
- **Complete WCAG 2.1 AA accessibility rules**
- Import ordering and organization
- Code quality rules

Key accessibility rules enforced:
- `jsx-a11y/alt-text` - Images must have alt text
- `jsx-a11y/aria-props` - Valid ARIA attributes
- `jsx-a11y/label-has-associated-control` - Form labels
- `jsx-a11y/no-autofocus` - No autofocus elements
- `jsx-a11y/click-events-have-key-events` - Keyboard accessibility

#### Prettier Configuration

**File:** `.prettierrc`

Standard formatting:
- 2-space indentation
- Single quotes
- 100 character line width
- Semicolons required
- Trailing commas (ES5)

---

## 💻 Development Workflow

### Prerequisites

```bash
# Node.js 18 or 20
node --version

# npm 9+
npm --version

# Rust 1.75+
cargo --version

# Docker (optional)
docker --version
```

### Initial Setup

```bash
# Install dependencies
npm install

# Start development databases
npm run docker:dev

# Run development servers
npm run dev
```

### Common Commands

#### Building

```bash
# Build everything (TypeScript + Rust)
npm run build:all

# Build only TypeScript packages
npm run build:ts

# Build only Rust workspace
npm run build:rust

# Build specific package
npm run build --workspace=web
```

#### Testing

```bash
# Run all tests
npm run test

# Run TypeScript tests
npm run test:ts

# Run Rust tests
npm run test:rust
```

#### Linting

```bash
# Lint everything
npm run lint

# Lint and fix TypeScript
npm run lint:fix

# Lint Rust
npm run lint:rust

# Check formatting
npm run format:check

# Auto-format code
npm run format
```

#### Cleaning

```bash
# Clean all build artifacts
npm run clean

# Clean only TypeScript
npm run clean:ts

# Clean only Rust
npm run clean:rust
```

---

## 🐳 Docker Setup

### Development Environment

**File:** `docker-compose.yml`

Includes:
- **PostgreSQL 16** with PostGIS - Port 5432
- **Redis 7** - Port 6379
- **Adminer** - Database UI on port 8080
- **Redis Commander** - Redis UI on port 8081
- **MinIO** - S3-compatible storage on ports 9000/9001

Start services:

```bash
npm run docker:dev
```

Access points:
- Database UI: http://localhost:8080
- Redis UI: http://localhost:8081
- MinIO Console: http://localhost:9001

Default credentials:
- PostgreSQL: `meridian` / `meridian_dev_password`
- Redis: `meridian_redis_password`
- MinIO: `meridian` / `meridian_minio_password`

### Production Container

**File:** `Dockerfile`

Multi-stage build:
1. **node-builder** - Builds all TypeScript packages
2. **rust-builder** - Compiles Rust workspace
3. **runtime** - Minimal Alpine-based production image
4. **development** - Full development environment

Build production image:

```bash
npm run docker:build
```

Run production container:

```bash
docker run -p 8000:8000 -p 3000:3000 \
  -e DATABASE_URL=postgres://... \
  -e REDIS_URL=redis://... \
  meridian-gis:latest
```

---

## 🚀 CI/CD Pipeline

**File:** `.github/workflows/ci.yml`

### Pipeline Stages

#### 1. Lint & Format
- Prettier formatting check
- ESLint for TypeScript
- Cargo fmt for Rust
- Clippy for Rust (with warnings as errors)

#### 2. TypeScript Build & Test
- Matrix testing on Node.js 18 and 20
- Type checking with tsc
- Build all packages
- Run tests
- Upload build artifacts

#### 3. Rust Build & Test
- Matrix testing on Ubuntu and macOS
- Test on stable and nightly Rust
- Cargo build with all features
- Run test suite
- Build release binaries

#### 4. Accessibility Tests
- WCAG 2.1 AA compliance checks
- Automated accessibility audits
- Screen reader compatibility tests

#### 5. Security Audit
- npm audit for JavaScript dependencies
- cargo audit for Rust dependencies
- Dependency vulnerability scanning

#### 6. Docker Build
- Multi-stage production build
- Development image build
- Layer caching for faster builds
- Optional push to registry

#### 7. Integration Tests
- Full stack testing
- Database integration tests
- Redis cache tests
- API endpoint tests

#### 8. Code Coverage
- Rust coverage with cargo-llvm-cov
- TypeScript coverage with Jest
- Upload to Codecov

#### 9. Deploy
- Automatic deployment on main branch
- Production environment deployment
- Health checks and rollback

### Triggering the Pipeline

The CI/CD pipeline runs on:
- Push to `main`, `develop`, or `claude/**` branches
- Pull requests to `main` or `develop`
- Manual workflow dispatch

### Viewing Results

All results are available in the GitHub Actions tab:
- Build logs
- Test results
- Artifact downloads
- Deployment status

---

## 📜 Scripts

### build-all.sh

**Location:** `scripts/build-all.sh`

Comprehensive build script that:
1. ✅ Checks for required tools (Node.js, npm, Cargo)
2. ✅ Cleans previous builds
3. ✅ Installs npm dependencies
4. ✅ Builds all TypeScript packages
5. ✅ Runs type checking
6. ✅ Builds Rust workspace in release mode
7. ✅ Runs Rust tests
8. ✅ Generates build report

Usage:

```bash
bash scripts/build-all.sh
```

Output includes:
- Colored status messages
- Build timing information
- Artifact locations
- Next steps recommendations

---

## 🔍 Troubleshooting

### Common Issues

#### Node.js Version Mismatch

```bash
# Use the correct version
nvm use
# or
nvm install
```

#### Dependencies Out of Sync

```bash
# Clean and reinstall
npm run clean
rm -rf node_modules package-lock.json
npm install
```

#### Rust Build Failures

```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo build --workspace
```

#### Docker Issues

```bash
# Stop and remove containers
npm run docker:down

# Clean volumes
docker volume prune

# Restart services
npm run docker:dev
```

#### Type Checking Errors

```bash
# Run type check to see all errors
npm run typecheck

# Check specific package
cd crates/accessibility-dashboard/ts
npm run typecheck
```

### Getting Help

- Check build logs: `BUILD_REPORT.txt`
- Review CI/CD results in GitHub Actions
- Check Docker logs: `docker-compose logs`
- Run with verbose output: `npm run build -- --verbose`

---

## 📊 Build Metrics

### Build Times (Approximate)

- **TypeScript (incremental)**: 30-60 seconds
- **TypeScript (clean)**: 2-4 minutes
- **Rust (incremental)**: 1-2 minutes
- **Rust (clean, release)**: 10-15 minutes
- **Full build (everything)**: 15-20 minutes
- **Docker image**: 20-30 minutes

### Artifact Sizes

- **TypeScript packages**: ~5-20 MB total
- **Web application**: ~2-5 MB
- **Rust release binaries**: ~50-100 MB
- **Docker image**: ~200-300 MB

---

## 🎉 Next Steps

After setting up the build infrastructure:

1. ✅ Run initial build: `npm run build:all`
2. ✅ Start development environment: `npm run docker:dev && npm run dev`
3. ✅ Run tests: `npm run test`
4. ✅ Set up GitHub secrets for CI/CD
5. ✅ Configure deployment targets
6. ✅ Review and customize ESLint rules
7. ✅ Set up monitoring and logging
8. ✅ Configure remote Turbo cache (optional)

---

## 📝 Version History

- **v0.3.0** - Enterprise Web-Accessibility SaaS
  - Complete build infrastructure
  - CI/CD pipeline
  - Docker development environment
  - Comprehensive linting and testing

- **v0.2.5** - Visualization & Data Processing
- **v0.1.5** - Enterprise Features
- **v0.1.0** - Core Platform

---

## 📄 License

MIT License - See LICENSE file for details

---

## 👥 Maintained By

HarborGrid - Enterprise GIS Solutions

For questions or support, please open an issue on GitHub.
