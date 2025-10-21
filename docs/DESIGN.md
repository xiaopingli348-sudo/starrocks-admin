# StarRocks 集群管理平台 - 详细设计文档

## 一、项目概述

StarRocks 集群管理平台是一个现代化的、美观的、智能的集群管理系统，用于替代 StarRocks 原生的简陋管理系统。

### 1.1 技术栈

- **后端**: Rust + Axum + SQLx + SQLite + JWT
- **前端**: Angular 15 + ngx-admin + Nebular UI + ECharts  
- **数据库**: SQLite (元数据) + StarRocks (业务数据查询)
- **部署**: Docker + Docker Compose + Kubernetes
- **架构模式**: **前后端完全分离**，后端提供 RESTful API，前端独立部署

### 1.2 核心特性

1. ✅ 多集群管理和注册
2. ✅ Backend/Frontend 节点实时监控
3. ✅ 查询管理（查看和终止）
4. ✅ 系统信息展示
5. ✅ 实时监控指标和图表
6. ✅ 集群健康检查
7. ✅ JWT 用户认证
8. ✅ OpenAPI 文档自动生成

## 二、后端架构设计

### 2.1 StarRocks 数据获取策略

采用 **HTTP REST API 优先** 的混合策略：

#### HTTP REST API（主要）
- 端口：8030
- 优势：性能更好，直接访问内存，延迟 <10ms
- 用途：节点信息、查询列表、系统信息、指标数据

| 功能 | API 端点 | 说明 |
|------|---------|------|
| Backend 节点 | GET /api/show_proc?path=/backends | JSON 格式节点列表 |
| Frontend 节点 | GET /api/show_proc?path=/frontends | 包含角色、状态 |
| 当前查询 | GET /api/show_proc?path=/current_queries | 实时查询列表 |
| 系统信息 | GET /api/show_runtime_info | FE 运行时信息 |
| 执行 SQL | POST /api/v1/catalogs/{catalog}/sql | 支持任意 SQL |
| 监控指标 | GET /metrics | Prometheus 格式 |

#### MySQL 协议（辅助）
- 端口：9030
- 用途：复杂查询、SET GLOBAL、KILL QUERY 等

### 2.2 核心模块

#### 认证模块 (auth)
- JWT Token 生成和验证
- 用户注册和登录
- 密码 bcrypt 哈希

#### 集群管理模块 (cluster)
- 集群 CRUD 操作
- 连接测试
- 健康检查

#### 节点管理模块
- Backend 节点信息查询
- Frontend 节点信息查询

#### 监控模块 (monitor)
- 指标采集和聚合
- Prometheus 格式解析
- 时间序列数据存储

### 2.3 数据库设计

#### users 表
```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### clusters 表
```sql
CREATE TABLE clusters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    fe_host VARCHAR(255) NOT NULL,
    fe_http_port INTEGER NOT NULL DEFAULT 8030,
    fe_query_port INTEGER NOT NULL DEFAULT 9030,
    username VARCHAR(100) NOT NULL,
    password_encrypted VARCHAR(255) NOT NULL,
    enable_ssl BOOLEAN DEFAULT 0,
    connection_timeout INTEGER DEFAULT 10,
    tags TEXT,
    catalog VARCHAR(100) DEFAULT 'default_catalog',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by INTEGER,
    FOREIGN KEY (created_by) REFERENCES users(id)
);
```

#### monitor_history 表
```sql
CREATE TABLE monitor_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value TEXT NOT NULL,
    collected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);
