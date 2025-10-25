# Cluster Overview 测试方案

## 📋 测试目标

验证 Cluster Overview 功能的完整性，包括：
1. **后端API**：所有接口正常响应，数据正确
2. **数据采集**：MetricsCollectorService 定时任务正常运行
3. **前端UI**：页面正常渲染，图表正确显示
4. **集成测试**：前后端数据流畅通
5. **性能测试**：响应时间和资源占用符合预期

---

## 🔧 测试环境准备

### 1. 数据库迁移

```bash
cd backend
DATABASE_URL="sqlite:../build/data/starrocks-admin.db" cargo sqlx database create
DATABASE_URL="sqlite:../build/data/starrocks-admin.db" cargo sqlx migrate run
```

**验证**：
```bash
sqlite3 build/data/starrocks-admin.db ".tables"
# 应该看到: metrics_snapshots, daily_snapshots, data_statistics
```

### 2. StarRocks 集群准备

本项目测试使用 Docker 快速启动 StarRocks 存算一体集群。

#### 2.1 启动 StarRocks Docker 集群

使用 StarRocks 官方提供的 all-in-one 镜像：

```bash
# 启动 StarRocks 集群（包含 FE + BE）
docker run -p 9030:9030 -p 8030:8030 -p 8040:8040 -itd \
  --name starrocks-test \
  starrocks/allin1-ubuntu

# 等待集群启动（约 30 秒）
sleep 30

# 检查容器状态
docker ps | grep starrocks-test

# 检查日志
docker logs starrocks-test
```

**端口说明**：
- `9030`: MySQL 协议端口（查询端口）
- `8030`: FE HTTP 端口（API 接口）
- `8040`: BE HTTP 端口（数据导入）

#### 2.2 验证集群启动

```bash
# 使用 MySQL 客户端连接（从 Docker 内部）
docker exec -it starrocks-test \
  mysql -P 9030 -h 127.0.0.1 -u root --prompt="StarRocks > "

# 或从本机连接（需要安装 MySQL 客户端）
mysql -P 9030 -h 127.0.0.1 -u root
```

**验证 SQL**：
```sql
-- 查看 FE 状态
SHOW FRONTENDS\G

-- 查看 BE 状态
SHOW BACKENDS\G

-- 预期结果
-- FE: Alive = true
-- BE: Alive = true
```

**健康检查**：
```bash
# 检查 FE HTTP 接口
curl http://localhost:8030/api/health

# 预期响应: {"status":"OK"}

# 检查 BE HTTP 接口
curl http://localhost:8040/api/health

# 预期响应: {"status":"OK"}
```

### 3. 创建测试数据库和表

连接到 StarRocks 后，创建测试数据：

```sql
-- 创建测试数据库
CREATE DATABASE IF NOT EXISTS test_db;
USE test_db;

-- 创建测试表1：用户表
CREATE TABLE IF NOT EXISTS users (
    user_id INT,
    username VARCHAR(50),
    email VARCHAR(100),
    created_at DATETIME,
    status VARCHAR(20)
) DUPLICATE KEY(user_id)
DISTRIBUTED BY HASH(user_id) BUCKETS 4;

-- 创建测试表2：订单表
CREATE TABLE IF NOT EXISTS orders (
    order_id BIGINT,
    user_id INT,
    product_name VARCHAR(200),
    amount DECIMAL(10, 2),
    order_date DATETIME
) DUPLICATE KEY(order_id)
DISTRIBUTED BY HASH(order_id) BUCKETS 8;

-- 创建测试表3：大表（用于容量测试）
CREATE TABLE IF NOT EXISTS large_table (
    id BIGINT,
    data VARCHAR(500),
    created_at DATETIME
) DUPLICATE KEY(id)
DISTRIBUTED BY HASH(id) BUCKETS 16;

-- 插入测试数据（用户表）
INSERT INTO users VALUES
(1, 'admin', 'admin@example.com', '2025-01-01 10:00:00', 'active'),
(2, 'user1', 'user1@example.com', '2025-01-02 11:00:00', 'active'),
(3, 'user2', 'user2@example.com', '2025-01-03 12:00:00', 'active'),
(4, 'user3', 'user3@example.com', '2025-01-04 13:00:00', 'inactive'),
(5, 'user4', 'user4@example.com', '2025-01-05 14:00:00', 'active');

-- 插入测试数据（订单表）
INSERT INTO orders VALUES
(1001, 1, 'Product A', 99.99, '2025-01-10 09:00:00'),
(1002, 1, 'Product B', 149.99, '2025-01-10 10:00:00'),
(1003, 2, 'Product C', 79.99, '2025-01-11 11:00:00'),
(1004, 2, 'Product A', 99.99, '2025-01-11 12:00:00'),
(1005, 3, 'Product D', 199.99, '2025-01-12 13:00:00'),
(1006, 3, 'Product B', 149.99, '2025-01-12 14:00:00'),
(1007, 4, 'Product C', 79.99, '2025-01-13 15:00:00'),
(1008, 5, 'Product A', 99.99, '2025-01-14 16:00:00');

-- 插入大量测试数据（用于容量预测测试）
INSERT INTO large_table 
SELECT 
    number as id,
    CONCAT('test_data_', number) as data,
    DATE_ADD('2025-01-01', INTERVAL number DAY) as created_at
FROM TABLE(generate_series(1, 10000));

-- 验证数据插入
SELECT 
    'users' as table_name, COUNT(*) as row_count FROM users
UNION ALL
SELECT 'orders', COUNT(*) FROM orders
UNION ALL
SELECT 'large_table', COUNT(*) FROM large_table;
```

**预期输出**：
```
+-------------+-----------+
| table_name  | row_count |
+-------------+-----------+
| users       |         5 |
| orders      |         8 |
| large_table |     10000 |
+-------------+-----------+
```

---

### 4. 配置审计日志（重要！）

审计日志是"Top表按访问量"和"慢查询"功能的数据源。

#### 4.1 启用审计日志

