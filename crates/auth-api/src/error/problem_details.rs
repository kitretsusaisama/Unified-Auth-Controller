//! RFC 7807 Problem Details for HTTP APIs

use serde::{Serialize, Deserialize};
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_url: String,

    pub title: String,

    pub status: u16,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,

    // Extensions
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

impl ProblemDetails {
    pub fn new(status: StatusCode, title: impl Into<String>) -> Self {
        Self {
            type_url: "about:blank".to_string(),
            title: title.into(),
            status: status.as_u16(),
            detail: None,
            instance: None,
            extensions: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_type(mut self, type_url: impl Into<String>) -> Self {
        self.type_url = type_url.into();
        self
    }

    pub fn with_extension(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.extensions.insert(key.into(), value.into());
        self
    }
}

impl IntoResponse for ProblemDetails {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // Content-Type: application/problem+json
        (
            status,
            [("content-type", "application/problem+json")],
            Json(self)
        ).into_response()
    }
}
