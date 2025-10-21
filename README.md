# StarRocks Admin

<div align="center">

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![License](https://img.shields.io/badge/license-MIT-blue)
![Version](https://img.shields.io/badge/version-0.1.0-orange)
![Rust](https://img.shields.io/badge/rust-1.75+-red)
![Angular](https://img.shields.io/badge/angular-15+-red)

**一个现代化、美观、智能的 StarRocks 集群管理平台**

[功能特性](#-功能特性) • [快速开始](#-快速开始) • [部署指南](#-部署指南) • [API 文档](#-api-文档) • [贡献](#-贡献)

</div>

## 📖 简介

StarRocks Admin 是一个专业的、企业级的 StarRocks 数据库集群管理工具，提供直观的 Web 界面来管理和监控多个 StarRocks 集群。相比 StarRocks 原生的管理界面，本平台提供了更丰富的功能和更好的用户体验。

### ✨ 核心特性

- 🚀 **一键部署** - 支持传统部署、Docker 和 Kubernetes
- 📊 **实时监控** - 查看集群的实时状态和性能指标
- 🔧 **集群管理** - 统一管理多个 StarRocks 集群
- 🎨 **现代 UI** - 基于 Angular + Nebular 的现代化界面
- 🔐 **安全认证** - JWT 认证和权限管理
- 📈 **性能分析** - 查询性能分析和优化建议

## 🚀 快速开始

### 方式一：一键部署（推荐）

```bash
# 1. 克隆项目
git clone https://github.com/your-org/starrocks-admin.git
cd starrocks-admin

# 2. 构建和打包
make package

# 3. 部署
make deploy

# 4. 启动服务
cd build/dist
./starrocks-admin.sh start

# 5. 访问应用
open http://localhost:8080
```

### 方式二：Docker 部署

```bash
# 1. 克隆项目
git clone https://github.com/your-org/starrocks-admin.git
cd starrocks-admin

# 2. 启动服务
make docker-up

# 3. 访问应用
open http://localhost:8080
```

### 方式三：开发环境

```bash
# 1. 克隆项目
git clone https://github.com/your-org/starrocks-admin.git
cd starrocks-admin

# 2. 启动开发环境
make dev

# 3. 访问应用
# 前端: http://localhost:4200
# 后端: http://localhost:8080
```

## 📁 项目结构

```
starrocks-admin/
├── backend/                 # Rust 后端
│   ├── src/
│   │   ├── handlers/       # API 处理器
│   │   ├── services/       # 业务逻辑
│   │   ├── models/         # 数据模型
│   │   └── middleware/     # 中间件
│   └── Cargo.toml
├── frontend/               # Angular 前端
│   ├── src/
│   │   ├── app/
│   │   │   ├── @core/      # 核心服务
│   │   │   ├── @theme/     # 主题
│   │   │   └── pages/      # 页面组件
│   │   └── environments/   # 环境配置
│   └── package.json
├── build/                  # 构建系统
│   ├── dist/              # 构建产物
│   │   ├── bin/           # 可执行文件
│   │   ├── conf/          # 配置文件
│   │   ├── web/           # 前端静态文件
│   │   ├── data/          # 数据目录
│   │   └── logs/          # 日志目录
│   └── *.sh               # 构建脚本
├── deploy/                 # 部署配置
│   ├── docker/            # Docker 部署
│   ├── k8s/               # Kubernetes 部署
│   └── scripts/           # 生产启动脚本
├── scripts/                # 开发脚本
│   └── dev/               # 开发环境脚本
└── docs/                   # 文档
```

## 🛠 技术栈

### 后端
- **语言**: Rust 1.75+
- **框架**: Axum 0.7
- **数据库**: SQLite (可扩展 MySQL/PostgreSQL)
- **认证**: JWT
- **配置**: TOML + 环境变量
- **日志**: tracing + tracing-appender

### 前端
- **框架**: Angular 15
- **UI**: ngx-admin + Nebular
- **图表**: ECharts
- **样式**: SCSS

### 部署
- **传统部署**: 一键启动脚本
- **容器化**: Docker + Docker Compose
- **编排**: Kubernetes
- **反向代理**: Nginx (可选)

## 📊 功能特性

### 集群管理
- ✅ 多集群统一管理
- ✅ 集群健康检查
- ✅ 节点状态监控
- ✅ 配置管理

### 监控指标
- 📈 QPS/RPS 实时监控
- ⏱️ 查询延迟分析（P50、P95、P99）
- 💾 资源使用率（CPU、内存、磁盘）
- 🔄 JVM 堆内存监控
- 📊 事务和加载统计

### 查询管理
- 🔍 查询历史查看
- ⏹️ 查询终止功能
- 📋 SQL 执行器
- 📈 查询性能分析

### 系统管理
- 👥 用户认证和权限
- ⚙️ 系统函数管理
- 📊 运行时信息
- 📝 操作日志

## 🔧 配置说明

### 主配置文件 (conf/config.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite://data/starrocks-admin.db"

[auth]
jwt_secret = "your-secret-key-change-in-production"
jwt_expires_in = "24h"

[cors]
allow_origin = "http://localhost:4200"

[logging]
level = "info,starrocks_admin_backend=debug"
file = "logs/starrocks-admin.log"

[static]
enabled = true
web_root = "web"
```

### 环境变量覆盖

所有配置项都支持环境变量覆盖：

```bash
export HOST=0.0.0.0
export PORT=8080
export DATABASE_URL="sqlite://data/starrocks-admin.db"
export JWT_SECRET="your-secret-key"
export RUST_LOG="info,starrocks_admin_backend=debug"
```

## 🚢 部署指南

### 传统部署

1. **构建项目**
   ```bash
   make package
   ```

2. **部署到服务器**
   ```bash
   # 解压部署包
   tar -xzf starrocks-admin-*.tar.gz
   cd starrocks-admin
   
   # 配置服务
   cp conf/config.toml.example conf/config.toml
   # 编辑 conf/config.toml
   
   # 启动服务
   ./starrocks-admin.sh start
   ```

3. **管理服务**
   ```bash
   ./starrocks-admin.sh start    # 启动
   ./starrocks-admin.sh stop     # 停止
   ./starrocks-admin.sh restart  # 重启
   ./starrocks-admin.sh status   # 状态
   ./starrocks-admin.sh logs     # 日志
   ```

### Docker 部署

1. **使用 Docker Compose**
   ```bash
   cd deploy/docker
   docker-compose up -d
   ```

2. **使用 Nginx 反向代理**
   ```bash
   docker-compose --profile nginx up -d
   ```

### Kubernetes 部署

```bash
kubectl apply -f deploy/k8s/
```

## 📚 API 文档

启动服务后，访问以下地址查看 API 文档：

- **Swagger UI**: http://localhost:8080/api-docs
- **OpenAPI JSON**: http://localhost:8080/api-docs/openapi.json

### 主要 API 端点

- `POST /api/auth/register` - 用户注册
- `POST /api/auth/login` - 用户登录
- `GET /api/clusters` - 获取集群列表
- `POST /api/clusters` - 创建集群
- `GET /api/clusters/:id/health` - 集群健康检查
- `GET /api/clusters/:id/queries` - 查询列表
- `GET /api/clusters/:id/metrics/summary` - 监控指标

## 🤝 贡献

我们欢迎所有形式的贡献！请遵循以下步骤：

1. **Fork 项目**
2. **创建特性分支** (`git checkout -b feature/AmazingFeature`)
3. **提交更改** (`git commit -m 'Add some AmazingFeature'`)
4. **推送分支** (`git push origin feature/AmazingFeature`)
5. **创建 Pull Request**

### 开发环境设置

```bash
# 1. 克隆项目
git clone https://github.com/your-org/starrocks-admin.git
cd starrocks-admin

# 2. 安装依赖
make check-env  # 检查环境
cd backend && cargo build
cd ../frontend && npm install

# 3. 启动开发环境
make dev
```

### 代码规范

- **Rust**: 遵循 Rust 官方编码规范
- **TypeScript**: 遵循 Angular 官方编码规范
- **提交信息**: 使用 Conventional Commits 格式

## 📝 开发原则

本项目遵循以下设计原则：

- **KISS**: 简洁至上，避免过度设计
- **YAGNI**: 只实现需要的功能
- **DRY**: 不重复造轮子
- **SOLID**: 单一职责、开放封闭原则

## 🗺️ 路线图

### v0.2.0 (计划中)
- [ ] 支持 MySQL/PostgreSQL 数据库
- [ ] 添加更多监控指标
- [ ] 支持集群自动发现
- [ ] 添加告警功能

### v0.3.0 (计划中)
- [ ] 支持多租户
- [ ] 添加 RBAC 权限控制
- [ ] 支持插件系统
- [ ] 添加数据导入导出

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [ngx-admin](https://github.com/akveo/ngx-admin) - 优秀的 Angular 管理模板
- [Nebular](https://akveo.github.io/nebular/) - 漂亮的 UI 组件库
- [Axum](https://github.com/tokio-rs/axum) - 强大的 Rust Web 框架
- [StarRocks](https://www.starrocks.io/) - 高性能分析数据库

## 📞 支持

- 📧 **邮箱**: support@starrocks-admin.com
- 🐛 **问题反馈**: [GitHub Issues](https://github.com/your-org/starrocks-admin/issues)
- 💬 **讨论**: [GitHub Discussions](https://github.com/your-org/starrocks-admin/discussions)
- 📖 **文档**: [项目文档](https://docs.starrocks-admin.com)

---

<div align="center">

**Made with ❤️ for StarRocks Community**

[⬆ 回到顶部](#starrocks-admin)

</div>