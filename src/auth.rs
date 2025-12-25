//! Bearer Token authentication middleware for MCP HTTP endpoint

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{debug, warn};

/// Extract and validate Bearer token from Authorization header
fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;

    if !auth_header.to_ascii_lowercase().starts_with("bearer ") {
        return None;
    }

    let token = auth_header[7..].trim();
    if token.is_empty() {
        return None;
    }

    Some(token.to_string())
}

/// Authentication state holding the expected token
#[derive(Clone, Default)]
pub struct AuthState {
    pub token: Option<Arc<String>>,
}

/// Create bearer authentication middleware
///
/// If `expected_token` is `None`, authentication is disabled (all requests pass).
/// If `expected_token` is `Some(token)`, requests must include a valid
/// `Authorization: Bearer <token>` header.
pub fn bearer_auth_middleware(expected_token: Option<String>) -> AuthState {
    AuthState {
        token: expected_token.map(Arc::new),
    }
}

/// Authentication middleware function
pub async fn auth_middleware(
    State(state): State<AuthState>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(expected) = &state.token {
        let token = extract_bearer_token(request.headers());

        match token {
            Some(ref t) if t == expected.as_str() => {
                debug!("Bearer token authentication successful");
            }
            Some(_) => {
                warn!(
                    "Bearer token authentication failed: invalid token, {}",
                    request.uri()
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
            None => {
                warn!(
                    "Bearer token authentication failed: missing token, {}",
                    request.uri()
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        debug!("Authentication disabled, allowing request");
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();

        // No token
        assert!(extract_bearer_token(&headers).is_none());

        // Valid Bearer token
        headers.insert("authorization", "Bearer my-token-123".parse().unwrap());
        assert_eq!(
            extract_bearer_token(&headers),
            Some("my-token-123".to_string())
        );

        // Lowercase bearer
        headers.insert("authorization", "bearer my-token-456".parse().unwrap());
        assert_eq!(
            extract_bearer_token(&headers),
            Some("my-token-456".to_string())
        );

        // Mixed case
        headers.insert("authorization", "BEARER my-token-789".parse().unwrap());
        assert_eq!(
            extract_bearer_token(&headers),
            Some("my-token-789".to_string())
        );

        // Empty token
        headers.insert("authorization", "Bearer ".parse().unwrap());
        assert!(extract_bearer_token(&headers).is_none());

        // Wrong scheme
        headers.insert("authorization", "Basic abc123".parse().unwrap());
        assert!(extract_bearer_token(&headers).is_none());
    }
}
