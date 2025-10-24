// Metrics Collector Service
// Purpose: Periodically collect metrics from StarRocks clusters and store them in SQLite
// Design Ref: ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md

use crate::models::{Cluster, MetricsSummary};
use crate::services::{ClusterService, StarRocksClient};
use crate::utils::ApiResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Metrics snapshot stored in database
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricsSnapshot {
    pub cluster_id: i64,
    pub collected_at: chrono::DateTime<Utc>,
    
    // Query performance
    pub qps: f64,
    pub rps: f64,
    pub query_latency_p50: f64,
    pub query_latency_p95: f64,
    pub query_latency_p99: f64,
    pub query_total: i64,
    pub query_success: i64,
    pub query_error: i64,
    pub query_timeout: i64,
    
    // Cluster health
    pub backend_total: i32,
    pub backend_alive: i32,
    pub frontend_total: i32,
    pub frontend_alive: i32,
    
    // Resource usage
    pub total_cpu_usage: f64,
    pub avg_cpu_usage: f64,
    pub total_memory_usage: f64,
    pub avg_memory_usage: f64,
    pub disk_total_bytes: i64,
    pub disk_used_bytes: i64,
    pub disk_usage_pct: f64,
    
    // Storage
    pub tablet_count: i64,
    pub max_compaction_score: f64,
    
    // Transactions
    pub txn_running: i32,
    pub txn_success_total: i64,
    pub txn_failed_total: i64,
    
    // Load jobs
    pub load_running: i32,
    pub load_finished_total: i64,
    
    // JVM metrics
    pub jvm_heap_total: i64,
    pub jvm_heap_used: i64,
    pub jvm_heap_usage_pct: f64,
    pub jvm_thread_count: i32,
    
    // Network metrics
    pub network_bytes_sent_total: i64,
    pub network_bytes_received_total: i64,
    pub network_send_rate: f64,
    pub network_receive_rate: f64,
    
    // IO metrics
    pub io_read_bytes_total: i64,
    pub io_write_bytes_total: i64,
    pub io_read_rate: f64,
    pub io_write_rate: f64,
}

#[derive(Clone)]
pub struct MetricsCollectorService {
    db: SqlitePool,
    cluster_service: Arc<ClusterService>,
    collection_interval: Duration,
    retention_days: i64,
}

impl MetricsCollectorService {
    /// Create a new MetricsCollectorService
    pub fn new(db: SqlitePool, cluster_service: Arc<ClusterService>) -> Self {
        Self {
            db,
            cluster_service,
            collection_interval: Duration::from_secs(30), // 30 seconds
            retention_days: 7,                             // 7 days retention
        }
    }

    /// Start the background collection task
    /// This should be called once when the application starts
    pub async fn start_collection(self: Arc<Self>) {
        let mut ticker = interval(self.collection_interval);
        
        tracing::info!(
            "Starting metrics collector (interval: {}s, retention: {} days)",
            self.collection_interval.as_secs(),
            self.retention_days
        );
        
        // Track last daily aggregation date
        let mut last_daily_aggregation = Utc::now().date_naive();
        
        loop {
            ticker.tick().await;
            
            // Collect metrics
            if let Err(e) = self.collect_all_clusters().await {
                tracing::error!("Failed to collect metrics: {}", e);
            }
            
            // Check if we need to run daily aggregation (once per day at midnight)
            let current_date = Utc::now().date_naive();
            if current_date > last_daily_aggregation {
                tracing::info!("Running daily aggregation for date: {}", last_daily_aggregation);
                if let Err(e) = self.run_daily_aggregation_all_clusters().await {
                    tracing::error!("Failed to run daily aggregation: {}", e);
                } else {
                    last_daily_aggregation = current_date;
                }
            }
        }
    }

    /// Collect metrics from all clusters
    async fn collect_all_clusters(&self) -> Result<(), anyhow::Error> {
        let clusters = self.cluster_service.list_clusters().await?;
        
        tracing::debug!("Collecting metrics from {} clusters", clusters.len());
        
        for cluster in clusters {
            if let Err(e) = self.collect_cluster_metrics(&cluster).await {
                tracing::error!(
                    "Failed to collect metrics for cluster {} ({}): {}",
                    cluster.id,
                    cluster.name,
                    e
                );
                // Continue with other clusters
            }
        }
        
        // Cleanup old metrics every collection cycle
        if let Err(e) = self.cleanup_old_metrics().await {
            tracing::error!("Failed to cleanup old metrics: {}", e);
        }
        
        Ok(())
    }

