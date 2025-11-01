#!/usr/bin/env bash

#
# StarRocks Admin - Optimized Backend Build Script
# 优化的后端构建脚本（支持增量构建）
#

set -e

# 导入构建工具函数
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/build-utils.sh"

# 获取项目路径
PROJECT_ROOT=$(get_project_root)
BACKEND_DIR="$PROJECT_ROOT/backend"
BUILD_DIR="$PROJECT_ROOT/build"
DIST_DIR="$BUILD_DIR/dist"
LAST_BUILD_FILE="$BUILD_DIR/.last_backend_build"

# 检查是否需要构建
NEEDS_BUILD=$(check_backend_changed "$PROJECT_ROOT")

if [ "$NEEDS_BUILD" = "false" ]; then
    show_build_status "后端" "false" "文件无修改"
    exit 0
fi

show_build_status "后端" "true" "检测到文件修改"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Building StarRocks Admin Backend${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 检查构建依赖
if ! check_build_dependencies; then
    exit 1
fi

# 清理旧的构建目录
echo -e "${YELLOW}[1/5]${NC} 清理旧的构建产物..."
rm -rf "$DIST_DIR"

# 创建输出目录
echo -e "${YELLOW}[2/5]${NC} 创建输出目录..."
mkdir -p "$DIST_DIR/bin"
mkdir -p "$DIST_DIR/conf"
mkdir -p "$DIST_DIR/lib"
mkdir -p "$DIST_DIR/data"
mkdir -p "$DIST_DIR/logs"
mkdir -p "$DIST_DIR/migrations"

# 构建后端
echo -e "${YELLOW}[3/5]${NC} 编译 Rust 后端 (release 模式)..."
cd "$BACKEND_DIR"

# 确保 Rust 环境已加载
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# 构建
cargo build --release

# 复制二进制文件
echo -e "${YELLOW}[4/5]${NC} 复制后端二进制文件..."
cp target/release/starrocks-admin "$DIST_DIR/bin/"

# 创建生产配置文件
echo -e "${YELLOW}[5/5]${NC} 创建生产配置文件..."
cat > "$DIST_DIR/conf/config.toml" << 'EOF'
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite://data/starrocks-admin.db"

[auth]
jwt_secret = "dev-secret-key-change-in-production"
jwt_expires_in = "24h"

[logging]
level = "info,starrocks_admin_backend=debug"
file = "logs/starrocks-admin.log"

[static_config]
enabled = true
web_root = "web"
EOF

# 复制数据库迁移文件
if [ -d "$BACKEND_DIR/migrations" ]; then
    cp -r "$BACKEND_DIR/migrations"/* "$DIST_DIR/migrations/"
    echo "复制了 $(ls "$DIST_DIR/migrations" | wc -l) 个迁移文件"
fi

# 创建启动脚本
cat > "$DIST_DIR/bin/starrocks-admin.sh" << 'EOF'
#!/bin/bash

# StarRocks Admin Backend - 生产环境管理脚本

set -e

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$SCRIPT_DIR/starrocks-admin"
CONFIG_DIR="$DIST_ROOT/conf"
DATA_DIR="$DIST_ROOT/data"
LOG_DIR="$DIST_ROOT/logs"
PID_FILE="$DIST_ROOT/starrocks-admin.pid"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 显示帮助信息
show_help() {
    echo -e "${GREEN}StarRocks Admin Backend 生产环境管理脚本${NC}"
    echo ""
    echo "用法: $0 {start|stop|restart|status|logs|help}"
    echo ""
    echo "命令:"
    echo "  start   - 启动后端服务"
    echo "  stop    - 停止后端服务"
    echo "  restart - 重启后端服务"
    echo "  status  - 查看服务状态"
    echo "  logs    - 查看实时日志"
    echo "  help    - 显示此帮助信息"
    echo ""
}

# 检查服务是否运行
is_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if ps -p "$pid" > /dev/null 2>&1; then
            return 0
        else
            rm -f "$PID_FILE"
            return 1
        fi
    fi
    return 1
}

# 获取服务PID
get_pid() {
    if [ -f "$PID_FILE" ]; then
        cat "$PID_FILE"
    else
        echo ""
    fi
}

# 检查并强杀占用端口的进程
kill_port_process() {
    local port=$1
    echo -e "${YELLOW}[INFO]${NC} 检查端口 $port 占用情况..."
    
    # 查找占用端口的进程
    local pids=$(lsof -ti:$port 2>/dev/null || netstat -tlnp 2>/dev/null | grep ":$port " | awk '{print $7}' | cut -d'/' -f1)
    
    if [ -z "$pids" ]; then
        echo -e "${GREEN}[INFO]${NC} 端口 $port 未被占用"
        return 0
    fi
    
    # 处理每个占用端口的进程
    for pid in $pids; do
        if [ -n "$pid" ] && [ "$pid" != "-" ]; then
            # 获取进程信息
            local proc_info=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
            echo -e "${YELLOW}[WARNING]${NC} 端口 $port 被进程占用: PID=$pid, 进程=$proc_info"
            
            # 尝试优雅停止
            echo -e "${YELLOW}[INFO]${NC} 尝试优雅停止进程 $pid..."
            kill -TERM $pid 2>/dev/null || true
            sleep 2
            
            # 检查进程是否还在运行
            if ps -p $pid > /dev/null 2>&1; then
                echo -e "${RED}[WARNING]${NC} 进程 $pid 未响应，强制终止..."
                kill -KILL $pid 2>/dev/null || true
                sleep 1
            fi
            
            # 最终检查
            if ps -p $pid > /dev/null 2>&1; then
                echo -e "${RED}[ERROR]${NC} 无法终止进程 $pid，请手动处理"
                return 1
            else
                echo -e "${GREEN}[SUCCESS]${NC} 已终止占用端口的进程 $pid"
            fi
        fi
    done
    
    # 再次确认端口已释放
    sleep 1
    local check_pids=$(lsof -ti:$port 2>/dev/null)
    if [ -n "$check_pids" ]; then
        echo -e "${RED}[ERROR]${NC} 端口 $port 仍被占用，请手动检查"
        return 1
    fi
    
    echo -e "${GREEN}[SUCCESS]${NC} 端口 $port 已释放"
    return 0
}

# 启动服务
start_service() {
    if is_running; then
        echo -e "${YELLOW}[WARNING]${NC} 后端服务已在运行 (PID: $(get_pid))"
        return 0
    fi

    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}StarRocks Admin Backend 启动脚本${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""

    # 检查二进制文件
    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}[ERROR]${NC} 二进制文件不存在: $BINARY_PATH"
        exit 1
    fi

    # 创建必要的目录
    echo -e "${YELLOW}[INFO]${NC} 创建必要的目录..."
    mkdir -p "$DATA_DIR"
    mkdir -p "$LOG_DIR"
    mkdir -p "$CONFIG_DIR"

    # 设置环境变量
    export DATABASE_URL="${DATABASE_URL:-sqlite://$DATA_DIR/starrocks-admin.db}"
    export HOST="${HOST:-0.0.0.0}"
    export PORT="${PORT:-8080}"

    # 显示配置信息
    echo -e "${GREEN}[CONFIG]${NC} 配置信息:"
    echo "  - 二进制文件: $BINARY_PATH"
    echo "  - 配置文件: $CONFIG_DIR/config.toml"
    echo "  - 数据目录: $DATA_DIR"
    echo "  - 日志目录: $LOG_DIR"
    echo "  - 监听地址: $HOST:$PORT"
    echo "  - 数据库: $DATABASE_URL"
    echo ""

    # 检查并清理端口占用
    if ! kill_port_process "$PORT"; then
        echo -e "${RED}[ERROR]${NC} 无法释放端口 $PORT"
        exit 1
    fi
    echo ""

    # 启动后端
    echo -e "${GREEN}[START]${NC} 启动后端服务..."
    cd "$DIST_ROOT"
    nohup "$BINARY_PATH" > "$LOG_DIR/starrocks-admin.log" 2>&1 &
    BACKEND_PID=$!
    echo $BACKEND_PID > "$PID_FILE"

    # 等待启动
    sleep 3

    # 检查进程
    if ps -p $BACKEND_PID > /dev/null; then
        echo -e "${GREEN}[SUCCESS]${NC} 后端启动成功!"
        echo "  - PID: $BACKEND_PID"
        echo "  - 健康检查: http://$HOST:$PORT/health"
        echo "  - Web UI: http://$HOST:$PORT"
        echo ""

        # 测试健康检查
        if curl -s "http://$HOST:$PORT/health" > /dev/null 2>&1; then
            echo -e "${GREEN}[健康检查]${NC} ✅ Backend运行正常"
        else
            echo -e "${YELLOW}[警告]${NC} Backend已启动但健康检查失败，请查看日志"
        fi
    else
        echo -e "${RED}[ERROR]${NC} 后端启动失败，请查看日志:"
        echo "  tail -f $LOG_DIR/starrocks-admin.log"
        rm -f "$PID_FILE"
        exit 1
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}后端已启动${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
}

# 停止服务
stop_service() {
    if ! is_running; then
        echo -e "${YELLOW}[WARNING]${NC} 后端服务未运行"
        return 0
    fi

    local pid=$(get_pid)
    echo -e "${YELLOW}[INFO]${NC} 停止后端服务 (PID: $pid)..."
    
    # 优雅停止
    kill -TERM "$pid" 2>/dev/null || true
    sleep 2
    
    # 检查是否还在运行
    if ps -p "$pid" > /dev/null 2>&1; then
        echo -e "${YELLOW}[INFO]${NC} 强制停止后端服务..."
        kill -KILL "$pid" 2>/dev/null || true
        sleep 1
    fi
    
    # 清理PID文件
    rm -f "$PID_FILE"
    
    if ps -p "$pid" > /dev/null 2>&1; then
        echo -e "${RED}[ERROR]${NC} 无法停止后端服务"
        exit 1
    else
        echo -e "${GREEN}[SUCCESS]${NC} 后端服务已停止"
    fi
}

# 重启服务
restart_service() {
    echo -e "${BLUE}[INFO]${NC} 重启后端服务..."
    stop_service
    sleep 1
    start_service
}

# 查看服务状态
show_status() {
    if is_running; then
        local pid=$(get_pid)
        echo -e "${GREEN}[STATUS]${NC} 后端服务正在运行"
        echo "  - PID: $pid"
        echo "  - 二进制文件: $BINARY_PATH"
        echo "  - 配置文件: $CONFIG_DIR/config.toml"
        echo "  - 日志文件: $LOG_DIR/starrocks-admin.log"
        echo "  - 数据目录: $DATA_DIR"
        echo "  - 健康检查: http://${HOST:-0.0.0.0}:${PORT:-8080}/health"
        echo "  - Web UI: http://${HOST:-0.0.0.0}:${PORT:-8080}"
        
        # 测试健康检查
        if curl -s "http://${HOST:-0.0.0.0}:${PORT:-8080}/health" > /dev/null 2>&1; then
            echo -e "  - 健康状态: ${GREEN}✅ 正常${NC}"
        else
            echo -e "  - 健康状态: ${RED}❌ 异常${NC}"
        fi
    else
        echo -e "${RED}[STATUS]${NC} 后端服务未运行"
    fi
}

# 查看日志
show_logs() {
    if [ ! -f "$LOG_DIR/starrocks-admin.log" ]; then
        echo -e "${YELLOW}[WARNING]${NC} 日志文件不存在: $LOG_DIR/starrocks-admin.log"
        return 1
    fi
    
    echo -e "${BLUE}[INFO]${NC} 显示实时日志 (按 Ctrl+C 退出)..."
    tail -f "$LOG_DIR/starrocks-admin.log"
}

# 主函数
main() {
    case "${1:-start}" in
        start)
            start_service
            ;;
        stop)
            stop_service
            ;;
        restart)
            restart_service
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}[ERROR]${NC} 未知命令: $1"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"
EOF

chmod +x "$DIST_DIR/bin/starrocks-admin.sh"

# 更新构建时间戳
update_build_timestamp "$LAST_BUILD_FILE"

echo ""
echo -e "${GREEN}✓ 后端构建完成!${NC}"
echo -e "  二进制文件: $DIST_DIR/bin/starrocks-admin"
echo -e "  启动脚本: $DIST_DIR/bin/starrocks-admin.sh"
echo -e "  配置文件: $DIST_DIR/conf/config.toml"
