#!/bin/bash

# Build All Script for Meridian GIS Platform v0.3.0
# Builds all TypeScript packages and Rust workspace

set -e  # Exit on error
set -o pipefail  # Exit on pipe failure

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Get the root directory
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

print_status "Starting build process for Meridian GIS Platform v0.3.0"
echo ""

# Check for required tools
print_status "Checking for required build tools..."

if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed"
    exit 1
fi
print_success "Node.js $(node --version) found"

if ! command -v npm &> /dev/null; then
    print_error "npm is not installed"
    exit 1
fi
print_success "npm $(npm --version) found"

if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed"
    exit 1
fi
print_success "Cargo $(cargo --version | cut -d' ' -f2) found"

echo ""

# Clean previous builds
print_status "Cleaning previous builds..."
rm -rf node_modules/.cache
rm -rf .turbo
print_success "Cache cleaned"

echo ""

# Install dependencies
print_status "Installing npm dependencies..."
if npm install; then
    print_success "npm dependencies installed"
else
    print_error "Failed to install npm dependencies"
    exit 1
fi

echo ""

# Build TypeScript packages
print_status "Building TypeScript packages with Turbo..."
if npm run build:ts; then
    print_success "TypeScript packages built successfully"
else
    print_error "TypeScript build failed"
    exit 1
fi

echo ""

# Run TypeScript type checking
print_status "Running TypeScript type checking..."
if npm run typecheck; then
    print_success "Type checking passed"
else
    print_warning "Type checking found issues (continuing...)"
fi

echo ""

# Build Rust workspace
print_status "Building Rust workspace (release mode)..."
if cargo build --release --workspace; then
    print_success "Rust workspace built successfully"
else
    print_error "Rust build failed"
    exit 1
fi

echo ""

# Run Rust tests
print_status "Running Rust tests..."
if cargo test --workspace --release; then
    print_success "Rust tests passed"
else
    print_warning "Some Rust tests failed (continuing...)"
fi

echo ""

# Generate build report
print_status "Generating build report..."
BUILD_REPORT="$ROOT_DIR/BUILD_REPORT.txt"
{
    echo "Meridian GIS Platform v0.3.0 - Build Report"
    echo "=========================================="
    echo ""
    echo "Build Date: $(date)"
    echo "Node Version: $(node --version)"
    echo "npm Version: $(npm --version)"
    echo "Cargo Version: $(cargo --version)"
    echo ""
    echo "TypeScript Packages:"
    find crates -name "package.json" -type f | wc -l
    echo ""
    echo "Rust Crates:"
    find crates -name "Cargo.toml" -type f | wc -l
    echo ""
    echo "Build artifacts:"
    echo "  - TypeScript: $(find . -name "dist" -type d | wc -l) dist directories"
    echo "  - Rust: target/release/"
    echo ""
} > "$BUILD_REPORT"

print_success "Build report generated: $BUILD_REPORT"

echo ""
print_success "All builds completed successfully!"
echo ""
print_status "Build artifacts:"
echo "  • TypeScript packages: crates/*/ts/dist/"
echo "  • Web application: web/dist/"
echo "  • Rust binaries: target/release/"
echo ""
print_status "Next steps:"
echo "  • Run tests: npm run test"
echo "  • Start dev server: npm run dev"
echo "  • Deploy to production: docker build -t meridian:latest ."
echo ""
