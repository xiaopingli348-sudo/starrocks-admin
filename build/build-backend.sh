#!/usr/bin/env bash

#
# StarRocks Admin - Backend Build Script
# Builds the Rust backend and outputs to build/dist/
#

set -e

# Get project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$PROJECT_ROOT/backend"
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$BUILD_DIR/dist"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Building StarRocks Admin Backend${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Create dist directories
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/conf"
mkdir -p "$DIST_DIR/lib"
mkdir -p "$DIST_DIR/data"
mkdir -p "$DIST_DIR/logs"
mkdir -p "$DIST_DIR/migrations"

# Build backend
echo -e "${YELLOW}[1/3]${NC} Compiling Rust backend (release mode)..."
cd "$BACKEND_DIR"
cargo build --release

# Copy binary
echo -e "${YELLOW}[2/3]${NC} Copying backend binary..."
cp target/release/starrocks-admin "$DIST_DIR/bin/"

# Copy configuration files
echo -e "${YELLOW}[3/4]${NC} Copying configuration files..."
# Copy config template if it exists
if [ -f "$DIST_DIR/conf/config.toml.example" ]; then
    cp "$DIST_DIR/conf/config.toml.example" "$DIST_DIR/conf/config.toml"
    echo "Created config.toml from template"
fi

# Copy migrations
echo -e "${YELLOW}[4/4]${NC} Copying database migrations..."
if [ -d "$BACKEND_DIR/migrations" ]; then
    cp -r "$BACKEND_DIR/migrations"/* "$DIST_DIR/migrations/"
    echo "Copied $(ls "$DIST_DIR/migrations" | wc -l) migration files"
else
    echo "Warning: No migrations directory found in backend"
fi

# Create a simple start script for backend
cat > "$DIST_DIR/bin/start-backend.sh" << 'EOF'
#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Set environment variables
export DATABASE_URL="${DATABASE_URL:-sqlite:///tmp/starrocks-admin/starrocks-admin.db}"
export HOST="${HOST:-0.0.0.0}"
export PORT="${PORT:-8081}"

# Create database directory
DB_DIR=$(dirname "${DATABASE_URL#sqlite://}")
mkdir -p "$DB_DIR"

echo "Starting StarRocks Admin Backend..."
echo "  - Host: $HOST:$PORT"
echo "  - Database: $DATABASE_URL"
echo ""

"$SCRIPT_DIR/starrocks-admin"
EOF

chmod +x "$DIST_DIR/bin/start-backend.sh"

echo ""
echo -e "${GREEN}✓ Backend build complete!${NC}"
echo -e "  Binary: $DIST_DIR/bin/starrocks-admin"
echo -e "  Startup script: $DIST_DIR/bin/start-backend.sh"

