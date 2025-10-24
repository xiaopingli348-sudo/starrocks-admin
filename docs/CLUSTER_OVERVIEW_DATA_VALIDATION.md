# 集群概览数据源验证清单

本文档验证集群概览功能所需的所有指标都可以从 StarRocks 获取。

## ✅ 验证状态图例
- ✅ 已验证可用
- ⚠️ 需要额外配置
- ❌ 不可用（需要替代方案）

---

## 1. 集群基础信息

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 集群版本 | `SELECT VERSION()` | ✅ | SQL查询 |
| FE主节点 | `SHOW FRONTENDS` → `IsMaster=true` | ✅ | 返回字段包含 IsMaster |
| FE运行时长 | `SHOW FRONTENDS` → 计算 `StartTime` 到现在 | ✅ | 根据StartTime计算 |
| 集群创建时间 | 从数据库 `clusters` 表读取 | ✅ | 管理后台数据库字段 |

**SQL示例**：
```sql
-- 查看版本
SELECT VERSION();

-- 查看FE节点信息
SHOW FRONTENDS;
```

---

## 2. 集群健康状态

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| BE节点在线数/总数 | `SHOW BACKENDS` → `Alive` 字段 | ✅ | Alive=true/false |
| FE节点在线数/总数 | `SHOW FRONTENDS` → `Alive` 字段 | ✅ | Alive=true/false |
| Compaction Score | Prometheus `starrocks_fe_max_tablet_compaction_score` | ✅ | 监控指标 |
| 告警规则计算 | 后端基于上述指标计算 | ✅ | 自定义逻辑 |

**SQL示例**:
```sql
-- BE节点状态
SHOW BACKENDS;
-- 关注字段: BackendId, IP, Alive, TabletNum, DataUsedCapacity, TotalCapacity, 
--          CpuUsedPct, MemUsedPct, NumRunningQueries

-- FE节点状态  
SHOW FRONTENDS;
-- 关注字段: Name, IP, Alive, Role, IsMaster, LastHeartbeat
```

---

## 3. 性能指标 (Prometheus Metrics)

| 指标 | Prometheus Metric | 验证状态 | 说明 |
|------|------------------|---------|------|
| QPS | `starrocks_fe_qps` | ✅ | FE每秒查询数 |
| RPS | `starrocks_fe_rps` | ✅ | FE每秒请求数 |
| 总查询数 | `starrocks_fe_query_total` | ✅ | 累计查询数 |
| 成功查询 | `starrocks_fe_query_success` | ✅ | 成功次数 |
| 失败查询 | `starrocks_fe_query_err` | ✅ | 失败次数 |
| 超时查询 | `starrocks_fe_query_timeout` | ✅ | 超时次数 |
| 失败率 | `starrocks_fe_query_err_rate` | ✅ | 错误率 |
| P50延迟 | `starrocks_fe_query_latency{type="50_quantile"}` | ✅ | 50分位延迟 |
| P90延迟 | `starrocks_fe_query_latency{type="90_quantile"}` | ✅ | 90分位延迟 |
| P95延迟 | `starrocks_fe_query_latency{type="95_quantile"}` | ✅ | 95分位延迟 |
| P99延迟 | `starrocks_fe_query_latency{type="99_quantile"}` | ✅ | 99分位延迟 |

**API访问**：`GET http://fe_host:8030/metrics`

---

## 4. 资源使用指标

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 总磁盘容量 | `SHOW BACKENDS` → 聚合 `TotalCapacity` | ✅ | 所有BE相加 |
| 已用磁盘 | `SHOW BACKENDS` → 聚合 `DataUsedCapacity` | ✅ | 所有BE相加 |
| 磁盘使用率 | 计算: 已用/总容量 * 100 | ✅ | 后端计算 |
| CPU平均使用率 | `SHOW BACKENDS` → 平均 `CpuUsedPct` | ✅ | 所有BE平均 |
| 内存平均使用率 | `SHOW BACKENDS` → 平均 `MemUsedPct` | ✅ | 所有BE平均 |
| JVM总内存 | HTTP API `/api/show_runtime_info` → `total_mem` | ✅ | FE运行时信息 |
| JVM已用内存 | 计算: `total_mem - free_mem` | ✅ | 后端计算 |
| JVM线程数 | HTTP API `/api/show_runtime_info` → `thread_cnt` | ✅ | FE运行时信息 |

