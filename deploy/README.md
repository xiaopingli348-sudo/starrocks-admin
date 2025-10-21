# StarRocks Admin - 部署指南

本目录包含 StarRocks Admin 的所有部署相关配置和脚本。

## 📁 目录结构

```
deploy/
├── docker/                 # Docker 部署
│   ├── docker-compose.yml # Docker Compose 配置
│   ├── Dockerfile         # Docker 镜像构建
│   ├── nginx.conf         # Nginx 配置
│   └── config.toml        # Docker 环境配置
├── k8s/                   # Kubernetes 部署
│   ├── backend/           # 后端 K8s 配置
│   ├── frontend/          # 前端 K8s 配置
│   ├── ingress.yaml       # Ingress 配置
│   └── namespace.yaml     # 命名空间配置
└── scripts/               # 生产环境脚本
    └── starrocks-admin.sh # 主启动脚本
```

## 🚀 部署方式

### 1. 传统部署（推荐）

传统部署方式适合大多数生产环境，提供最大的灵活性和控制力。

#### 快速开始

```bash
# 1. 构建项目
make package

# 2. 解压部署包
tar -xzf starrocks-admin-*.tar.gz
cd starrocks-admin

# 3. 配置服务
cp conf/config.toml.example conf/config.toml
# 编辑 conf/config.toml

# 4. 启动服务
./starrocks-admin.sh start
```

#### 服务管理

```bash
# 启动服务
./starrocks-admin.sh start

# 停止服务
./starrocks-admin.sh stop

# 重启服务
./starrocks-admin.sh restart

# 查看状态
./starrocks-admin.sh status

# 查看日志
./starrocks-admin.sh logs
```

#### 目录结构

```
starrocks-admin/
├── bin/                    # 可执行文件
│   ├── starrocks-admin
│   └── start-backend.sh
├── conf/                   # 配置文件
│   ├── config.toml
│   └── config.toml.example
├── web/                    # 前端静态文件
├── data/                   # 数据目录
├── logs/                   # 日志目录
├── lib/                    # 依赖库
└── starrocks-admin.sh      # 主启动脚本
```

#### 配置说明

编辑 `conf/config.toml` 文件：

```toml
[server]
host = "0.0.0.0"          # 监听地址
port = 8080               # 监听端口

[database]
url = "sqlite://data/starrocks-admin.db"  # 数据库连接

[auth]
jwt_secret = "your-secret-key-change-in-production"  # JWT 密钥
jwt_expires_in = "24h"    # Token 过期时间

[cors]
allow_origin = "http://localhost:4200"  # CORS 允许的源

[logging]
level = "info,starrocks_admin_backend=debug"  # 日志级别
file = "logs/starrocks-admin.log"             # 日志文件

[static]
enabled = true            # 是否启用静态文件服务
web_root = "web"          # 静态文件根目录
```

### 2. Docker 部署

Docker 部署方式适合容器化环境，提供一致的运行环境。

#### 快速开始

```bash
# 1. 启动服务
make docker-up

# 2. 查看日志
make docker-logs

# 3. 停止服务
make docker-down
```

#### 使用 Nginx 反向代理

```bash
# 启动带 Nginx 的完整服务
cd deploy/docker
docker-compose --profile nginx up -d
```

#### 自定义配置

1. **修改 Docker 配置**
   ```bash
   # 编辑 deploy/docker/config.toml
   vim deploy/docker/config.toml
   ```

2. **修改 Nginx 配置**
   ```bash
   # 编辑 deploy/docker/nginx.conf
   vim deploy/docker/nginx.conf
   ```

3. **重新构建和启动**
   ```bash
   make docker-build
   make docker-up
   ```

### 3. Kubernetes 部署

Kubernetes 部署方式适合大规模生产环境，提供高可用性和自动扩缩容。

#### 快速开始

```bash
# 1. 创建命名空间
kubectl apply -f deploy/k8s/namespace.yaml

# 2. 部署后端
kubectl apply -f deploy/k8s/backend/

# 3. 部署前端
kubectl apply -f deploy/k8s/frontend/

# 4. 配置 Ingress
kubectl apply -f deploy/k8s/ingress.yaml
```

#### 配置说明

- **后端配置**: `deploy/k8s/backend/deployment.yaml`
- **前端配置**: `deploy/k8s/frontend/deployment.yaml`
- **服务配置**: `deploy/k8s/backend/service.yaml`
- **Ingress 配置**: `deploy/k8s/ingress.yaml`

## 🔧 高级配置

### 环境变量

所有配置项都支持环境变量覆盖：

```bash
export HOST=0.0.0.0
export PORT=8080
export DATABASE_URL="sqlite://data/starrocks-admin.db"
export JWT_SECRET="your-secret-key"
export RUST_LOG="info,starrocks_admin_backend=debug"
```

### 数据库配置

