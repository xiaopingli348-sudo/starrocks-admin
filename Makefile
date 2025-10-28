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
	@echo "Code Quality:"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make fmt-check    - Check code formatting"
	@echo "  make clippy       - Run clippy checks (fix + strict)"
	@echo "  make check        - Run cargo check"
	@echo "  make pre-commit   - Run all pre-commit checks (fmt + clippy + check)"
	@echo "  make lint         - Alias for clippy"
	@echo ""
	@echo "Build:"
	@echo "  make build        - Build backend and frontend (runs pre-commit first)"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-up    - Start Docker container"
	@echo "  make docker-down  - Stop Docker container"
	@echo "  make clean        - Clean build artifacts"
	@echo ""

# Format code
fmt:
	@echo "[fmt] Formatting code..."
	@cd $(BACKEND_DIR) && cargo fmt --all

# Check code formatting
fmt-check:
	@echo "[fmt-check] Checking code formatting..."
	@cd $(BACKEND_DIR) && cargo fmt --all --check

# Run clippy checks (following rustfs standard)
clippy:
	@echo "[clippy] Running clippy checks..."
	@cd $(BACKEND_DIR) && DATABASE_URL="sqlite:$(BUILD_DIR)/data/starrocks-admin.db" cargo clippy --fix --allow-dirty --all-targets
	@cd $(BACKEND_DIR) && DATABASE_URL="sqlite:$(BUILD_DIR)/data/starrocks-admin.db" cargo clippy --all-targets --all-features -- -D warnings

# Run cargo check
check:
	@echo "[check] Running cargo check..."
	@cd $(BACKEND_DIR) && cargo check --all-targets

# Legacy alias for clippy
lint: clippy

# Run pre-commit checks (following rustfs standard)
pre-commit: fmt clippy check
	@echo "[pre-commit] All pre-commit checks passed!"

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