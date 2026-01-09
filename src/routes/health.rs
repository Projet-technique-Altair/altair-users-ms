use axum::{extract::State, Json};
use serde_json::json;
use crate::state::AppState;

pub async fn health(
    State(_): State<AppState>,
) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        //"service": "labs-ms"
    }))
}
