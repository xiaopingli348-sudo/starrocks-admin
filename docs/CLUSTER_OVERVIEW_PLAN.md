# 集群概览功能设计与实现计划 ⭐️

> **版本**: v2.0  
> **更新日期**: 2025-10-24  
> **文档类型**: 📋 核心实施计划（总纲）  
> **核心变更**: 不依赖 Prometheus，自主采集所有指标数据

---

## 📚 文档导航

本文档是**集群概览功能的核心实施计划**，包含完整的需求分析、架构设计、实施步骤。

### 相关辅助文档

本计划依赖以下辅助文档，实施时请参考：

```
CLUSTER_OVERVIEW_PLAN.md (本文档 ⭐️)
    │
    ├─► CLUSTER_OVERVIEW_DATA_VALIDATION.md
    │   └─ 用途：验证所有指标的数据来源可用性
    │   └─ 阅读时机：实施前，确认技术可行性
    │
    ├─► METRICS_COLLECTION_DESIGN.md
    │   └─ 用途：后端自主采集系统的详细技术设计
    │   └─ 阅读时机：实施 P0 后端部分时参考
    │
    └─► FRONTEND_VISUAL_DESIGN.md
        └─ 用途：前端组件视觉设计参考（基于 ngx-admin）
        └─ ⚠️ 重要：优先使用 ngx-admin 原生组件
        └─ 阅读时机：实施前端页面时参考
```

### 快速导航

- **快速了解**：阅读"一、设计理念"和"二、核心架构设计"
- **开始实施**：跳转到"七、实施优先级与阶段划分"
- **技术细节**：参考对应章节或辅助文档

### 推荐阅读路径

#### 对于新读者（第一次接触）
1. **先读**：`README_CLUSTER_OVERVIEW.md`（3分钟，了解文档体系）
2. **再读**：本文档的"一、设计理念"和"二、核心架构设计"（10分钟）
3. **参考**：根据实施阶段查阅对应的辅助文档

#### 对于项目经理/产品经理
1. 阅读本文档的"一、设计理念"
2. 查看"三、页面布局架构"
3. 了解"七、实施优先级与阶段划分"
4. 关注"核心设计决策回顾"表格

#### 对于后端开发（⚠️ 重要）
1. ⭐️ **必读**：`ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md`（架构分析）
2. 先读本文档的"五、后端实现"部分
3. **详细阅读** `METRICS_COLLECTION_DESIGN.md`
4. 查看 `CLUSTER_OVERVIEW_DATA_VALIDATION.md` 确认数据源
5. 参考"八、技术要点"进行优化

#### 对于前端开发
1. ⭐️ **必读**：本文档的"六、前端实现 → 核心实施原则"
2. **详细阅读** `FRONTEND_VISUAL_DESIGN.md` 的实施原则部分
3. 了解 ngx-admin 的组件使用方式
4. 熟悉 Nebular 官方文档（https://akveo.github.io/nebular/）
5. 确认 ECharts 已集成

#### 对于测试工程师
1. 阅读本文档的"九、测试计划"
2. 参考各模块的数据来源进行测试设计
3. 查看 `CLUSTER_OVERVIEW_DATA_VALIDATION.md` 了解数据验证点

---

## 一、设计理念

基于业界成熟监控系统（Grafana、Datadog、New Relic）的设计思路，结合 StarRocks OLAP 数据库特性，打造一个**信息密度高、视觉直观、可操作性强、酷炫现代**的集群概览页面。

### 核心目标

1. ✅ 管理员在 30 秒内了解集群的健康状态、性能表现、资源使用和潜在问题
2. ✅ **所有指标必须可以从 StarRocks 直接获取**（不依赖 Prometheus、Grafana 等外部组件）
3. ✅ 前端设计要现代、酷炫，合理使用各种图表类型
4. ✅ 强交互性，所有关键指标可点击跳转到详情页面
5. ✅ 支持历史数据查询和趋势分析（最长 3 天）

### 设计风格

- 🎨 **基于 ngx-admin**：使用 Nebular 组件库，保持风格一致
- 📊 **数据驱动**：用图表讲故事，不是简单堆砌数字
- 🔄 **实时感**：30秒自动刷新 + 适度的数字动画
- 🎯 **重点突出**：使用 Nebular status 颜色（success/warning/danger）
- 💫 **交互丰富**：Hover 显示详情，Click 跳转管理页
- 🌈 **颜色语义**：使用 Nebular 标准颜色系统

---

## 二、核心架构设计

### 2.1 数据采集架构（⭐️ 核心变更）

**原则**：❌ 不依赖 Prometheus  ✅ 自主采集存储