    /// Collect metrics from a single cluster
    async fn collect_cluster_metrics(&self, cluster: &Cluster) -> ApiResult<()> {
        tracing::debug!("Collecting metrics for cluster: {} ({})", cluster.id, cluster.name);
        
        let client = StarRocksClient::new(cluster.clone());
        
        // Collect data from StarRocks
        let (metrics_text, backends, frontends, runtime_info) = tokio::try_join!(
            client.get_metrics(),
            client.get_backends(),
            client.get_frontends(),
            client.get_runtime_info(),
        )?;
        
        // Parse Prometheus metrics
        let metrics_map = client.parse_prometheus_metrics(&metrics_text)?;
        
        // Aggregate backend metrics
        let backend_total = backends.len() as i32;
        let backend_alive = backends.iter().filter(|b| b.alive == "true").count() as i32;
        
        let frontend_total = frontends.len() as i32;
        let frontend_alive = frontends.iter().filter(|f| f.alive == "true").count() as i32;
        
        let tablet_count: i64 = backends
            .iter()
            .filter_map(|b| b.tablet_num.parse::<i64>().ok())
            .sum();
        
        let total_cpu_usage: f64 = backends
            .iter()
            .filter_map(|b| b.cpu_used_pct.trim_end_matches('%').parse::<f64>().ok())
            .sum();
        
        let avg_cpu_usage = if backend_total > 0 {
            total_cpu_usage / backend_total as f64
        } else {
            0.0
        };
        
        let total_memory_usage: f64 = backends
            .iter()
            .filter_map(|b| b.mem_used_pct.trim_end_matches('%').parse::<f64>().ok())
            .sum();
        
        let avg_memory_usage = if backend_total > 0 {
            total_memory_usage / backend_total as f64
        } else {
            0.0
        };
        
        let disk_used_bytes: i64 = backends
            .iter()
            .filter_map(|b| parse_storage_size(&b.data_used_capacity))
            .sum();
        
        let disk_total_bytes: i64 = backends
            .iter()
            .filter_map(|b| parse_storage_size(&b.total_capacity))
            .sum();
        
        let disk_usage_pct = if disk_total_bytes > 0 {
            (disk_used_bytes as f64 / disk_total_bytes as f64) * 100.0
        } else {
            0.0
        };
        
        // JVM metrics
        let jvm_heap_used = runtime_info.total_mem - runtime_info.free_mem;
        let jvm_heap_usage_pct = if runtime_info.total_mem > 0 {
            (jvm_heap_used as f64 / runtime_info.total_mem as f64) * 100.0
        } else {
            0.0
        };
        
        // Network metrics (BE)
        let network_bytes_sent_total = metrics_map.get("starrocks_be_network_send_bytes").copied().unwrap_or(0.0) as i64;
        let network_bytes_received_total = metrics_map.get("starrocks_be_network_receive_bytes").copied().unwrap_or(0.0) as i64;
        let network_send_rate = metrics_map.get("starrocks_be_network_send_rate").copied().unwrap_or(0.0);
        let network_receive_rate = metrics_map.get("starrocks_be_network_receive_rate").copied().unwrap_or(0.0);
        
        // IO metrics (BE)
        let io_read_bytes_total = metrics_map.get("starrocks_be_disk_read_bytes").copied().unwrap_or(0.0) as i64;
        let io_write_bytes_total = metrics_map.get("starrocks_be_disk_write_bytes").copied().unwrap_or(0.0) as i64;
        let io_read_rate = metrics_map.get("starrocks_be_disk_read_rate").copied().unwrap_or(0.0);
        let io_write_rate = metrics_map.get("starrocks_be_disk_write_rate").copied().unwrap_or(0.0);
        
        // Create snapshot
        let snapshot = MetricsSnapshot {
            cluster_id: cluster.id,
            collected_at: Utc::now(),
            
            // Query metrics from Prometheus
            qps: metrics_map.get("starrocks_fe_qps").copied().unwrap_or(0.0),
            rps: metrics_map.get("starrocks_fe_rps").copied().unwrap_or(0.0),
            query_latency_p50: metrics_map.get("starrocks_fe_query_latency_p50").copied().unwrap_or(0.0),
            query_latency_p95: metrics_map.get("starrocks_fe_query_latency_p95").copied().unwrap_or(0.0),
            query_latency_p99: metrics_map.get("starrocks_fe_query_latency_p99").copied().unwrap_or(0.0),
            query_total: metrics_map.get("starrocks_fe_query_total").copied().unwrap_or(0.0) as i64,
            query_success: metrics_map.get("starrocks_fe_query_success").copied().unwrap_or(0.0) as i64,
            query_error: metrics_map.get("starrocks_fe_query_err").copied().unwrap_or(0.0) as i64,
            query_timeout: metrics_map.get("starrocks_fe_query_timeout").copied().unwrap_or(0.0) as i64,
            
            // Cluster health
            backend_total,
            backend_alive,
            frontend_total,
            frontend_alive,
            
            // Resource usage
            total_cpu_usage,
            avg_cpu_usage,
            total_memory_usage,
            avg_memory_usage,
            disk_total_bytes,
            disk_used_bytes,
            disk_usage_pct,
            
            // Storage
            tablet_count,
            max_compaction_score: metrics_map
                .get("starrocks_fe_max_tablet_compaction_score")
                .copied()
                .unwrap_or(0.0),
            
            // Transactions
            txn_running: 0, // TODO: Need to get from appropriate metric
            txn_success_total: metrics_map.get("starrocks_fe_txn_success").copied().unwrap_or(0.0) as i64,
            txn_failed_total: metrics_map.get("starrocks_fe_txn_failed").copied().unwrap_or(0.0) as i64,
            
            // Load jobs
            load_running: 0, // TODO: Need to get from appropriate metric
            load_finished_total: metrics_map.get("starrocks_fe_load_finished").copied().unwrap_or(0.0) as i64,
            
            // JVM metrics
            jvm_heap_total: runtime_info.total_mem,
            jvm_heap_used,
            jvm_heap_usage_pct,
            jvm_thread_count: runtime_info.thread_cnt,
            
            // Network metrics
            network_bytes_sent_total,
            network_bytes_received_total,
            network_send_rate,
            network_receive_rate,
            
            // IO metrics
            io_read_bytes_total,
            io_write_bytes_total,
            io_read_rate,
            io_write_rate,
        };
        
        // Save to database
        self.save_snapshot(&snapshot).await?;
        
        tracing::debug!(
            "Metrics collected for cluster {} ({}): QPS={:.2}, CPU={:.1}%, Disk={:.1}%",
            cluster.id,
            cluster.name,
            snapshot.qps,
            snapshot.avg_cpu_usage,
            snapshot.disk_usage_pct
        );
        
        Ok(())
    }

