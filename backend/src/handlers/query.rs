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
use crate::models::{CatalogWithDatabases, CatalogsWithDatabasesResponse, Query, QueryExecuteRequest, QueryExecuteResponse};
use crate::services::mysql_client::MySQLClient;
use crate::services::StarRocksClient;
use crate::utils::ApiResult;

// Get list of catalogs using MySQL client
#[utoipa::path(
    get,
    path = "/api/clusters/catalogs",
    responses(
        (status = 200, description = "List of catalogs", body = Vec<String>),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn list_catalogs(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<String>>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    
    // Use MySQL client to execute SHOW CATALOGS
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);
    
    let (_, rows) = mysql_client.query_raw("SHOW CATALOGS", None, None).await?;
    
    let mut catalogs = Vec::new();
    for row in rows {
        if let Some(catalog_name) = row.first() {
            let name = catalog_name.trim().to_string();
            if !name.is_empty() {
                catalogs.push(name);
            }
        }
    }
    
    tracing::debug!("Found {} catalogs via MySQL client", catalogs.len());
    Ok(Json(catalogs))
}

// Get list of databases in a catalog using MySQL client
#[utoipa::path(
    get,
    path = "/api/clusters/databases",
    params(
        ("catalog" = Option<String>, Query, description = "Catalog name (optional)")
    ),
    responses(
        (status = 200, description = "List of databases", body = Vec<String>),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn list_databases(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Vec<String>>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    
    // Use MySQL client
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);
    
    // Get catalog parameter if provided
    if let Some(catalog_name) = params.get("catalog") {
        // First switch to the catalog, then show databases
        // Try without backticks - StarRocks may not support them for catalog names
        let use_catalog_sql = format!("USE CATALOG {}", catalog_name);
        if let Err(e) = mysql_client.execute(&use_catalog_sql).await {
            tracing::warn!("Failed to switch to catalog {}: {}", catalog_name, e);
            // Continue anyway, might be using default catalog
        }
    }
    
    // Execute SHOW DATABASES
    let (_, rows) = mysql_client.query_raw("SHOW DATABASES", None, None).await?;
    
    let mut databases = Vec::new();
    for row in rows {
        if let Some(db_name) = row.first() {
            let name = db_name.trim().to_string();
            // Skip system databases
            if !name.is_empty()
                && name != "information_schema" 
                && name != "_statistics_" {
                databases.push(name);
            }
        }
    }
    
    tracing::debug!("Found {} databases via MySQL client", databases.len());
    Ok(Json(databases))
}

// Get all catalogs with their databases using MySQL client (one-time response)
#[utoipa::path(
    get,
    path = "/api/clusters/catalogs-databases",
    responses(
        (status = 200, description = "All catalogs with their databases", body = CatalogsWithDatabasesResponse),
        (status = 404, description = "No active cluster found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Queries"
)]
pub async fn list_catalogs_with_databases(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<CatalogsWithDatabasesResponse>> {
    let cluster = state.cluster_service.get_active_cluster().await?;
    
    // Use MySQL client
    let pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);
    
    // Step 1: Get all catalogs
    let (_, catalog_rows) = mysql_client.query_raw("SHOW CATALOGS", None, None).await?;
    
    let mut catalogs = Vec::new();
    
    // Extract catalog names
    let mut catalog_names = Vec::new();
    for row in catalog_rows {
        if let Some(catalog_name) = row.first() {
            let name = catalog_name.trim().to_string();
            if !name.is_empty() {
                catalog_names.push(name);
            }
        }
    }
    
    tracing::debug!("Found {} catalogs, fetching databases for each...", catalog_names.len());
    
    // Step 2: For each catalog, switch to it and get databases
    for catalog_name in &catalog_names {
        // Switch to catalog (without backticks - StarRocks may not support them for catalog names)
        let use_catalog_sql = format!("USE CATALOG {}", catalog_name);
        if let Err(e) = mysql_client.execute(&use_catalog_sql).await {
            tracing::warn!("Failed to switch to catalog {}: {}", catalog_name, e);
            catalogs.push(CatalogWithDatabases {
                catalog: catalog_name.clone(),
                databases: Vec::new(),
            });
            continue;
        }
        
        // Get databases for this catalog
        let (_, db_rows) = match mysql_client.query_raw("SHOW DATABASES", None, None).await {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!("Failed to get databases for catalog {}: {}", catalog_name, e);
                catalogs.push(CatalogWithDatabases {
                    catalog: catalog_name.clone(),
                    databases: Vec::new(),
                });
                continue;
            }
        };
        
        let mut databases = Vec::new();
        for row in db_rows {
            if let Some(db_name) = row.first() {
                let name = db_name.trim().to_string();
                // Skip system databases
                if !name.is_empty()
                    && name != "information_schema" 
                    && name != "_statistics_" {
                    databases.push(name);
                }
            }
        }
        
        tracing::debug!("Catalog {} has {} databases", catalog_name, databases.len());
        
        catalogs.push(CatalogWithDatabases {
            catalog: catalog_name.clone(),
            databases,
        });
    }
    
    Ok(Json(CatalogsWithDatabasesResponse { catalogs }))
}

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
// If database is provided, will execute USE database before the SQL query
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
    let pool: mysql_async::Pool = state.mysql_pool_manager.get_pool(&cluster).await?;
    let mysql_client = MySQLClient::from_pool(pool);

    // If catalog is specified, switch to it first
    if let Some(ref cat) = request.catalog {
        if !cat.is_empty() {
            let use_catalog_sql = format!("USE CATALOG `{}`", cat);
            tracing::debug!("Executing USE CATALOG: {}", use_catalog_sql);
            if let Err(e) = mysql_client.execute(&use_catalog_sql).await {
                tracing::warn!("Failed to execute USE CATALOG {}: {}", cat, e);
                // Continue execution anyway - catalog might already be active
            }
        }
    }

    // If database is specified, execute USE database
    if let Some(ref db) = request.database {
        if !db.is_empty() {
            let use_db_sql = format!("USE `{}`", db);
            tracing::debug!("Executing USE DATABASE: {}", use_db_sql);
            if let Err(e) = mysql_client.execute(&use_db_sql).await {
                tracing::warn!("Failed to execute USE database {}: {}", db, e);
                // Continue execution anyway - database might not exist
            }
        }
    }

    let original_sql = &request.sql;
    let sql = apply_query_limit(original_sql, request.limit.unwrap_or(1000));

    let start = Instant::now();

    // Execute query with catalog and database context
    let query_result = mysql_client.query_raw(
        &sql,
        request.catalog.as_deref(),
        request.database.as_deref(),
    ).await;

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