```
┌─────────────────────────────────────────┐
│           前端 Dashboard                │
└────────────┬────────────────────────────┘
             │ HTTP API
             ↓
┌─────────────────────────────────────────┐
│      OverviewService (数据聚合)         │
│  ┌──────────────┬───────────────────┐   │
│  │ 实时查询      │  历史数据查询     │   │
│  └──────┬───────┴────────┬──────────┘   │
└─────────┼────────────────┼──────────────┘
          │                │
          ↓                ↓
┌──────────────────┐  ┌────────────────┐
│ StarRocksClient  │  │MetricsRepository│
│(实时查询 SR)     │  │(查询历史数据)  │
└──────────────────┘  └────────────────┘
          ↑                │
          │                ↓
┌─────────┴────────┐  ┌────────────────┐
│ StarRocks 集群   │  │ SQLite 数据库  │
│- SHOW BACKENDS   │  │- metrics_      │
│- SHOW FRONTENDS  │  │  snapshots     │
│- HTTP /metrics   │  │- daily_        │
│- SQL查询         │  │  snapshots     │
└──────────────────┘  └────────────────┘
          ↑                ↑
          │                │
          └────────────────┘
                  │
       ┌──────────┴──────────┐
       │  MetricsCollector   │
       │  (后台定时任务)      │
       │  - 每30秒采集一次    │
       │  - 存储到 SQLite    │
       │  - 自动清理过期数据  │
       └─────────────────────┘
```

### 2.2 数据来源

| 数据类型 | 数据来源 | 说明 |
|---------|---------|------|
| 节点状态 | `SHOW BACKENDS/FRONTENDS` | BE/FE 节点信息 |
| 性能指标 | HTTP `/metrics` 端口8030 | Prometheus 格式指标 |
| 元数据 | `information_schema` | 数据库、表、物化视图 |
| 会话信息 | `SHOW PROCESSLIST` | 当前活跃查询 |
| 任务状态 | SQL 查询 | Routine Load、事务等 |
| 历史数据 | SQLite `metrics_snapshots` | 自采集时间序列数据 |

### 2.3 数据库设计

#### metrics_snapshots 表（时间序列快照）
```sql
CREATE TABLE metrics_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    collected_at TIMESTAMP NOT NULL,
    
    -- 查询性能指标
    query_total BIGINT,
    query_success BIGINT,
    query_error BIGINT,
    query_latency_p50 REAL,
    query_latency_p90 REAL,
    query_latency_p95 REAL,
    query_latency_p99 REAL,
    qps REAL,
    rps REAL,
    
    -- 资源使用
    disk_total_bytes BIGINT,
    disk_used_bytes BIGINT,
    cpu_usage_pct REAL,
    mem_usage_pct REAL,
    
    -- 集群状态
    be_node_total INTEGER,
    be_node_alive INTEGER,
    running_queries INTEGER,
    
    -- 更多字段见 METRICS_COLLECTION_DESIGN.md
    
    FOREIGN KEY (cluster_id) REFERENCES clusters(id)
);
```

#### daily_snapshots 表（每日汇总）
```sql
CREATE TABLE daily_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cluster_id INTEGER NOT NULL,
    snapshot_date DATE NOT NULL,
    
    -- 数据量
    total_data_bytes BIGINT,
    daily_data_growth_bytes BIGINT,
    
    -- 每日统计
    daily_query_total BIGINT,
    avg_qps REAL,
    avg_p95_latency REAL,
    
    UNIQUE(cluster_id, snapshot_date),
    FOREIGN KEY (cluster_id) REFERENCES clusters(id)
);
```

---

## 三、页面布局架构

### 3.1 整体结构

**方案**：升级现有 `dashboard` 为全功能集群概览，删除 `monitor` 页面

```
┌──────────────────────────────────────────────────┐
│ 顶部控制栏 (玻璃态效果)                           │
│ [集群选择器 ▼] [时间范围: 1h/6h/24h/3d]          │
│ [自动刷新: 30s ▼] [🔄 手动刷新] [最后更新: 5秒前] │
└──────────────────────────────────────────────────┘

第一屏（核心健康状态）
├── 集群基础信息卡片（新增）
├── 集群健康总览卡片（Hero Card，发光效果）
├── 关键性能指标 KPI（5个并排，可点击跳转，数字跳动动画）
│   ├── QPS → 查询管理
│   ├── P99延迟 → 查询管理
│   ├── 成功率 → 查询管理
│   ├── 在线节点 → Backend节点管理
│   └── 运行查询 → 查询管理
├── 查询性能趋势图（双Y轴折线图，渐变填充）
└── 资源使用状态（仪表盘 + 饼图，可点击跳转）

第二屏（深入分析）
├── 数据统计概览（带迷你图）
├── 活跃会话与用户（新增，可点击 → 会话管理）
├── 物化视图状态（可点击 → 物化视图管理）
├── 导入任务状态（可点击 → 系统管理）
├── 事务状态
├── Schema Change & Clone 任务（新增，可点击 → 系统管理）
├── Compaction 状态（可点击 → Backend节点管理）
├── 网络与IO（新增）
├── Top 20 表（按大小，渐变进度条，可点击）
├── Top 20 表（按访问次数，P2阶段，可点击）
├── 慢查询列表（可点击 → 查询历史）
├── 容量预测与告警（新增，可点击 → Backend节点管理）
└── 智能告警与建议（闪烁动画）
```

### 3.2 与现有页面的关系

