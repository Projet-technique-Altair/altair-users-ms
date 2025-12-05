use axum::{response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "status": "error",
            "message": self.message
        }));

        (axum::http::StatusCode::BAD_REQUEST, body).into_response()
    }
}

impl<E: std::fmt::Display> From<E> for ApiError {
    fn from(err: E) -> Self {
        ApiError { message: err.to_string() }
    }
}
