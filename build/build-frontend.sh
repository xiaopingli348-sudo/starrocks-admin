#!/usr/bin/env bash

#
# StarRocks Admin - Frontend Build Script
# Builds the Angular frontend and outputs to build/dist/
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

# Create dist directories (web directory already exists from backend build)
mkdir -p "$DIST_DIR/web"

# Install dependencies if needed
echo -e "${YELLOW}[1/3]${NC} Checking npm dependencies..."
cd "$FRONTEND_DIR"
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    # 确保 .npmrc 存在
    if [ ! -f ".npmrc" ]; then
        echo "legacy-peer-deps=true" > .npmrc
    fi
    npm install
else
    echo "Dependencies already installed"
fi

# Build frontend
echo -e "${YELLOW}[2/3]${NC} Building Angular frontend (production mode)..."
npm run build -- --configuration production

# Copy built files
echo -e "${YELLOW}[3/3]${NC} Copying frontend build artifacts..."
if [ -d "$FRONTEND_DIR/dist/ngx-admin" ]; then
    cp -r "$FRONTEND_DIR/dist/ngx-admin/"* "$DIST_DIR/web/"
elif [ -d "$FRONTEND_DIR/dist" ]; then
    cp -r "$FRONTEND_DIR/dist/"* "$DIST_DIR/web/"
else
    echo "Error: Frontend build output not found"
    exit 1
fi

# Create a simple start script for frontend (nginx config example)
cat > "$DIST_DIR/web/README.md" << 'EOF'
# StarRocks Admin Frontend

This directory contains the built Angular frontend application.

## Serving the Frontend

### Option 1: Using Node.js http-server

```bash
npm install -g http-server
http-server -p 4200
```

### Option 2: Using Python

```bash
# Python 3
python3 -m http.server 4200

# Python 2
python -m SimpleHTTPServer 4200
```

### Option 3: Using Nginx

Create an nginx configuration:

```nginx
server {
    listen 4200;
    server_name localhost;
    
    root /path/to/starrocks-admin/build/dist/web;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
    
    # Proxy API requests to backend
    location /api/ {
        proxy_pass http://0.0.0.0:8081/api/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Development

For development mode, use the npm dev server from the frontend directory:

```bash
cd frontend
npm start
```
EOF

echo ""
echo -e "${GREEN}✓ Frontend build complete!${NC}"
echo -e "  Output: $DIST_DIR/web/"
echo -e "  Size: $(du -sh "$DIST_DIR/web" | cut -f1)"

