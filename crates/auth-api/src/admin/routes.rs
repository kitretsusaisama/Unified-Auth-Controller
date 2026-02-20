//! Admin UI routes

use axum::{
    routing::get,
    Router,
};
use crate::AppState;

use super::handlers;

/// Create admin UI router
///
/// Mounts all admin interface routes under `/admin`
pub fn admin_router() -> Router<AppState> {
    Router::new()
        // Public auth pages
        .route("/login", get(handlers::login_page))
        .route("/register", get(handlers::register_page))

        // Protected dashboard pages
        // TODO: Add auth middleware
        .route("/dashboard", get(handlers::dashboard_page))
        .route("/users", get(handlers::users_page))
        .route("/roles", get(handlers::roles_page))
        .route("/settings", get(handlers::settings_page))

        // Logout
        .route("/logout", get(handlers::logout))
}