```

## 三、API 设计

### 3.1 认证 API

- POST /api/auth/register - 用户注册
- POST /api/auth/login - 用户登录
- GET /api/auth/me - 获取当前用户（需认证）

### 3.2 集群管理 API

- POST /api/clusters - 创建集群（需认证）
- GET /api/clusters - 获取所有集群（需认证）
- GET /api/clusters/:id - 获取集群详情（需认证）
- PUT /api/clusters/:id - 更新集群（需认证）
- DELETE /api/clusters/:id - 删除集群（需认证）
- POST /api/clusters/:id/test - 测试连接（需认证）
- GET /api/clusters/:id/health - 健康检查（需认证）

### 3.3 节点管理 API

- GET /api/clusters/:id/backends - Backend 节点列表
- GET /api/clusters/:id/frontends - Frontend 节点列表

### 3.4 查询管理 API

- GET /api/clusters/:id/queries - 查询列表
- DELETE /api/clusters/:id/queries/:query_id - 终止查询

### 3.5 会话管理 API

- GET /api/clusters/:id/sessions - 会话列表
- DELETE /api/clusters/:id/sessions/:session_id - 终止会话

### 3.6 变量管理 API

- GET /api/clusters/:id/variables?type=global|session&filter=xxx - 变量列表
- PUT /api/clusters/:id/variables/:name - 修改变量

### 3.7 系统信息 API

- GET /api/clusters/:id/system/runtime_info - 运行时信息

### 3.8 监控 API

- GET /api/clusters/:id/metrics/summary - 指标汇总

## 四、监控指标体系

### 4.1 查询性能指标

| 指标名 | Prometheus 指标 | 说明 |
|-------|----------------|------|
| QPS | starrocks_fe_qps | 每秒查询数 |
| RPS | starrocks_fe_rps | 每秒请求数 |
| 总查询数 | starrocks_fe_query_total | 累计查询 |
| 成功查询 | starrocks_fe_query_success | 成功数 |
| 错误查询 | starrocks_fe_query_err | 失败数 |
| P50 延迟 | starrocks_fe_query_latency{type="50_quantile"} | 中位数 |
| P95 延迟 | starrocks_fe_query_latency{type="95_quantile"} | 95分位 |
| P99 延迟 | starrocks_fe_query_latency{type="99_quantile"} | 99分位 |

### 4.2 FE 系统资源指标

- JVM 总堆内存
- JVM 已用堆内存
- JVM 堆内存使用率
- JVM 线程数

### 4.3 Backend 聚合指标

- 在线节点数
- Tablet 总数
- 磁盘使用量
- CPU 平均使用率
- 内存平均使用率
- 运行中查询数

## 五、前端设计（基于 ngx-admin）

### 5.1 页面结构

```
pages/starrocks/
├── dashboard/              # 集群概览
├── clusters/              # 集群管理
│   ├── cluster-list/
│   ├── cluster-form/
│   └── cluster-detail/
├── backends/              # Backend 节点
├── frontends/             # Frontend 节点
├── queries/               # 查询管理
├── system/                # 系统信息
└── monitor/               # 监控指标
```

### 5.2 使用的 ngx-admin 组件

- **nb-card**: 卡片布局
- **ng2-smart-table**: 数据表格
- **echarts**: 图表展示
- **nb-select**: 下拉选择
- **nb-checkbox**: 复选框
- **nb-button**: 按钮
- **NbDialogService**: 对话框
- **NbToastrService**: 提示消息

## 六、部署方案

### 6.1 Docker 部署

**后端 Dockerfile**:
- 多阶段构建
- Rust 1.75 编译
- Debian bookworm-slim 运行时

**前端 Dockerfile**:
- Node.js 18 构建
- Nginx Alpine 运行时

### 6.2 Docker Compose

```yaml
services:
  backend:
    build: ./backend
    ports: ["8080:8080"]
    volumes: ["./data:/app/data"]
    
  frontend:
    build: ./frontend
    ports: ["80:80"]
    depends_on: [backend]
```

### 6.3 Kubernetes 部署

- Namespace 隔离
- Backend Deployment (2 副本 + PVC)
- Frontend Deployment (3 副本)
- Service (ClusterIP)
- Ingress (TLS)

## 七、安全设计

### 7.1 认证授权
- JWT Token 认证
- HTTP Basic Auth（StarRocks）
- CORS 配置

### 7.2 错误处理
- 统一错误码体系
- 详细错误信息

### 7.3 数据加密
- 密码 bcrypt 哈希
- StarRocks 密码加密存储（TODO）

## 八、测试集群

- URL: http://10.212.200.149:8030/
- 用户名: root
- 密码: tZcXi*^he5g5

## 九、开发指南

### 9.1 后端开发

```bash
cd backend
cargo run
# API 文档: http://localhost:8080/api-docs
```

### 9.2 前端开发

```bash
cd frontend
npm install
npm start
# 访问: http://localhost:4200
```

### 9.3 Docker 部署

```bash
docker-compose up -d
```

## 十、设计原则

- **KISS**: API 设计简洁直观
- **YAGNI**: 只实现当前需要的功能
- **DRY**: 抽象通用逻辑
- **SOLID**: 单一职责、开放封闭
