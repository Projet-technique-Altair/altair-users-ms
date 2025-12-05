use axum::{response::{IntoResponse, Response}, Json};
use serde_json::json;

pub struct ApiError {
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.message
        }));

        (axum::http::StatusCode::BAD_REQUEST, body).into_response()
    }
}
