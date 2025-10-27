-- Complete Database Schema Migration
-- This migration creates all tables and inserts default data

-- ==============================================
-- 1. Create users table
-- ==============================================
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100),
    avatar VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create index on username
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- ==============================================
-- 2. Create clusters table
-- ==============================================
CREATE TABLE IF NOT EXISTS clusters (
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
    created_by INTEGER
    -- Removed foreign key constraint to allow insertion without user reference
);

-- Create index on name
CREATE INDEX IF NOT EXISTS idx_clusters_name ON clusters(name);

-- ==============================================
-- 3. Create monitor_history table
-- ==============================================
CREATE TABLE IF NOT EXISTS monitor_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value TEXT NOT NULL,
    collected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cluster_id) REFERENCES clusters(id) ON DELETE CASCADE
);

-- Create index on cluster_id and metric_name for faster queries
CREATE INDEX IF NOT EXISTS idx_monitor_history_cluster_metric ON monitor_history(cluster_id, metric_name);
CREATE INDEX IF NOT EXISTS idx_monitor_history_collected_at ON monitor_history(collected_at);

-- ==============================================
-- 4. Create system_functions table
-- ==============================================
CREATE TABLE IF NOT EXISTS system_functions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NULL, -- NULL 表示系统默认功能，属于所有集群
    category_name TEXT NOT NULL,
    function_name TEXT NOT NULL,
    description TEXT NOT NULL,
    sql_query TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    category_order INTEGER NOT NULL DEFAULT 0,
    is_favorited BOOLEAN NOT NULL DEFAULT FALSE,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cluster_id) REFERENCES clusters (id) ON DELETE CASCADE
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_system_functions_cluster_id ON system_functions (cluster_id);
CREATE INDEX IF NOT EXISTS idx_system_functions_category_order ON system_functions (category_order);
CREATE INDEX IF NOT EXISTS idx_system_functions_display_order ON system_functions (display_order);
CREATE INDEX IF NOT EXISTS idx_system_functions_is_system ON system_functions (is_system);

-- ==============================================
-- 5. Create system_function_preferences table
-- ==============================================
-- 创建系统功能偏好设置表（统一管理所有功能的排序）
CREATE TABLE IF NOT EXISTS system_function_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    function_id INTEGER NOT NULL,  -- 指向 system_functions.id（系统或自定义功能）
    category_order INTEGER NOT NULL DEFAULT 0,
    display_order INTEGER NOT NULL DEFAULT 0,
    is_favorited BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(cluster_id, function_id),
    FOREIGN KEY (cluster_id) REFERENCES clusters (id) ON DELETE CASCADE,
    FOREIGN KEY (function_id) REFERENCES system_functions (id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_preferences_cluster_id ON system_function_preferences (cluster_id);
CREATE INDEX IF NOT EXISTS idx_preferences_function_id ON system_function_preferences (function_id);
CREATE INDEX IF NOT EXISTS idx_preferences_cluster_function ON system_function_preferences (cluster_id, function_id);

-- ==============================================
-- 6. Insert default admin user
-- ==============================================
-- Password: admin (bcrypt hash with DEFAULT_COST=12)
-- Hash generated using: bcrypt::hash("admin", DEFAULT_COST)
INSERT OR IGNORE INTO users (username, password_hash, email)
VALUES ('admin', '$2b$12$LFxvzXbmyBPO9Zp.1MFU4OX3fb8kID8AHYHklokkZvgyzmHuRTc56', 'admin@example.com');

-- ==============================================
-- 7. Insert default system functions
-- ==============================================
-- 系统默认功能（cluster_id 为 NULL，属于所有集群）
INSERT OR IGNORE INTO system_functions (cluster_id, category_name, function_name, description, sql_query, display_order, category_order, is_favorited, is_system, created_by, created_at, updated_at) VALUES
-- 数据库管理
(NULL, '数据库管理', 'tables', '表信息', 'HTTP_QUERY', 0, 0, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '数据库管理', 'dbs', '数据库信息', 'HTTP_QUERY', 1, 0, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '数据库管理', 'tablet_schema', 'Tablet Schema', 'HTTP_QUERY', 2, 0, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '数据库管理', 'partitions', '分区信息', 'HTTP_QUERY', 3, 0, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 集群信息
(NULL, '集群信息', 'backends', 'Backend节点信息', 'HTTP_QUERY', 0, 1, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '集群信息', 'frontends', 'Frontend节点信息', 'HTTP_QUERY', 1, 1, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '集群信息', 'brokers', 'Broker节点信息', 'HTTP_QUERY', 2, 1, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '集群信息', 'statistic', '统计信息', 'HTTP_QUERY', 3, 1, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 事务管理
(NULL, '事务管理', 'transactions', '事务信息', 'HTTP_QUERY', 0, 2, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 任务管理
(NULL, '任务管理', 'routine_loads', 'Routine Load任务', 'HTTP_QUERY', 0, 3, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '任务管理', 'stream_loads', 'Stream Load任务', 'HTTP_QUERY', 1, 3, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '任务管理', 'loads', 'Load任务', 'HTTP_QUERY', 2, 3, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '任务管理', 'load_error_hub', 'Load错误信息', 'HTTP_QUERY', 3, 3, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 元数据管理
(NULL, '元数据管理', 'catalog', 'Catalog信息', 'HTTP_QUERY', 0, 4, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '元数据管理', 'resources', '资源信息', 'HTTP_QUERY', 1, 4, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '元数据管理', 'workgroups', '工作组信息', 'HTTP_QUERY', 2, 4, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '元数据管理', 'warehouses', '数据仓库信息', 'HTTP_QUERY', 3, 4, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 存储管理
(NULL, '存储管理', 'tablets', 'Tablet信息', 'HTTP_QUERY', 0, 5, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '存储管理', 'colocate_group', 'Colocate Group信息', 'HTTP_QUERY', 1, 5, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '存储管理', 'compactions', '压缩任务信息', 'HTTP_QUERY', 2, 5, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '存储管理', 'routine_load_jobs', 'Routine Load作业信息', 'HTTP_QUERY', 3, 5, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),

-- 作业管理
(NULL, '作业管理', 'jobs', '作业信息', 'HTTP_QUERY', 0, 6, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '作业管理', 'tasks', '任务信息', 'HTTP_QUERY', 1, 6, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '作业管理', 'routine_load_jobs', 'Routine Load作业', 'HTTP_QUERY', 2, 6, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
(NULL, '作业管理', 'stream_load_jobs', 'Stream Load作业', 'HTTP_QUERY', 3, 6, false, true, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);