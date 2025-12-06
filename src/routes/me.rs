use axum::{Json, extract::State};
use serde_json::Value;
use crate::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new().route("/", axum::routing::get(get_me))
}

async fn get_me(
    State(state): State<AppState>
) -> Json<Value> {
    Json(state.users_service.get_mock_user())
}
