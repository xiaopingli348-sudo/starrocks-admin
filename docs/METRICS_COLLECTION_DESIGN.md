# 自主指标采集系统设计

## 一、设计目标

**核心原则**：
- ❌ 不依赖 Prometheus、Grafana 等外部组件
- ✅ 自主采集所有监控指标
- ✅ 支持历史数据查询和趋势分析
- ✅ 轻量级、低侵入、高性能

---

## 二、数据来源分析

### 2.1 StarRocks 原生数据源

| 数据源 | 获取方式 | 包含指标 | 是否实时 |
|-------|---------|---------|---------|
| SHOW BACKENDS | SQL | BE节点状态、资源使用、Tablet数 | ✅ 实时 |
| SHOW FRONTENDS | SQL | FE节点状态、版本信息 | ✅ 实时 |
| SHOW PROCESSLIST | SQL | 当前活跃查询、会话信息 | ✅ 实时 |
| information_schema.tables | SQL | 表元数据、大小、行数 | ✅ 实时 |
| information_schema.materialized_views | SQL | 物化视图元数据 | ✅ 实时 |
| SHOW ROUTINE LOAD | SQL | 导入任务状态 | ✅ 实时 |
| SHOW TRANSACTION | SQL | 事务状态 | ✅ 实时 |
| SHOW PROFILELIST | SQL | 查询历史（有限） | ✅ 实时 |
| HTTP API /metrics | HTTP | Prometheus格式指标 | ✅ 实时 |
| HTTP API /api/show_runtime_info | HTTP | FE运行时信息 | ✅ 实时 |

**结论**：
1. ✅ **基础数据都可以从 StarRocks 直接获取**
2. ⚠️ **时间序列数据需要自己采集存储**（如 QPS、延迟趋势）
3. ⚠️ **聚合统计需要自己计算**（如成功率、增长率）

---

## 三、指标分类与采集策略

### 3.1 实时指标（不需要历史数据）

直接查询即可，无需定时采集：

| 指标类别 | 具体指标 | 数据来源 |
|---------|---------|---------|
| 节点状态 | BE/FE在线数 | SHOW BACKENDS/FRONTENDS |
| 资源使用 | 磁盘/内存/CPU使用率 | SHOW BACKENDS |
| 当前查询 | 运行中查询数 | SHOW PROCESSLIST |
| 数据统计 | 数据库/表/Tablet数量 | SQL查询 |
| 任务状态 | 导入/事务/Schema Change | SQL查询 |

**实现方式**：API 调用时实时查询

---

### 3.2 时间序列指标（需要历史数据）

需要定时采集并存储：

| 指标类别 | 具体指标 | 采集频率 | 保留时长 |
|---------|---------|---------|---------|
| 查询性能 | QPS、查询成功数、失败数 | 30秒 | 7天 |
| 查询延迟 | P50/P90/P95/P99 延迟 | 30秒 | 7天 |
| 资源趋势 | CPU/内存/磁盘使用率 | 1分钟 | 7天 |
| 数据增长 | 总数据量、Tablet数 | 5分钟 | 30天 |
| 物化视图 | 刷新成功/失败数 | 1分钟 | 7天 |
| 导入统计 | 导入行数、成功率 | 1分钟 | 7天 |
| 事务统计 | 事务成功/失败数 | 1分钟 | 7天 |

**实现方式**：后台定时任务 + 数据库存储

---

### 3.3 计算指标（基于原始数据计算）

| 指标 | 计算公式 | 数据来源 |
|------|---------|---------|
| 查询成功率 | success / (success + error) * 100 | 历史数据 |
| 环比变化 | (当前值 - 历史值) / 历史值 * 100 | 历史数据 |
| 磁盘使用率 | used / total * 100 | SHOW BACKENDS |
| 日增长量 | 今日数据量 - 昨日数据量 | 历史快照 |
| 容量预测 | (total - used) / 日增长量 | 历史趋势 |

**实现方式**：后端 Service 层实时计算

---

## 四、数据采集架构设计

### 4.1 整体架构