```sql
-- 连接到 StarRocks
mysql -P 9030 -h 127.0.0.1 -u root

-- 启用审计日志插件
INSTALL PLUGIN auditloader 
SONAME "auditloader.so";

-- 查看插件状态
SHOW PLUGINS\G

-- 预期输出
-- Name: auditloader
-- Status: INSTALLED
```

**通过 FE 配置文件启用**（推荐）：

```bash
# 进入 Docker 容器
docker exec -it starrocks-test bash

# 编辑 FE 配置文件
vi /opt/starrocks/fe/conf/fe.conf

# 添加以下配置
enable_audit_plugin = true
audit_log_dir = /opt/starrocks/fe/log
audit_log_modules = slow_query, query, load
audit_log_roll_mode = TIME-DAY
audit_log_roll_num = 30

# 重启 FE
/opt/starrocks/fe/bin/stop_fe.sh
/opt/starrocks/fe/bin/start_fe.sh --daemon

# 退出容器
exit
```

#### 4.2 验证审计日志

```sql
-- 查看审计数据库
SHOW DATABASES LIKE '%audit%';

-- 预期结果
+-------------------------+
| Database                |
+-------------------------+
| starrocks_audit_db__    |
+-------------------------+

-- 查看审计表
USE starrocks_audit_db__;
SHOW TABLES;

-- 预期结果
+-------------------------------+
| Tables_in_starrocks_audit_db__|
+-------------------------------+
| starrocks_audit_tbl__         |
+-------------------------------+

-- 查看审计表结构
DESC starrocks_audit_tbl__;

-- 查看审计日志数量
SELECT COUNT(*) FROM starrocks_audit_tbl__;
```

#### 4.3 生成审计日志测试数据

执行一些查询以生成审计日志：

```sql
-- 执行各种查询（会被记录到审计日志）
USE test_db;

-- 快速查询
SELECT * FROM users WHERE user_id = 1;
SELECT * FROM users WHERE status = 'active';

-- 表连接查询
SELECT u.username, o.product_name, o.amount 
FROM users u 
JOIN orders o ON u.user_id = o.user_id 
WHERE u.status = 'active';

-- 聚合查询
SELECT user_id, COUNT(*) as order_count, SUM(amount) as total_amount 
FROM orders 
GROUP BY user_id;

-- 慢查询（模拟耗时查询）
SELECT COUNT(*) FROM large_table WHERE data LIKE '%999%';

-- 全表扫描
SELECT * FROM large_table ORDER BY created_at DESC LIMIT 1000;

-- 重复查询同一个表（用于测试访问量统计）
SELECT * FROM orders WHERE order_id > 1000;
SELECT * FROM orders WHERE amount > 100;
SELECT * FROM orders WHERE order_date >= '2025-01-10';
SELECT COUNT(*) FROM orders;
SELECT MAX(amount) FROM orders;
```

#### 4.4 验证审计日志数据

```sql
-- 查看最近的审计日志
SELECT 
    queryId,
    `user`,
    `db`,
    queryTime,
    scanRows,
    LEFT(stmt, 100) as query_preview
FROM starrocks_audit_db__.starrocks_audit_tbl__
ORDER BY queryTime DESC
LIMIT 20;

-- 验证慢查询
SELECT 
    queryId,
    queryTime,
    scanRows,
    LEFT(stmt, 50) as query
FROM starrocks_audit_db__.starrocks_audit_tbl__
WHERE queryTime > 100  -- 大于100ms
ORDER BY queryTime DESC;

-- 验证表访问统计
SELECT 
    `db`,
    COUNT(*) as access_count
FROM starrocks_audit_db__.starrocks_audit_tbl__
WHERE `db` = 'test_db'
GROUP BY `db`;
```

**预期结果**：
- ✅ 审计日志表有数据
- ✅ 能看到刚才执行的查询记录
- ✅ `queryTime`、`scanRows` 等字段有值

---

### 5. 在 StarRocks Admin 中添加测试集群

现在需要在 StarRocks Admin 中添加刚创建的测试集群：

#### 5.1 准备集群配置信息

```bash
# 集群信息
名称: test_cluster
描述: Docker 测试集群
FE Host: 127.0.0.1 或 host.docker.internal
FE HTTP Port: 8030
FE Query Port: 9030
用户名: root
密码: (空)
Catalog: default_catalog
```

**注意**：如果 StarRocks Admin 后端也在 Docker 中运行，需要使用：
- macOS/Windows Docker Desktop: `host.docker.internal`
- Linux: `172.17.0.1` (Docker bridge IP)

#### 5.2 通过 API 添加集群

```bash
# 获取 JWT Token（先登录）
TOKEN=$(curl -s -X POST http://localhost:8081/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.token')

# 添加测试集群
curl -X POST http://localhost:8081/api/clusters \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "test_cluster",
    "description": "Docker 测试集群",
    "fe_host": "host.docker.internal",
    "fe_http_port": 8030,
    "fe_query_port": 9030,
    "username": "root",
    "password": "",
    "catalog": "default_catalog"
  }'
```

#### 5.3 验证集群连接

```bash
# 测试集群连接
curl http://localhost:8081/api/clusters/1/test \
  -H "Authorization: Bearer $TOKEN"

# 预期响应
{"status":"success","message":"Connection successful"}

# 查看集群列表
curl http://localhost:8081/api/clusters \
  -H "Authorization: Bearer $TOKEN"
```

---

### 6. 等待数据采集

启动 StarRocks Admin 后端后，`MetricsCollectorService` 会自动开始采集指标：

```bash
# 查看后端日志，确认采集正在进行
tail -f backend/logs/starrocks-admin.log | grep metrics-collector

# 预期日志（每30秒）
[INFO] Scheduled task 'metrics-collector' running...
[INFO] Collecting metrics for cluster 1: test_cluster
[INFO] Fetched Prometheus metrics: 150 data points
[INFO] Saved metric snapshot for cluster 1

# 等待至少 2-3 分钟，确保有足够的历史数据
```

**验证数据采集**：

