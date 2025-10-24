use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

use crate::models::{ProfileListItem, ProfileDetail};
use crate::services::{ClusterService, MySQLClient};
use crate::utils::ApiResult;

// List all query profiles for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/{cluster_id}/profiles",
    params(
        ("cluster_id" = i64, Path, description = "Cluster ID")
    ),
    responses(
        (status = 200, description = "List of query profiles", body = Vec<ProfileListItem>),
        (status = 404, description = "Cluster not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Profiles"
)]
pub async fn list_profiles(
    State(state): State<Arc<crate::AppState>>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<Json<Vec<ProfileListItem>>> {
    let cluster_service = ClusterService::new(state.db.clone());
    let cluster = cluster_service.get_cluster(cluster_id).await?;
    
    tracing::info!("Fetching profile list for cluster {}", cluster_id);
    
    // Get connection pool and execute SHOW PROFILELIST
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);
    
    let (columns, rows) = mysql_client.query_raw("SHOW PROFILELIST").await?;
    
    tracing::info!("Profile list query returned {} rows with {} columns", rows.len(), columns.len());
    
    // Convert rows to ProfileListItem
    let profiles: Vec<ProfileListItem> = rows
        .into_iter()
        .map(|row| {
            // SHOW PROFILELIST returns: QueryId, StartTime, Time, State, Statement
            ProfileListItem {
                query_id: row.first().cloned().unwrap_or_default(),
                start_time: row.get(1).cloned().unwrap_or_default(),
                time: row.get(2).cloned().unwrap_or_default(),
                state: row.get(3).cloned().unwrap_or_default(),
                statement: row.get(4).cloned().unwrap_or_default(),
            }
        })
        .collect();
    
    tracing::info!("Successfully converted {} profiles", profiles.len());
    Ok(Json(profiles))
}

// Get detailed profile for a specific query
#[utoipa::path(
    get,
    path = "/api/clusters/{cluster_id}/profiles/{query_id}",
    params(
        ("cluster_id" = i64, Path, description = "Cluster ID"),
        ("query_id" = String, Path, description = "Query ID")
    ),
    responses(
        (status = 200, description = "Query profile detail", body = ProfileDetail),
        (status = 404, description = "Cluster or profile not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Profiles"
)]
pub async fn get_profile(
    State(state): State<Arc<crate::AppState>>,
    Path((cluster_id, query_id)): Path<(i64, String)>,
) -> ApiResult<Json<ProfileDetail>> {
    let cluster_service = ClusterService::new(state.db.clone());
    let cluster = cluster_service.get_cluster(cluster_id).await?;
    
    tracing::info!("Fetching profile detail for query {} in cluster {}", query_id, cluster_id);
    
    // Get connection pool and execute SELECT get_query_profile()
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);
    
    let sql = format!("SELECT get_query_profile('{}')", query_id);
    let (_, rows) = mysql_client.query_raw(&sql).await?;
    
    // Extract profile content from result
      let profile_content = rows
        .first()
        .and_then(|row| row.first())
        .cloned()
        .unwrap_or_else(|| "Profile not found or unavailable".to_string());
    
    tracing::info!("Profile content length: {} bytes", profile_content.len());
    
    Ok(Json(ProfileDetail {
        query_id,
        profile_content,
    }))
}

