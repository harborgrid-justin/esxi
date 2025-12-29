#!/bin/sh

# Meridian GIS Platform - Docker Entrypoint Script
# Handles initialization and startup for containerized environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "${BLUE}========================================${NC}"
echo "${BLUE}Meridian GIS Platform v0.3.0${NC}"
echo "${BLUE}========================================${NC}"
echo ""

# Function to wait for a service
wait_for_service() {
    local host=$1
    local port=$2
    local service_name=$3
    local max_attempts=30
    local attempt=0

    echo "${YELLOW}Waiting for ${service_name} at ${host}:${port}...${NC}"

    while [ $attempt -lt $max_attempts ]; do
        if nc -z "$host" "$port" 2>/dev/null; then
            echo "${GREEN}✓ ${service_name} is ready${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 2
    done

    echo "${RED}✗ ${service_name} is not available after ${max_attempts} attempts${NC}"
    return 1
}

# Parse DATABASE_URL to get host and port
if [ -n "$DATABASE_URL" ]; then
    DB_HOST=$(echo "$DATABASE_URL" | sed -n 's/.*@\([^:]*\):.*/\1/p')
    DB_PORT=$(echo "$DATABASE_URL" | sed -n 's/.*:\([0-9]*\)\/.*/\1/p')

    if [ -n "$DB_HOST" ] && [ -n "$DB_PORT" ]; then
        wait_for_service "$DB_HOST" "$DB_PORT" "PostgreSQL" || true
    fi
fi

# Parse REDIS_URL to get host and port
if [ -n "$REDIS_URL" ]; then
    REDIS_HOST=$(echo "$REDIS_URL" | sed -n 's/.*@\([^:]*\):.*/\1/p')
    REDIS_PORT=$(echo "$REDIS_URL" | sed -n 's/.*:\([0-9]*\).*/\1/p')

    if [ -n "$REDIS_HOST" ] && [ -n "$REDIS_PORT" ]; then
        wait_for_service "$REDIS_HOST" "$REDIS_PORT" "Redis" || true
    fi
fi

echo ""
echo "${GREEN}Starting Meridian GIS Platform...${NC}"
echo ""

# Execute the main command
exec "$@"
