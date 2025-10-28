# 集群概览功能文档导航 📚

> 本目录包含集群概览功能的完整设计文档

---

## 📋 文档体系结构

```
集群概览功能文档
│
├─► CLUSTER_OVERVIEW_PLAN.md ⭐️ 核心文档
│   └─ 总体实施计划，包含需求、架构、实施步骤
│   └─ **必读**：开始实施前请先阅读本文档
│
├─► ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md (辅助文档0 - 架构分析)
│   └─ 后端架构深度分析与集成方案（架构师视角）
│   └─ 包含：现有架构分析、集成设计、实施路线图
│   └─ 🔥 重要程度：后端开发必读
│   └─ 阅读时机：开始实施前，了解如何将新功能嵌入现有架构
│
├─► CLUSTER_OVERVIEW_DATA_VALIDATION.md (辅助文档1)
│   └─ 数据源可用性验证
│   └─ 验证所有 75+ 个指标都可以从 StarRocks 获取
│   └─ 阅读时机：实施前确认技术可行性
│
├─► METRICS_COLLECTION_DESIGN.md (辅助文档2)
│   └─ 后端自主采集系统详细设计
│   └─ 包含：数据库设计、MetricsCollector 实现、API 设计
│   └─ 阅读时机：实施 P0 后端部分时参考
│
└─► FRONTEND_VISUAL_DESIGN.md (辅助文档3)
    └─ 前端视觉设计参考（基于 ngx-admin）
    └─ 包含：组件设计、颜色系统、动画效果
    └─ ⚠️ 重要：优先使用 ngx-admin 原生组件，不轻易自定义
    └─ 阅读时机：实施前端页面时参考
```

---

## 🎯 快速开始

### 1. 了解需求和设计（5 分钟）
阅读 `CLUSTER_OVERVIEW_PLAN.md` 的前两章：
- 一、设计理念
- 二、核心架构设计

### 2. 确认技术可行性（10 分钟）
浏览 `CLUSTER_OVERVIEW_DATA_VALIDATION.md`：
- 查看"总结与建议"部分
- 确认 P0 阶段所需指标全部可用

### 3. 开始实施（按阶段）

#### P0 阶段（5-7 天）- 核心功能
1. **架构理解**（⭐️ 必读）
   - 先读 `ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md`
   - 了解如何将新功能嵌入现有架构

2. **后端开发**（参考 `METRICS_COLLECTION_DESIGN.md`）
   - Day 1-2：数据库表创建 + MetricsCollector 实现
   - Day 3-4：后端 API 实现（OverviewService）

3. **前端开发**（参考 `FRONTEND_VISUAL_DESIGN.md`）
   - Day 5-7：页面布局 + 组件开发
   - **重点**：使用 ngx-admin 原生组件

#### P1 阶段（3-4 天）- 任务监控
- 参考核心计划的 P1 模块清单

#### P2 阶段（3-4 天）- 高级功能
- 参考核心计划的 P2 模块清单

---

## ✅ 核心设计决策

### 1. 不依赖外部组件
- ❌ 不依赖 Prometheus、Grafana
- ✅ 自主采集指标数据到 SQLite
- ✅ 每 30 秒采集一次，保留 7 天

### 2. 前端实施原则
- ✅ **优先使用 ngx-admin 原生组件**
- ✅ 使用 Nebular UI 的 status 颜色系统
- ✅ 使用 ECharts（ngx-admin 已集成）
- ⚠️ 必要时可自定义，但要基于主题变量
- ❌ 不创建独立的 CSS 样式体系

### 3. 数据来源
| 数据类型 | 来源 |
|---------|-----|
| 节点状态 | `SHOW BACKENDS/FRONTENDS` |
| 性能指标 | HTTP `/metrics` (Prometheus 格式) |
| 元数据 | `information_schema` |
| 历史数据 | SQLite `metrics_snapshots` 表 |

---

## 📊 实施进度跟踪

### P0 阶段 ✅/❌
- [ ] 数据库表创建
- [ ] MetricsCollector 后台任务
- [ ] 后端 API（OverviewService）
- [ ] 前端基础布局
- [ ] 健康状态卡片
- [ ] KPI 性能指标
- [ ] 性能趋势图
- [ ] 资源使用状态

### P1 阶段 ✅/❌
- [ ] 物化视图状态
- [ ] 导入任务状态
- [ ] 事务状态
- [ ] Compaction 状态
- [ ] Top 20 表（按大小）

### P2 阶段 ✅/❌
- [ ] 慢查询列表
- [ ] 容量预测
- [ ] 智能告警

---

## 📖 阅读路径推荐

### 对于项目经理/产品经理
1. 阅读核心计划的"一、设计理念"
2. 查看核心计划的"三、页面布局架构"
3. 了解"七、实施优先级"

### 对于后端开发
1. 阅读核心计划的"五、后端实现"
2. **详细阅读** `METRICS_COLLECTION_DESIGN.md`
3. 查看数据验证文档确认数据源

### 对于前端开发
1. 阅读核心计划的"六、前端实现"
2. **详细阅读** `FRONTEND_VISUAL_DESIGN.md`
3. **特别注意**：组件选用优先级和实施原则

### 对于测试工程师
1. 阅读核心计划的"九、测试计划"
2. 参考各模块的数据来源进行测试设计

---

## 🔗 相关链接

- [StarRocks 官方文档](https://docs.starrocks.io/)
- [ngx-admin GitHub](https://github.com/akveo/ngx-admin)
- [Nebular UI Components](https://akveo.github.io/nebular/)
- [ECharts 文档](https://echarts.apache.org/)

---

## 📝 更新日志

- 2025-10-24: v2.0 - 重大更新，不依赖 Prometheus，自主采集
- 2025-10-24: 明确文档体系结构和依赖关系
- 2025-10-24: 强调前端使用 ngx-admin 原生组件的原则

---

**祝实施顺利！如有疑问请参考对应的详细文档。** 🚀

