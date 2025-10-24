// Cluster Overview Handlers
// Purpose: HTTP API endpoints for cluster overview functionality
// Design Ref: ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::services::{
    ClusterOverview, DataStatistics, HealthCard, OverviewService, PerformanceTrends, ResourceTrends, TimeRange,
};
use crate::utils::ApiResult;

pub type OverviewServiceState = Arc<OverviewService>;

/// Query parameters for overview endpoints
#[derive(Debug, Deserialize)]
pub struct OverviewQueryParams {
    #[serde(default = "default_time_range")]
    pub time_range: TimeRange,
}

fn default_time_range() -> TimeRange {
    TimeRange::Hours24
}

/// Query parameters for trend endpoints
#[derive(Debug, Deserialize)]
pub struct TrendQueryParams {
    #[serde(default = "default_time_range")]
    pub time_range: TimeRange,
}

// ========================================
// API Handlers
// ========================================

/// Get cluster overview
/// 
/// Returns comprehensive cluster overview including:
/// - Latest metrics snapshot
/// - Performance trends (QPS, latency, error rate)
/// - Resource trends (CPU, memory, disk)
/// - Aggregated statistics
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview",
    params(
        ("id" = i64, Path, description = "Cluster ID"),
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Cluster overview data", body = ClusterOverview),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_cluster_overview(
    State(overview_service): State<OverviewServiceState>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<OverviewQueryParams>,
) -> ApiResult<Json<ClusterOverview>> {
    tracing::debug!(
        "GET /api/clusters/{}/overview?time_range={:?}",
        cluster_id,
        params.time_range
    );

    let overview = overview_service
        .get_cluster_overview(cluster_id, params.time_range)
        .await?;

    Ok(Json(overview))
}

/// Get health status cards
/// 
/// Returns health status cards for quick overview:
/// - Cluster status (BE/FE availability)
/// - Query load (QPS)
/// - CPU usage
/// - Disk usage
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview/health",
    params(
        ("id" = i64, Path, description = "Cluster ID")
    ),
    responses(
        (status = 200, description = "Health status cards", body = Vec<HealthCard>),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_health_cards(
    State(overview_service): State<OverviewServiceState>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<Json<Vec<HealthCard>>> {
    tracing::debug!("GET /api/clusters/{}/overview/health", cluster_id);

    let cards = overview_service.get_health_cards(cluster_id).await?;

    Ok(Json(cards))
}

/// Get performance trends
/// 
/// Returns time series data for performance metrics:
/// - QPS (Queries per second)
/// - Latency P99
/// - Error rate
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview/performance",
    params(
        ("id" = i64, Path, description = "Cluster ID"),
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Performance trends", body = PerformanceTrends),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_performance_trends(
    State(overview_service): State<OverviewServiceState>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<TrendQueryParams>,
) -> ApiResult<Json<PerformanceTrends>> {
    tracing::debug!(
        "GET /api/clusters/{}/overview/performance?time_range={:?}",
        cluster_id,
        params.time_range
    );

    let trends = overview_service
        .get_performance_trends(cluster_id, params.time_range)
        .await?;

    Ok(Json(trends))
}

/// Get resource trends
/// 
/// Returns time series data for resource usage:
/// - CPU usage
/// - Memory usage
/// - Disk usage
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview/resources",
    params(
        ("id" = i64, Path, description = "Cluster ID"),
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Resource trends", body = ResourceTrends),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_resource_trends(
    State(overview_service): State<OverviewServiceState>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<TrendQueryParams>,
) -> ApiResult<Json<ResourceTrends>> {
    tracing::debug!(
        "GET /api/clusters/{}/overview/resources?time_range={:?}",
        cluster_id,
        params.time_range
    );

    let trends = overview_service
        .get_resource_trends(cluster_id, params.time_range)
        .await?;

    Ok(Json(trends))
}

/// Get data statistics
/// 
/// Returns cached data statistics including:
/// - Database and table counts
/// - Top 20 tables by size
/// - Top 20 tables by access count
/// - Materialized view statistics
/// - Schema change statistics
/// - Active users
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/overview/data-stats",
    params(
        ("id" = i64, Path, description = "Cluster ID")
    ),
    responses(
        (status = 200, description = "Data statistics", body = DataStatistics),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_data_statistics(
    State(overview_service): State<OverviewServiceState>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<Json<DataStatistics>> {
    tracing::debug!("GET /api/clusters/{}/overview/data-stats", cluster_id);

    let stats = overview_service.get_data_statistics(cluster_id).await?;

    Ok(Json(stats))
}

