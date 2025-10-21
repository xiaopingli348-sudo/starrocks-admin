.PHONY: help check-env build build-backend build-frontend clean dist dist-only run-backend run-frontend run all

# Default target when running 'make' without arguments
.DEFAULT_GOAL := help

# Detect system type
SYSTEM_TYPE := $(shell uname -s)
IS_UBUNTU := $(shell grep -q -i ubuntu /etc/os-release 2>/dev/null && echo 1 || echo 0)

# Set shell command based on system type
SHELL_CMD := sh
ifeq ($(IS_UBUNTU),1)
  SHELL_CMD := bash
endif

# Project paths
PROJECT_ROOT := $(shell pwd)
BUILD_DIR := $(PROJECT_ROOT)/build
DIST_DIR := $(BUILD_DIR)/dist
BACKEND_DIR := $(PROJECT_ROOT)/backend
FRONTEND_DIR := $(PROJECT_ROOT)/frontend

# Version (can be overridden by environment variable)
VERSION ?= $(shell grep '^version' backend/Cargo.toml | head -1 | cut -d'"' -f2)

# Default target - show help
help:
	@echo "StarRocks Admin Build System - Available Commands:"
	@echo ""
	@echo "Environment:"
	@echo "  make check-env                   - Check build environment dependencies"
	@echo ""
	@echo "Building:"
	@echo "  make build                       - Build both backend and frontend"
	@echo "  make build-backend               - Build backend only (Rust)"
	@echo "  make build-frontend              - Build frontend only (Angular)"
	@echo "  make all                         - Same as 'make build'"
	@echo "  make clean                       - Clean build artifacts"
	@echo ""
	@echo "Distribution:"
	@echo "  make dist                        - Build and create distribution package"
	@echo "  make dist-only                   - Create distribution package without building"
	@echo "  make package                     - Create complete deployment package"
	@echo ""
	@echo "Deployment:"
	@echo "  make deploy                      - Deploy using production scripts"
	@echo "  make docker-build                - Build Docker image"
	@echo "  make docker-up                   - Start with Docker Compose"
	@echo "  make docker-down                 - Stop Docker Compose"
	@echo ""
	@echo "Development:"
	@echo "  make dev                         - Start development environment (one-click)"
	@echo "  make run-backend                 - Run backend only"
	@echo "  make run-frontend                - Run frontend only"
	@echo ""
	@echo "Parameters:"
	@echo "  VERSION='...'                    - Version string for distribution (default: from Cargo.toml)"
	@echo ""
	@echo "Examples:"
	@echo "  make build                       - Build entire project"
	@echo "  make package                     - Create complete deployment package"
	@echo "  make deploy                      - Deploy to production"
	@echo "  make docker-up                   - Start with Docker"
	@echo "  VERSION=v1.0.0 make dist         - Build and package with specific version"

# Check build environment dependencies
check-env:
	@echo "Checking build environment..."
	@$(SHELL_CMD) build/check-env.sh

# Build backend (Rust)
build-backend: check-env
	@echo "Building backend (Rust)..."
	@$(SHELL_CMD) build/build-backend.sh

# Build frontend (Angular)
build-frontend: check-env
	@echo "Building frontend (Angular)..."
	@$(SHELL_CMD) build/build-frontend.sh

# Build both backend and frontend
build: build-backend build-frontend
	@echo "Build complete! Output: $(DIST_DIR)"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)
	@cd $(BACKEND_DIR) && cargo clean
	@cd $(FRONTEND_DIR) && rm -rf dist node_modules/.cache
	@echo "Clean complete!"

# Run backend in development mode
run-backend:
	@echo "Running backend in development mode..."
	@$(SHELL_CMD) scripts/dev/start_backend.sh

# Run frontend in development mode
run-frontend:
	@echo "Running frontend in development mode..."
	@$(SHELL_CMD) scripts/dev/start_frontend.sh

# Run both backend and frontend in development mode (legacy)
run:
	@echo "Starting StarRocks Admin (dev environment)..."
	@$(SHELL_CMD) ./dev.sh

