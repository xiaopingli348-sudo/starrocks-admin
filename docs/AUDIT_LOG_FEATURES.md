# 审计日志相关功能说明

> **文档类型**: 📘 功能说明  
> **创建日期**: 2025-10-24  
> **状态**: 📝 待实现（依赖外部配置）

---

## 一、概述

以下功能依赖 StarRocks 的审计日志（Audit Log）功能。审计日志需要在 StarRocks FE 配置中启用。

**依赖功能**：
1. **Top 20 表（按访问量排序）** - 统计最频繁访问的表
2. **慢查询列表** - 识别性能问题查询

---

## 二、StarRocks 审计日志配置

### 2.1 启用审计日志

在 StarRocks FE 配置文件 `fe.conf` 中添加：

```properties
# Enable audit log
audit_log_enable = true

# Audit log directory
audit_log_dir = ${STARROCKS_HOME}/log

# Audit log file prefix
audit_log_file_pattern = audit_log_

# Keep audit logs for 7 days
audit_log_delete_age = 7d

# Rotate audit log every day
audit_log_roll_interval = DAY

# Max audit log file size (1GB)
audit_log_roll_size_mb = 1024
```

### 2.2 重启 FE

```bash
sh bin/stop_fe.sh
sh bin/start_fe.sh
```

### 2.3 验证

审计日志文件位于：`$STARROCKS_HOME/log/audit_log_*.log`

---

## 三、审计日志格式

StarRocks 审计日志是 JSON 格式，包含以下关键字段：

```json
{
  "timestamp": "2025-10-24 10:30:15",
  "queryId": "12345678-1234-1234-1234-123456789abc",
  "user": "admin",
  "clientIp": "192.168.1.100",
  "db": "test_db",
  "state": "EOF",
  "time": 125,  // Query time in ms
  "scanBytes": 1024000,
  "scanRows": 10000,
  "returnRows": 100,
  "cpuCostMs": 50,
  "memCostBytes": 1048576,
  "stmt": "SELECT * FROM users WHERE age > 18",
  "tables": ["test_db.users"]
}
```

**关键字段说明**：
- `time`: 查询执行时间（毫秒）
- `tables`: 涉及的表列表
- `stmt`: SQL 语句
- `user`: 执行用户
- `state`: 查询状态（EOF=成功）

---

## 四、实现方案

### 4.1 Top 20 表按访问量

#### 方案 A：使用审计日志（推荐）

```sql
-- 从审计日志表统计（如果导入到 StarRocks）
SELECT 
    table_name,
    COUNT(*) as access_count,
    MAX(timestamp) as last_access
FROM audit_log_table
WHERE timestamp >= NOW() - INTERVAL 24 HOUR
GROUP BY table_name
ORDER BY access_count DESC
LIMIT 20;
```

#### 方案 B：使用 SHOW PROFILELIST（替代方案）

```sql
-- 获取最近的查询 Profile
SHOW PROFILELIST;

-- 解析 Profile 中的表信息
-- 缺点：只保留最近 1000 条，数据不够全面
```

### 4.2 慢查询列表

#### 方案 A：使用审计日志（推荐）

```sql
-- 从审计日志表统计慢查询
SELECT 
    queryId,
    user,
    db,
    time as duration_ms,
    timestamp,
    LEFT(stmt, 200) as query_preview
FROM audit_log_table
WHERE timestamp >= NOW() - INTERVAL 24 HOUR
    AND time > 1000  -- 超过 1 秒的查询
    AND state = 'EOF'
ORDER BY time DESC
LIMIT 20;
```

#### 方案 B：使用 SHOW PROFILELIST（替代方案）

```sql
SHOW PROFILELIST;

-- 提取 StartTime > 1000ms 的查询
```

---

## 五、实施计划

### 5.1 Phase 1: 审计日志导入（可选）

**目标**：将审计日志文件导入到 StarRocks 表

```sql
-- 创建审计日志表
CREATE TABLE audit_logs (
    timestamp DATETIME,
    queryId VARCHAR(64),
    user VARCHAR(64),
    clientIp VARCHAR(32),
    db VARCHAR(128),
    state VARCHAR(32),
    time BIGINT,  -- ms
    scanBytes BIGINT,
    scanRows BIGINT,
    returnRows BIGINT,
    cpuCostMs BIGINT,
    memCostBytes BIGINT,
    stmt STRING,
    tables ARRAY<VARCHAR(256)>
) DUPLICATE KEY (timestamp)
PARTITION BY RANGE(timestamp) (
    PARTITION p1 VALUES [("2025-01-01"), ("2025-02-01")),
    PARTITION p2 VALUES [("2025-02-01"), ("2025-03-01"))
)
DISTRIBUTED BY HASH(queryId) BUCKETS 10;

-- 使用 Routine Load 或 Broker Load 导入审计日志
```

**工作量**：约 4-6 小时

### 5.2 Phase 2: 实现 Top 表和慢查询 API

**API 端点**：

```rust
// Top tables by access count
GET /api/clusters/:id/overview/top-tables-by-access?hours=24&limit=20

// Slow queries
GET /api/clusters/:id/overview/slow-queries?hours=24&limit=20&min_duration=1000
```

**工作量**：约 3-4 小时

### 5.3 Phase 3: 前端展示

- Top 表（按访问）- ECharts 柱状图
- 慢查询列表 - Nebular Table

**工作量**：约 2-3 小时

---

## 六、当前状态

### ✅ 已实现
- Top 20 表（按大小） - 使用 `information_schema.tables`
- 数据统计缓存机制

### ⚠️ 待实现（需审计日志）
- Top 20 表（按访问量）
- 慢查询列表

### 占位实现
- `DataStatisticsService::get_top_tables_by_access()` - 返回空列表
- 注释说明：需要审计日志或 PROFILELIST

---

## 七、替代方案（无审计日志）

如果无法启用审计日志，可以考虑：

### 方案 1：使用 SHOW PROFILELIST

```sql
SHOW PROFILELIST;
```

**优点**：
- 无需额外配置
- 可以获取最近查询信息

**缺点**：
- 只保留最近 ~1000 条
- 无法获取历史数据
- 数据不完整

### 方案 2：自定义查询拦截器

在 StarRocks Admin 中拦截所有查询，记录到 SQLite：

```rust
// Query interceptor
pub async fn intercept_query(
    cluster_id: i64,
    user: &str,
    query: &str,
    tables: Vec<String>,
    duration_ms: i64,
) {
    // Record to SQLite query_history table
}
```

**优点**：
- 完全自主控制
- 可以记录任意信息

**缺点**：
- 需要所有查询都通过 StarRocks Admin
- 直连数据库的查询无法记录
- 增加系统复杂度

---

## 八、建议

**推荐方案**：

1. **生产环境**：启用 StarRocks 审计日志
   - 审计日志是生产环境的最佳实践
   - 可用于安全审计、性能分析、问题排查

2. **开发/测试环境**：使用 PROFILELIST 替代方案
   - 快速验证功能
   - 无需额外配置

3. **功能优先级**：
   - P0: Top 表（按大小）✅ 已完成
   - P1: 慢查询（PROFILELIST 方案）⚠️ 可选
   - P2: Top 表（按访问，审计日志）⚠️ 可选

---

**当前实施建议**：

由于审计日志需要用户自行配置 StarRocks，建议：
1. 在文档中说明配置方法
2. 代码中保留占位实现
3. 前端显示"需要启用审计日志"提示
4. 用户启用后自动生效

**预估总工作量**（如启用审计日志）：约 10-13 小时

---

**文档状态**: 完成  
**功能状态**: 占位实现（等待审计日志配置）