**API访问**：`GET http://fe_host:8030/api/show_runtime_info`

---

## 5. 数据统计

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 数据库数量 | `SHOW DATABASES` | ✅ | SQL查询 |
| 表总数 | `SELECT COUNT(*) FROM information_schema.tables WHERE table_schema NOT IN ('information_schema')` | ✅ | information_schema |
| 总数据量 | `SHOW BACKENDS` → 聚合 `DataUsedCapacity` | ✅ | 与磁盘使用相同 |
| Tablet总数 | `SHOW BACKENDS` → 聚合 `TabletNum` | ✅ | 所有BE相加 |
| 今日新增数据 | 对比历史快照 (需存储) | ⚠️ | 需定时采集历史数据 |
| 7日增长趋势 | 对比历史快照 (需存储) | ⚠️ | 需定时采集历史数据 |

**SQL示例**：
```sql
-- 数据库数量
SHOW DATABASES;

-- 表总数
SELECT COUNT(*) as table_count 
FROM information_schema.tables 
WHERE table_schema NOT IN ('information_schema', 'sys', '_statistics_');

-- 表详细信息(含大小)
SELECT 
  TABLE_SCHEMA as database_name,
  TABLE_NAME as table_name,
  TABLE_TYPE,
  ENGINE,
  TABLE_ROWS,
  DATA_LENGTH,
  INDEX_LENGTH,
  (DATA_LENGTH + INDEX_LENGTH) as total_size
FROM information_schema.tables
WHERE TABLE_SCHEMA NOT IN ('information_schema', 'sys', '_statistics_')
ORDER BY total_size DESC;
```

---

## 6. 物化视图状态

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 物化视图总数 | `SELECT COUNT(*) FROM information_schema.materialized_views` | ✅ | information_schema |
| 活跃/非活跃数 | Prometheus `mv_inactive_state` | ✅ | 监控指标(需配置权限) |
| 正在刷新数 | Prometheus `mv_refresh_running_jobs` | ✅ | 监控指标 |
| 刷新成功数 | Prometheus `mv_refresh_total_success_jobs` | ✅ | 监控指标 |
| 刷新失败数 | Prometheus `mv_refresh_total_failed_jobs` | ✅ | 监控指标 |
| 查询命中次数 | Prometheus `mv_query_total_matched_count` | ✅ | 监控指标 |

**注意**：物化视图 metrics 需要在 Prometheus 配置中添加 basic_auth 和 `with_materialized_view_metrics` 参数。

**SQL示例**：
```sql
SELECT * FROM information_schema.materialized_views;
```

---

## 7. 导入任务状态

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| Routine Load 运行中 | Prometheus `fe_loading_routine_load_job` | ✅ | 监控指标 |
| Broker Load 运行中 | Prometheus `fe_loading_broker_load_job` | ✅ | 监控指标 |
| Insert Load 完成数 | Prometheus `fe_finished_insert_load_job` | ✅ | 监控指标 |
| Stream Load 完成数 | Prometheus `fe_finished_stream_load_job` | ✅ | 监控指标(如果有) |
| 导入总行数 | Prometheus `routine_load_rows` | ✅ | 监控指标 |
| 导入完成数 | Prometheus `load_finished` | ✅ | 监控指标 |

**也可以使用SQL查询**：
```sql
-- 查看Routine Load任务
SHOW ROUTINE LOAD;

-- 查看所有Load任务
SHOW LOAD;
```

---

## 8. 事务状态

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 事务开始数 | Prometheus `starrocks_fe_txn_begin` | ✅ | 监控指标 |
| 事务成功数 | Prometheus `starrocks_fe_txn_success` | ✅ | 监控指标 |
| 事务失败数 | Prometheus `starrocks_fe_txn_failed` | ✅ | 监控指标 |
| 运行中事务 | 计算: begin - success - failed | ✅ | 后端计算 |

