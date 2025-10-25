// Overview Service
// Purpose: Provide aggregated cluster overview data (real-time + historical)
// Design Ref: ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md

use crate::services::{ClusterService, DataStatistics, DataStatisticsService, MetricsSnapshot};
use crate::utils::{ApiError, ApiResult, ErrorCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use utoipa::ToSchema;

/// Time range for querying historical data
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    #[serde(rename = "1h")]
    Hours1,
    #[serde(rename = "6h")]
    Hours6,
    #[serde(rename = "24h")]
    Hours24,
    #[serde(rename = "3d")]
    Days3,
}

impl TimeRange {
    pub fn to_duration(&self) -> chrono::Duration {
        match self {
            TimeRange::Hours1 => chrono::Duration::hours(1),
            TimeRange::Hours6 => chrono::Duration::hours(6),
            TimeRange::Hours24 => chrono::Duration::hours(24),
            TimeRange::Days3 => chrono::Duration::days(3),
        }
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        Utc::now() - self.to_duration()
    }

    pub fn end_time(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Cluster overview data
#[derive(Debug, Serialize, ToSchema)]
pub struct ClusterOverview {
    pub cluster_id: i64,
    pub cluster_name: String,
    pub timestamp: DateTime<Utc>,
    
    // Real-time snapshot
    pub latest_snapshot: Option<MetricsSnapshot>,
    
    // Historical trends (time series data)
    pub performance_trends: PerformanceTrends,
    pub resource_trends: ResourceTrends,
    
    // Aggregated statistics
    pub statistics: AggregatedStatistics,
}

/// Performance trends over time
#[derive(Debug, Serialize, ToSchema)]
pub struct PerformanceTrends {
    pub qps: Vec<TimeSeriesPoint>,
    pub latency_p99: Vec<TimeSeriesPoint>,
    pub error_rate: Vec<TimeSeriesPoint>,
}

/// Resource trends over time
#[derive(Debug, Serialize, ToSchema)]
pub struct ResourceTrends {
    pub cpu_usage: Vec<TimeSeriesPoint>,
    pub memory_usage: Vec<TimeSeriesPoint>,
    pub disk_usage: Vec<TimeSeriesPoint>,
}

/// Time series data point
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Capacity prediction result
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct CapacityPrediction {
    pub disk_total_bytes: i64,
    pub disk_used_bytes: i64,
    pub disk_usage_pct: f64,
    pub daily_growth_bytes: i64,
    pub days_until_full: Option<i32>,
    pub predicted_full_date: Option<String>,
    pub growth_trend: String, // "increasing", "stable", "decreasing"
}

/// Aggregated statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct AggregatedStatistics {
    pub avg_qps: f64,
    pub max_qps: f64,
    pub avg_latency_p99: f64,
    pub avg_cpu_usage: f64,
    pub avg_memory_usage: f64,
    pub avg_disk_usage: f64,
}

/// Health status card
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthCard {
    pub title: String,
    pub value: String,
    pub status: HealthStatus,
    pub description: String,
}

/// Health status enum
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Clone)]
pub struct OverviewService {
    db: SqlitePool,
    cluster_service: Arc<ClusterService>,
    data_statistics_service: Option<Arc<DataStatisticsService>>,
}

impl OverviewService {
    /// Create a new OverviewService
    pub fn new(db: SqlitePool, cluster_service: Arc<ClusterService>) -> Self {
        Self {
            db,
            cluster_service,
            data_statistics_service: None,
        }
    }

    /// Set data statistics service (optional dependency)
    pub fn with_data_statistics(mut self, service: Arc<DataStatisticsService>) -> Self {
        self.data_statistics_service = Some(service);
        self
    }

    /// Get cluster overview (main API)
    pub async fn get_cluster_overview(
        &self,
        cluster_id: i64,
        time_range: TimeRange,
    ) -> ApiResult<ClusterOverview> {
        tracing::debug!("Getting overview for cluster {} with time range {:?}", cluster_id, time_range);
        
        // Get cluster info
        let cluster = self.cluster_service.get_cluster(cluster_id).await?;
        
        // Get latest snapshot
        let latest_snapshot = self.get_latest_snapshot(cluster_id).await?;
        
        // Get historical snapshots for trends
        let history = self.get_history_snapshots(cluster_id, &time_range).await?;
        
        // Calculate trends and statistics
        let performance_trends = self.calculate_performance_trends(&history);
        let resource_trends = self.calculate_resource_trends(&history);
        let statistics = self.calculate_aggregated_statistics(&history);
        
        Ok(ClusterOverview {
            cluster_id,
            cluster_name: cluster.name,
            timestamp: Utc::now(),
            latest_snapshot,
            performance_trends,
            resource_trends,
            statistics,
        })
    }