    /// Save metrics snapshot to database
    async fn save_snapshot(&self, snapshot: &MetricsSnapshot) -> ApiResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO metrics_snapshots (
                cluster_id, collected_at,
                qps, rps, query_latency_p50, query_latency_p95, query_latency_p99,
                query_total, query_success, query_error, query_timeout,
                backend_total, backend_alive, frontend_total, frontend_alive,
                total_cpu_usage, avg_cpu_usage, total_memory_usage, avg_memory_usage,
                disk_total_bytes, disk_used_bytes, disk_usage_pct,
                tablet_count, max_compaction_score,
                txn_running, txn_success_total, txn_failed_total,
                load_running, load_finished_total,
                jvm_heap_total, jvm_heap_used, jvm_heap_usage_pct, jvm_thread_count,
                network_bytes_sent_total, network_bytes_received_total, network_send_rate, network_receive_rate,
                io_read_bytes_total, io_write_bytes_total, io_read_rate, io_write_rate
            ) VALUES (
                ?, ?,
                ?, ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?, ?,
                ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?
            )
            "#,
            snapshot.cluster_id,
            snapshot.collected_at,
            snapshot.qps,
            snapshot.rps,
            snapshot.query_latency_p50,
            snapshot.query_latency_p95,
            snapshot.query_latency_p99,
            snapshot.query_total,
            snapshot.query_success,
            snapshot.query_error,
            snapshot.query_timeout,
            snapshot.backend_total,
            snapshot.backend_alive,
            snapshot.frontend_total,
            snapshot.frontend_alive,
            snapshot.total_cpu_usage,
            snapshot.avg_cpu_usage,
            snapshot.total_memory_usage,
            snapshot.avg_memory_usage,
            snapshot.disk_total_bytes,
            snapshot.disk_used_bytes,
            snapshot.disk_usage_pct,
            snapshot.tablet_count,
            snapshot.max_compaction_score,
            snapshot.txn_running,
            snapshot.txn_success_total,
            snapshot.txn_failed_total,
            snapshot.load_running,
            snapshot.load_finished_total,
            snapshot.jvm_heap_total,
            snapshot.jvm_heap_used,
            snapshot.jvm_heap_usage_pct,
            snapshot.jvm_thread_count,
            snapshot.network_bytes_sent_total,
            snapshot.network_bytes_received_total,
            snapshot.network_send_rate,
            snapshot.network_receive_rate,
            snapshot.io_read_bytes_total,
            snapshot.io_write_bytes_total,
            snapshot.io_read_rate,
            snapshot.io_write_rate
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }

    /// Cleanup old metrics data based on retention policy
    async fn cleanup_old_metrics(&self) -> Result<(), sqlx::Error> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.retention_days);
        
        let result = sqlx::query!(
            "DELETE FROM metrics_snapshots WHERE collected_at < ?",
            cutoff_date
        )
        .execute(&self.db)
        .await?;
        
        if result.rows_affected() > 0 {
            tracing::info!(
                "Cleaned up {} old metric snapshots (older than {} days)",
                result.rows_affected(),
                self.retention_days
            );
        }
        
        Ok(())
    }

    /// Get the latest snapshot for a cluster
    pub async fn get_latest_snapshot(&self, cluster_id: i64) -> ApiResult<Option<MetricsSnapshot>> {
        let row = sqlx::query!(
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
        
        if let Some(r) = row {
            Ok(Some(MetricsSnapshot {
                cluster_id: r.cluster_id,
                collected_at: r.collected_at.and_utc(),
                qps: r.qps,
                rps: r.rps,
                query_latency_p50: r.query_latency_p50,
                query_latency_p95: r.query_latency_p95,
                query_latency_p99: r.query_latency_p99,
                query_total: r.query_total,
                query_success: r.query_success,
                query_error: r.query_error,
                query_timeout: r.query_timeout,
                backend_total: r.backend_total,
                backend_alive: r.backend_alive,
                frontend_total: r.frontend_total,
                frontend_alive: r.frontend_alive,
                total_cpu_usage: r.total_cpu_usage,
                avg_cpu_usage: r.avg_cpu_usage,
                total_memory_usage: r.total_memory_usage,
                avg_memory_usage: r.avg_memory_usage,
                disk_total_bytes: r.disk_total_bytes,
                disk_used_bytes: r.disk_used_bytes,
                disk_usage_pct: r.disk_usage_pct,
                tablet_count: r.tablet_count,
                max_compaction_score: r.max_compaction_score,
                txn_running: r.txn_running,
                txn_success_total: r.txn_success_total,
                txn_failed_total: r.txn_failed_total,
                load_running: r.load_running,
                load_finished_total: r.load_finished_total,
                jvm_heap_total: r.jvm_heap_total,
                jvm_heap_used: r.jvm_heap_used,
                jvm_heap_usage_pct: r.jvm_heap_usage_pct,
                jvm_thread_count: r.jvm_thread_count,
            }))
        } else {
            Ok(None)
        }
    }
}

