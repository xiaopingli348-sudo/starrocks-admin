# Compaction 功能实现文档

## 概述

根据 [StarRocks 存算分离 Compaction 原理文档](https://forum.mirrorship.cn/t/topic/13256)，我们实现了符合存算分离架构的 Compaction 监控功能。

## 核心特点

### 1. 架构理解

在 StarRocks 存算分离模式下：
- **FE 作为 Compaction Scheduler**：负责调度发起 Compaction 任务
- **BE/CN 作为 Compaction Executor**：负责执行 Compaction 任务
- **Partition 为调度单位**：FE 以 Partition 为单位发起 Compaction，每个 Partition 有自己的 Compaction Score
- **统一的 Compaction**：不区分 base compaction 和 cumulative compaction

### 2. Compaction Score

**Compaction Score 的意义：**
- 表示 Partition 内所有 Tablet 需要进行 Compaction 的优先级
- Score 越高，表示 Partition 需要合并的紧急程度越高
- FE 会挑选 Score 最高的 Partition 优先进行 Compaction

**Compaction Score 的来源：**
- FE 掌握每个 Partition 的 Compaction Score
- 通过 FE 的 `/metrics` 端点暴露：`starrocks_fe_max_tablet_compaction_score`

## 实现细节

### 1. 数据收集（MetricsCollectorService）

```rust
// backend/src/services/metrics_collector_service.rs

// 从 FE 的 Prometheus metrics 端点获取 Compaction Score
max_compaction_score: metrics_map
    .get("starrocks_fe_max_tablet_compaction_score")
    .copied()
    .unwrap_or(0.0),
```

**采集频率：** 30秒（可配置）

**存储位置：** SQLite `metrics_snapshots` 表

### 2. 数据查询（OverviewService）

```rust
// backend/src/services/overview_service.rs

async fn get_compaction_stats(&self, cluster_id: i64) -> ApiResult<CompactionStats> {
    // 1. 查询当前运行的 Compaction 任务
    let query = "SHOW PROC '/compactions'";
    let (_headers, rows) = client.query_raw(query).await.unwrap_or((vec![], vec![]));
    let total_running = rows.len() as i32;
    
    // 2. 从历史快照获取 max_compaction_score
    let latest_snapshot = self.get_latest_snapshot(cluster_id).await?;
    let max_score = latest_snapshot
        .as_ref()
        .map(|s| s.max_compaction_score)
        .unwrap_or(0.0);
    
    Ok(CompactionStats {
        base_compaction_running: 0,                    // 存算分离不适用
        cumulative_compaction_running: total_running,  // 总的运行任务数
        max_score,                                     // Partition级别的最大Score
        avg_score: max_score,                          // 与max_score相同
        be_scores: Vec::new(),                         // 存算分离不适用
    })
}
```

**关键SQL命令：**
- `SHOW PROC '/compactions'`：查看当前运行的 Compaction 任务
- 可选：`SHOW PROC '/compactions/{txn_id}'`：查看特定任务的详细进度

### 3. 前端接口定义

```typescript
// frontend/src/app/@core/data/overview.service.ts

// Compaction Stats for Storage-Compute Separation Architecture
export interface CompactionStats {
  baseCompactionRunning: number;           // Always 0 in shared-data mode
  cumulativeCompactionRunning: number;     // Total compaction tasks running
  maxScore: number;                        // Max compaction score across all partitions (from FE)
  avgScore: number;                        // Same as maxScore in shared-data mode
  beScores: BECompactionScore[];           // Empty in shared-data mode
}
```

## 数据库Schema

```sql
-- backend/migrations/20250125000000_unified_database_schema.sql

CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    collected_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- ... other metrics ...
    
    -- Storage Metrics
    tablet_count BIGINT NOT NULL DEFAULT 0,
    max_compaction_score REAL NOT NULL DEFAULT 0.0,  -- Partition级别的最大Compaction Score
    
    -- ... other metrics ...
);

CREATE INDEX idx_metrics_snapshots_cluster_time 
    ON metrics_snapshots(cluster_id, collected_at DESC);
```

## API端点

### 1. 获取 Compaction 统计（作为 Extended Overview 的一部分）

```http
GET /api/clusters/{id}/overview/extended?time_range=24h
```

**响应示例：**
```json
{
  "clusterId": 1,
  "clusterName": "prod-cluster",
  "timestamp": "2025-01-25T10:30:00Z",
  "compaction": {
    "baseCompactionRunning": 0,
    "cumulativeCompactionRunning": 5,
    "maxScore": 87.3,
    "avgScore": 87.3,
    "beScores": []
  }
}
```

## 监控建议

根据 StarRocks 官方文档，建议：

### 1. Compaction Score 告警

```yaml
# Prometheus 告警规则示例
- alert: HighCompactionScore
  expr: starrocks_fe_max_tablet_compaction_score > 100
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High compaction score detected"
    description: "Compaction score is {{ $value }}, exceeding threshold 100"

- alert: CriticalCompactionScore
  expr: starrocks_fe_max_tablet_compaction_score > 500
  for: 10m
  labels:
    severity: critical
  annotations:
    summary: "Critical compaction score detected"
    description: "Compaction score is {{ $value }}, immediate action required"
```

### 2. FE 参数调优

```sql
-- 查看当前配置
ADMIN SHOW FRONTEND CONFIG LIKE '%compaction%';

-- 调整最小 Compaction Score（低于该值不会发起任务）
ADMIN SET FRONTEND CONFIG ("lake_compaction_score_selector_min_score" = "10.0");

-- 调整最大并发任务数（-1为自动，0为禁用）
ADMIN SET FRONTEND CONFIG ("lake_compaction_max_tasks" = "-1");

-- 查看历史任务数量
ADMIN SET FRONTEND CONFIG ("lake_compaction_history_size" = "12");
```

### 3. BE/CN 参数调优

```sql
-- 查看BE配置
SELECT * FROM information_schema.be_configs WHERE name LIKE '%compact%';

-- 调整并发线程数（默认4）
UPDATE information_schema.be_configs 
SET value = '8' 
WHERE name = 'compact_threads';

-- 调整单次合并文件数（建议100，默认1000）
UPDATE information_schema.be_configs 
SET value = '100' 
WHERE name = 'max_cumulative_compaction_num_singleton_deltas';
```

## 查看 Compaction 状态

### 1. 查看所有 Compaction 任务

```sql
-- 查看当前所有 Compaction 任务
SHOW PROC '/compactions';

-- 输出字段：TXN_ID, DB_ID, TABLE_ID, PARTITION, START_TIME, COMMIT_TIME, VISIBLE_TIME, FINISH_TIME, STATUS
```

### 2. 查看特定任务的详细进度

```sql
-- 查看 TXN_ID=197562 的任务详情
SELECT * FROM information_schema.be_cloud_native_compactions WHERE TXN_ID = 197562;

-- 输出字段：
-- - BE_ID: Backend ID
-- - TXN_ID: Transaction ID
-- - TABLET_ID: Tablet ID
-- - VERSION: 版本号
-- - PROGRESS: 进度百分比
-- - STATUS: 状态（OK, FAILED等）
-- - START_TIME: 开始时间
-- - FINISH_TIME: 结束时间
```

### 3. 取消 Compaction 任务

```sql
-- 连接 Leader FE 执行
CANCEL COMPACTION WHERE TXN_ID = 123;
```

## 最佳实践

1. **关注 Compaction Score**
   - 正常范围：< 100
   - 警告范围：100-500
   - 危险范围：> 500

2. **调整线程数**
   - 在资源空闲时增加 `compact_threads`
   - 在资源紧张时减少以释放CPU和内存

3. **优化单次合并文件数**
   - 建议设置为 100（默认 1000 过大）
   - 更小的值可以：
     - 更快完成单个任务
     - 消耗更少资源
     - 更均匀的资源使用

4. **监控资源消耗**
   - Compaction 会消耗大量 CPU 和内存
   - 建议在 Grafana 中监控：
     - `starrocks_be_compaction_mem_bytes`
     - `starrocks_be_compaction_cpu_cost_ns`

## 与存算一体的区别

| 特性 | 存算一体 | 存算分离 |
|-----|---------|---------|
| 调度者 | BE 自行调度 | FE 统一调度 |
| 调度单位 | Tablet | Partition |
| Compaction类型 | Base + Cumulative | 统一Compaction |
| Score位置 | 每个BE | 每个Partition（FE） |
| 查询命令 | `SHOW PROC '/backends'` | `SHOW PROC '/compactions'` |
| 监控指标 | BE级别 | Partition级别 |

## 参考资料

- [StarRocks 存算分离 Compaction 原理 & 调优指南](https://forum.mirrorship.cn/t/topic/13256)
- [StarRocks 官方文档 - 使用 Prometheus 和 Grafana 监控报警](https://docs.starrocks.io/zh/docs/administration/management/monitoring/Monitor_and_Alert/)

