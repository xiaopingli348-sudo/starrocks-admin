#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Set environment variables
export DATABASE_URL="${DATABASE_URL:-sqlite:///tmp/starrocks-admin/starrocks-admin.db}"
export HOST="${HOST:-10.119.43.216}"
export PORT="${PORT:-8081}"

# Create database directory
DB_DIR=$(dirname "${DATABASE_URL#sqlite://}")
mkdir -p "$DB_DIR"

echo "Starting StarRocks Admin Backend..."
echo "  - Host: $HOST:$PORT"
echo "  - Database: $DATABASE_URL"
echo ""

"$SCRIPT_DIR/starrocks-admin"
