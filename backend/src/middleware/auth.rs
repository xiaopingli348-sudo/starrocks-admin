use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::utils::{ApiError, JwtUtil};

#[derive(Clone)]
pub struct AuthState {
    pub jwt_util: Arc<JwtUtil>,
}

// Extract user ID from JWT token
pub async fn auth_middleware(
    State(state): State<AuthState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let uri = req.uri().to_string();
    let method = req.method().to_string();

    tracing::debug!("Auth middleware processing: {} {}", method, uri);

    // Get Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = if let Some(auth) = auth_header {
        // Extract token from "Bearer <token>"
        if let Some(stripped) = auth.strip_prefix("Bearer ") {
            stripped
        } else {
            tracing::warn!("Invalid authorization header format for {} {}", method, uri);
            return Err(ApiError::unauthorized("Invalid authorization header format"));
        }
    } else {
        tracing::warn!("Missing authorization header for {} {}", method, uri);
        return Err(ApiError::unauthorized("Missing authorization header"));
    };

    tracing::debug!("Verifying JWT token for {} {}", method, uri);
    // Verify token
    let claims = state.jwt_util.verify_token(token).map_err(|e| {
        tracing::warn!("JWT token verification failed for {} {}: {:?}", method, uri, e);
        e
    })?;

    let user_id = claims.sub.parse::<i64>().unwrap_or(0);
    tracing::debug!(
        "JWT token verified for user {} (ID: {}) on {} {}",
        claims.username,
        user_id,
        method,
        uri
    );

    // Add user ID to request extensions
    req.extensions_mut().insert(user_id);
    req.extensions_mut().insert(claims.username.clone());

    Ok(next.run(req).await)
}

// Extract user ID from request extensions
#[allow(dead_code)]
pub fn get_user_id_from_request(req: &Request) -> Option<i64> {
    req.extensions().get::<i64>().copied()
}

#[allow(dead_code)]
pub fn get_username_from_request(req: &Request) -> Option<String> {
    req.extensions().get::<String>().cloned()
}
