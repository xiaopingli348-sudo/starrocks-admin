#!/usr/bin/env bash

#
# StarRocks Admin - Frontend Build Script
# Builds the Angular frontend and outputs to build/dist/web/
#

set -e

# Get project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_DIR="$PROJECT_ROOT/frontend"
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$BUILD_DIR/dist"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Building StarRocks Admin Frontend${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Create web directory
mkdir -p "$DIST_DIR/web"

# Clean up old frontend build
echo -e "${YELLOW}[1/3]${NC} Installing frontend dependencies..."
cd "$FRONTEND_DIR"
npm install

echo -e "${YELLOW}[2/3]${NC} Building Angular frontend (production mode)..."
npm run build -- --configuration production

# Copy built files
echo -e "${YELLOW}[3/3]${NC} Copying built frontend files..."
cp -r dist/* "$DIST_DIR/web/"

echo ""
echo -e "${GREEN}✓ Frontend build complete!${NC}"
echo -e "  Output: $DIST_DIR/web/"