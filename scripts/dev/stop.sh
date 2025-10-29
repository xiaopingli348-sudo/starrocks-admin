#!/bin/bash

# StarRocks Admin - Development Environment Stop Script
# 停止开发环境中的所有服务

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Stopping development environment...${NC}"
echo ""

# Stop backend
if [ -f "$PROJECT_ROOT/backend/starrocks-admin.pid" ]; then
    echo "Stopping backend..."
    bash scripts/dev/start_backend.sh stop
else
    echo -e "${GREEN}Backend not running${NC}"
fi

echo ""

# Stop frontend
if [ -f "$PROJECT_ROOT/frontend/frontend.pid" ]; then
    echo "Stopping frontend..."
    bash scripts/dev/start_frontend.sh stop
else
    echo -e "${GREEN}Frontend not running${NC}"
fi

echo ""
echo -e "${GREEN}All services stopped!${NC}"

