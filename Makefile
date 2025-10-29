.PHONY: help build build-force build-clean docker-build docker-up docker-down clean
.PHONY: dev dev-start dev-stop dev-restart dev-status dev-logs

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
	@echo ""
	@echo "Development:"
	@echo "  make dev          - 启动开发环境（前后端热加载）"
	@echo "  make dev-start    - 启动开发服务器（后台）"
	@echo "  make dev-stop     - 停止开发服务器"
	@echo "  make dev-restart  - 重启开发服务器"
	@echo "  make dev-status   - 查看服务状态"
	@echo "  make dev-logs     - 查看实时日志"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build - Build Docker image"
	@echo "  make docker-up    - Start Docker container"
	@echo "  make docker-down  - Stop Docker container"
	@echo ""
	@echo "Clean:"
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

# ==================== Development Commands ====================

# 开发环境 - 热加载模式（前台运行，Ctrl+C 停止）
dev:
	@echo "Starting development environment with hot reload..."
	@bash scripts/dev/start.sh

# 启动开发服务器（后台运行）
dev-start:
	@bash scripts/dev/start.sh start

# 停止开发服务器
dev-stop:
	@bash scripts/dev/stop.sh

# 重启开发服务器
dev-restart:
	@bash scripts/dev/stop.sh
	@sleep 2
	@bash scripts/dev/start.sh start

# 查看服务状态
dev-status:
	@echo "=== Backend Status ==="
	@bash scripts/dev/start_backend.sh status || true
	@echo ""
	@echo "=== Frontend Status ==="
	@bash scripts/dev/start_frontend.sh status || true

# 查看实时日志
dev-logs:
	@echo "Showing real-time logs (Ctrl+C to exit)..."
	@bash scripts/dev/logs.sh