```
┌─────────────────────────────────────────────────┐
│                  前端 Dashboard                  │
└────────────────┬────────────────────────────────┘
                 │ HTTP Request
                 ↓
┌─────────────────────────────────────────────────┐
│            API Layer (Axum Handlers)            │
└────────────────┬────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────┐
│         OverviewService (数据聚合层)            │
│  ┌──────────────────┬────────────────────────┐  │
│  │  实时数据查询     │   历史数据查询         │  │
│  └──────┬───────────┴────────┬───────────────┘  │
└─────────┼────────────────────┼──────────────────┘
          │                    │
          ↓                    ↓
┌──────────────────┐  ┌────────────────────────┐
│  StarRocksClient │  │   MetricsRepository    │
│  (实时查询 SR)    │  │  (查询历史指标数据)     │
└──────────────────┘  └────────────────────────┘
          ↑                    ↑
          │                    │
          │                    │
┌─────────┴────────┐  ┌────────┴───────────────┐
│  StarRocks 集群   │  │  SQLite (本地数据库)   │
│  - SHOW commands │  │  - metrics_snapshots   │
│  - HTTP /metrics │  │  - daily_snapshots     │
└──────────────────┘  └────────────────────────┘
          ↑                    ↑
          │                    │
          │                    │
          └────────────────────┘
                     │
          ┌──────────┴──────────┐
          │  MetricsCollector   │
          │   (后台定时任务)     │
          │  - 每30秒采集一次    │
          │  - 存储到 SQLite    │
          └─────────────────────┘
```

---

### 4.2 数据库设计

#### 表1：metrics_snapshots（时间序列快照）

存储所有需要趋势分析的指标。

```sql
CREATE TABLE metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    collected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 查询性能指标
    query_total BIGINT NOT NULL DEFAULT 0,
    query_success BIGINT NOT NULL DEFAULT 0,
    query_error BIGINT NOT NULL DEFAULT 0,
    query_timeout BIGINT NOT NULL DEFAULT 0,
    
    -- 查询延迟指标 (毫秒)
    query_latency_p50 REAL NOT NULL DEFAULT 0,
    query_latency_p90 REAL NOT NULL DEFAULT 0,
    query_latency_p95 REAL NOT NULL DEFAULT 0,
    query_latency_p99 REAL NOT NULL DEFAULT 0,
    
    -- QPS/RPS
    qps REAL NOT NULL DEFAULT 0,
    rps REAL NOT NULL DEFAULT 0,
    
    -- 资源使用指标
    disk_total_bytes BIGINT NOT NULL DEFAULT 0,
    disk_used_bytes BIGINT NOT NULL DEFAULT 0,
    disk_usage_pct REAL NOT NULL DEFAULT 0,
    cpu_usage_pct REAL NOT NULL DEFAULT 0,
    mem_usage_pct REAL NOT NULL DEFAULT 0,
    
    -- 集群状态指标
    be_node_total INTEGER NOT NULL DEFAULT 0,
    be_node_alive INTEGER NOT NULL DEFAULT 0,
    fe_node_total INTEGER NOT NULL DEFAULT 0,
    fe_node_alive INTEGER NOT NULL DEFAULT 0,
    
    -- 数据量指标
    total_tablet_count BIGINT NOT NULL DEFAULT 0,
    total_database_count INTEGER NOT NULL DEFAULT 0,
    total_table_count INTEGER NOT NULL DEFAULT 0,
    
    -- 任务指标
    running_queries INTEGER NOT NULL DEFAULT 0,
    mv_refresh_running INTEGER NOT NULL DEFAULT 0,
    mv_refresh_success_total BIGINT NOT NULL DEFAULT 0,
    mv_refresh_failed_total BIGINT NOT NULL DEFAULT 0,
    
    -- 事务指标
    txn_begin_total BIGINT NOT NULL DEFAULT 0,
    txn_success_total BIGINT NOT NULL DEFAULT 0,
    txn_failed_total BIGINT NOT NULL DEFAULT 0,
    
    -- 导入指标
    load_finished_total BIGINT NOT NULL DEFAULT 0,
    routine_load_rows_total BIGINT NOT NULL DEFAULT 0,
    
    -- Compaction指标
    max_compaction_score REAL NOT NULL DEFAULT 0,
    base_compaction_requests_total BIGINT NOT NULL DEFAULT 0,
    cumulative_compaction_requests_total BIGINT NOT NULL DEFAULT 0,
    
    -- 索引优化
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX idx_metrics_cluster_time 
ON metrics_snapshots(cluster_id, collected_at DESC);

CREATE INDEX idx_metrics_collected_at 
ON metrics_snapshots(collected_at DESC);
```