// Helper function to parse storage size strings like "1.5 TB", "500 GB", etc.
fn parse_storage_size(size_str: &str) -> Option<i64> {
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let value: f64 = parts[0].parse().ok()?;
    let unit = parts[1].to_uppercase();

    let bytes = match unit.as_str() {
        "B" | "BYTES" => value,
        "KB" => value * 1024.0,
        "MB" => value * 1024.0 * 1024.0,
        "GB" => value * 1024.0 * 1024.0 * 1024.0,
        "TB" => value * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "PB" => value * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some(bytes as i64)
}

impl MetricsCollectorService {
    /// Run daily aggregation for all clusters
    async fn run_daily_aggregation_all_clusters(&self) -> Result<(), anyhow::Error> {
        let clusters = self.cluster_service.list_clusters().await?;
        
        tracing::info!("Starting daily aggregation for {} clusters", clusters.len());
        
        for cluster in clusters {
            if let Err(e) = self.run_daily_aggregation_for_cluster(cluster.id).await {
                tracing::error!(
                    "Failed to run daily aggregation for cluster {} ({}): {}",
                    cluster.id,
                    cluster.name,
                    e
                );
            }
        }
        
        // Cleanup old daily snapshots (keep 90 days)
        self.cleanup_old_daily_snapshots().await?;
        
        Ok(())
    }

    /// Run daily aggregation for a single cluster
    /// Aggregates yesterday's metrics_snapshots into daily_snapshots
    async fn run_daily_aggregation_for_cluster(&self, cluster_id: i64) -> ApiResult<()> {
        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
        let yesterday_start = yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let yesterday_end = yesterday.and_hms_opt(23, 59, 59).unwrap().and_utc();
        
        tracing::debug!(
            "Aggregating metrics for cluster {} from {} to {}",
            cluster_id,
            yesterday_start,
            yesterday_end
        );
        
        // Query all snapshots from yesterday
        let snapshots = sqlx::query!(
            r#"
            SELECT
                AVG(qps) as avg_qps,
                MAX(qps) as max_qps,
                MIN(qps) as min_qps,
                AVG(query_latency_p99) as avg_latency_p99,
                MAX(query_latency_p99) as max_latency_p99,
                SUM(query_total) as total_queries,
                SUM(query_error) as total_errors,
                AVG(avg_cpu_usage) as avg_cpu_usage,
                MAX(avg_cpu_usage) as max_cpu_usage,
                AVG(avg_memory_usage) as avg_memory_usage,
                MAX(avg_memory_usage) as max_memory_usage,
                AVG(disk_usage_pct) as avg_disk_usage,
                MAX(disk_usage_pct) as max_disk_usage,
                AVG(disk_used_bytes) as avg_disk_used_bytes,
                MAX(disk_used_bytes) as max_disk_used_bytes
            FROM metrics_snapshots
            WHERE cluster_id = ?
                AND collected_at >= ?
                AND collected_at <= ?
            "#,
            cluster_id,
            yesterday_start,
            yesterday_end
        )
        .fetch_one(&self.db)
        .await?;
        
        // Calculate derived metrics
        let total_queries = snapshots.total_queries.unwrap_or(0);
        let total_errors = snapshots.total_errors.unwrap_or(0);
        let error_rate = if total_queries > 0 {
            (total_errors as f64 / total_queries as f64) * 100.0
        } else {
            0.0
        };
        
        // Insert or update daily snapshot
        sqlx::query!(
            r#"
            INSERT INTO daily_snapshots (
                cluster_id, snapshot_date,
                avg_qps, max_qps, min_qps,
                avg_latency_p99, max_latency_p99,
                total_queries, total_errors, error_rate,
                avg_cpu_usage, max_cpu_usage,
                avg_memory_usage, max_memory_usage,
                avg_disk_usage, max_disk_usage,
                total_data_bytes, daily_data_growth_bytes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(cluster_id, snapshot_date) DO UPDATE SET
                avg_qps = excluded.avg_qps,
                max_qps = excluded.max_qps,
                min_qps = excluded.min_qps,
                avg_latency_p99 = excluded.avg_latency_p99,
                max_latency_p99 = excluded.max_latency_p99,
                total_queries = excluded.total_queries,
                total_errors = excluded.total_errors,
                error_rate = excluded.error_rate,
                avg_cpu_usage = excluded.avg_cpu_usage,
                max_cpu_usage = excluded.max_cpu_usage,
                avg_memory_usage = excluded.avg_memory_usage,
                max_memory_usage = excluded.max_memory_usage,
                avg_disk_usage = excluded.avg_disk_usage,
                max_disk_usage = excluded.max_disk_usage,
                total_data_bytes = excluded.total_data_bytes,
                daily_data_growth_bytes = excluded.daily_data_growth_bytes
            "#,
            cluster_id,
            yesterday,
            snapshots.avg_qps.unwrap_or(0.0),
            snapshots.max_qps.unwrap_or(0.0),
            snapshots.min_qps.unwrap_or(0.0),
            snapshots.avg_latency_p99.unwrap_or(0.0),
            snapshots.max_latency_p99.unwrap_or(0.0),
            total_queries,
            total_errors,
            error_rate,
            snapshots.avg_cpu_usage.unwrap_or(0.0),
            snapshots.max_cpu_usage.unwrap_or(0.0),
            snapshots.avg_memory_usage.unwrap_or(0.0),
            snapshots.max_memory_usage.unwrap_or(0.0),
            snapshots.avg_disk_usage.unwrap_or(0.0),
            snapshots.max_disk_usage.unwrap_or(0.0),
            snapshots.avg_disk_used_bytes.unwrap_or(0) as i64,
            0 as i64 // daily_data_growth_bytes (would need previous day's value)
        )
        .execute(&self.db)
        .await?;
        
        tracing::info!(
            "Daily aggregation completed for cluster {} (date: {})",
            cluster_id,
            yesterday
        );
        
        Ok(())
    }

    /// Cleanup old daily snapshots (keep 90 days)
    async fn cleanup_old_daily_snapshots(&self) -> Result<(), sqlx::Error> {
        let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(90);
        
        let result = sqlx::query!(
            "DELETE FROM daily_snapshots WHERE snapshot_date < ?",
            cutoff_date
        )
        .execute(&self.db)
        .await?;
        
        if result.rows_affected() > 0 {
            tracing::info!(
                "Cleaned up {} old daily snapshots (older than 90 days)",
                result.rows_affected()
            );
        }
        
        Ok(())
    }
}

