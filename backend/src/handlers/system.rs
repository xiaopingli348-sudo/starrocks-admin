use axum::{Json, extract::State};
use std::sync::Arc;

use crate::AppState;
use crate::models::RuntimeInfo;
use crate::services::StarRocksClient;
use crate::utils::ApiResult;

// Get runtime info for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/system/runtime_info",
    responses(
        (status = 200, description = "Runtime information", body = RuntimeInfo),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "System"
)]
pub async fn get_runtime_info(State(state): State<Arc<AppState>>) -> ApiResult<Json<RuntimeInfo>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    let client = StarRocksClient::new(cluster);
    let runtime_info = client.get_runtime_info().await?;
    Ok(Json(runtime_info))
}