#### 表2：daily_snapshots（每日汇总快照）

用于长期趋势分析和容量规划。

```sql
CREATE TABLE daily_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    snapshot_date DATE NOT NULL,
    
    -- 数据量统计
    total_data_bytes BIGINT NOT NULL DEFAULT 0,
    total_tablet_count BIGINT NOT NULL DEFAULT 0,
    total_table_count INTEGER NOT NULL DEFAULT 0,
    total_database_count INTEGER NOT NULL DEFAULT 0,
    
    -- 每日增长量
    daily_data_growth_bytes BIGINT NOT NULL DEFAULT 0,
    daily_tablet_growth INTEGER NOT NULL DEFAULT 0,
    
    -- 查询统计
    daily_query_total BIGINT NOT NULL DEFAULT 0,
    daily_query_success BIGINT NOT NULL DEFAULT 0,
    daily_query_error BIGINT NOT NULL DEFAULT 0,
    
    -- 平均性能
    avg_qps REAL NOT NULL DEFAULT 0,
    avg_p95_latency REAL NOT NULL DEFAULT 0,
    avg_disk_usage_pct REAL NOT NULL DEFAULT 0,
    avg_cpu_usage_pct REAL NOT NULL DEFAULT 0,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(cluster_id, snapshot_date),
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

CREATE INDEX idx_daily_cluster_date 
ON daily_snapshots(cluster_id, snapshot_date DESC);
```

---

## 五、指标采集实现

### 5.1 MetricsCollector（定时采集器）

**文件**: `backend/src/services/metrics_collector.rs`

