use axum::{response::IntoResponse, Json};
use serde_json::json;
use auth_protocols::SamlService;

pub async fn metadata() -> impl IntoResponse {
    let saml_service = SamlService::new();
    match saml_service.generate_metadata() {
        Ok(xml) => (
            [(axum::http::header::CONTENT_TYPE, "application/xml")],
            xml
        ).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()}))
        ).into_response()
    }
}

pub async fn acs() -> impl IntoResponse {
    // Assertion Consumer Service endpoint
    Json(json!({
        "status": "success",
        "action": "saml_consume",
        "message": "SAML assertion would be processed here"
    }))
}