```bash
# 查询采集的指标数据
sqlite3 /tmp/starrocks-admin.db "
SELECT 
    cluster_id,
    COUNT(*) as snapshot_count,
    MIN(collected_at) as first_collection,
    MAX(collected_at) as latest_collection
FROM metrics_snapshots
GROUP BY cluster_id;
"

# 预期输出（等待5分钟后）
# cluster_id|snapshot_count|first_collection|latest_collection
# 1|10|2025-01-25 10:00:00|2025-01-25 10:05:00
```

### 7. 启动 StarRocks Admin 测试环境

本项目提供了完善的开发脚本，位于 `scripts/dev/` 目录。

#### 方式一：一键启动（推荐）

同时启动前后端：

```bash
# 从项目根目录执行
bash scripts/dev/start.sh
```

**该脚本会自动：**
- ✅ 检查 Rust/Cargo 和 Node.js/npm 环境
- ✅ 启动后端服务（端口 8081）
- ✅ 启动前端服务（端口 4200）
- ✅ 健康检查（自动测试服务是否正常）

**访问地址**：
- Frontend: `http://localhost:4200`
- Backend: `http://0.0.0.0:8081`
- API Docs: `http://0.0.0.0:8081/api-docs`

**停止服务**：按 `Ctrl+C`

---

#### 方式二：分别启动

**启动后端**：

```bash
# 启动后端
bash scripts/dev/start_backend.sh

# 其他命令
bash scripts/dev/start_backend.sh stop     # 停止
bash scripts/dev/start_backend.sh restart  # 重启
bash scripts/dev/start_backend.sh status   # 查看状态
bash scripts/dev/start_backend.sh logs     # 查看日志
```

**后端特性**：
- ✅ 自动创建配置文件 `backend/conf/config.toml`
- ✅ 自动创建数据目录 `backend/data/`
- ✅ 编译为 release 模式（性能优化）
- ✅ 后台运行（使用 nohup）
- ✅ PID 文件管理
- ✅ 健康检查：`http://localhost:8081/health`

**启动前端**：

```bash
# 启动前端
bash scripts/dev/start_frontend.sh

# 其他命令
bash scripts/dev/start_frontend.sh stop     # 停止
bash scripts/dev/start_frontend.sh restart  # 重启
bash scripts/dev/start_frontend.sh status   # 查看状态
bash scripts/dev/start_frontend.sh logs     # 查看日志
```

**前端特性**：
- ✅ 自动检查并安装 npm 依赖
- ✅ 自动创建 `.npmrc` 配置
- ✅ 后台运行（使用 nohup）
- ✅ PID 文件管理
- ✅ 端口冲突自动处理
- ✅ 访问地址：`http://0.0.0.0:4200`

---

#### 方式三：手动启动（开发调试用）

**手动启动后端**：

```bash
cd backend
DATABASE_URL="sqlite:../build/data/starrocks-admin.db" cargo run
```

**手动启动前端**：

```bash
cd frontend
npm install  # 首次运行
npm start
```

---

### 8. 验证服务启动

**检查后端**：

```bash
# 健康检查
curl http://localhost:8081/health

# 预期响应
{"status":"ok","timestamp":"2025-01-24T10:00:00Z"}

# 检查日志
tail -f backend/logs/starrocks-admin.log

# 预期日志
[2025-01-24T10:00:00Z INFO ] Server started on 0.0.0.0:8081
[2025-01-24T10:00:01Z INFO ] Scheduled task 'metrics-collector' running...
[2025-01-24T10:00:02Z INFO ] MetricsCollectorService collecting metrics...
```

**检查前端**：

```bash
# 访问首页
curl http://localhost:4200

# 预期：返回 HTML 内容（Angular 应用）

# 检查日志
tail -f frontend/frontend.log

# 预期日志
✔ Browser application bundle generation complete.
Angular Live Development Server is listening on 0.0.0.0:4200
```

**浏览器访问**：
- 打开浏览器访问 `http://localhost:4200`
- 登录系统
- 导航到 `/pages/starrocks/overview`

---

## 🧪 测试用例

## 一、后端 API 测试

### 1.1 健康卡片 API

**测试接口**：`GET /api/clusters/:id/overview/health`

**测试命令**：
```bash
# 替换 :id 为实际集群ID
curl -X GET "http://localhost:3000/api/clusters/1/overview/health" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**（示例）：
```json
[
  {
    "title": "QPS",
    "value": 1234,
    "status": "success",
    "trend": 5.2,
    "unit": "q/s",
    "icon": "activity-outline",
    "navigateTo": "/pages/starrocks/queries"
  },
  {
    "title": "P99 Latency",
    "value": 45.3,
    "status": "success",
    "trend": -2.1,
    "unit": "ms"
  },
  ...
]
```

**验证点**：
- ✅ 状态码：200
- ✅ 返回数组长度：4-8 个卡片
- ✅ 每个卡片包含必需字段：title, value, status
- ✅ trend 为正数表示上升，负数表示下降

---

### 1.2 性能趋势 API

**测试接口**：`GET /api/clusters/:id/overview/performance?time_range=1h`

**测试命令**：
```bash
curl -X GET "http://localhost:3000/api/clusters/1/overview/performance?time_range=1h" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**：
```json
{
  "qpsSeries": [
    {"timestamp": "2025-01-24T10:00:00Z", "value": 1200},
    {"timestamp": "2025-01-24T10:01:00Z", "value": 1250},
    ...
  ],
  "latencyP99Series": [...],
  "errorRateSeries": [...]
}
```

**验证点**：
- ✅ 状态码：200
- ✅ 包含 3 个时间序列：qpsSeries, latencyP99Series, errorRateSeries
- ✅ 每个序列是数组，包含时间戳和数值
- ✅ 时间戳按升序排列
- ✅ 数据点数量合理（1h 约 60-120 个点）

**测试不同时间范围**：
```bash
# 测试 6h
curl "http://localhost:3000/api/clusters/1/overview/performance?time_range=6h"

# 测试 24h
curl "http://localhost:3000/api/clusters/1/overview/performance?time_range=24h"

# 测试 3d
curl "http://localhost:3000/api/clusters/1/overview/performance?time_range=3d"
```

---

### 1.3 资源趋势 API

**测试接口**：`GET /api/clusters/:id/overview/resources?time_range=1h`