    /// Get health status cards
    pub async fn get_health_cards(&self, cluster_id: i64) -> ApiResult<Vec<HealthCard>> {
        let snapshot = self.get_latest_snapshot(cluster_id).await?;
        
        let snapshot = match snapshot {
            Some(s) => s,
            None => {
                return Ok(vec![HealthCard {
                    title: "No Data".to_string(),
                    value: "N/A".to_string(),
                    status: HealthStatus::Warning,
                    description: "No metrics data available yet".to_string(),
                }]);
            }
        };
        
        let mut cards = Vec::new();
        
        // Cluster Status Card
        let cluster_status = if snapshot.backend_alive == snapshot.backend_total 
                                && snapshot.frontend_alive == snapshot.frontend_total {
            HealthStatus::Healthy
        } else if snapshot.backend_alive > 0 && snapshot.frontend_alive > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        cards.push(HealthCard {
            title: "Cluster Status".to_string(),
            value: format!("{}/{} BE, {}/{} FE", 
                          snapshot.backend_alive, snapshot.backend_total,
                          snapshot.frontend_alive, snapshot.frontend_total),
            status: cluster_status,
            description: "Backend and Frontend nodes availability".to_string(),
        });
        
        // QPS Card
        let qps_status = if snapshot.qps < 100.0 {
            HealthStatus::Healthy
        } else if snapshot.qps < 1000.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        cards.push(HealthCard {
            title: "Query Load".to_string(),
            value: format!("{:.1} QPS", snapshot.qps),
            status: qps_status,
            description: "Current queries per second".to_string(),
        });
        
        // CPU Usage Card
        let cpu_status = if snapshot.avg_cpu_usage < 70.0 {
            HealthStatus::Healthy
        } else if snapshot.avg_cpu_usage < 85.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        cards.push(HealthCard {
            title: "CPU Usage".to_string(),
            value: format!("{:.1}%", snapshot.avg_cpu_usage),
            status: cpu_status,
            description: "Average CPU usage across all BE nodes".to_string(),
        });
        
        // Disk Usage Card
        let disk_status = if snapshot.disk_usage_pct < 70.0 {
            HealthStatus::Healthy
        } else if snapshot.disk_usage_pct < 85.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };
        
        cards.push(HealthCard {
            title: "Disk Usage".to_string(),
            value: format!("{:.1}%", snapshot.disk_usage_pct),
            status: disk_status,
            description: "Total disk space usage".to_string(),
        });
        
