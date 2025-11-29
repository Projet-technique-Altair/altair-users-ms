use axum::{Router, routing::get, Json};
use serde_json::json;

pub fn metrics_routes() -> Router {
    Router::new().route("/", get(basic_metrics))
}

async fn basic_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "uptime": 12345,
        "requests_total": 42,
        "service": "users-ms"
    }))
}