**测试命令**：
```bash
curl -X GET "http://localhost:3000/api/clusters/1/overview/resources?time_range=1h" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**：
```json
{
  "cpuUsageSeries": [...],
  "memoryUsageSeries": [...],
  "diskUsageSeries": [...],
  "networkTxSeries": [...],
  "networkRxSeries": [...],
  "ioReadSeries": [...],
  "ioWriteSeries": [...]
}
```

**验证点**：
- ✅ 状态码：200
- ✅ 包含 7 个时间序列
- ✅ CPU/Memory/Disk 使用率在 0-100 之间
- ✅ 网络/IO 速率为正数

---

### 1.4 数据统计 API

**测试接口**：`GET /api/clusters/:id/overview/data-stats`

**测试命令**：
```bash
curl -X GET "http://localhost:3000/api/clusters/1/overview/data-stats" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**：
```json
{
  "databaseCount": 5,
  "tableCount": 120,
  "totalDataSizeBytes": 1073741824000,
  "topTablesBySize": [
    {
      "database": "db1",
      "table": "large_table",
      "sizeBytes": 50000000000,
      "rowCount": 1000000000
    },
    ...
  ],
  "topTablesByAccess": [...],
  "slowQueries": [...],
  "mvTotal": 10,
  "mvRunning": 2,
  "mvFailed": 0,
  "mvSuccess": 8,
  "schemaChangeRunning": 1,
  "schemaChangePending": 0,
  "schemaChangeFinished": 5,
  "schemaChangeFailed": 0,
  "activeUsers1h": 3,
  "activeUsers24h": 15
}
```

**验证点**：
- ✅ 状态码：200
- ✅ 所有计数字段为非负整数
- ✅ `topTablesBySize` 按大小降序排列
- ✅ `topTablesByAccess` 按访问次数降序排列
- ✅ `slowQueries` 按耗时降序排列

---

### 1.5 容量预测 API

**测试接口**：`GET /api/clusters/:id/overview/capacity-prediction`

**测试命令**：
```bash
curl -X GET "http://localhost:3000/api/clusters/1/overview/capacity-prediction" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**：
```json
{
  "diskTotalBytes": 1099511627776,
  "diskUsedBytes": 549755813888,
  "diskUsagePct": 50.0,
  "dailyGrowthBytes": 10737418240,
  "daysUntilFull": 51,
  "predictedFullDate": "2025-03-16",
  "growthTrend": "stable"
}
```

**验证点**：
- ✅ 状态码：200
- ✅ `diskUsagePct` 在 0-100 之间
- ✅ `growthTrend` 为 "increasing", "stable", 或 "decreasing"
- ✅ 如果 `daysUntilFull` 存在，应为正整数
- ✅ `predictedFullDate` 格式为 YYYY-MM-DD

---

### 1.6 慢查询 API

**测试接口**：`GET /api/clusters/:id/overview/slow-queries?hours=24&min_duration_ms=1000&limit=20`

**测试命令**：
```bash
curl -X GET "http://localhost:3000/api/clusters/1/overview/slow-queries?hours=24&min_duration_ms=1000&limit=20" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**预期响应**：
```json
[
  {
    "queryId": "abc-123-def",
    "user": "admin",
    "database": "test_db",
    "durationMs": 5000,
    "scanRows": 1000000,
    "scanBytes": 50000000,
    "returnRows": 100,
    "cpuCostMs": 3000,
    "memCostBytes": 100000000,
    "timestamp": "2025-01-24T10:00:00Z",
    "state": "EOF",
    "queryPreview": "SELECT * FROM large_table WHERE..."
  },
  ...
]
```

**验证点**：
- ✅ 状态码：200
- ✅ 返回数组长度 ≤ limit
- ✅ 所有查询 `durationMs` ≥ `min_duration_ms`
- ✅ 按 `durationMs` 降序排列

---

## 二、数据采集服务测试

### 2.1 MetricsCollectorService 运行状态

**检查日志**：
```bash
# 启动后端后，观察日志
tail -f backend/logs/app.log  # 如果有日志文件
```

**预期日志（每30秒）**：
```
[2025-01-24T10:00:00Z] Scheduled task 'metrics-collector' running...
[2025-01-24T10:00:01Z] Collecting metrics for cluster 1: test_cluster
[2025-01-24T10:00:02Z] Fetched Prometheus metrics: 150 data points
[2025-01-24T10:00:03Z] Saved metric snapshot for cluster 1
[2025-01-24T10:00:30Z] Scheduled task 'metrics-collector' running...
```

**验证数据库**：
```bash
sqlite3 build/data/starrocks-admin.db "
SELECT 
  cluster_id, 
  COUNT(*) as snapshot_count,
  MAX(collected_at) as latest_collection
FROM metrics_snapshots 
GROUP BY cluster_id;
"
```

**预期**：
- ✅ 每个集群每30秒有新的快照
- ✅ `snapshot_count` 持续增长
- ✅ `latest_collection` 在最近 30 秒内

---

### 2.2 每日聚合任务测试

**触发条件**：每天首次运行时自动聚合前一天数据

**手动测试**：
修改 `metrics_collector_service.rs` 的聚合时间：
```rust
// 临时修改为立即触发（仅测试用）
let yesterday = Local::now().date_naive();
```

**验证数据库**：
```bash
sqlite3 build/data/starrocks-admin.db "
SELECT 
  cluster_id,
  snapshot_date,
  avg_qps,
  max_qps,
  avg_cpu_usage,
  max_cpu_usage
FROM daily_snapshots 
ORDER BY snapshot_date DESC
LIMIT 10;
"
```

**预期**：
- ✅ 每天有一条聚合记录
- ✅ `avg_*` 和 `max_*` 字段有合理值
- ✅ `snapshot_date` 为昨天日期

---

### 2.3 DataStatisticsService 缓存测试

**检查缓存表**：
```bash
sqlite3 build/data/starrocks-admin.db "
SELECT 
  cluster_id,
  database_count,
  table_count,
  total_data_size_bytes,
  updated_at
FROM data_statistics
ORDER BY updated_at DESC;
"
```