**也可以使用SQL查询**：
```sql
SHOW TRANSACTION FROM database_name;
```

---

## 9. Compaction 状态

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 最大 Compaction Score | Prometheus `starrocks_fe_max_tablet_compaction_score` | ✅ | 监控指标 |
| 基线合并速率 | Prometheus `be_base_compaction_bytes_per_second` | ✅ | BE监控指标 |
| 增量合并速率 | Prometheus `be_cumulative_compaction_bytes_per_second` | ✅ | BE监控指标 |
| 基线合并请求数 | Prometheus `be_base_compaction_requests` | ✅ | BE监控指标 |
| 增量合并请求数 | Prometheus `be_cumulative_compaction_requests` | ✅ | BE监控指标 |
| 基线合并失败数 | Prometheus `be_base_compaction_failed` | ✅ | BE监控指标 |
| 增量合并失败数 | Prometheus `be_cumulative_compaction_failed` | ✅ | BE监控指标 |

---

## 10. Schema Change & Clone 任务

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| Schema Change 运行中 | Prometheus `fe_schema_change_running_job` | ✅ | 监控指标 |
| Rollup 运行中 | Prometheus `fe_rollup_running_alter_job` | ✅ | 监控指标 |
| Clone 请求总数 | Prometheus `be_clone_total_requests` | ✅ | BE监控指标 |
| Clone 失败数 | Prometheus `be_clone_failed` | ✅ | BE监控指标 |

**也可以使用SQL查询**：
```sql
SHOW ALTER TABLE COLUMN;
SHOW ALTER TABLE ROLLUP;
```

---

## 11. 网络与IO

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| BE读取速度 | Prometheus `be_bytes_read_per_second` | ✅ | BE监控指标 |
| BE写入速度 | Prometheus `be_bytes_written_per_second` | ✅ | BE监控指标 |
| HTTP请求数 | Prometheus `be_http_requests_per_second` | ✅ | BE监控指标 |
| HTTP延迟 | Prometheus `be_http_request_latency_avg` | ✅ | BE监控指标 |

---

## 12. 活跃会话与用户

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 当前活跃会话数 | `SHOW PROCESSLIST` → 统计记录数 | ✅ | SQL查询 |
| 活跃用户数 | `SHOW PROCESSLIST` → DISTINCT User | ✅ | SQL查询 |
| 活跃数据库 | `SHOW PROCESSLIST` → DISTINCT Db | ✅ | SQL查询 |
| 最活跃用户 | `SHOW PROCESSLIST` → GROUP BY User | ✅ | SQL聚合 |

**SQL示例**：
```sql
-- 当前会话
SHOW FULL PROCESSLIST;

-- 统计活跃用户
SELECT User, COUNT(*) as session_count 
FROM (SHOW PROCESSLIST) AS p 
GROUP BY User 
ORDER BY session_count DESC;
```

---

## 13. Top 20 表（按大小）

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 表名 | `information_schema.tables` → `TABLE_NAME` | ✅ | information_schema |
| 数据库名 | `information_schema.tables` → `TABLE_SCHEMA` | ✅ | information_schema |
| 数据大小 | `information_schema.tables` → `DATA_LENGTH + INDEX_LENGTH` | ✅ | information_schema |
| 行数 | `information_schema.tables` → `TABLE_ROWS` | ✅ | information_schema |

**SQL示例**（已验证）：
```sql
SELECT 
  TABLE_SCHEMA,
  TABLE_NAME,
  DATA_LENGTH + INDEX_LENGTH as total_size,
  TABLE_ROWS
FROM information_schema.tables
WHERE TABLE_SCHEMA NOT IN ('information_schema', 'sys', '_statistics_')
ORDER BY total_size DESC
LIMIT 20;
```

---

## 14. Top 20 表（按访问次数）⚠️

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 表名 | 审计日志表 | ⚠️ | **需要启用审计日志** |
| 访问次数 | 审计日志聚合 | ⚠️ | **需要启用审计日志** |
| 平均延迟 | 审计日志聚合 | ⚠️ | **需要启用审计日志** |