```rust
use tokio::time::{interval, Duration};
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct MetricsCollector {
    db: SqlitePool,
    cluster_service: Arc<ClusterService>,
}

impl MetricsCollector {
    pub fn new(db: SqlitePool, cluster_service: Arc<ClusterService>) -> Self {
        Self { db, cluster_service }
    }
    
    /// Start the background collection task
    pub async fn start(self: Arc<Self>) {
        // Spawn metrics collection task (every 30 seconds)
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.collect_all_clusters().await {
                    tracing::error!("Metrics collection error: {}", e);
                }
            }
        });
        
        // Spawn daily aggregation task (runs at midnight)
        let collector_clone = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(3600)); // Check every hour
            
            loop {
                interval.tick().await;
                
                // Check if it's around midnight (00:00 - 00:59)
                let now = chrono::Local::now();
                if now.hour() == 0 {
                    if let Err(e) = collector_clone.aggregate_daily_snapshots().await {
                        tracing::error!("Daily aggregation error: {}", e);
                    }
                }
            }
        });
    }
    
    /// Collect metrics for all clusters
    async fn collect_all_clusters(&self) -> Result<(), ApiError> {
        let clusters = self.cluster_service.list_clusters().await?;
        
        for cluster in clusters {
            if let Err(e) = self.collect_cluster_metrics(cluster.id).await {
                tracing::error!("Failed to collect metrics for cluster {}: {}", 
                              cluster.id, e);
                // Continue with next cluster even if one fails
            }
        }
        
        Ok(())
    }
    
    /// Collect metrics for a single cluster
    async fn collect_cluster_metrics(&self, cluster_id: i64) -> Result<(), ApiError> {
        let cluster = self.cluster_service.get_cluster(cluster_id).await?;
        let client = StarRocksClient::new(cluster);
        
        // Fetch all required data concurrently
        let (backends, frontends, processlist, metrics_text) = tokio::try_join!(
            client.get_backends(),
            client.get_frontends(),
            client.query("SHOW PROCESSLIST"),
            client.get_metrics(), // HTTP /metrics endpoint
        )?;
        
        // Parse metrics from Prometheus format
        let metrics_map = client.parse_prometheus_metrics(&metrics_text)?;
        
        // Calculate aggregate metrics
        let snapshot = self.build_metrics_snapshot(
            cluster_id,
            &backends,
            &frontends,
            &processlist,
            &metrics_map,
        ).await?;
        
        // Save to database
        self.save_snapshot(&snapshot).await?;
        
        tracing::debug!("Collected metrics for cluster {}", cluster_id);
        
        Ok(())
    }
    
    /// Build metrics snapshot from raw data
    async fn build_metrics_snapshot(
        &self,
        cluster_id: i64,
        backends: &[Backend],
        frontends: &[Frontend],
        processlist: &[ProcessInfo],
        metrics_map: &HashMap<String, f64>,
    ) -> Result<MetricsSnapshot, ApiError> {
        // Aggregate backend metrics
        let be_total = backends.len();
        let be_alive = backends.iter().filter(|b| b.alive == "true").count();
        
        let disk_total: u64 = backends.iter()
            .filter_map(|b| parse_storage_size(&b.total_capacity))
            .sum();
        let disk_used: u64 = backends.iter()
            .filter_map(|b| parse_storage_size(&b.data_used_capacity))
            .sum();
        let disk_usage_pct = if disk_total > 0 {
            (disk_used as f64 / disk_total as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_cpu = if be_total > 0 {
            backends.iter()
                .filter_map(|b| b.cpu_used_pct.trim_end_matches('%').parse::<f64>().ok())
                .sum::<f64>() / be_total as f64
        } else {
            0.0
        };
        
        let avg_mem = if be_total > 0 {
            backends.iter()
                .filter_map(|b| b.mem_used_pct.trim_end_matches('%').parse::<f64>().ok())
                .sum::<f64>() / be_total as f64
        } else {
            0.0
        };
        
        let total_tablets: i64 = backends.iter()
            .filter_map(|b| b.tablet_num.parse::<i64>().ok())
            .sum();
        
        let running_queries = processlist.len() as i32;
        
        // Aggregate frontend metrics
        let fe_total = frontends.len();
        let fe_alive = frontends.iter().filter(|f| f.alive == "true").count();
        
        // Extract metrics from Prometheus format
        Ok(MetricsSnapshot {
            cluster_id,
            collected_at: chrono::Utc::now(),
            
            // Query metrics from /metrics
            query_total: metrics_map.get("starrocks_fe_query_total")
                .map(|v| *v as i64).unwrap_or(0),
            query_success: metrics_map.get("starrocks_fe_query_success")
                .map(|v| *v as i64).unwrap_or(0),
            query_error: metrics_map.get("starrocks_fe_query_err")
                .map(|v| *v as i64).unwrap_or(0),
            query_timeout: metrics_map.get("starrocks_fe_query_timeout")
                .map(|v| *v as i64).unwrap_or(0),
            
            // Latency metrics (parse from metrics with labels)
            query_latency_p50: self.extract_latency_quantile(metrics_map, "50_quantile"),
            query_latency_p90: self.extract_latency_quantile(metrics_map, "90_quantile"),
            query_latency_p95: self.extract_latency_quantile(metrics_map, "95_quantile"),
            query_latency_p99: self.extract_latency_quantile(metrics_map, "99_quantile"),
            
            qps: metrics_map.get("starrocks_fe_qps").copied().unwrap_or(0.0),
            rps: metrics_map.get("starrocks_fe_rps").copied().unwrap_or(0.0),
            
            // Resource metrics
            disk_total_bytes: disk_total as i64,
            disk_used_bytes: disk_used as i64,
            disk_usage_pct,
            cpu_usage_pct: avg_cpu,
            mem_usage_pct: avg_mem,
            
            // Cluster status
            be_node_total: be_total as i32,
            be_node_alive: be_alive as i32,
            fe_node_total: fe_total as i32,
            fe_node_alive: fe_alive as i32,
            
            // Data volume (will be filled separately)
            total_tablet_count: total_tablets,
            total_database_count: 0, // Needs separate query
            total_table_count: 0,    // Needs separate query
            
            running_queries,
            
            // MV metrics
            mv_refresh_running: metrics_map.get("mv_refresh_running_jobs")
                .map(|v| *v as i32).unwrap_or(0),
            mv_refresh_success_total: metrics_map.get("mv_refresh_total_success_jobs")
                .map(|v| *v as i64).unwrap_or(0),
            mv_refresh_failed_total: metrics_map.get("mv_refresh_total_failed_jobs")
                .map(|v| *v as i64).unwrap_or(0),
            
            // Transaction metrics
            txn_begin_total: metrics_map.get("starrocks_fe_txn_begin")
                .map(|v| *v as i64).unwrap_or(0),
            txn_success_total: metrics_map.get("starrocks_fe_txn_success")
                .map(|v| *v as i64).unwrap_or(0),
            txn_failed_total: metrics_map.get("starrocks_fe_txn_failed")
                .map(|v| *v as i64).unwrap_or(0),
            
            // Load metrics
            load_finished_total: metrics_map.get("starrocks_fe_load_finished")
                .map(|v| *v as i64).unwrap_or(0),
            routine_load_rows_total: metrics_map.get("starrocks_fe_routine_load_rows")
                .map(|v| *v as i64).unwrap_or(0),
            
            // Compaction metrics
            max_compaction_score: metrics_map.get("starrocks_fe_max_tablet_compaction_score")
                .copied().unwrap_or(0.0),
            base_compaction_requests_total: metrics_map.get("be_base_compaction_requests")
                .map(|v| *v as i64).unwrap_or(0),
            cumulative_compaction_requests_total: metrics_map.get("be_cumulative_compaction_requests")
                .map(|v| *v as i64).unwrap_or(0),
        })
    }
    
    /// Extract latency quantile from metrics map
    fn extract_latency_quantile(&self, metrics_map: &HashMap<String, f64>, quantile: &str) -> f64 {
        // Prometheus format: starrocks_fe_query_latency{type="95_quantile"}
        // Our parser should handle this
        metrics_map.get(&format!("starrocks_fe_query_latency_{}", quantile))
            .copied()
            .unwrap_or(0.0)
    }
    
    /// Save snapshot to database
    async fn save_snapshot(&self, snapshot: &MetricsSnapshot) -> Result<(), ApiError> {
        sqlx::query!(
            r#"
            INSERT INTO metrics_snapshots (
                cluster_id, collected_at,
                query_total, query_success, query_error, query_timeout,
                query_latency_p50, query_latency_p90, query_latency_p95, query_latency_p99,
                qps, rps,
                disk_total_bytes, disk_used_bytes, disk_usage_pct,
                cpu_usage_pct, mem_usage_pct,
                be_node_total, be_node_alive, fe_node_total, fe_node_alive,
                total_tablet_count, running_queries,
                mv_refresh_running, mv_refresh_success_total, mv_refresh_failed_total,
                txn_begin_total, txn_success_total, txn_failed_total,
                load_finished_total, routine_load_rows_total,
                max_compaction_score,
                base_compaction_requests_total, cumulative_compaction_requests_total
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            "#,
            snapshot.cluster_id,
            snapshot.collected_at,
            snapshot.query_total,
            snapshot.query_success,
            snapshot.query_error,
            snapshot.query_timeout,
            snapshot.query_latency_p50,
            snapshot.query_latency_p90,
            snapshot.query_latency_p95,
            snapshot.query_latency_p99,
            snapshot.qps,
            snapshot.rps,
            snapshot.disk_total_bytes,
            snapshot.disk_used_bytes,
            snapshot.disk_usage_pct,
            snapshot.cpu_usage_pct,
            snapshot.mem_usage_pct,
            snapshot.be_node_total,
            snapshot.be_node_alive,
            snapshot.fe_node_total,
            snapshot.fe_node_alive,
            snapshot.total_tablet_count,
            snapshot.running_queries,
            snapshot.mv_refresh_running,
            snapshot.mv_refresh_success_total,
            snapshot.mv_refresh_failed_total,
            snapshot.txn_begin_total,
            snapshot.txn_success_total,
            snapshot.txn_failed_total,
            snapshot.load_finished_total,
            snapshot.routine_load_rows_total,
            snapshot.max_compaction_score,
            snapshot.base_compaction_requests_total,
            snapshot.cumulative_compaction_requests_total,
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
    
    /// Aggregate daily snapshots (runs at midnight)
    async fn aggregate_daily_snapshots(&self) -> Result<(), ApiError> {
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        
        // For each cluster, aggregate yesterday's data
        let clusters = self.cluster_service.list_clusters().await?;
        
        for cluster in clusters {
            if let Err(e) = self.aggregate_cluster_daily(cluster.id, yesterday).await {
                tracing::error!("Failed to aggregate daily data for cluster {}: {}", 
                              cluster.id, e);
            }
        }
        
        // Clean up old metrics_snapshots (keep only 7 days)
        self.cleanup_old_snapshots(7).await?;
        
        Ok(())
    }
    
    /// Aggregate one cluster's daily data
    async fn aggregate_cluster_daily(
        &self,
        cluster_id: i64,
        date: chrono::NaiveDate,
    ) -> Result<(), ApiError> {
        // Query average metrics for the day
        let stats = sqlx::query!(
            r#"
            SELECT 
                AVG(qps) as avg_qps,
                AVG(query_latency_p95) as avg_p95_latency,
                AVG(disk_usage_pct) as avg_disk_usage_pct,
                AVG(cpu_usage_pct) as avg_cpu_usage_pct,
                MAX(query_total) - MIN(query_total) as daily_query_total,
                MAX(query_success) - MIN(query_success) as daily_query_success,
                MAX(query_error) - MIN(query_error) as daily_query_error,
                MAX(disk_used_bytes) as total_data_bytes,
                MAX(total_tablet_count) as total_tablet_count
            FROM metrics_snapshots
            WHERE cluster_id = ?
              AND DATE(collected_at) = ?
            "#,
            cluster_id,
            date
        )
        .fetch_optional(&self.db)
        .await?;
        
        if let Some(stats) = stats {
            // Calculate daily growth
            let prev_day = date - chrono::Duration::days(1);
            let prev_stats = sqlx::query!(
                r#"
                SELECT total_data_bytes, total_tablet_count
                FROM daily_snapshots
                WHERE cluster_id = ? AND snapshot_date = ?
                "#,
                cluster_id,
                prev_day
            )
            .fetch_optional(&self.db)
            .await?;
            
            let daily_data_growth = if let Some(prev) = prev_stats {
                stats.total_data_bytes.unwrap_or(0) - prev.total_data_bytes
            } else {
                0
            };
            
            // Insert or update daily snapshot
            sqlx::query!(
                r#"
                INSERT INTO daily_snapshots (
                    cluster_id, snapshot_date,
                    total_data_bytes, total_tablet_count,
                    daily_data_growth_bytes,
                    daily_query_total, daily_query_success, daily_query_error,
                    avg_qps, avg_p95_latency, avg_disk_usage_pct, avg_cpu_usage_pct
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(cluster_id, snapshot_date) DO UPDATE SET
                    total_data_bytes = excluded.total_data_bytes,
                    daily_data_growth_bytes = excluded.daily_data_growth_bytes,
                    avg_qps = excluded.avg_qps
                "#,
                cluster_id,
                date,
                stats.total_data_bytes.unwrap_or(0),
                stats.total_tablet_count.unwrap_or(0),
                daily_data_growth,
                stats.daily_query_total.unwrap_or(0),
                stats.daily_query_success.unwrap_or(0),
                stats.daily_query_error.unwrap_or(0),
                stats.avg_qps.unwrap_or(0.0),
                stats.avg_p95_latency.unwrap_or(0.0),
                stats.avg_disk_usage_pct.unwrap_or(0.0),
                stats.avg_cpu_usage_pct.unwrap_or(0.0),
            )
            .execute(&self.db)
            .await?;
        }
        
        Ok(())
    }
    
    /// Cleanup old snapshots
    async fn cleanup_old_snapshots(&self, keep_days: i64) -> Result<(), ApiError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(keep_days);
        
        sqlx::query!(
            "DELETE FROM metrics_snapshots WHERE collected_at < ?",
            cutoff
        )
        .execute(&self.db)
        .await?;
        
        tracing::info!("Cleaned up metrics snapshots older than {} days", keep_days);
        
        Ok(())
    }
}
```