**预期**：
- ✅ 每个集群有一条记录
- ✅ `updated_at` 在最近时间内（取决于更新策略）
- ✅ 统计数据与实际集群一致

---

## 三、前端 UI 测试

### 3.1 页面加载测试

**步骤**：
1. 登录系统
2. 选择一个集群（如果有集群选择器）
3. 导航到 `/pages/starrocks/overview`

**验证点**：
- ✅ 页面加载成功，无白屏
- ✅ Loading spinner 显示
- ✅ 页面标题"Cluster Overview"显示
- ✅ 控制区域正常显示（时间选择器、刷新按钮等）

---

### 3.2 健康卡片渲染

**验证点**：
- ✅ 卡片横向排列（4-5列）
- ✅ 每个卡片显示：图标、标题、数值、单位
- ✅ 趋势指示器正确显示（上升↑/下降↓）
- ✅ 状态颜色正确（success=绿色, warning=黄色, danger=红色）
- ✅ 卡片有 hover 效果（轻微上移）
- ✅ 点击可导航卡片能跳转到相应页面

---

### 3.3 图表渲染测试

**性能图表**（左列）：
- ✅ QPS 图表显示，使用蓝色渐变
- ✅ P99 Latency 图表显示，使用橙色渐变
- ✅ Error Rate 图表显示，使用红色渐变
- ✅ 所有图表使用平滑曲线
- ✅ 鼠标悬停显示详细数据
- ✅ 时间轴标签清晰可读

**资源图表**（左列）：
- ✅ CPU 使用率图表（绿色渐变）
- ✅ Memory 使用率图表（紫色渐变）
- ✅ Disk 使用率图表（黄色渐变）
- ✅ Network 流量图表（双线：TX + RX）
- ✅ Disk I/O 图表（双线：Read + Write）
- ✅ Y轴范围合理（使用率 0-100%）

---

### 3.4 统计信息展示

**数据统计卡片**（右列）：
- ✅ 数据库数量显示
- ✅ 表数量显示
- ✅ 总数据大小正确格式化（GB/TB）
- ✅ 活跃用户数（1h/24h）显示

**物化视图卡片**：
- ✅ 四宫格布局（Total/Running/Success/Failed）
- ✅ 数字清晰可读
- ✅ 颜色编码（Running=黄色, Success=绿色, Failed=红色）
- ✅ 点击卡片可跳转到物化视图页面

**Schema变更卡片**：
- ✅ 四宫格布局（Running/Pending/Finished/Failed）
- ✅ 状态颜色正确

**容量预测卡片**：
- ✅ 进度条正确显示使用百分比
- ✅ 进度条颜色根据使用率变化（>90%=红色, >70%=黄色, 否则绿色）
- ✅ 已用/总容量显示
- ✅ 每日增长量显示
- ✅ 预计满盘时间显示（如果有）
- ✅ 增长趋势标签（increasing/stable/decreasing）

**Top表（按大小）**：
- ✅ 表格显示 Database.Table, Size, Rows
- ✅ 按大小降序排列
- ✅ 大小单位正确（KB/MB/GB/TB）
- ✅ 最多显示 10 条

**Top表（按访问量）**：
- ✅ 表格显示 Database.Table, Accesses, Users
- ✅ 按访问次数降序排列
- ✅ 访问次数格式化（1K, 10K, 1M等）
- ✅ 点击卡片跳转到查询页面

**慢查询列表**：
- ✅ 表格显示 User, Database, Duration, Query
- ✅ 按耗时降序排列
- ✅ 耗时格式化（ms/s/m/h）
- ✅ Query 预览文本截断
- ✅ 鼠标悬停显示完整 Query（title属性）
- ✅ 点击卡片跳转到查询页面

---

### 3.5 交互功能测试

**时间范围切换**：
1. 点击时间选择器
2. 选择不同时间范围（1h → 6h → 24h → 3d）
3. 验证：
   - ✅ 图表重新加载
   - ✅ X轴时间跨度改变
   - ✅ 数据点数量变化合理

**刷新间隔切换**：
1. 点击刷新间隔选择器
2. 选择不同间隔（15s → 30s → 1m → Manual）
3. 验证：
   - ✅ 自动刷新按选定间隔执行
   - ✅ 选择 "Manual" 时，自动刷新停止
   - ✅ "Auto" toggle 自动更新状态

**手动刷新**：
1. 点击"Refresh"按钮
2. 验证：
   - ✅ Loading spinner 显示
   - ✅ 所有数据重新加载
   - ✅ 加载完成后 spinner 消失

**自动刷新 Toggle**：
1. 切换"Auto" toggle 开关
2. 验证：
   - ✅ 开启时自动刷新启用
   - ✅ 关闭时自动刷新停止
   - ✅ 状态与刷新间隔选择器联动

---

### 3.6 响应式布局测试

**桌面端（>1200px）**：
- ✅ 健康卡片：5列布局
- ✅ 主内容：2列布局（左列图表，右列统计）

**平板端（768px-1200px）**：
- ✅ 健康卡片：4列布局
- ✅ 主内容：单列布局（图表和统计垂直排列）

**移动端（<768px）**：
- ✅ 健康卡片：2列布局
- ✅ 主内容：单列布局
- ✅ 控制区域自适应
- ✅ 图表高度自适应

---

### 3.7 主题兼容性测试

**测试步骤**：
1. 切换到暗色主题（如果 ngx-admin 支持）
2. 导航到 Cluster Overview 页面

**验证点**：
- ✅ 卡片背景色适配暗色主题
- ✅ 文字颜色可读性良好
- ✅ 图表颜色在暗色背景下清晰
- ✅ 边框和分隔线颜色适配
- ✅ 无白色"闪光"元素

---

## 四、集成测试

### 4.1 完整数据流测试

**场景**：从数据采集到前端展示的完整流程

**步骤**：
1. 启动后端服务（MetricsCollectorService 自动运行）
2. 等待 30 秒（一个采集周期）
3. 打开前端页面
4. 验证数据显示

**验证点**：
- ✅ 健康卡片数据来自最新快照
- ✅ 图表数据点对应采集时间
- ✅ 统计信息反映当前集群状态
- ✅ 趋势计算正确（与上一周期对比）