**删除**：
- ❌ `monitor` 页面（功能被集群概览替代）
- ❌ `monitor` 路由配置
- ❌ 侧边栏"监控指标"菜单项

**升级**：
- ✅ `dashboard` → 全功能集群概览
- ✅ 保留集群选择能力
- ✅ 整合 `monitor` 的图表和指标
- ✅ 增加可点击跳转的交互
- ✅ 增加自主数据采集系统

---

## 四、核心模块详细设计

### 模块 1：集群基础信息卡片（新增）

**展示内容**：
- 集群名称
- StarRocks 版本
- 运行时长（Uptime）
- FE主节点（标注 ⭐）
- 创建时间

**数据来源**：
```sql
-- 版本
SELECT VERSION();

-- FE节点信息（包含主节点）
SHOW FRONTENDS;  -- IsMaster=true

-- 运行时长通过 StartTime 计算
```

---

### 模块 2：集群健康总览卡片（Hero Card）

**视觉效果**：
- 超大卡片，渐变背景
- 健康状态大图标，发光脉冲动画
- 状态颜色：绿色/黄色/红色

**告警规则**：
- 🔴 危险：任何 BE 节点离线，Compaction Score > 100
- 🟡 警告：Compaction Score > 50，磁盘使用 > 80%
- 🟢 健康：其他情况

**数据来源**：
- BE/FE状态：`SHOW BACKENDS/FRONTENDS`
- Compaction Score：HTTP `/metrics` → `starrocks_fe_max_tablet_compaction_score`
- 告警：后端基于规则计算

---

### 模块 3：关键性能指标 KPI（5个卡片）

**指标项**：
1. **QPS**：当前每秒查询数 + 环比变化 ↑12%
2. **P99延迟**：99分位查询延迟 + 环比变化 ↓8%
3. **成功率**：查询成功率 + 环比变化 ↑0.1%
4. **在线节点**：BE节点在线数/总数
5. **运行查询**：当前运行中查询数 + 环比变化

**视觉效果**：
- 数字超大显示（48px），渐变色
- CountUp.js 数字跳动动画
- 趋势箭头（绿色向上，红色向下）
- Hover 放大 + 发光效果
- 可点击跳转

**数据来源**：
- QPS: HTTP `/metrics` → `starrocks_fe_qps`
- P99: HTTP `/metrics` → `starrocks_fe_query_latency{type="99_quantile"}`
- 成功率: 计算 `(query_success / query_total) * 100`
- 在线节点: `SHOW BACKENDS` → 聚合 `Alive`
- 运行查询: `SHOW BACKENDS` → 聚合 `NumRunningQueries`

**趋势计算**：
```rust
// 与 5 分钟前的值对比
let trend = ((current - previous) / previous) * 100.0;
```

---

### 模块 4：查询性能趋势图

**图表类型**：ECharts 双Y轴折线图

**展示曲线**：
- QPS（蓝色）- 左Y轴，渐变填充
- P90延迟（橙色）- 右Y轴
- P99延迟（红色，虚线）- 右Y轴

**时间范围**：1h / 6h / 24h / 3d

**数据来源**：
- **实时模式**（默认）：查询 `metrics_snapshots` 表最近数据
- **历史模式**：查询指定时间范围内的所有快照

```rust
// API: GET /api/clusters/:id/metrics/timeseries?range=1h&metrics=qps,p90,p99
pub async fn get_timeseries_metrics(
    cluster_id: i64,
    time_range: &str,  // "1h", "6h", "24h", "3d"
    metrics: Vec<String>,
) -> ApiResult<Vec<MetricsSnapshot>>
```

---

### 模块 5：资源使用状态

**展示方式**：
- 三个半圆仪表盘（ECharts Gauge）
- 饼图（各BE节点磁盘分布）

**指标项**：
- 磁盘：使用率 82% + 8.2TB/10TB
- 内存：使用率 45% + 36GB/80GB
- CPU：平均使用率 23%

**颜色规则**：
- 🟢 绿色：< 60%
- 🟡 黄色：60% - 80%
- 🔴 红色：> 80%

**数据来源**：
```sql
SHOW BACKENDS;
-- 聚合计算:
-- disk_total = SUM(TotalCapacity)
-- disk_used = SUM(DataUsedCapacity)
-- cpu_avg = AVG(CpuUsedPct)
-- mem_avg = AVG(MemUsedPct)
```

---

### 模块 6：数据统计概览

**指标项**：
- 数据库数量：15
- 表总数：1,234
- 总数据量：125.6 TB
- Tablet总数：456,789
- 今日新增：↑ 2.3 TB（带迷你折线图）
- 近7日增长：↑ 15.8 TB

**数据来源**：
```sql
-- 数据库数
SHOW DATABASES;

-- 表总数
SELECT COUNT(*) FROM information_schema.tables
WHERE table_schema NOT IN ('information_schema', 'sys', '_statistics_');

-- 总数据量、Tablet数：SHOW BACKENDS 聚合
-- 增长数据：从 daily_snapshots 查询
```

---

