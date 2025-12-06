use axum::{Json, extract::{State, Path}};
use serde_json::Value;
use crate::state::AppState;

pub fn routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/:id", axum::routing::get(get_user))
}

async fn get_user(
    Path(id): Path<String>,
    State(state): State<AppState>
) -> Json<Value> {
    Json(state.users_service.get_mock_user_by_id(id))
}
