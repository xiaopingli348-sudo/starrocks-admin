use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiError(code={}, message={})", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorCode {
    // Authentication errors 1xxx
    Unauthorized = 1001,
    TokenExpired = 1002,
    InvalidCredentials = 1003,

    // Cluster errors 2xxx
    ClusterNotFound = 2001,
    ClusterConnectionFailed = 2002,
    ClusterTimeout = 2003,
    ClusterAuthFailed = 2004,

    // Resource errors 3xxx
    QueryNotFound = 3001,
    QueryKillFailed = 3002,

    // Validation errors 4xxx
    ValidationError = 4001,
    InvalidInput = 4002,

    // System errors 5xxx
    InternalError = 5001,
    DatabaseError = 5002,

    // System Function errors 6xxx
    SystemFunctionNotFound = 6001,
    SystemFunctionDuplicate = 6002,
    CategoryFull = 6003,
    InvalidSQL = 6004,
    SQLSafetyViolation = 6005,
    CategoryCannotDelete = 6006,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, message)
    }

    pub fn cluster_not_found(cluster_id: i64) -> Self {
        Self::new(
            ErrorCode::ClusterNotFound,
            format!("Cluster with ID {} not found", cluster_id),
        )
    }

    pub fn cluster_connection_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ClusterConnectionFailed, message)
    }

    pub fn invalid_credentials() -> Self {
        Self::new(ErrorCode::InvalidCredentials, "Invalid credentials")
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn database_error(err: impl fmt::Display) -> Self {
        Self::new(ErrorCode::DatabaseError, format!("Database error: {}", err))
    }

    // System Function specific constructors
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::SystemFunctionNotFound, message)
    }

    pub fn invalid_sql(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidSQL, message)
    }

    pub fn category_full(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::CategoryFull, message)
    }

    pub fn sql_safety_violation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::SQLSafetyViolation, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code {
            1001..=1999 => StatusCode::UNAUTHORIZED,
            2001..=2999 => StatusCode::BAD_REQUEST,
            3001..=3999 => StatusCode::NOT_FOUND,
            4001..=4999 => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

// Implement From for common error types
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::database_error(err)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::internal_error(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::internal_error(format!("JSON serialization error: {}", err))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

