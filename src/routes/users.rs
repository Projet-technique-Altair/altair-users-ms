use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::json;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(me))
        .route("/:id", get(get_user))
}

async fn me(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "me": {
            "id": 1,
            "name": "Test User"
        }
    }))
}

async fn get_user(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "user": {
            "id": id,
            "name": format!("User {}", id)
        }
    }))
}