---

### 5.2 MetricsRepository（历史数据查询）

**文件**: `backend/src/services/metrics_repository.rs`

```rust
pub struct MetricsRepository {
    db: SqlitePool,
}

impl MetricsRepository {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
    
    /// Query metrics for a time range
    pub async fn query_metrics(
        &self,
        cluster_id: i64,
        time_range: TimeRange,
    ) -> Result<Vec<MetricsSnapshot>, ApiError> {
        let (start_time, end_time) = time_range.to_timestamps();
        
        let snapshots = sqlx::query_as!(
            MetricsSnapshot,
            r#"
            SELECT * FROM metrics_snapshots
            WHERE cluster_id = ?
              AND collected_at BETWEEN ? AND ?
            ORDER BY collected_at ASC
            "#,
            cluster_id,
            start_time,
            end_time
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(snapshots)
    }
    
    /// Get latest snapshot
    pub async fn get_latest_snapshot(
        &self,
        cluster_id: i64,
    ) -> Result<Option<MetricsSnapshot>, ApiError> {
        let snapshot = sqlx::query_as!(
            MetricsSnapshot,
            r#"
            SELECT * FROM metrics_snapshots
            WHERE cluster_id = ?
            ORDER BY collected_at DESC
            LIMIT 1
            "#,
            cluster_id
        )
        .fetch_optional(&self.db)
        .await?;
        
        Ok(snapshot)
    }
    
    /// Calculate trend (compare with previous time window)
    pub async fn calculate_trend(
        &self,
        cluster_id: i64,
        metric_name: &str,
    ) -> Result<f64, ApiError> {
        // Get current average (last 5 minutes)
        let current = self.get_recent_average(cluster_id, metric_name, 5).await?;
        
        // Get previous average (5-10 minutes ago)
        let previous = self.get_previous_average(cluster_id, metric_name, 5, 10).await?;
        
        // Calculate percentage change
        let trend = if previous > 0.0 {
            ((current - previous) / previous) * 100.0
        } else {
            0.0
        };
        
        Ok(trend)
    }
    
    /// Get daily growth statistics
    pub async fn get_daily_growth(
        &self,
        cluster_id: i64,
        days: i32,
    ) -> Result<Vec<DailySnapshot>, ApiError> {
        let snapshots = sqlx::query_as!(
            DailySnapshot,
            r#"
            SELECT * FROM daily_snapshots
            WHERE cluster_id = ?
            ORDER BY snapshot_date DESC
            LIMIT ?
            "#,
            cluster_id,
            days
        )
        .fetch_all(&self.db)
        .await?;
        
        Ok(snapshots)
    }
    
    /// Predict capacity (how many days until full)
    pub async fn predict_capacity(
        &self,
        cluster_id: i64,
    ) -> Result<Option<i32>, ApiError> {
        // Get last 7 days growth
        let growth = self.get_daily_growth(cluster_id, 7).await?;
        
        if growth.len() < 2 {
            return Ok(None);
        }
        
        // Calculate average daily growth
        let total_growth: i64 = growth.iter()
            .map(|s| s.daily_data_growth_bytes)
            .sum();
        let avg_daily_growth = total_growth / growth.len() as i64;
        
        if avg_daily_growth <= 0 {
            return Ok(None);
        }
        
        // Get current usage
        let latest = growth.first().unwrap();
        let available = latest.total_data_bytes; // Simplified
        
        let days_remaining = available / avg_daily_growth;
        
        Ok(Some(days_remaining as i32))
    }
}
```

