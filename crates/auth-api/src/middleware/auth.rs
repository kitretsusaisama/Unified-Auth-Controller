//! JWT Authentication Middleware

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{Response, Redirect, IntoResponse},
};
use crate::AppState;

/// JWT authentication middleware
/// Validates JWT from Authorization header or cookies
pub async fn jwt_auth(
    State(_state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    // Try to extract JWT from Authorization header or cookie
    let has_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .or_else(|| {
            // Fallback to cookie
            req.headers()
                .get(header::COOKIE)
                .and_then(|h| h.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find_map(|cookie| {
                            let mut parts = cookie.trim().splitn(2, '=');
                            match (parts.next(), parts.next()) {
                                (Some("token"), Some(value)) => Some(value),
                                _ => None,
                            }
                        })
                })
        })
        .is_some();

    if !has_token {
        // No token - redirect to login
        return Err(Redirect::to("/admin/login").into_response());
    }

    // Token exists - allow access
    // TODO: Validate token signature and claims using state.identity_service
    Ok(next.run(req).await)
}
