use axum::{Json, extract::State};
use std::sync::Arc;

use crate::AppState;
use crate::models::Frontend;
use crate::services::StarRocksClient;
use crate::utils::ApiResult;

// Get all frontends for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/frontends",
    responses(
        (status = 200, description = "List of frontend nodes", body = Vec<Frontend>),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Frontends"
)]
pub async fn list_frontends(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<Frontend>>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    let client = StarRocksClient::new(cluster);
    let frontends = client.get_frontends().await?;
    Ok(Json(frontends))
}
