use axum::{extract::{Path, State}, Json};
use std::sync::Arc;

use crate::models::Backend;
use crate::services::{ClusterService, StarRocksClient};
use crate::utils::ApiResult;

pub type ClusterServiceState = Arc<ClusterService>;

// Get all backends for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/{id}/backends",
    params(
        ("id" = i64, Path, description = "Cluster ID")
    ),
    responses(
        (status = 200, description = "List of backend nodes", body = Vec<Backend>),
        (status = 404, description = "Cluster not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Backends"
)]
pub async fn list_backends(
    State(cluster_service): State<ClusterServiceState>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<Json<Vec<Backend>>> {
    let cluster = cluster_service.get_cluster(cluster_id).await?;
    let client = StarRocksClient::new(cluster);
    let backends = client.get_backends().await?;
    Ok(Json(backends))
}