### 模块 7：物化视图状态

**指标项**：
- 总数 / 活跃数 / 非活跃数
- 正在刷新：3
- 刷新成功率：98.5%
- 今日刷新：156 成功 / 2 失败
- 查询命中：2,345 次

**数据来源**：
```sql
-- 总数
SELECT COUNT(*) FROM information_schema.materialized_views;

-- 刷新状态：HTTP /metrics
-- mv_refresh_running_jobs
-- mv_refresh_total_success_jobs
-- mv_refresh_total_failed_jobs
-- mv_query_total_matched_count
```

**注意**：需要在 Prometheus 配置中添加 `with_materialized_view_metrics`

---

### 模块 8-13：其他模块

详细设计见完整文档，包括：
- 导入任务状态
- 事务状态
- Schema Change & Clone
- Compaction 状态
- Top 20 表（按大小）
- 慢查询列表
- 容量预测
- 智能告警

---

## 五、后端实现

### 5.1 新增模型

**文件**：`backend/src/models/overview.rs`

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ClusterOverview {
    pub cluster_info: ClusterBasicInfo,
    pub health: ClusterHealthOverview,
    pub performance: PerformanceMetrics,
    pub resources: ResourceUsage,
    pub data_stats: DataStatistics,
    pub mv_stats: MaterializedViewStats,
    pub load_jobs: LoadJobStats,
    pub transactions: TransactionStats,
    pub compaction: CompactionStats,
    pub schema_change: SchemaChangeStats,
    pub sessions: SessionStats,
    pub network_io: NetworkIOStats,
    pub top_tables_by_size: Vec<TopTable>,
    pub top_tables_by_access: Option<Vec<TopTable>>,  // P2
    pub slow_queries: Option<Vec<SlowQuery>>,  // P2
    pub capacity_prediction: CapacityPrediction,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClusterBasicInfo {
    pub name: String,
    pub version: String,
    pub uptime_seconds: i64,
    pub master_fe: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PerformanceMetrics {
    pub qps: f64,
    pub qps_trend: f64,  // 环比变化百分比
    pub p99_latency_ms: f64,
    pub p99_trend: f64,
    pub success_rate: f64,
    pub success_rate_trend: f64,
    pub running_queries: i32,
    pub running_queries_trend: f64,
}

// 更多结构体定义...
```

### 5.2 MetricsCollector（定时采集器）

**文件**：`backend/src/services/metrics_collector.rs`

```rust
pub struct MetricsCollector {
    db: SqlitePool,
    cluster_service: Arc<ClusterService>,
}

impl MetricsCollector {
    /// 启动后台采集任务
    pub async fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if let Err(e) = self.collect_all_clusters().await {
                    tracing::error!("Metrics collection error: {}", e);
                }
            }
        });
    }
    
    /// 采集单个集群指标
    async fn collect_cluster_metrics(&self, cluster_id: i64) -> ApiResult<()> {
        let cluster = self.cluster_service.get_cluster(cluster_id).await?;
        let client = StarRocksClient::new(cluster);
        
        // 并发获取所有数据源
        let (backends, frontends, metrics_text) = tokio::try_join!(
            client.get_backends(),
            client.get_frontends(),
            client.get_metrics(),  // HTTP /metrics
        )?;
        
        // 解析 Prometheus 指标
        let metrics_map = client.parse_prometheus_metrics(&metrics_text)?;
        
        // 构建快照
        let snapshot = self.build_snapshot(cluster_id, &backends, &frontends, &metrics_map).await?;
        
        // 保存到数据库
        self.save_snapshot(&snapshot).await?;
        
        Ok(())
    }
}
```

### 5.3 OverviewService

**文件**：`backend/src/services/overview_service.rs`

```rust
pub struct OverviewService {
    db: SqlitePool,
    cluster_service: Arc<ClusterService>,
    metrics_repo: Arc<MetricsRepository>,
}

impl OverviewService {
    pub async fn get_cluster_overview(
        &self,
        cluster_id: i64,
        time_range: TimeRange,
    ) -> ApiResult<ClusterOverview> {
        // 获取实时数据
        let cluster = self.cluster_service.get_cluster(cluster_id).await?;
        let client = StarRocksClient::new(cluster);
        
        // 并发获取所有数据
        let (backends, frontends, processlist, metrics_text, mv_count, db_count) = 
            tokio::try_join!(
                client.get_backends(),
                client.get_frontends(),
                client.query("SHOW PROCESSLIST"),
                client.get_metrics(),
                client.query("SELECT COUNT(*) FROM information_schema.materialized_views"),
                client.query("SHOW DATABASES"),
            )?;
        
        // 获取历史数据（用于趋势计算）
        let historical = self.metrics_repo.get_recent_snapshots(cluster_id, 2).await?;
        
        // 聚合计算所有指标
        let overview = self.build_overview(
            cluster,
            backends,
            frontends,
            processlist,
            metrics_text,
            historical,
        ).await?;
        
        Ok(overview)
    }
}
```

### 5.4 API Handler

**文件**：`backend/src/handlers/overview.rs`

```rust
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview",
    params(
        ("id" = i64, Path, description = "Cluster ID"),
        ("time_range" = Option<String>, Query, description = "1h|6h|24h|3d")
    ),
    responses(
        (status = 200, description = "Cluster overview", body = ClusterOverview)
    ),
    tag = "Overview"
)]
pub async fn get_cluster_overview(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<OverviewQuery>,
) -> ApiResult<Json<ClusterOverview>> {
    let time_range = params.time_range.unwrap_or_else(|| "1h".to_string());
    let time_range = TimeRange::from_str(&time_range)?;
    
    let overview_service = OverviewService::new(
        state.db.clone(),
        state.cluster_service.clone(),
        state.metrics_repo.clone(),
    );
    
    let overview = overview_service.get_cluster_overview(cluster_id, time_range).await?;
    
    Ok(Json(overview))
}