#### SQLite（默认）

```toml
[database]
url = "sqlite://data/starrocks-admin.db"
```

#### MySQL

```toml
[database]
url = "mysql://user:password@localhost:3306/starrocks_admin"
```

#### PostgreSQL

```toml
[database]
url = "postgresql://user:password@localhost:5432/starrocks_admin"
```

### 日志配置

#### 文件日志

```toml
[logging]
level = "info,starrocks_admin_backend=debug"
file = "logs/starrocks-admin.log"
```

#### 控制台日志

```toml
[logging]
level = "info,starrocks_admin_backend=debug"
# file = "logs/starrocks-admin.log"  # 注释掉文件日志
```

### 静态文件配置

#### 启用静态文件服务（默认）

```toml
[static]
enabled = true
web_root = "web"
```

#### 禁用静态文件服务（使用 Nginx）

```toml
[static]
enabled = false
```

## 🔍 故障排除

### 常见问题

#### 1. 服务启动失败

**问题**: 服务启动后立即退出

**解决方案**:
```bash
# 查看日志
./starrocks-admin.sh logs

# 检查配置文件
cat conf/config.toml

# 检查端口是否被占用
netstat -tlnp | grep 8080
```

#### 2. 数据库连接失败

**问题**: 无法连接到数据库

**解决方案**:
```bash
# 检查数据库文件权限
ls -la data/

# 创建数据目录
mkdir -p data

# 检查配置文件
grep -A 2 "\[database\]" conf/config.toml
```

#### 3. 前端页面无法访问

**问题**: 访问 http://localhost:8080 显示 404

**解决方案**:
```bash
# 检查静态文件目录
ls -la web/

# 检查配置
grep -A 2 "\[static\]" conf/config.toml

# 重新构建前端
make build-frontend
```

#### 4. 权限问题

**问题**: 权限不足导致启动失败

**解决方案**:
```bash
# 检查文件权限
ls -la bin/starrocks-admin

# 添加执行权限
chmod +x bin/starrocks-admin

# 检查目录权限
ls -la data/ logs/
```

### 日志分析

#### 查看实时日志

```bash
# 查看所有日志
./starrocks-admin.sh logs

# 查看错误日志
tail -f logs/starrocks-admin.log | grep ERROR

# 查看访问日志
tail -f logs/starrocks-admin.log | grep "GET\|POST"
```

#### 日志级别调整

```toml
[logging]
level = "debug"  # 最详细
# level = "info"   # 一般信息
# level = "warn"   # 警告
# level = "error"  # 仅错误
```

## 📊 监控和维护

### 健康检查

```bash
# 检查服务状态
curl http://localhost:8080/health

# 检查就绪状态
curl http://localhost:8080/ready

# 检查 API 文档
curl http://localhost:8080/api-docs
```

### 性能监控

```bash
# 查看进程资源使用
ps aux | grep starrocks-admin

# 查看内存使用
free -h

# 查看磁盘使用
df -h
```

### 备份和恢复

#### 数据备份

```bash
# 备份数据库
cp data/starrocks-admin.db backup/starrocks-admin-$(date +%Y%m%d).db

# 备份配置
cp conf/config.toml backup/config-$(date +%Y%m%d).toml
```

#### 数据恢复

```bash
# 停止服务
./starrocks-admin.sh stop

# 恢复数据库
cp backup/starrocks-admin-20240101.db data/starrocks-admin.db

# 启动服务
./starrocks-admin.sh start
```

## 🔄 升级指南

### 升级步骤

1. **备份数据**
   ```bash
   ./starrocks-admin.sh stop
   cp -r data/ backup/
   cp conf/config.toml backup/
   ```

2. **下载新版本**
   ```bash
   # 下载新的部署包
   wget https://github.com/your-org/starrocks-admin/releases/latest/download/starrocks-admin-latest.tar.gz
   ```

3. **解压新版本**
   ```bash
   tar -xzf starrocks-admin-latest.tar.gz
   cd starrocks-admin
   ```

4. **恢复配置**
   ```bash
   cp backup/config.toml conf/
   ```

5. **启动服务**
   ```bash
   ./starrocks-admin.sh start
   ```

### 回滚步骤

如果升级出现问题，可以快速回滚：

```bash
# 停止新版本
./starrocks-admin.sh stop

# 恢复旧版本
cd ..
tar -xzf starrocks-admin-previous.tar.gz
cd starrocks-admin

# 恢复数据
cp backup/starrocks-admin.db data/
cp backup/config.toml conf/

# 启动服务
./starrocks-admin.sh start
```

## 📞 支持

如果您在部署过程中遇到问题，请：

1. 查看本文档的故障排除部分
2. 检查 [GitHub Issues](https://github.com/your-org/starrocks-admin/issues)
3. 提交新的 Issue 并附上详细的错误信息

---

**祝您部署顺利！** 🚀