        Ok(cards)
    }

    /// Get performance trends
    pub async fn get_performance_trends(
        &self,
        cluster_id: i64,
        time_range: TimeRange,
    ) -> ApiResult<PerformanceTrends> {
        let history = self.get_history_snapshots(cluster_id, &time_range).await?;
        Ok(self.calculate_performance_trends(&history))
    }

    /// Get resource trends
    pub async fn get_resource_trends(
        &self,
        cluster_id: i64,
        time_range: TimeRange,
    ) -> ApiResult<ResourceTrends> {
        let history = self.get_history_snapshots(cluster_id, &time_range).await?;
        Ok(self.calculate_resource_trends(&history))
    }

    /// Get data statistics (database/table counts, top tables, etc.)
    pub async fn get_data_statistics(&self, cluster_id: i64) -> ApiResult<DataStatistics> {
        if let Some(ref service) = self.data_statistics_service {
            // Try to get cached statistics first
            if let Some(stats) = service.get_statistics(cluster_id).await? {
                // Check if cache is recent (< 10 minutes old)
                let age = Utc::now() - stats.updated_at;
                if age.num_minutes() < 10 {
                    tracing::debug!("Using cached data statistics (age: {} minutes)", age.num_minutes());
                    return Ok(stats);
                }
            }
            
            // Cache is stale or doesn't exist, update it
            tracing::debug!("Updating data statistics for cluster {}", cluster_id);
            service.update_statistics(cluster_id).await
        } else {
            Err(ApiError::new(
                ErrorCode::InternalError,
                "Data statistics service not configured"
            ))
        }
    }

    /// Predict disk capacity
    /// 
    /// Uses linear regression on historical disk usage data to predict when disk will be full
    pub async fn predict_capacity(&self, cluster_id: i64) -> ApiResult<CapacityPrediction> {
        // Get last 7 days of disk usage data
        let cutoff = Utc::now() - chrono::Duration::days(7);
        
        let snapshots = sqlx::query!(
            r#"
            SELECT 
                disk_total_bytes,
                disk_used_bytes,
                disk_usage_pct,
                collected_at
            FROM metrics_snapshots
            WHERE cluster_id = ? AND collected_at >= ?
            ORDER BY collected_at ASC
            "#,
            cluster_id,
            cutoff
        )
        .fetch_all(&self.db)
        .await?;
        
        if snapshots.is_empty() {
            return Err(ApiError::new(
                ErrorCode::InternalError,
                "No historical data available for capacity prediction"
            ));
        }
        
        // Get latest values
        let latest = snapshots.last().unwrap();
        let disk_total_bytes = latest.disk_total_bytes;
        let disk_used_bytes = latest.disk_used_bytes;
        let disk_usage_pct = latest.disk_usage_pct;
        
        // Perform linear regression on disk_used_bytes over time
        // y = disk_used_bytes, x = days since first snapshot
        let first_time = snapshots.first().unwrap().collected_at.and_utc().timestamp();
        
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let n = snapshots.len() as f64;
        
        for snapshot in &snapshots {
            let x = (snapshot.collected_at.and_utc().timestamp() - first_time) as f64 / 86400.0; // days
            let y = snapshot.disk_used_bytes as f64;
            
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        // Calculate slope (daily growth rate in bytes)
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let daily_growth_bytes = slope as i64;
        
        // Determine growth trend
        let growth_trend = if daily_growth_bytes > 1_000_000_000 {
            // > 1GB/day
            "increasing"
        } else if daily_growth_bytes > 0 {
            "stable"
        } else {
            "decreasing"
        };
        
        // Calculate days until full (if disk is growing)
        let (days_until_full, predicted_full_date) = if daily_growth_bytes > 0 {
            let remaining_bytes = disk_total_bytes - disk_used_bytes;
            let days = (remaining_bytes as f64 / daily_growth_bytes as f64).ceil() as i32;
            
            let full_date = Utc::now() + chrono::Duration::days(days as i64);
            let full_date_str = full_date.format("%Y-%m-%d").to_string();
            
            (Some(days), Some(full_date_str))
        } else {
            (None, None)
        };
        
        Ok(CapacityPrediction {
            disk_total_bytes,
            disk_used_bytes,
            disk_usage_pct,
            daily_growth_bytes,
            days_until_full,
            predicted_full_date,
            growth_trend: growth_trend.to_string(),
        })
    }

    // ========================================
    // Internal helper methods
    // ========================================

    /// Get the latest snapshot for a cluster
    async fn get_latest_snapshot(&self, cluster_id: i64) -> ApiResult<Option<MetricsSnapshot>> {
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
                backend_total: r.backend_total as i32,
                backend_alive: r.backend_alive as i32,
                frontend_total: r.frontend_total as i32,
                frontend_alive: r.frontend_alive as i32,
                total_cpu_usage: r.total_cpu_usage,
                avg_cpu_usage: r.avg_cpu_usage,
                total_memory_usage: r.total_memory_usage,
                avg_memory_usage: r.avg_memory_usage,
                disk_total_bytes: r.disk_total_bytes,
                disk_used_bytes: r.disk_used_bytes,
                disk_usage_pct: r.disk_usage_pct,
                tablet_count: r.tablet_count,
                max_compaction_score: r.max_compaction_score,
                txn_running: r.txn_running as i32,
                txn_success_total: r.txn_success_total,
                txn_failed_total: r.txn_failed_total,
                load_running: r.load_running as i32,
                load_finished_total: r.load_finished_total,
                jvm_heap_total: r.jvm_heap_total,
                jvm_heap_used: r.jvm_heap_used,
                jvm_heap_usage_pct: r.jvm_heap_usage_pct,
                jvm_thread_count: r.jvm_thread_count as i32,
                network_bytes_sent_total: r.network_bytes_sent_total,
                network_bytes_received_total: r.network_bytes_received_total,
                network_send_rate: r.network_send_rate,
                network_receive_rate: r.network_receive_rate,
                io_read_bytes_total: r.io_read_bytes_total,
                io_write_bytes_total: r.io_write_bytes_total,
                io_read_rate: r.io_read_rate,
                io_write_rate: r.io_write_rate,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get historical snapshots for a time range
    async fn get_history_snapshots(
        &self,
        cluster_id: i64,
        time_range: &TimeRange,
    ) -> ApiResult<Vec<MetricsSnapshot>> {
        let start_time = time_range.start_time();
        let end_time = time_range.end_time();
        
        let rows = sqlx::query!(
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
        
        let snapshots = rows
            .into_iter()
            .map(|r| MetricsSnapshot {
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
                backend_total: r.backend_total as i32,
                backend_alive: r.backend_alive as i32,
                frontend_total: r.frontend_total as i32,
                frontend_alive: r.frontend_alive as i32,
                total_cpu_usage: r.total_cpu_usage,
                avg_cpu_usage: r.avg_cpu_usage,
                total_memory_usage: r.total_memory_usage,
                avg_memory_usage: r.avg_memory_usage,
                disk_total_bytes: r.disk_total_bytes,
                disk_used_bytes: r.disk_used_bytes,
                disk_usage_pct: r.disk_usage_pct,
                tablet_count: r.tablet_count,
                max_compaction_score: r.max_compaction_score,
                txn_running: r.txn_running as i32,
                txn_success_total: r.txn_success_total,
                txn_failed_total: r.txn_failed_total,
                load_running: r.load_running as i32,
                load_finished_total: r.load_finished_total,
                jvm_heap_total: r.jvm_heap_total,
                jvm_heap_used: r.jvm_heap_used,
                jvm_heap_usage_pct: r.jvm_heap_usage_pct,
                jvm_thread_count: r.jvm_thread_count as i32,
                network_bytes_sent_total: r.network_bytes_sent_total,
                network_bytes_received_total: r.network_bytes_received_total,
                network_send_rate: r.network_send_rate,
                network_receive_rate: r.network_receive_rate,
                io_read_bytes_total: r.io_read_bytes_total,
                io_write_bytes_total: r.io_write_bytes_total,
                io_read_rate: r.io_read_rate,
                io_write_rate: r.io_write_rate,
            })
            .collect();
        
        Ok(snapshots)
    }

    /// Calculate performance trends from snapshots
    fn calculate_performance_trends(&self, snapshots: &[MetricsSnapshot]) -> PerformanceTrends {
        let qps: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| TimeSeriesPoint {
                timestamp: s.collected_at,
                value: s.qps,
            })
            .collect();
        
        let latency_p99: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| TimeSeriesPoint {
                timestamp: s.collected_at,
                value: s.query_latency_p99,
            })
            .collect();
        
        let error_rate: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| {
                let total = s.query_total as f64;
                let errors = s.query_error as f64;
                let rate = if total > 0.0 { (errors / total) * 100.0 } else { 0.0 };
                TimeSeriesPoint {
                    timestamp: s.collected_at,
                    value: rate,
                }
            })
            .collect();
        
        PerformanceTrends {
            qps,
            latency_p99,
            error_rate,
        }
    }

    /// Calculate resource trends from snapshots
    fn calculate_resource_trends(&self, snapshots: &[MetricsSnapshot]) -> ResourceTrends {
        let cpu_usage: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| TimeSeriesPoint {
                timestamp: s.collected_at,
                value: s.avg_cpu_usage,
            })
            .collect();
        
        let memory_usage: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| TimeSeriesPoint {
                timestamp: s.collected_at,
                value: s.avg_memory_usage,
            })
            .collect();
        
        let disk_usage: Vec<TimeSeriesPoint> = snapshots
            .iter()
            .map(|s| TimeSeriesPoint {
                timestamp: s.collected_at,
                value: s.disk_usage_pct,
            })
            .collect();
        
        ResourceTrends {
            cpu_usage,
            memory_usage,
            disk_usage,
        }
    }

    /// Calculate aggregated statistics from snapshots
    fn calculate_aggregated_statistics(&self, snapshots: &[MetricsSnapshot]) -> AggregatedStatistics {
        if snapshots.is_empty() {
            return AggregatedStatistics {
                avg_qps: 0.0,
                max_qps: 0.0,
                avg_latency_p99: 0.0,
                avg_cpu_usage: 0.0,
                avg_memory_usage: 0.0,
                avg_disk_usage: 0.0,
            };
        }
        
        let count = snapshots.len() as f64;
        
        let avg_qps = snapshots.iter().map(|s| s.qps).sum::<f64>() / count;
        let max_qps = snapshots.iter().map(|s| s.qps).fold(0.0, f64::max);
        let avg_latency_p99 = snapshots.iter().map(|s| s.query_latency_p99).sum::<f64>() / count;
        let avg_cpu_usage = snapshots.iter().map(|s| s.avg_cpu_usage).sum::<f64>() / count;
        let avg_memory_usage = snapshots.iter().map(|s| s.avg_memory_usage).sum::<f64>() / count;
        let avg_disk_usage = snapshots.iter().map(|s| s.disk_usage_pct).sum::<f64>() / count;
        
        AggregatedStatistics {
            avg_qps,
            max_qps,
            avg_latency_p99,
            avg_cpu_usage,
            avg_memory_usage,
            avg_disk_usage,
        }
    }
}