/// 获取时间序列数据（用于图表）
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/metrics/timeseries",
    params(
        ("id" = i64, Path),
        ("range" = String, Query, description = "1h|6h|24h|3d"),
        ("metrics" = String, Query, description = "qps,p90,p99")
    ),
    tag = "Overview"
)]
pub async fn get_timeseries_metrics(
    State(metrics_repo): State<Arc<MetricsRepository>>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<TimeSeriesQuery>,
) -> ApiResult<Json<Vec<MetricsSnapshot>>> {
    let snapshots = metrics_repo.query_metrics(cluster_id, params.range).await?;
    Ok(Json(snapshots))
}
```

---

## 六、前端实现

### ⚠️ 核心实施原则（必读）

#### 组件选用优先级

1. **优先使用 ngx-admin 原生组件**（最高优先级）
   - ✅ `nb-card`, `nb-card-header`, `nb-card-body`, `nb-card-footer`
   - ✅ `nb-button`, `nb-select`, `nb-checkbox`, `nb-radio`
   - ✅ `nb-progress-bar`, `nb-badge`, `nb-alert`
   - ✅ `nb-icon` (Eva Icons)
   - ✅ `ng2-smart-table`（表格）
   - ✅ `echarts`（ngx-admin 已集成）

2. **使用 Nebular 主题系统**
   ```typescript
   // ✅ 使用 status 属性
   <nb-card status="success">  // 健康
   <nb-card status="warning">  // 警告
   <nb-card status="danger">   // 危险
   ```

   ```scss
   // ✅ 使用主题变量
   .custom-style {
     background-color: nb-theme(card-background-color);
     color: nb-theme(color-success-default);
   }
   ```

3. **必要时自定义（但需遵循规则）**
   - ⚠️ 自定义样式必须基于 Nebular 主题变量
   - ⚠️ 自定义组件要与 ngx-admin 风格兼容
   - ⚠️ 避免破坏响应式布局
   - ⚠️ 确保暗色主题兼容

#### 实施原则总结

**"不轻易自定义 ≠ 不能自定义"**

- ✅ 能用原生组件就用原生
- ✅ 需要自定义时，使用主题变量保证风格统一
- ✅ 简单动画可以添加（transition, animation）
- ❌ 不要创建完全独立的 CSS 样式体系
- ❌ 不要引入与 ngx-admin 冲突的 UI 库
- ❌ 不要自定义颜色变量（使用 Nebular 的）

#### 示例对比

**✅ 正确做法**：
```html
<!-- 使用 Nebular 组件 + status -->
<nb-card status="success">
  <nb-card-header>集群健康</nb-card-header>
  <nb-card-body>
    <nb-icon icon="checkmark-circle-2-outline" status="success"></nb-icon>
    <h3>健康</h3>
  </nb-card-body>
</nb-card>
```

```scss
// 使用主题变量
.health-card {
  background-color: nb-theme(card-background-color);
  border-left: 3px solid nb-theme(color-success-default);
  
  // 适度动画
  transition: transform 0.3s;
  &:hover {
    transform: translateY(-4px);
  }
}
```

**❌ 错误做法**：
```css
/* 不要自定义颜色变量 */
:root {
  --my-custom-color: #00ff00;
}

/* 不要创建独立样式体系 */
.my-custom-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 0 30px rgba(102, 126, 234, 0.5);
}
```

---

### 6.1 主组件

**文件**：`frontend/src/app/pages/starrocks/cluster-overview/cluster-overview.component.ts`

```typescript
@Component({
  selector: 'ngx-cluster-overview',
  templateUrl: './cluster-overview.component.html',
  styleUrls: ['./cluster-overview.component.scss'],
})
export class ClusterOverviewComponent implements OnInit, OnDestroy {
  overview: ClusterOverview | null = null;
  clusterId: number;
  timeRange: string = '1h';
  loading = false;
  
  private destroy$ = new Subject<void>();
  
  constructor(
    private overviewService: OverviewService,
    private clusterContext: ClusterContextService,
    private router: Router,
  ) {}
  
