use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

use crate::AppState;
use crate::models::{Query, QueryExecuteRequest, QueryExecuteResponse};
use crate::services::{MySQLClient, StarRocksClient};
use crate::utils::ApiResult;

// Get all running queries for a cluster
#[utoipa::path(
    get,
    path = "/api/clusters/queries",
    responses(
        (status = 200, description = "List of running queries", body = Vec<Query>),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn list_queries(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<Query>>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    let client = StarRocksClient::new(cluster);
    let queries = client.get_queries().await?;
    Ok(Json(queries))
}

// Kill a query
#[utoipa::path(
    delete,
    path = "/api/clusters/queries/{query_id}",
    params(
        ("query_id" = String, Path, description = "Query ID")
    ),
    responses(
        (status = 200, description = "Query killed successfully"),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn kill_query(
    State(state): State<Arc<crate::AppState>>,
    Path(query_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let cluster = state.cluster_service.get_active_cluster().await?;

    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);

    // Execute KILL QUERY
    let sql = format!("KILL QUERY '{}'", query_id);
    mysql_client.execute(&sql).await?;

    Ok((StatusCode::OK, Json(json!({ "message": "Query killed successfully" }))))
}

// Execute SQL query
#[utoipa::path(
    post,
    path = "/api/clusters/queries/execute",
    request_body = QueryExecuteRequest,
    responses(
        (status = 200, description = "Query executed successfully", body = QueryExecuteResponse),
        (status = 400, description = "Invalid SQL or query error"),
        (status = 404, description = "No active cluster found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn execute_sql(
    State(state): State<Arc<crate::AppState>>,
    Json(request): Json<QueryExecuteRequest>,
) -> ApiResult<Json<QueryExecuteResponse>> {
    let cluster = state.cluster_service.get_active_cluster().await?;

    // Use pool manager to get cached pool (avoid intermittent failures from creating new pools)
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);

    let original_sql = &request.sql;
    let sql = apply_query_limit(original_sql, request.limit.unwrap_or(1000));

    let start = Instant::now();

    // Execute query
    let query_result = mysql_client.query_raw(&sql).await;

    let execution_time_ms = start.elapsed().as_millis();

    match query_result {
        Ok((columns, data_rows)) => {
            let row_count = data_rows.len();

            Ok(Json(QueryExecuteResponse {
                columns,
                rows: data_rows,
                row_count,
                execution_time_ms,
            }))
        },
        Err(e) => Err(e),
    }
}

fn apply_query_limit(sql: &str, limit: i32) -> String {
    let sql_upper = sql.trim().to_uppercase();

    if sql_upper.contains("LIMIT") {
        return sql.to_string();
    }

    if sql_upper.starts_with("SELECT") {
        if sql_upper.contains("GET_QUERY_PROFILE")
            || sql_upper.contains("SHOW_PROFILE")
            || sql_upper.contains("EXPLAIN")
        {
            return sql.to_string();
        }

        format!("{} LIMIT {}", sql.trim().trim_end_matches(';'), limit)
    } else {
        sql.to_string()
    }
}