---

### 4.2 多集群切换测试

**步骤**：
1. 添加多个 StarRocks 集群
2. 在集群选择器中切换集群
3. 观察 Cluster Overview 页面变化

**验证点**：
- ✅ 切换后立即加载新集群数据
- ✅ 所有指标更新为新集群数据
- ✅ 图表重新渲染
- ✅ 无数据混淆（旧集群数据）

---

### 4.3 错误处理测试

**场景1：集群不可用**
1. 停止 StarRocks FE 服务
2. 刷新 Cluster Overview 页面
3. 验证：
   - ✅ 显示友好错误提示
   - ✅ 不显示过期数据
   - ✅ 不崩溃或白屏

**场景2：API 超时**
1. 模拟网络延迟（修改后端代码加 sleep）
2. 刷新页面
3. 验证：
   - ✅ Loading 状态持续显示
   - ✅ 超时后显示错误提示
   - ✅ 可重试

**场景3：无历史数据**
1. 清空 `metrics_snapshots` 表
2. 刷新页面
3. 验证：
   - ✅ 显示"No historical data available"
   - ✅ 部分实时数据仍可显示（如 DB/Table 数量）
   - ✅ 图表显示空状态提示

**场景4：审计日志未启用**
1. StarRocks 未启用审计日志
2. 访问 Cluster Overview
3. 验证：
   - ✅ "Top表按访问量" 显示"No data available"
   - ✅ "慢查询列表" 显示"No slow queries found"
   - ✅ 其他功能正常

---

## 五、性能测试

### 5.1 API 响应时间

**测试工具**：`curl` + `time` 或 `wrk`

**测试命令**：
```bash
# 单次请求
time curl "http://localhost:3000/api/clusters/1/overview/health"

# 并发请求（使用 wrk）
wrk -t4 -c100 -d30s http://localhost:3000/api/clusters/1/overview/health
```

**性能目标**：
- ✅ 单次请求响应时间 < 500ms
- ✅ 并发 100 请求时，P95 响应时间 < 1s
- ✅ 无请求失败（5xx错误）

---

### 5.2 前端渲染性能

**测试工具**：Chrome DevTools → Performance

**测试步骤**：
1. 打开 Chrome DevTools
2. 切换到 Performance 标签
3. 点击"Record"
4. 导航到 Cluster Overview 页面
5. 停止录制

**性能目标**：
- ✅ First Contentful Paint (FCP) < 1.5s
- ✅ Time to Interactive (TTI) < 3s
- ✅ ECharts 渲染时间 < 500ms per chart
- ✅ 无长任务（Long Tasks > 50ms）

---

### 5.3 数据采集性能

**监控指标**：
```bash
# 观察 MetricsCollectorService 执行时间
# 后端日志应显示每次采集耗时
```

**性能目标**：
- ✅ 单集群采集时间 < 5s
- ✅ 数据库写入时间 < 100ms
- ✅ 不影响主服务性能（CPU/内存占用稳定）

---

### 5.4 内存占用测试

**测试步骤**：
1. 启动后端服务
2. 运行 24 小时
3. 观察内存占用

**性能目标**：
- ✅ 后端内存占用 < 500MB
- ✅ 无内存泄漏（内存占用稳定或缓慢增长）
- ✅ SQLite 数据库大小合理（< 1GB/天）

---

## 六、数据准确性验证

### 6.1 QPS 计算验证

**验证步骤**：
1. 手动查询 StarRocks Prometheus metrics：
   ```bash
   curl http://fe-host:8030/metrics | grep query_total
   ```
2. 计算 QPS：`(current_total - previous_total) / time_interval`
3. 对比 Cluster Overview 显示的 QPS

**预期**：
- ✅ 误差 < 5%

---

### 6.2 资源使用率验证

**验证步骤**：
1. 手动查询 BE 节点资源：
   ```bash
   curl http://be-host:8040/api/show_proc?path=/backends
   ```
2. 对比 Cluster Overview 显示的 CPU/Memory/Disk 使用率

**预期**：
- ✅ 误差 < 2%

---

### 6.3 容量预测准确性

**验证步骤**：
1. 记录 3 天的实际数据增长
2. 对比容量预测的增长趋势
3. 验证预测日期合理性

**预期**：
- ✅ 增长趋势判断正确（增长/稳定/下降）
- ✅ 预测满盘日期误差 < 10 天（假设线性增长）

---

## 七、回归测试

### 7.1 已有功能兼容性

**验证点**：
- ✅ Backends 页面功能正常
- ✅ Frontends 页面功能正常
- ✅ Materialized Views 页面功能正常
- ✅ Queries 页面功能正常
- ✅ Sessions 页面功能正常
- ✅ Variables 页面功能正常
- ✅ System Management 页面功能正常

---

### 7.2 旧 Monitor 页面

**说明**：保留了 `/pages/starrocks/monitor` 路由用于向后兼容

**验证点**：
- ✅ 旧 URL 仍可访问
- ✅ 功能正常（或显示迁移提示）

---

## 八、故障排查指南

### 8.1 后端无法启动

**症状**：后端服务启动失败

**排查步骤**：

1. **使用脚本启动时**：
   ```bash
   # 查看后端状态
   bash scripts/dev/start_backend.sh status
   
   # 查看日志
   bash scripts/dev/start_backend.sh logs
   
   # 或直接查看日志文件
   tail -f backend/logs/starrocks-admin.log
   ```

2. **检查配置文件**：
   ```bash
   cat backend/conf/config.toml
   # 确认配置正确（脚本会自动生成）
   ```

3. **检查数据库**：
   ```bash
   ls -lh /tmp/starrocks-admin.db
   # 脚本使用 /tmp 目录（开发环境）
   
   # 检查迁移
   cd backend
   DATABASE_URL="sqlite:///tmp/starrocks-admin.db" cargo sqlx migrate info
   ```

4. **手动测试编译**：
   ```bash
   cd backend
   cargo build --release
   ```

