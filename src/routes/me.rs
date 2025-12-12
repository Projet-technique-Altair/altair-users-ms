use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use crate::state::AppState;
use crate::models::user::User;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(me))
}

async fn me(
    State(state): State<AppState>,
) -> Result<Json<User>, axum::http::StatusCode> {
    let user = state
        .users_service
        .get_first_user()
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}