---

## 六、集成到应用启动流程

**文件**: `backend/src/main.rs`

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup code ...
    
    // Initialize services
    let cluster_service = Arc::new(ClusterService::new(db.clone()));
    
    // Initialize and start metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new(
        db.clone(),
        Arc::clone(&cluster_service),
    ));
    
    // Start background collection tasks
    metrics_collector.clone().start().await;
    
    tracing::info!("Metrics collector started");
    
    // ... rest of application setup ...
}
```

---

## 七、数据迁移脚本

**文件**: `backend/migrations/20241025000000_metrics_collection.sql`

```sql
-- 创建 metrics_snapshots 表
CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    collected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Query metrics
    query_total BIGINT NOT NULL DEFAULT 0,
    query_success BIGINT NOT NULL DEFAULT 0,
    query_error BIGINT NOT NULL DEFAULT 0,
    query_timeout BIGINT NOT NULL DEFAULT 0,
    
    -- Latency metrics
    query_latency_p50 REAL NOT NULL DEFAULT 0,
    query_latency_p90 REAL NOT NULL DEFAULT 0,
    query_latency_p95 REAL NOT NULL DEFAULT 0,
    query_latency_p99 REAL NOT NULL DEFAULT 0,
    
    -- QPS/RPS
    qps REAL NOT NULL DEFAULT 0,
    rps REAL NOT NULL DEFAULT 0,
    
    -- Resource metrics
    disk_total_bytes BIGINT NOT NULL DEFAULT 0,
    disk_used_bytes BIGINT NOT NULL DEFAULT 0,
    disk_usage_pct REAL NOT NULL DEFAULT 0,
    cpu_usage_pct REAL NOT NULL DEFAULT 0,
    mem_usage_pct REAL NOT NULL DEFAULT 0,
    
    -- Cluster status
    be_node_total INTEGER NOT NULL DEFAULT 0,
    be_node_alive INTEGER NOT NULL DEFAULT 0,
    fe_node_total INTEGER NOT NULL DEFAULT 0,
    fe_node_alive INTEGER NOT NULL DEFAULT 0,
    
    -- Data volume
    total_tablet_count BIGINT NOT NULL DEFAULT 0,
    total_database_count INTEGER NOT NULL DEFAULT 0,
    total_table_count INTEGER NOT NULL DEFAULT 0,
    
    -- Tasks
    running_queries INTEGER NOT NULL DEFAULT 0,
    mv_refresh_running INTEGER NOT NULL DEFAULT 0,
    mv_refresh_success_total BIGINT NOT NULL DEFAULT 0,
    mv_refresh_failed_total BIGINT NOT NULL DEFAULT 0,
    
    -- Transactions
    txn_begin_total BIGINT NOT NULL DEFAULT 0,
    txn_success_total BIGINT NOT NULL DEFAULT 0,
    txn_failed_total BIGINT NOT NULL DEFAULT 0,
    
    -- Loading
    load_finished_total BIGINT NOT NULL DEFAULT 0,
    routine_load_rows_total BIGINT NOT NULL DEFAULT 0,
    
    -- Compaction
    max_compaction_score REAL NOT NULL DEFAULT 0,
    base_compaction_requests_total BIGINT NOT NULL DEFAULT 0,
    cumulative_compaction_requests_total BIGINT NOT NULL DEFAULT 0,
    
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

