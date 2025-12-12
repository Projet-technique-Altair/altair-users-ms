use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::state::AppState;
use crate::models::user::User;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_user_by_id))
}

async fn get_user_by_id(
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<User>, axum::http::StatusCode> {
    let user = state
        .users_service
        .get_user_by_id(user_id)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}
