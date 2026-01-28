use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::AppState;
use crate::config::AuthMode;
use crate::domain::{Operator, OperatorRole};

use crate::auth::AuthSession;

/// Check if request is authenticated via session cookie or bearer token (hybrid auth)
/// Returns Ok(()) if authenticated, Err(StatusCode) if not
/// When auth mode is None, always returns Ok
pub fn require_auth(
    state: &AppState,
    auth_session: &AuthSession,
    auth_header: Option<&str>,
) -> Result<(), StatusCode> {
    // If auth is disabled, allow all
    if matches!(state.settings.http.auth.mode, AuthMode::None) {
        return Ok(());
    }

    // mTLS is validated at transport layer - if request reached here, client is authenticated
    if matches!(state.settings.http.auth.mode, AuthMode::Mtls) {
        return Ok(());
    }

    // Session-based auth (Credentials mode or hybrid)
    if auth_session.user.is_some() {
        return Ok(());
    }

    // Credentials mode requires session auth only (no bearer fallback)
    if matches!(state.settings.http.auth.mode, AuthMode::Credentials) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Fall back to bearer token (CLI/detectors) for Bearer mode
    if let Some(header_str) = auth_header {
        if let Some(provided_token) = header_str.strip_prefix("Bearer ") {
            if let Some(ref expected_token) = state.bearer_token {
                if constant_time_eq(provided_token.as_bytes(), expected_token.as_bytes()) {
                    return Ok(());
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Check if the authenticated user has the required role
/// Returns the operator if authorized, or UNAUTHORIZED/FORBIDDEN status
/// When auth mode is None, returns a synthetic admin operator
pub fn require_role(
    state: &AppState,
    auth_session: &AuthSession,
    auth_header: Option<&str>,
    required_role: OperatorRole,
) -> Result<Operator, StatusCode> {
    // If auth is disabled, return synthetic admin
    if matches!(state.settings.http.auth.mode, AuthMode::None) {
        return Ok(Operator {
            operator_id: uuid::Uuid::nil(),
            username: "anonymous".to_string(),
            password_hash: String::new(),
            role: OperatorRole::Admin,
            created_at: chrono::Utc::now(),
            created_by: None,
            last_login_at: None,
        });
    }

    // mTLS users get admin privileges (certificate-based trust)
    if matches!(state.settings.http.auth.mode, AuthMode::Mtls) {
        return Ok(Operator {
            operator_id: uuid::Uuid::nil(),
            username: "mtls-client".to_string(),
            password_hash: String::new(),
            role: OperatorRole::Admin,
            created_at: chrono::Utc::now(),
            created_by: None,
            last_login_at: None,
        });
    }

    // Check session cookie first (browser/dashboard)
    if let Some(ref operator) = auth_session.user {
        if operator.role.has_permission(&required_role) {
            return Ok(operator.clone());
        } else {
            tracing::warn!(
                username = %operator.username,
                role = %operator.role,
                required = %required_role,
                "insufficient permissions"
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // Bearer token gets operator-level access (can view and withdraw, but not admin)
    if let Some(header_str) = auth_header {
        if let Some(provided_token) = header_str.strip_prefix("Bearer ") {
            if let Some(ref expected_token) = state.bearer_token {
                if constant_time_eq(provided_token.as_bytes(), expected_token.as_bytes()) {
                    // Bearer tokens get operator role (can withdraw but not manage users/safelist)
                    let bearer_role = OperatorRole::Operator;
                    if bearer_role.has_permission(&required_role) {
                        return Ok(Operator {
                            operator_id: uuid::Uuid::nil(),
                            username: "bearer-token".to_string(),
                            password_hash: String::new(),
                            role: bearer_role,
                            created_at: chrono::Utc::now(),
                            created_by: None,
                            last_login_at: None,
                        });
                    } else {
                        tracing::warn!(
                            role = %bearer_role,
                            required = %required_role,
                            "bearer token has insufficient permissions"
                        );
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Bearer token authentication middleware (legacy, for CLI/detectors only)
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match state.settings.http.auth.mode {
        AuthMode::None => Ok(next.run(request).await),
        AuthMode::Bearer => validate_bearer_token(&state, request, next).await,
        AuthMode::Credentials => {
            // Credentials mode uses session auth, handled by axum-login layer
            // This middleware is for bearer-only routes, so reject here
            Err(StatusCode::UNAUTHORIZED)
        }
        AuthMode::Mtls => {
            // mTLS is handled at the transport layer, not here
            // If we reach this point with mTLS configured, connection was already validated
            Ok(next.run(request).await)
        }
    }
}

/// Bearer token authentication middleware for API routes
/// Session-based auth is used only for WebSocket and /v1/auth/* endpoints
pub async fn hybrid_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If auth is disabled, allow all
    if matches!(state.settings.http.auth.mode, AuthMode::None) {
        return Ok(next.run(request).await);
    }

    // Check bearer token (CLI/detectors)
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(header_str) = auth_header.to_str() {
            if header_str.starts_with("Bearer ") {
                return validate_bearer_token(&state, request, next).await;
            }
        }
    }

    tracing::debug!("no valid session or bearer token");
    Err(StatusCode::UNAUTHORIZED)
}

async fn validate_bearer_token(
    state: &AppState,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Use cached token from startup (avoids per-request env lookups)
    let expected_token = match &state.bearer_token {
        Some(token) => token.as_str(),
        None => {
            // Token was not loaded at startup - this is a configuration error
            tracing::error!("bearer auth enabled but no token was loaded at startup");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let provided_token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            tracing::warn!("missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Constant-time comparison to prevent timing attacks
    if !constant_time_eq(provided_token.as_bytes(), expected_token.as_bytes()) {
        tracing::warn!("invalid bearer token");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
