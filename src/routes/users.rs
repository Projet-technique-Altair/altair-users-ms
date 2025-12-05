use axum::{
    Router,
    routing::get,
    extract::{Path, State},
    Json
};
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_user))
}

async fn get_user(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let user = state.users.find_user(id).await;
    Json(user)
}
