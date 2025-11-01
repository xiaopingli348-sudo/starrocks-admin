// Cluster Overview Handlers
// Purpose: HTTP API endpoints for cluster overview functionality
// Design Ref: ARCHITECTURE_ANALYSIS_AND_INTEGRATION.md

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;
use crate::services::{
    CapacityPrediction, ClusterOverview, DataStatistics, ExtendedClusterOverview, HealthCard,
    PerformanceTrends, ResourceTrends, TimeRange,
};
use crate::utils::ApiResult;

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
    path = "/api/clusters/overview",
    params(
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Cluster overview data", body = ClusterOverview),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_cluster_overview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OverviewQueryParams>,
) -> ApiResult<Json<ClusterOverview>> {
    tracing::debug!("GET /api/clusters/overview?time_range={:?}", params.time_range);

    // Get the active cluster
    let active_cluster = state.cluster_service.get_active_cluster().await?;
    let cluster_id = active_cluster.id;

    let overview = state
        .overview_service
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
    path = "/api/clusters/overview/health",
    responses(
        (status = 200, description = "Health status cards", body = Vec<HealthCard>),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_health_cards(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<HealthCard>>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/health");

    let cards = state.overview_service.get_health_cards(cluster.id).await?;

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
    path = "/api/clusters/overview/performance",
    params(
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Performance trends", body = PerformanceTrends),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_performance_trends(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TrendQueryParams>,
) -> ApiResult<Json<PerformanceTrends>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/performance?time_range={:?}", params.time_range);

    let trends = state
        .overview_service
        .get_performance_trends(cluster.id, params.time_range)
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
    path = "/api/clusters/overview/resources",
    params(
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Resource trends", body = ResourceTrends),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_resource_trends(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TrendQueryParams>,
) -> ApiResult<Json<ResourceTrends>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/resources?time_range={:?}", params.time_range);

    let trends = state
        .overview_service
        .get_resource_trends(cluster.id, params.time_range)
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
    path = "/api/clusters/overview/data-stats",
    responses(
        (status = 200, description = "Data statistics", body = DataStatistics),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_data_statistics(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<DataStatistics>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/data-stats");

    let stats = state
        .overview_service
        .get_data_statistics(cluster.id)
        .await?;

    Ok(Json(stats))
}

/// Get capacity prediction
///
/// Returns disk capacity prediction based on historical growth trend
#[utoipa::path(
    get,
    path = "/api/clusters/overview/capacity-prediction",
    responses(
        (status = 200, description = "Capacity prediction", body = CapacityPrediction),
        (status = 404, description = "No active cluster found or no historical data"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_capacity_prediction(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<CapacityPrediction>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/capacity-prediction");

    let prediction = state.overview_service.predict_capacity(cluster.id).await?;

    Ok(Json(prediction))
}

/// Get extended cluster overview (All 18 modules)
///
/// Returns comprehensive cluster overview with all modules including:
/// - Module 1: Cluster Health
/// - Module 2: Key Performance Indicators
/// - Module 3: Resource Metrics
/// - Module 4-5: Performance & Resource Trends
/// - Module 6: Data Statistics
/// - Module 7-13: Task and session monitoring
/// - Module 17: Capacity Prediction
/// - Module 18: Alerts
#[utoipa::path(
    get,
    path = "/api/clusters/overview/extended",
    params(
        ("time_range" = Option<String>, Query, description = "Time range: 1h, 6h, 24h, 3d (default: 24h)")
    ),
    responses(
        (status = 200, description = "Extended cluster overview with all modules", body = ExtendedClusterOverview),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Cluster Overview"
)]
pub async fn get_extended_cluster_overview(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OverviewQueryParams>,
) -> ApiResult<Json<ExtendedClusterOverview>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    tracing::debug!("GET /api/clusters/overview/extended?time_range={:?}", params.time_range);

    let overview = state
        .overview_service
        .get_extended_overview(cluster.id, params.time_range)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to get extended cluster overview for cluster {}: {}",
                cluster.id,
                e
            );
            e
        })?;

    Ok(Json(overview))
}