**常见问题**：
- ❌ `DATABASE_URL` 配置错误（检查 `config.toml`）
- ❌ SQLite 数据库文件权限不足
- ❌ 迁移文件缺失或损坏
- ❌ 端口 8081 被占用：
  ```bash
  # 查看端口占用
  lsof -i :8081
  
  # 停止占用进程
  bash scripts/dev/start_backend.sh stop
  ```
- ❌ Rust 编译失败：检查 Rust 版本
  ```bash
  rustc --version  # 应该 >= 1.85
  ```

---

### 8.2 MetricsCollectorService 不运行

**症状**：`metrics_snapshots` 表无新数据

**排查步骤**：
1. 检查后端日志：
   ```bash
   grep "metrics-collector" backend/logs/*.log
   ```
2. 检查集群配置：
   ```bash
   sqlite3 build/data/starrocks-admin.db "SELECT * FROM clusters;"
   ```
3. 手动测试集群连接：
   ```bash
   curl http://fe-host:8030/api/show_proc?path=/frontends
   ```

**常见问题**：
- ❌ StarRocks FE 无法访问
- ❌ ScheduledExecutor 初始化失败
- ❌ 数据库写入权限不足

---

### 8.3 前端图表不显示

**症状**：图表区域空白或显示错误

**排查步骤**：
1. 打开浏览器 Console，查看错误日志
2. 检查 Network 标签，查看 API 请求状态
3. 验证 API 响应数据格式

**常见问题**：
- ❌ API 返回 500 错误
- ❌ 数据格式不匹配（前后端类型不一致）
- ❌ ECharts 未正确导入
- ❌ 时间序列数据为空

---

### 8.4 数据不准确

**症状**：显示的指标与实际不符

**排查步骤**：
1. 检查数据采集日志
2. 查询数据库原始数据：
   ```bash
   sqlite3 build/data/starrocks-admin.db "
   SELECT * FROM metrics_snapshots 
   ORDER BY collected_at DESC 
   LIMIT 10;
   "
   ```
3. 对比 StarRocks 原始 metrics

**常见问题**：
- ❌ Prometheus metrics 解析错误
- ❌ 聚合计算逻辑错误
- ❌ 时间戳时区问题

---

## 九、测试完成清单

完成所有测试后，确认以下清单：

### 后端 API
- [ ] 所有 6 个 API 端点正常响应
- [ ] 数据格式符合前端预期
- [ ] 错误处理完善（返回友好错误信息）
- [ ] 性能达标（响应时间 < 500ms）

### 数据采集
- [ ] MetricsCollectorService 定时运行（30秒间隔）
- [ ] 数据正确写入 `metrics_snapshots` 表
- [ ] 每日聚合任务正常执行
- [ ] 旧数据自动清理（保留策略生效）

### 前端 UI
- [ ] 所有卡片/图表正确渲染
- [ ] 交互功能正常（时间切换、刷新等）
- [ ] 响应式布局在各尺寸屏幕下正常
- [ ] 暗色主题兼容
- [ ] 导航跳转正确

### 集成测试
- [ ] 数据从采集到展示全流程畅通
- [ ] 多集群切换正常
- [ ] 错误情况处理得当

### 性能测试
- [ ] API 响应时间达标
- [ ] 前端渲染性能达标
- [ ] 内存占用合理

### 数据准确性
- [ ] QPS 计算准确
- [ ] 资源使用率准确
- [ ] 容量预测合理

### 回归测试
- [ ] 已有功能未受影响
- [ ] 旧 Monitor 页面兼容

---

## 十、测试报告模板

测试完成后，填写以下报告：

```markdown
# Cluster Overview 测试报告

## 测试信息
- **测试人员**：[姓名]
- **测试日期**：2025-01-24
- **测试环境**：
  - 后端版本：[commit hash]
  - 前端版本：[commit hash]
  - StarRocks 版本：[版本号]
  - 操作系统：macOS 21.6.0

## 测试结果总览
- **通过用例**：X / Y
- **失败用例**：Z
- **阻塞问题**：N

## 详细测试结果

### 后端 API 测试
| API 端点 | 状态 | 响应时间 | 备注 |
|---------|------|---------|------|
| /overview/health | ✅ | 120ms | |
| /overview/performance | ✅ | 350ms | |
| /overview/resources | ✅ | 280ms | |
| /overview/data-stats | ⚠️ | 1.2s | 较慢，需优化 |
| /overview/capacity-prediction | ✅ | 90ms | |
| /overview/slow-queries | ✅ | 450ms | |

### 前端 UI 测试
| 功能模块 | 状态 | 问题描述 |
|---------|------|---------|
| 健康卡片 | ✅ | 无 |
| 性能图表 | ✅ | 无 |
| 资源图表 | ✅ | 无 |
| 统计信息 | ✅ | 无 |
| 交互功能 | ✅ | 无 |
| 响应式布局 | ⚠️ | 移动端图表重叠 |

### 已知问题
1. **[P1] 数据统计 API 响应慢**
   - 描述：首次加载耗时 > 1s
   - 原因：未使用缓存
   - 解决方案：DataStatisticsService 缓存

2. **[P2] 移动端图表显示问题**
   - 描述：小屏幕下图表重叠
   - 原因：响应式 CSS 未完全适配
   - 解决方案：调整图表高度

## 测试结论
✅ **功能完整，可以发布**

## 后续优化建议
1. 优化数据统计 API 性能
2. 完善移动端响应式布局
3. 添加更多错误提示
```

---

## 附录：自动化测试脚本

### A.1 后端 API 自动化测试

创建 `backend/tests/overview_api_test.sh`：