CREATE INDEX idx_metrics_cluster_time ON metrics_snapshots(cluster_id, collected_at DESC);
CREATE INDEX idx_metrics_collected_at ON metrics_snapshots(collected_at DESC);

-- 创建 daily_snapshots 表
CREATE TABLE IF NOT EXISTS daily_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    snapshot_date DATE NOT NULL,
    
    total_data_bytes BIGINT NOT NULL DEFAULT 0,
    total_tablet_count BIGINT NOT NULL DEFAULT 0,
    total_table_count INTEGER NOT NULL DEFAULT 0,
    total_database_count INTEGER NOT NULL DEFAULT 0,
    
    daily_data_growth_bytes BIGINT NOT NULL DEFAULT 0,
    daily_tablet_growth INTEGER NOT NULL DEFAULT 0,
    
    daily_query_total BIGINT NOT NULL DEFAULT 0,
    daily_query_success BIGINT NOT NULL DEFAULT 0,
    daily_query_error BIGINT NOT NULL DEFAULT 0,
    
    avg_qps REAL NOT NULL DEFAULT 0,
    avg_p95_latency REAL NOT NULL DEFAULT 0,
    avg_disk_usage_pct REAL NOT NULL DEFAULT 0,
    avg_cpu_usage_pct REAL NOT NULL DEFAULT 0,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(cluster_id, snapshot_date),
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

CREATE INDEX idx_daily_cluster_date ON daily_snapshots(cluster_id, snapshot_date DESC);
```

---

## 八、总结

### 优势

1. ✅ **完全自主**：不依赖 Prometheus、Grafana 等外部组件
2. ✅ **轻量级**：使用 SQLite 存储，无需额外数据库
3. ✅ **高性能**：30秒采集一次，异步非阻塞
4. ✅ **数据完整**：支持 7 天详细数据 + 30 天汇总数据
5. ✅ **容量可控**：自动清理过期数据
6. ✅ **易于扩展**：可轻松添加新指标

### 数据保留策略

- **详细指标**：保留最近 7 天，每 30 秒一次
- **每日汇总**：保留最近 30 天
- **自动清理**：每天凌晨清理过期数据

### 存储空间估算

假设单个集群：
- 每次快照约 500 字节
- 每天采集次数: (24 * 3600) / 30 ≈ 2,880 次
- 每天数据量: 2,880 * 500 字节 ≈ 1.4 MB
- 7 天数据量: 1.4 * 7 ≈ 10 MB
- **单集群总计: < 15 MB**

多集群环境：
- 10 个集群约 150 MB
- 100 个集群约 1.5 GB

**结论**：存储成本极低，完全可控。

