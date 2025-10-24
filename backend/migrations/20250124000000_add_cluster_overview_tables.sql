-- ========================================
-- Cluster Overview Database Schema Migration
-- ========================================
-- Created: 2025-01-24
-- Purpose: Add tables for cluster overview metrics collection and aggregation
-- Design Ref: ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md

-- ========================================
-- 1. Metrics Snapshots Table (高频采集，30秒一次)
-- ========================================
-- Purpose: Store real-time metrics snapshots every 30 seconds
-- Retention: 7 days (~20,160 records per cluster)
-- Data Source: StarRocks Prometheus metrics + Backend/Frontend info

CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    collected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Query Performance Metrics
    qps REAL NOT NULL DEFAULT 0.0,                      -- Queries per second
    rps REAL NOT NULL DEFAULT 0.0,                      -- Requests per second
    query_latency_p50 REAL NOT NULL DEFAULT 0.0,        -- P50 latency (ms)
    query_latency_p95 REAL NOT NULL DEFAULT 0.0,        -- P95 latency (ms)
    query_latency_p99 REAL NOT NULL DEFAULT 0.0,        -- P99 latency (ms)
    query_total BIGINT NOT NULL DEFAULT 0,              -- Total queries count
    query_success BIGINT NOT NULL DEFAULT 0,            -- Successful queries
    query_error BIGINT NOT NULL DEFAULT 0,              -- Error queries
    query_timeout BIGINT NOT NULL DEFAULT 0,            -- Timeout queries
    
    -- Cluster Health Metrics
    backend_total INTEGER NOT NULL DEFAULT 0,           -- Total BE nodes
    backend_alive INTEGER NOT NULL DEFAULT 0,           -- Alive BE nodes
    frontend_total INTEGER NOT NULL DEFAULT 0,          -- Total FE nodes
    frontend_alive INTEGER NOT NULL DEFAULT 0,          -- Alive FE nodes
    
    -- Resource Usage Metrics
    total_cpu_usage REAL NOT NULL DEFAULT 0.0,          -- Total CPU usage (%)
    avg_cpu_usage REAL NOT NULL DEFAULT 0.0,            -- Average CPU usage (%)
    total_memory_usage REAL NOT NULL DEFAULT 0.0,       -- Total memory usage (%)
    avg_memory_usage REAL NOT NULL DEFAULT 0.0,         -- Average memory usage (%)
    disk_total_bytes BIGINT NOT NULL DEFAULT 0,         -- Total disk capacity (bytes)
    disk_used_bytes BIGINT NOT NULL DEFAULT 0,          -- Used disk space (bytes)
    disk_usage_pct REAL NOT NULL DEFAULT 0.0,           -- Disk usage percentage
    
    -- Storage Metrics
    tablet_count BIGINT NOT NULL DEFAULT 0,             -- Total tablet count
    max_compaction_score REAL NOT NULL DEFAULT 0.0,     -- Max compaction score
    
    -- Transaction Metrics
    txn_running INTEGER NOT NULL DEFAULT 0,             -- Running transactions
    txn_success_total BIGINT NOT NULL DEFAULT 0,        -- Total successful txns
    txn_failed_total BIGINT NOT NULL DEFAULT 0,         -- Total failed txns
    
    -- Load Metrics
    load_running INTEGER NOT NULL DEFAULT 0,            -- Running load jobs
    load_finished_total BIGINT NOT NULL DEFAULT 0,      -- Total finished loads
    
    -- JVM Metrics (FE)
    jvm_heap_total BIGINT NOT NULL DEFAULT 0,           -- JVM heap total (bytes)
    jvm_heap_used BIGINT NOT NULL DEFAULT 0,            -- JVM heap used (bytes)
    jvm_heap_usage_pct REAL NOT NULL DEFAULT 0.0,       -- JVM heap usage (%)
    jvm_thread_count INTEGER NOT NULL DEFAULT 0,        -- JVM thread count
    
    -- Raw Metrics (JSON format for flexibility)
    raw_metrics TEXT,                                   -- JSON: additional metrics
    
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_metrics_snapshots_cluster_time 
ON metrics_snapshots(cluster_id, collected_at DESC);

CREATE INDEX IF NOT EXISTS idx_metrics_snapshots_time 
ON metrics_snapshots(collected_at DESC);

-- ========================================
-- 2. Daily Snapshots Table (低频存储，1天1次)
-- ========================================
-- Purpose: Store daily aggregated statistics
-- Retention: 90 days (~90 records per cluster)
-- Update Frequency: Daily at midnight