```bash
#!/bin/bash

BASE_URL="http://localhost:3000"
CLUSTER_ID=1
TOKEN="YOUR_TOKEN"

echo "=== Cluster Overview API Tests ==="

# 1. Health Cards
echo -n "Testing /overview/health... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/health")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

# 2. Performance Trends
echo -n "Testing /overview/performance... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/performance?time_range=1h")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

# 3. Resource Trends
echo -n "Testing /overview/resources... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/resources?time_range=1h")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

# 4. Data Statistics
echo -n "Testing /overview/data-stats... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/data-stats")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

# 5. Capacity Prediction
echo -n "Testing /overview/capacity-prediction... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/capacity-prediction")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

# 6. Slow Queries
echo -n "Testing /overview/slow-queries... "
RESPONSE=$(curl -s -w "\n%{http_code}" -H "Authorization: Bearer $TOKEN" \
  "$BASE_URL/api/clusters/$CLUSTER_ID/overview/slow-queries?hours=24&min_duration_ms=1000&limit=20")
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" == "200" ]; then
  echo "✅ PASS"
else
  echo "❌ FAIL (HTTP $HTTP_CODE)"
fi

echo "=== Tests Complete ==="
```

运行：
```bash
chmod +x backend/tests/overview_api_test.sh
./backend/tests/overview_api_test.sh
```

---

## 测试检查表

复制以下清单，逐项测试并勾选：

```
## Phase 1: 环境准备
- [ ] 数据库迁移完成
- [ ] StarRocks 集群可用
- [ ] 后端服务启动成功
- [ ] 前端服务启动成功

## Phase 2: 后端 API
- [ ] GET /overview/health
- [ ] GET /overview/performance
- [ ] GET /overview/resources
- [ ] GET /overview/data-stats
- [ ] GET /overview/capacity-prediction
- [ ] GET /overview/slow-queries

## Phase 3: 数据采集
- [ ] MetricsCollectorService 运行
- [ ] 数据写入 metrics_snapshots
- [ ] 每日聚合任务正常

## Phase 4: 前端 UI
- [ ] 页面加载
- [ ] 健康卡片渲染
- [ ] 性能图表渲染
- [ ] 资源图表渲染
- [ ] 统计信息显示
- [ ] 交互功能正常

## Phase 5: 集成测试
- [ ] 完整数据流
- [ ] 多集群切换
- [ ] 错误处理

## Phase 6: 性能测试
- [ ] API 响应时间
- [ ] 前端渲染性能
- [ ] 内存占用

## Phase 7: 数据准确性
- [ ] QPS 计算
- [ ] 资源使用率
- [ ] 容量预测

## Phase 8: 回归测试
- [ ] 已有功能正常
```

---

## 结语

按照本测试方案逐步执行，可以全面验证 Cluster Overview 功能的完整性和正确性。

**测试优先级**：
1. 🔴 P0：环境准备 + 后端 API（必须通过）
2. 🟡 P1：前端 UI + 集成测试（核心功能）
3. 🟢 P2：性能测试 + 数据准确性（优化目标）
4. 🔵 P3：回归测试（质量保证）

---

## 快速开始测试

### 完整测试流程（从零开始）

```bash
# ============================================
# Step 1: 启动 StarRocks 测试集群
# ============================================
docker run -p 9030:9030 -p 8030:8030 -p 8040:8040 -itd \
  --name starrocks-test \
  starrocks/allin1-ubuntu

# 等待 StarRocks 启动
sleep 30

# 验证 StarRocks
curl http://localhost:8030/api/health
docker exec -it starrocks-test mysql -P 9030 -h 127.0.0.1 -u root -e "SHOW BACKENDS\G"

# ============================================
# Step 2: 创建测试数据和配置审计日志
# ============================================
# 连接到 StarRocks
docker exec -it starrocks-test mysql -P 9030 -h 127.0.0.1 -u root

# 执行以下 SQL（复制粘贴）
CREATE DATABASE IF NOT EXISTS test_db;
USE test_db;

-- 创建表并插入数据（参考上面的"创建测试数据库和表"章节）
-- ...

-- 启用审计日志（参考上面的"配置审计日志"章节）
INSTALL PLUGIN auditloader SONAME "auditloader.so";
SHOW PLUGINS\G

-- 生成一些查询以产生审计日志
SELECT * FROM test_db.users;
SELECT * FROM test_db.orders;
-- ...

# 退出 MySQL
exit

# ============================================
# Step 3: 启动 StarRocks Admin
# ============================================
# 一键启动前后端
bash scripts/dev/start.sh

# 等待服务启动（约10秒）
sleep 10

# ============================================
# Step 4: 添加测试集群到 StarRocks Admin
# ============================================
# 登录并获取 Token
TOKEN=$(curl -s -X POST http://localhost:8081/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.token')

# 添加 Docker 集群
curl -X POST http://localhost:8081/api/clusters \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "test_cluster",
    "description": "Docker 测试集群",
    "fe_host": "host.docker.internal",
    "fe_http_port": 8030,
    "fe_query_port": 9030,
    "username": "root",
    "password": "",
    "catalog": "default_catalog"
  }'

# ============================================
# Step 5: 等待数据采集（至少5分钟）
# ============================================
echo "等待 MetricsCollectorService 采集数据..."
echo "可以查看日志: tail -f backend/logs/starrocks-admin.log"
sleep 300  # 等待5分钟

# 验证数据采集
sqlite3 /tmp/starrocks-admin.db "
SELECT COUNT(*) FROM metrics_snapshots;
"

# ============================================
# Step 6: 访问前端测试
# ============================================
# 打开浏览器
open http://localhost:4200

# 登录：admin / admin
# 选择集群：test_cluster
# 导航到：/pages/starrocks/overview

# ============================================
# Step 7: 执行自动化API测试（可选）
# ============================================
bash backend/tests/overview_api_test.sh

# ============================================
# Step 8: 停止所有服务
# ============================================
# 停止 StarRocks Admin
bash scripts/dev/start_backend.sh stop
bash scripts/dev/start_frontend.sh stop

# 停止 StarRocks 集群
docker stop starrocks-test
docker rm starrocks-test
```

### 最简测试流程（假设环境已准备）

```bash
# 1. 启动 StarRocks
docker start starrocks-test || docker run -p 9030:9030 -p 8030:8030 -p 8040:8040 -itd \
  --name starrocks-test starrocks/allin1-ubuntu

# 2. 启动 StarRocks Admin
bash scripts/dev/start.sh

# 3. 等待3分钟数据采集

# 4. 访问前端
open http://localhost:4200
# 登录 -> 选择集群 -> /pages/starrocks/overview
```

祝测试顺利！🎉

