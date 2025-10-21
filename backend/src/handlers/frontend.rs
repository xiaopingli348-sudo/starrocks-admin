use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

use crate::models::Frontend;
use crate::services::{ClusterService, StarRocksClient};
use crate::utils::ApiResult;

pub type ClusterServiceState = Arc<ClusterService>;

// Get all frontends for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/frontends",
    params(
        ("id" = i64, Path, description = "Cluster ID")
    ),
    responses(
        (status = 200, description = "List of frontend nodes", body = Vec<Frontend>),
        (status = 404, description = "Cluster not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Frontends"
)]
pub async fn list_frontends(
    State(cluster_service): State<ClusterServiceState>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<Json<Vec<Frontend>>> {
    let cluster = cluster_service.get_cluster(cluster_id).await?;
    let client = StarRocksClient::new(cluster);
    let frontends = client.get_frontends().await?;
    Ok(Json(frontends))
}

