use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::{
    models::starrocks::{UpdateVariableRequest, Variable},
    services::{cluster_service::ClusterService, mysql_client::MySQLClient},
    utils::error::{ApiError, ApiResult},
};

#[derive(Debug, Deserialize)]
pub struct VariableQueryParams {
    #[serde(default = "default_type")]
    pub r#type: String, // "global" or "session"
    pub filter: Option<String>,
}

fn default_type() -> String {
    "global".to_string()
}

/// Get system variables
#[utoipa::path(
    get,
    path = "/api/clusters/{cluster_id}/variables",
    params(
        ("cluster_id" = i64, Path, description = "Cluster ID"),
        ("type" = Option<String>, Query, description = "Variable type: global or session"),
        ("filter" = Option<String>, Query, description = "Filter variable name")
    ),
    responses(
        (status = 200, description = "Variables list", body = Vec<Variable>),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_variables(
    State(state): State<Arc<crate::AppState>>,
    Path(cluster_id): Path<i64>,
    Query(params): Query<VariableQueryParams>,
) -> ApiResult<impl IntoResponse> {
    // Get cluster info
    let cluster_service = ClusterService::new(state.db.clone());
    let cluster = cluster_service.get_cluster(cluster_id).await?;

    // Get MySQL client from pool
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);

    // Build SQL query
    let sql = match params.r#type.as_str() {
        "session" => "SHOW SESSION VARIABLES",
        _ => "SHOW GLOBAL VARIABLES",
    };

    let sql_with_filter = if let Some(filter) = params.filter {
        format!("{} LIKE '%{}%'", sql, filter)
    } else {
        sql.to_string()
    };

    // Execute query
    let (_, rows) = mysql_client.query(&sql_with_filter).await?;

    // Parse results
    let variables: Vec<Variable> = rows
        .into_iter()
        .map(|row| {
            Variable {
                name: row.first().cloned().unwrap_or_default(),
                value: row.get(1).cloned().unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(variables))
}

/// Update a variable
#[utoipa::path(
    put,
    path = "/api/clusters/{cluster_id}/variables/{variable_name}",
    params(
        ("cluster_id" = i64, Path, description = "Cluster ID"),
        ("variable_name" = String, Path, description = "Variable name")
    ),
    request_body = UpdateVariableRequest,
    responses(
        (status = 200, description = "Variable updated successfully"),
        (status = 404, description = "Cluster not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn update_variable(
    State(state): State<Arc<crate::AppState>>,
    Path((cluster_id, variable_name)): Path<(i64, String)>,
    Json(request): Json<UpdateVariableRequest>,
) -> ApiResult<impl IntoResponse> {
    // Get cluster info
    let cluster_service = ClusterService::new(state.db.clone());
    let cluster = cluster_service.get_cluster(cluster_id).await?;

    // Get MySQL client from pool
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);

    // Validate scope
    let scope = match request.scope.to_uppercase().as_str() {
        "GLOBAL" => "GLOBAL",
        "SESSION" => "SESSION",
        _ => {
            return Err(ApiError::invalid_data(
                "Invalid scope. Must be GLOBAL or SESSION",
            ))
        }
    };

    // Build SET command
    let sql = format!(
        "SET {} {} = {}",
        scope, variable_name, request.value
    );

    // Execute command
    mysql_client.execute(&sql).await?;

    Ok((
        StatusCode::OK,
        Json(json!({ "message": "Variable updated successfully" })),
    ))
}
