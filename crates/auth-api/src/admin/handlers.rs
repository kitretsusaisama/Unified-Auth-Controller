//! Admin UI template handlers

use askama_axum::Template;
use axum::response::Redirect;

// ============================================================================
// Auth Page Templates
// ============================================================================

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate;

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate;

// Dashboard Templates
// ============================================================================

#[derive(Template)]
#[template(path = "dashboard/index.html")]
pub struct DashboardTemplate {
    pub title: String,
    pub page_title: String,
    pub active: String,
    pub user_email: String,
    pub user_initial: String,
    pub active_sessions: u32,
    pub total_roles: u32,
}

// ============================================================================
// Handler Functions
// ============================================================================

/// GET /admin/login - Login page
pub async fn login_page() -> LoginTemplate {
    LoginTemplate
}

/// GET /admin/register - Register page
pub async fn register_page() -> RegisterTemplate {
    RegisterTemplate
}

/// GET /admin/dashboard - Dashboard home
pub async fn dashboard_page() -> DashboardTemplate {
    DashboardTemplate {
        title: "Dashboard".to_string(),
        page_title: "Dashboard".to_string(),
        active: "dashboard".to_string(),
        user_email: "admin@example.com".to_string(),
        user_initial: "A".to_string(),
        active_sessions: 42,
        total_roles: 5,
    }
}

/// GET /admin/users - User management page
pub async fn users_page() -> askama_axum::Response {
    askama_axum::IntoResponse::into_response("User Management - Coming Soon")
}

/// GET /admin/roles - Role management page
pub async fn roles_page() -> askama_axum::Response {
    askama_axum::IntoResponse::into_response("Role Management - Coming Soon")
}

/// GET /admin/settings - Settings page
pub async fn settings_page() -> askama_axum::Response {
    askama_axum::IntoResponse::into_response("Settings - Coming Soon")
}

/// GET /admin/logout - Logout handler
pub async fn logout() -> Redirect {
    // TODO: Clear JWT cookie
    Redirect::to("/admin/login")
}

/// Helper to extract user email from JWT
/// TODO: Implement proper JWT extraction
fn get_user_email_from_jwt() -> Option<String> {
    Some("admin@example.com".to_string())
}
