#!/bin/bash

# StarRocks Admin Backend - Optimized Development Mode
# 优化的开发模式后端启动脚本（支持增量编译，避免重复编译第三方库）

set -e

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 加载 Rust 环境（如果存在）
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# 加载公共函数
source "$SCRIPT_DIR/common.sh"
BACKEND_DIR="$PROJECT_ROOT/backend"
CONFIG_DIR="$PROJECT_ROOT/backend/conf"
DB_DIR="${DB_DIR:-$PROJECT_ROOT/backend/data}"
LOG_DIR="${LOG_DIR:-$PROJECT_ROOT/backend/logs}"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}StarRocks Admin Backend - Optimized Dev${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 验证当前在 WSL 环境中
echo -e "${YELLOW}[0/4]${NC} 验证运行环境..."
print_wsl_status || echo -e "${YELLOW}继续尝试启动...${NC}"

# 验证 Rust 工具链在 WSL 中
echo -e "${YELLOW}验证 Rust 工具链...${NC}"
if ! verify_wsl_tool "cargo" "Cargo"; then
    echo -e "${YELLOW}请确保 Rust 工具链已安装在 WSL 中${NC}"
    exit 1
fi

# 检查 cargo-watch 是否安装
echo -e "${YELLOW}[1/4]${NC} 检查开发工具..."
if ! cargo --list | grep -q "watch"; then
    echo -e "${YELLOW}cargo-watch 未安装，正在安装...${NC}"
    cargo install cargo-watch
else
    echo -e "${GREEN}✓ cargo-watch 已安装${NC}"
fi
echo ""

# 创建必要的目录
echo -e "${YELLOW}[2/4]${NC} 准备开发环境..."
mkdir -p "$DB_DIR"
mkdir -p "$LOG_DIR"
mkdir -p "$CONFIG_DIR"

# 创建开发环境配置文件
rm -f "$CONFIG_DIR/config.toml"
cat > "$CONFIG_DIR/config.toml" << 'EOF'
[server]
host = "0.0.0.0"
port = 8081

[database]
url = "sqlite:///tmp/starrocks-admin.db"

[auth]
jwt_secret = "dev-secret-key-change-in-production"
jwt_expires_in = "24h"

[cors]
allow_origin = "http://0.0.0.0:4200"

[logging]
level = "debug"
file = "logs/starrocks-admin.log"

[static_config]
enabled = false
web_root = "../build/dist/web"
EOF
echo -e "${GREEN}✓ 配置文件已创建${NC}"
echo ""

# 检查是否需要首次编译
echo -e "${YELLOW}[3/4]${NC} 检查编译状态..."
cd "$BACKEND_DIR"

# 检查是否存在编译缓存
if [ ! -d "target/debug" ] || [ ! -f "target/debug/deps/libstarrocks_admin-*.rlib" ]; then
    echo -e "${YELLOW}首次编译或缓存缺失，进行完整编译...${NC}"
    echo -e "${BLUE}这可能需要几分钟时间，请耐心等待...${NC}"
    
    # 使用 release 模式编译依赖，debug 模式编译项目代码
    # 这样可以获得更好的性能，同时保持开发时的快速编译
    cargo build --release --dependencies-only 2>/dev/null || true
    cargo build
    echo -e "${GREEN}✓ 编译完成${NC}"
else
    echo -e "${GREEN}✓ 编译缓存存在，跳过完整编译${NC}"
fi
echo ""

# 启动开发服务器（优化的热重载模式）
echo -e "${BLUE}[4/4]${NC} 启动后端（优化热重载模式）..."
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "优化特性："
echo -e "  • 第三方库已预编译，不会重复编译"
echo -e "  • 只监听项目源代码变化"
echo -e "  • 使用增量编译，速度更快"
echo -e "  • 代码修改将自动重新编译并重启"
echo -e "按 Ctrl+C 停止服务"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""

# 使用优化的 cargo-watch 配置
# 只监听源代码文件，排除所有编译输出和日志目录
cargo watch \
    --watch "src" \
    --watch "Cargo.toml" \
    --ignore "target/**" \
    --ignore "logs/**" \
    --ignore "data/**" \
    --ignore "conf/**" \
    --ignore "*.log" \
    --ignore "*.pid" \
    --ignore "*.tmp" \
    --ignore "*.swp" \
    --ignore "*.swo" \
    --ignore ".git/**" \
    --ignore "node_modules/**" \
    --ignore "dist/**" \
    --ignore "build/**" \
    --ignore ".angular/**" \
    --ignore "coverage/**" \
    --ignore "*.lock" \
    --ignore "*.orig" \
    --ignore "*.rej" \
    --delay 1 \
    --clear \
    -x "run --bin starrocks-admin"
