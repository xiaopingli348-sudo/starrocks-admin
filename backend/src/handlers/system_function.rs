use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
    Extension,
};
use std::sync::Arc;
use validator::Validate;

use crate::models::{CreateFunctionRequest, UpdateOrderRequest, UpdateFunctionRequest};
use crate::utils::{ApiResult, ApiError};
use crate::AppState;

// GET /api/clusters/:id/system-functions
pub async fn get_system_functions(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
) -> ApiResult<impl IntoResponse> {
    let functions = state.system_function_service.get_functions(cluster_id).await?;
    Ok(Json(functions))
}

// POST /api/clusters/:id/system-functions
pub async fn create_system_function(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(req): Json<CreateFunctionRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate request
    if let Err(validation_errors) = req.validate() {
        return Err(ApiError::validation_error(format!("请求参数验证失败：{}", validation_errors)));
    }

    // Check if cluster exists
    state.cluster_service.get_cluster(cluster_id).await?;

    let function = state.system_function_service.create_function(cluster_id, req, user_id).await?;
    Ok((StatusCode::CREATED, Json(function)))
}

// POST /api/clusters/:id/system-functions/:function_id/execute
pub async fn execute_system_function(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, function_id)): Path<(i64, i64)>,
) -> ApiResult<impl IntoResponse> {
    let result = state.system_function_service.execute_function(cluster_id, function_id).await?;
    Ok(Json(result))
}

// PUT /api/clusters/:id/system-functions/orders
pub async fn update_function_orders(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
    Json(req): Json<UpdateOrderRequest>,
) -> ApiResult<impl IntoResponse> {
    state.system_function_service.update_orders(cluster_id, req).await?;
    Ok(StatusCode::NO_CONTENT)
}

// PUT /api/clusters/:id/system-functions/:function_id/favorite
pub async fn toggle_function_favorite(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, function_id)): Path<(i64, i64)>,
) -> ApiResult<impl IntoResponse> {
    let function = state.system_function_service.toggle_favorite(cluster_id, function_id).await?;
    Ok(Json(function))
}

// DELETE /api/clusters/:id/system-functions/:function_id
pub async fn delete_system_function(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, function_id)): Path<(i64, i64)>,
) -> ApiResult<impl IntoResponse> {
    state.system_function_service.delete_function(cluster_id, function_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// PUT /api/clusters/:id/system-functions/:function_id
pub async fn update_function(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, function_id)): Path<(i64, i64)>,
    Json(req): Json<UpdateFunctionRequest>,
) -> ApiResult<impl IntoResponse> {
    // Validate request
    if let Err(validation_errors) = req.validate() {
        return Err(ApiError::validation_error(format!("请求参数验证失败：{}", validation_errors)));
    }

    // Check if cluster exists
    state.cluster_service.get_cluster(cluster_id).await?;

    let function = state.system_function_service.update_function(cluster_id, function_id, req).await?;
    Ok(Json(function))
}

// PUT /api/system-functions/:function_name/access-time
pub async fn update_system_function_access_time(
    State(state): State<Arc<AppState>>,
    Path(function_name): Path<String>,
) -> ApiResult<impl IntoResponse> {
    state.system_function_service.update_system_function_access_time(&function_name).await?;
    Ok(StatusCode::NO_CONTENT)
}

// DELETE /api/system-functions/category/:category_name
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(category_name): Path<String>,
) -> ApiResult<impl IntoResponse> {
    state.system_function_service.delete_category(&category_name).await?;
    Ok(StatusCode::NO_CONTENT)
}
