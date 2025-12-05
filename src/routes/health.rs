use axum::{Router, routing::get, Json};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health))
}

async fn health() -> Json<&'static str> {
    Json("ok")
}