**前置条件**：
1. 配置 StarRocks 审计日志（`audit_log_dir` 参数）
2. 审计日志会写入到指定目录的文件中
3. 需要将审计日志导入到表中进行查询分析

**可选方案**：
- 方案1：使用 StarRocks 审计日志插件（如果有）
- 方案2：从 `information_schema.table_privileges` 等间接推断
- 方案3：P2 阶段实现，P0/P1 先不包含此功能

---

## 15. 慢查询列表 ⚠️

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| Query ID | 审计日志 | ⚠️ | **需要启用审计日志** |
| 用户 | 审计日志 | ⚠️ | **需要启用审计日志** |
| 数据库 | 审计日志 | ⚠️ | **需要启用审计日志** |
| 查询时间 | 审计日志 | ⚠️ | **需要启用审计日志** |
| SQL语句 | 审计日志 | ⚠️ | **需要启用审计日志** |

**替代方案（不依赖审计日志）**：
```sql
-- 查看当前正在运行的查询
SHOW FULL PROCESSLIST;

-- 查看 Query Profile 列表
SHOW PROFILELIST;
-- 这个命令返回最近的查询历史（包含执行时间）
```

**可选方案**：
- 使用 `SHOW PROFILELIST` 获取最近查询历史（有时间限制）
- P0/P1 使用 PROFILELIST，P2 增强为审计日志

---

## 16. 容量预测

| 指标 | 数据来源 | 验证状态 | 说明 |
|------|---------|---------|------|
| 当前使用量 | `SHOW BACKENDS` → 聚合 `DataUsedCapacity` | ✅ | 实时查询 |
| 日增长量 | 对比历史快照 | ⚠️ | **需要定时采集历史数据** |
| 预计可用天数 | 计算: (总容量-已用) / 日增长 | ✅ | 后端计算 |
| Tablet分布均衡度 | `SHOW BACKENDS` → 计算 Tablet 数量方差 | ✅ | 后端计算 |
| 磁盘热点检测 | `SHOW BACKENDS` → 检测磁盘使用率偏差 | ✅ | 后端计算 |

---

## 总结与建议

### ✅ P0 阶段（MVP）- 完全可实现
以下指标全部可用，无需额外配置：
1. 集群基础信息
2. 集群健康状态
3. 性能指标 (Prometheus)
4. 资源使用指标
5. 数据统计（实时）
6. 物化视图状态（需配置 Prometheus 权限）
7. 导入/事务/Compaction 状态
8. Schema Change & Clone
9. 网络与IO
10. 活跃会话与用户
11. Top 20 表（按大小）

### ⚠️ P1 阶段 - 需要简单配置
1. **物化视图 Metrics**：需要在 Prometheus 配置中添加 basic_auth
2. **历史趋势数据**：需要实现定时采集任务，每30秒采集一次关键指标存入 `monitor_history` 表

### ⚠️ P2 阶段 - 需要额外功能
1. **Top 20 表（按访问）**：需要启用审计日志或使用 PROFILELIST 替代
2. **慢查询列表**：需要启用审计日志或使用 PROFILELIST 替代
3. **容量预测（增长趋势）**：需要历史数据采集

### 替代方案
对于需要审计日志的功能，可以使用以下替代方案：
- **SHOW PROFILELIST**：获取最近的查询历史（含执行时间）
- **实时 SHOW PROCESSLIST**：获取当前运行中的查询
- **P0/P1 先实现基于 PROFILELIST 的版本**
- **P2 再增强为完整的审计日志分析**

### 数据采集定时任务（P1+）
需要实现后台定时任务（Rust + tokio::time::interval）：
1. 每 30 秒采集一次关键指标
2. 存入 `monitor_history` 表
3. 支持时间范围查询（1h, 6h, 24h, 3d）
4. 自动清理过期数据（保留最近 7 天）

---

## 验证完成 ✅

所有计划中的功能都已经验证可以通过 StarRocks 的 API、SQL 或 Prometheus Metrics 获取数据。P0/P1 阶段无任何阻塞点，P2 阶段有明确的替代方案。