  ngOnInit() {
    // 监听活跃集群变化
    this.clusterContext.activeCluster$
      .pipe(takeUntil(this.destroy$))
      .subscribe(cluster => {
        if (cluster) {
          this.clusterId = cluster.id;
          this.loadOverview();
        }
      });
    
    // 30秒自动刷新
    interval(30000)
      .pipe(
        takeUntil(this.destroy$),
        switchMap(() => this.overviewService.getClusterOverview(this.clusterId, this.timeRange))
      )
      .subscribe(data => {
        this.overview = data;
        this.updateCharts();
      });
  }
  
  loadOverview() {
    this.loading = true;
    this.overviewService.getClusterOverview(this.clusterId, this.timeRange)
      .subscribe({
        next: (data) => {
          this.overview = data;
          this.loading = false;
          this.initCharts();
        },
        error: (err) => {
          this.loading = false;
          console.error(err);
        }
      });
  }
  
  onTimeRangeChange(range: string) {
    this.timeRange = range;
    this.loadOverview();
  }
  
  // 点击 KPI 卡片跳转
  navigateToQueries() {
    this.router.navigate(['/pages/starrocks/queries']);
  }
  
  navigateToBackends() {
    this.router.navigate(['/pages/starrocks/backends']);
  }
}
```

### 6.2 Service 层

**文件**：`frontend/src/app/@core/data/overview.service.ts`

```typescript
@Injectable()
export class OverviewService {
  constructor(private api: ApiService) {}
  
  getClusterOverview(clusterId: number, timeRange: string): Observable<ClusterOverview> {
    return this.api.get(`/api/clusters/${clusterId}/overview`, { time_range: timeRange });
  }
  
  getTimeSeriesMetrics(
    clusterId: number, 
    range: string, 
    metrics: string[]
  ): Observable<MetricsSnapshot[]> {
    return this.api.get(`/api/clusters/${clusterId}/metrics/timeseries`, {
      range,
      metrics: metrics.join(','),
    });
  }
}
```

### 6.3 子组件示例

**KPI Card Component**:

```typescript
@Component({
  selector: 'ngx-kpi-card',
  template: `
    <div class="kpi-card" (click)="onClick()" [class.clickable]="clickable">
      <div class="kpi-label">{{ label }}</div>
      <div class="kpi-value" #valueElement>{{ value | number:'1.2-2' }}</div>
      <div class="kpi-trend" [class.up]="trend > 0" [class.down]="trend < 0">
        <nb-icon [icon]="trend > 0 ? 'arrow-up' : 'arrow-down'"></nb-icon>
        {{ Math.abs(trend) }}%
      </div>
    </div>
  `,
})
export class KpiCardComponent implements OnChanges {
  @Input() label: string;
  @Input() value: number;
  @Input() trend: number;
  @Input() clickable: boolean = true;
  @Output() cardClick = new EventEmitter<void>();
  
  @ViewChild('valueElement') valueElement: ElementRef;
  
  ngOnChanges(changes: SimpleChanges) {
    if (changes['value'] && this.valueElement) {
      // CountUp.js 动画
      const countUp = new CountUp(this.valueElement.nativeElement, this.value, {
        duration: 1.5,
        useEasing: true,
      });
      countUp.start();
    }
  }
  
  onClick() {
    if (this.clickable) {
      this.cardClick.emit();
    }
  }
}
```

### 6.4 图表配置（使用 ECharts）

**ngx-admin 已集成 ECharts**，直接使用即可。

**基础配置示例**：
```typescript
// 使用 Nebular 主题色
import { NbThemeService } from '@nebular/theme';

export class ChartComponent {
  chartOption: any;
  
  constructor(private themeService: NbThemeService) {}
  
  ngOnInit() {
    // 监听主题变化
    this.themeService.getJsTheme()
      .pipe(takeUntil(this.destroy$))
      .subscribe(config => {
        const colors: any = config.variables;
        
        this.chartOption = {
          color: [
            colors.primary,
            colors.success,
            colors.warning,
            colors.danger,
          ],
          tooltip: {
            trigger: 'axis',
            backgroundColor: colors.bg,
            textStyle: { color: colors.fgText },
          },
          // ... 其他配置
        };
      });
  }
}
```

**使用方式**：
```html
<nb-card>
  <nb-card-header>查询性能趋势</nb-card-header>
  <nb-card-body>
    <div echarts [options]="chartOption" class="echart"></div>
  </nb-card-body>