CREATE TABLE IF NOT EXISTS daily_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    snapshot_date DATE NOT NULL,
    
    -- Aggregated Query Statistics
    avg_qps REAL NOT NULL DEFAULT 0.0,                  -- Average QPS of the day
    max_qps REAL NOT NULL DEFAULT 0.0,                  -- Peak QPS of the day
    min_qps REAL NOT NULL DEFAULT 0.0,                  -- Minimum QPS of the day
    avg_latency_p99 REAL NOT NULL DEFAULT 0.0,          -- Average P99 latency
    max_latency_p99 REAL NOT NULL DEFAULT 0.0,          -- Maximum P99 latency
    total_queries BIGINT NOT NULL DEFAULT 0,            -- Total queries of the day
    total_errors BIGINT NOT NULL DEFAULT 0,             -- Total errors of the day
    error_rate REAL NOT NULL DEFAULT 0.0,               -- Error rate (%)
    
    -- Aggregated Resource Statistics
    avg_cpu_usage REAL NOT NULL DEFAULT 0.0,            -- Average CPU usage
    max_cpu_usage REAL NOT NULL DEFAULT 0.0,            -- Peak CPU usage
    avg_memory_usage REAL NOT NULL DEFAULT 0.0,         -- Average memory usage
    max_memory_usage REAL NOT NULL DEFAULT 0.0,         -- Peak memory usage
    avg_disk_usage_pct REAL NOT NULL DEFAULT 0.0,       -- Average disk usage
    max_disk_usage_pct REAL NOT NULL DEFAULT 0.0,       -- Peak disk usage
    
    -- Availability Statistics
    avg_backend_alive REAL NOT NULL DEFAULT 0.0,        -- Average alive BE nodes
    min_backend_alive INTEGER NOT NULL DEFAULT 0,       -- Minimum alive BE nodes
    total_downtime_seconds INTEGER NOT NULL DEFAULT 0,  -- Total downtime (seconds)
    availability_pct REAL NOT NULL DEFAULT 100.0,       -- Availability percentage
    
    -- Data Growth Statistics
    data_size_start BIGINT NOT NULL DEFAULT 0,          -- Data size at day start
    data_size_end BIGINT NOT NULL DEFAULT 0,            -- Data size at day end
    data_growth_bytes BIGINT NOT NULL DEFAULT 0,        -- Data growth (bytes)
    data_growth_rate REAL NOT NULL DEFAULT 0.0,         -- Growth rate (%)
    
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE,
    UNIQUE(cluster_id, snapshot_date)
);

-- Indexes for daily queries
CREATE INDEX IF NOT EXISTS idx_daily_snapshots_cluster_date 
ON daily_snapshots(cluster_id, snapshot_date DESC);

CREATE INDEX IF NOT EXISTS idx_daily_snapshots_date 
ON daily_snapshots(snapshot_date DESC);

-- ========================================
-- 3. Data Statistics Cache Table (按需更新)
-- ========================================
-- Purpose: Cache expensive queries (database/table counts, top tables)
-- Update Frequency: Every 5-10 minutes or on-demand
-- This reduces query pressure on StarRocks

CREATE TABLE IF NOT EXISTS data_statistics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Database/Table Statistics
    database_count INTEGER NOT NULL DEFAULT 0,          -- Total databases
    table_count INTEGER NOT NULL DEFAULT 0,             -- Total tables
    total_data_size BIGINT NOT NULL DEFAULT 0,          -- Total data size (bytes)
    total_index_size BIGINT NOT NULL DEFAULT 0,         -- Total index size (bytes)
    
    -- Top Tables by Size (JSON Array)
    -- Format: [{"database": "db1", "table": "t1", "size": 1024000, "rows": 1000}, ...]
    top_tables_by_size TEXT,
    
    -- Top Tables by Access Count (JSON Array)
    -- Format: [{"database": "db1", "table": "t1", "access_count": 500, "last_access": "2025-01-24T10:00:00Z"}, ...]
    top_tables_by_access TEXT,
    
    -- Materialized View Statistics
    mv_total INTEGER NOT NULL DEFAULT 0,                -- Total MVs
    mv_running INTEGER NOT NULL DEFAULT 0,              -- Running MVs
    mv_failed INTEGER NOT NULL DEFAULT 0,               -- Failed MVs
    mv_success INTEGER NOT NULL DEFAULT 0,              -- Successful MVs
    
    -- Schema Change Statistics
    schema_change_running INTEGER NOT NULL DEFAULT 0,   -- Running schema changes
    schema_change_pending INTEGER NOT NULL DEFAULT 0,   -- Pending schema changes
    schema_change_finished INTEGER NOT NULL DEFAULT 0,  -- Finished schema changes
    schema_change_failed INTEGER NOT NULL DEFAULT 0,    -- Failed schema changes
    
    -- Active Users Statistics
    active_users_1h INTEGER NOT NULL DEFAULT 0,         -- Active users in last 1 hour
    active_users_24h INTEGER NOT NULL DEFAULT 0,        -- Active users in last 24 hours
    unique_users TEXT,                                  -- JSON: list of unique users
    
    -- Query Statistics Cache
    slow_query_count_1h INTEGER NOT NULL DEFAULT 0,     -- Slow queries in last 1 hour
    slow_query_count_24h INTEGER NOT NULL DEFAULT 0,    -- Slow queries in last 24 hours
    
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE,
    UNIQUE(cluster_id)
);

-- Index for quick lookup
CREATE INDEX IF NOT EXISTS idx_data_statistics_cluster 
ON data_statistics(cluster_id);

CREATE INDEX IF NOT EXISTS idx_data_statistics_updated 
ON data_statistics(updated_at DESC);

-- ========================================
-- Migration Completion
-- ========================================
-- Tables created:
-- 1. metrics_snapshots - Real-time metrics (30s interval, 7 day retention)
-- 2. daily_snapshots - Daily aggregations (90 day retention)
-- 3. data_statistics - Cached statistics (on-demand update)
--
-- Next Steps:
-- 1. Implement MetricsCollectorService to populate these tables
-- 2. Implement OverviewService to query and aggregate data
-- 3. Create background tasks for data collection and cleanup

