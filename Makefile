.PHONY: help build build-force build-clean docker-build docker-up docker-down clean

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
	@echo "Build:"
	@echo "  make build        - 增量构建（仅构建有修改的组件）"
	@echo "  make build-force  - 强制构建所有组件"
	@echo "  make build-clean  - 清理缓存后构建"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-up    - Start Docker container"
	@echo "  make docker-down  - Stop Docker container"
	@echo "  make clean        - Clean build artifacts"
	@echo ""

# 增量构建（默认）
build:
	@bash build/build-optimized.sh

# 强制构建所有组件
build-force:
	@bash build/build-optimized.sh --force

# 清理缓存后构建
build-clean:
	@bash build/build-optimized.sh --clean

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