</nb-card>
```

```scss
.echart {
  height: 400px;
}
```

**完整图表配置详见**：`FRONTEND_VISUAL_DESIGN.md`

---

## 七、实现优先级与阶段划分

### P0 - MVP 第一阶段（核心功能）

**目标**：实现最核心的集群健康和性能监控

**包含模块**：
1. ✅ 数据库表创建（metrics_snapshots, daily_snapshots）
2. ✅ MetricsCollector 后台任务（30秒采集）
3. ✅ 集群基础信息卡片
4. ✅ 集群健康总览卡片（使用 nb-card + status）
5. ✅ 关键性能指标 KPI（5个，使用 nb-card）
6. ✅ 查询性能趋势图（ECharts，ngx-admin 已集成）
7. ✅ 资源使用状态（ECharts Gauge）
8. ✅ 数据统计概览
9. ✅ 基础 API 和数据模型

**工作量评估**：约 5-7 天

**关键里程碑**：
- **Day 1-2**：后端基础
  - 创建数据库迁移脚本
  - 实现 MetricsCollector（参考 `METRICS_COLLECTION_DESIGN.md`）
  - 测试定时采集任务

- **Day 3-4**：后端 API
  - 实现 OverviewService
  - 实现 GET /api/clusters/:id/overview
  - 实现 GET /api/clusters/:id/metrics/timeseries
  - API 测试

- **Day 5-7**：前端页面
  - ⚠️ **重要**：先阅读 `FRONTEND_VISUAL_DESIGN.md` 实施原则
  - 创建主组件（cluster-overview.component）
  - 使用 **Nebular 原生组件** 构建布局
  - 集成 ECharts 图表
  - 测试交互和跳转

---

### P1 - 第二阶段（任务状态监控）

**目标**：增加物化视图、导入、事务等任务状态监控

**包含模块**：
7. ✅ 物化视图状态卡片
8. ✅ 导入任务状态卡片
9. ✅ 事务状态卡片
10. ✅ Schema Change & Clone 任务
11. ✅ Compaction 状态卡片
12. ✅ 活跃会话与用户
13. ✅ 网络与IO
14. ✅ Top 20 表（按大小）

**工作量评估**：约 3-4 天

---

### P2 - 第三阶段（高级分析）

**目标**：增加审计日志分析和智能告警

**包含模块**：
15. ✅ Top 20 表（按访问次数）- 需审计日志或 PROFILELIST
16. ✅ 慢查询列表 - 需审计日志或 PROFILELIST
17. ✅ 容量预测与告警
18. ✅ 智能告警与建议
19. ✅ 每日汇总任务（daily_snapshots）
20. ✅ 历史数据清理任务

**工作量评估**：约 3-4 天

**前置条件**：
- StarRocks 审计日志启用（可选）
- 或使用 SHOW PROFILELIST 替代方案

---

### P3 - 第四阶段（优化增强）

**目标**：性能优化、用户体验提升、视觉效果增强

**包含功能**：
21. ✅ 数据缓存机制（Redis，可选）
22. ✅ 发光效果、玻璃态、粒子背景
23. ✅ 导出报表功能（PDF/Excel）
24. ✅ 自定义时间范围
25. ✅ 移动端适配
26. ✅ WebSocket 实时推送（可选）

**工作量评估**：约 2-3 天

---

## 八、技术要点

### 8.1 性能优化

1. **并发数据获取**：
```rust
// 使用 tokio::join! 并发调用
let (backends, frontends, metrics) = tokio::try_join!(
    client.get_backends(),
    client.get_frontends(),
    client.get_metrics(),
)?;
```

2. **数据库索引优化**：
```sql
CREATE INDEX idx_metrics_cluster_time 
ON metrics_snapshots(cluster_id, collected_at DESC);
```

3. **前端虚拟滚动**（大表格）：
```html
<cdk-virtual-scroll-viewport itemSize="50">
  <tr *cdkVirtualFor="let row of data">...</tr>
