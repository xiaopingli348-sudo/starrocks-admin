#!/usr/bin/env bash

#
# StarRocks Admin - Environment Check Script
# Checks if all required build dependencies are installed
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Checking build environment..."
echo ""

# Track errors
HAS_ERROR=0

# Function to check if a command exists
check_command() {
    local cmd=$1
    local name=$2
    local install_hint=$3
    
    if command -v "$cmd" &> /dev/null; then
        local version=$($cmd --version 2>&1 | head -1)
        echo -e "${GREEN}✓${NC} $name: $version"
        return 0
    else
        echo -e "${RED}✗${NC} $name: not found"
        if [ -n "$install_hint" ]; then
            echo -e "  ${YELLOW}→${NC} $install_hint"
        fi
        HAS_ERROR=1
        return 1
    fi
}

# Check Rust
echo "Backend (Rust):"
check_command "cargo" "Cargo" "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_command "rustc" "Rust Compiler"

echo ""

# Check Node.js and npm
echo "Frontend (Angular):"
check_command "node" "Node.js" "Install Node.js: https://nodejs.org/"
check_command "npm" "npm"

# Check Node.js version (should be >= 14)
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -lt 14 ]; then
        echo -e "${YELLOW}⚠${NC}  Node.js version is $NODE_VERSION, recommended >= 14"
    fi
fi

echo ""

# Check optional dependencies
echo "Optional:"
check_command "jq" "jq (JSON processor)" "Install: sudo apt-get install jq (Ubuntu) or brew install jq (Mac)" || true
check_command "docker" "Docker" "Install Docker: https://docs.docker.com/get-docker/" || true

echo ""

# Summary
if [ $HAS_ERROR -eq 0 ]; then
    echo -e "${GREEN}✓ All required dependencies are installed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some required dependencies are missing. Please install them and try again.${NC}"
    exit 1
fi