# Development environment (one-click start)
dev:
	@$(SHELL_CMD) ./dev.sh

# All in one
all: build

# Distribution packaging
dist: all
	@$(MAKE) dist-only

dist-only:
	@echo "Creating distribution package..."
	@if [ ! -d "$(DIST_DIR)" ]; then \
		echo "Error: $(DIST_DIR) directory not found. Please run 'make build' first."; \
		exit 1; \
	fi
	@PLATFORM=$$(uname -s | tr '[:upper:]' '[:lower:]'); \
	ARCH=$$(uname -m); \
	if [ -n "$$VERSION" ]; then \
		VER="$$VERSION"; \
	else \
		VER="$(VERSION)"; \
	fi; \
	if [ -n "$$GITHUB_ACTIONS" ]; then \
		DIST_NAME="starrocks-admin-$${VER}-$${PLATFORM}-$${ARCH}"; \
		echo "GitHub Actions detected - using clean naming"; \
	else \
		TIMESTAMP=$$(date +%Y%m%d-%H%M%S); \
		DIST_NAME="starrocks-admin-$${VER}-$${PLATFORM}-$${ARCH}-$${TIMESTAMP}"; \
		echo "Local build - adding timestamp"; \
	fi; \
	echo "Packaging as: $${DIST_NAME}.tar.gz"; \
	cd $(DIST_DIR) && tar -czf "../$${DIST_NAME}.tar.gz" . && cd ..; \
	echo "Distribution package created: $${DIST_NAME}.tar.gz"; \
	ls -lh "$${DIST_NAME}.tar.gz"

# Complete deployment package (includes scripts)
package: build
	@echo "Creating complete deployment package..."
	@if [ ! -d "$(DIST_DIR)" ]; then \
		echo "Error: $(DIST_DIR) directory not found. Please run 'make build' first."; \
		exit 1; \
	fi
	@# Copy deployment scripts to dist
	@cp deploy/scripts/starrocks-admin.sh $(DIST_DIR)/bin/
	@chmod +x $(DIST_DIR)/bin/starrocks-admin.sh
	@# Create README for deployment
	@echo "# StarRocks Admin - Deployment Package" > $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "## Quick Start" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "1. Extract this package: \`tar -xzf starrocks-admin-*.tar.gz\`" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "2. Start the service: \`bin/starrocks-admin.sh start\`" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "3. Access the web UI: http://localhost:8080" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "## Management Commands" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/starrocks-admin.sh start\`   - Start the service" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/starrocks-admin.sh stop\`    - Stop the service" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/starrocks-admin.sh restart\` - Restart the service" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/starrocks-admin.sh status\`  - Show service status" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/starrocks-admin.sh logs\`    - Show live logs" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "## Configuration" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "Edit \`conf/config.toml\` to customize settings." >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "## Directory Structure" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`bin/\`     - Executable files" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`conf/\`    - Configuration files" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`web/\`     - Frontend static files" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`data/\`    - Database files" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@echo "- \`logs/\`    - Log files" >> $(DIST_DIR)/README-DEPLOYMENT.md
	@$(MAKE) dist-only

# Production deployment
deploy: package
	@echo "Deploying StarRocks Admin..."
	@if [ ! -f "$(DIST_DIR)/bin/starrocks-admin.sh" ]; then \
		echo "Error: Deployment package not found. Run 'make package' first."; \
		exit 1; \
	fi
	@echo "Deployment package ready in $(DIST_DIR)/"
	@echo "To deploy:"
	@echo "  1. Copy the dist/ directory to your server"
	@echo "  2. Run: ./starrocks-admin.sh start"

# Docker targets
docker-build:
	@echo "Building Docker image..."
	@cd deploy/docker && docker-compose build

docker-up:
	@echo "Starting with Docker Compose..."
	@cd deploy/docker && docker-compose up -d

docker-down:
	@echo "Stopping Docker Compose..."
	@cd deploy/docker && docker-compose down

docker-logs:
	@echo "Showing Docker logs..."
	@cd deploy/docker && docker-compose logs -f

