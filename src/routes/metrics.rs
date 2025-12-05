use axum::{Router, routing::get, Json};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_metrics))
}

async fn get_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "service": "users-ms",
        "status": "ok"
    }))
}
