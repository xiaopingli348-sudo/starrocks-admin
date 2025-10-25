.PHONY: help build lint fmt check pre-commit docker-build docker-up docker-down clean

# Project paths
PROJECT_ROOT := $(shell pwd)
BACKEND_DIR := $(PROJECT_ROOT)/backend
FRONTEND_DIR := $(PROJECT_ROOT)/frontend
BUILD_DIR := $(PROJECT_ROOT)/build
DIST_DIR := $(BUILD_DIR)/dist

# Default target - show help
help:
	@echo "StarRocks Admin - Build Commands:"
	@echo ""
	@echo "Development:"
	@echo "  make lint         - Run clippy with strict checks"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make check        - Run cargo check"
	@echo "  make pre-commit   - Run pre-commit checks (clippy + fmt)"
	@echo ""
	@echo "Build:"
	@echo "  make build        - Build backend and frontend (runs pre-commit first)"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-up    - Start Docker container"
	@echo "  make docker-down  - Stop Docker container"
	@echo "  make clean        - Clean build artifacts"
	@echo ""

# Run clippy with strict checks
lint:
	@echo "Running clippy with strict checks..."
	@cd $(BACKEND_DIR) && cargo clippy --all-targets -- --deny warnings --allow clippy::uninlined-format-args

# Format code
fmt:
	@echo "Formatting code..."
	@cd $(BACKEND_DIR) && cargo fmt

# Run cargo check
check:
	@echo "Running cargo check..."
	@cd $(BACKEND_DIR) && cargo check

# Run pre-commit checks
pre-commit:
	@echo "Running pre-commit checks..."
	@bash $(BUILD_DIR)/pre-commit.sh

# Build both backend and frontend (runs pre-commit first)
build: pre-commit
	@echo "Building StarRocks Admin..."
	@bash build/build-backend.sh
	@bash build/build-frontend.sh
	@echo "Build complete! Output: $(DIST_DIR)"

# Build Docker image
docker-build:
	@echo "Building Docker image..."
	@docker build -f deploy/docker/Dockerfile -t starrocks-admin .

# Start Docker container
docker-up:
	@echo "Starting Docker container..."
	@cd deploy/docker && docker compose up -d

# Stop Docker container
docker-down:
	@echo "Stopping Docker container..."
	@cd deploy/docker && docker compose down

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf $(BUILD_DIR)
	@cd $(BACKEND_DIR) && cargo clean
	@cd $(FRONTEND_DIR) && rm -rf dist node_modules/.cache
	@echo "Clean complete!"