</cdk-virtual-scroll-viewport>
```

4. **增量更新**：只更新变化的数据，不全量刷新

### 8.2 错误处理

1. **部分数据失败不影响整体**
2. **超时控制**：API 调用 10 秒超时
3. **重试机制**：失败自动重试 1-2 次
4. **降级展示**：数据获取失败时显示 "暂无数据"

### 8.3 安全考虑

1. **权限控制**：需要 JWT 认证
2. **SQL 注入防护**：使用参数化查询
3. **敏感信息脱敏**：SQL 语句截断显示

---

## 九、测试计划

### 单元测试
- MetricsCollector 采集逻辑
- 告警规则计算
- 趋势计算
- 容量预测算法

### 集成测试
- API 端点测试
- 数据库查询测试
- Prometheus 指标解析

### E2E 测试
- 页面加载测试
- 数据刷新测试
- 时间范围切换
- 点击跳转测试

### 性能测试
- 10 个集群并发采集
- 3 天历史数据查询
- 前端渲染性能

---

## 十、文档清单

### 已完成文档
1. ✅ `CLUSTER_OVERVIEW_DATA_VALIDATION.md` - 数据源验证
2. ✅ `METRICS_COLLECTION_DESIGN.md` - 自主采集系统设计
3. ✅ `FRONTEND_VISUAL_DESIGN.md` - 前端视觉设计
4. ✅ `CLUSTER_OVERVIEW_PLAN.md` - 本实现计划（本文档）

### 待编写文档
- [ ] API 接口文档（OpenAPI 自动生成）
- [ ] 用户使用手册
- [ ] 运维部署指南
- [ ] 故障排查手册

---

## 十一、风险与依赖

### 依赖项
- ✅ StarRocks HTTP `/metrics` 接口
- ✅ `SHOW BACKENDS/FRONTENDS/PROCESSLIST` SQL 命令
- ✅ `information_schema` 表
- ⚠️ 审计日志（P2 可选，可用 PROFILELIST 替代）

### 风险点

1. **StarRocks /metrics 格式变化**
   - 缓解：版本兼容性检查，解析失败时降级

2. **采集任务性能影响**
   - 缓解：30秒间隔足够长，并发控制，超时保护

3. **历史数据存储增长**
   - 缓解：自动清理策略，7 天详细 + 30 天汇总

4. **多集群并发采集**
   - 缓解：使用 tokio 异步，每个集群独立任务

---

## 十二、总结

### 核心亮点

1. ✅ **完全自主**：不依赖 Prometheus、Grafana 等外部组件
2. ✅ **轻量高效**：SQLite 存储，30秒采集，自动清理
3. ✅ **风格一致**：基于 ngx-admin，使用 Nebular 组件
4. ✅ **交互丰富**：所有关键指标可点击跳转
5. ✅ **生产就绪**：错误处理、性能优化、安全考虑

### 核心设计决策回顾

| 方面 | 决策 | 原因 |
|------|------|------|
| **数据采集** | 不依赖 Prometheus，自主采集 | 减少外部依赖，完全可控 |
| **存储方案** | SQLite（metrics_snapshots） | 轻量级，无需额外数据库 |
| **采集频率** | 30 秒一次 | 平衡实时性和性能 |
| **数据保留** | 7天详细 + 30天汇总 | 满足趋势分析需求，控制存储 |
| **前端组件** | 优先 ngx-admin 原生 | 风格一致，维护成本低 |
| **图表库** | ECharts（已集成） | 功能强大，ngx-admin 已包含 |
| **颜色系统** | Nebular status | 语义化，支持主题切换 |

### 预期成果

一个**与 ngx-admin 风格完美融合、专业级别**的集群概览 Dashboard，让管理员能够：
- 30 秒内全面掌握集群健康状态
- 通过丰富的图表理解性能趋势
- 快速发现异常并定位问题
- 点击任何指标跳转到详细管理页面
- 无缝切换 ngx-admin 的 4 种主题

### 实施前检查清单

#### 开始前必读
- [ ] 阅读本文档的"文档导航"部分
- [ ] 理解"核心实施原则"（特别是前端部分）
- [ ] 查看 `CLUSTER_OVERVIEW_DATA_VALIDATION.md` 确认数据可用性

#### 后端开发前
- [ ] 详细阅读 `METRICS_COLLECTION_DESIGN.md`
- [ ] 准备 SQLite 数据库环境
- [ ] 了解 StarRocks HTTP `/metrics` 接口

#### 前端开发前
- [ ] **必读** `FRONTEND_VISUAL_DESIGN.md` 的实施原则部分
- [ ] 熟悉 Nebular 组件库（https://akveo.github.io/nebular/）
- [ ] 了解 ngx-admin 的代码结构
- [ ] 确认 ECharts 已集成（检查 package.json）

### 下一步行动

1. ✅ 阅读 `README_CLUSTER_OVERVIEW.md`（文档导航）
2. ✅ 按照 P0 → P1 → P2 顺序实施
3. ✅ 实施过程中随时参考对应辅助文档
4. ✅ 遇到问题回到核心计划查找答案

---

## 📚 相关文档索引

| 文档 | 用途 | 何时阅读 |
|------|------|---------|
| `README_CLUSTER_OVERVIEW.md` | 文档导航和快速开始 | ⭐️ 最先阅读 |
| 本文档 | 核心实施计划（总纲） | 开始实施前通读 |
| `ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md` | 后端架构分析与集成方案 | 🔥 **实施前必读**（架构师视角） |
| `CLUSTER_OVERVIEW_DATA_VALIDATION.md` | 数据源可用性验证 | 实施前确认技术可行性 |
| `METRICS_COLLECTION_DESIGN.md` | 后端采集系统详细设计 | 实施后端时参考 |
| `FRONTEND_VISUAL_DESIGN.md` | 前端视觉设计参考 | 实施前端时参考（⚠️ 注意实施原则） |

---

## 📝 文档版本历史

### v2.0 (2025-10-24) - 重大更新
**核心变更**：
- ✅ 不依赖 Prometheus，改为自主采集所有指标
- ✅ 明确文档体系结构（1个核心 + 3个辅助）
- ✅ 强调前端使用 ngx-admin 原生组件
- ✅ 增加详细的实施原则和示例对比
- ✅ 添加推荐阅读路径和检查清单

**设计决策**：
- 后端：SQLite 存储，30秒采集，7天历史
- 前端：Nebular 组件，status 颜色，ECharts 图表

---

**现在，让我们开始构建这个令人兴奋的功能吧！** 🚀

**核心原则**：
- 📋 **文档优先**：实施前先读对应的文档
- 🔧 **后端原则**：不依赖 Prometheus，自主采集
- 🎨 **前端原则**：优先使用 ngx-admin 原生组件，不轻易自定义
- ✅ **质量保证**：参考检查清单，确保实施质量

