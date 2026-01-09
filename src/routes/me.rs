use axum::{
    extract::State,
    routing::get,
    Json, Router,
};

use crate::{
    error::AppError,
    models::{api::ApiResponse, auth::AuthUser},
    state::AppState,
};


pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(me))
}

async fn me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {
    let user = state
        .users_service
        .get_user_by_id(claims.user_id)
        .await?;

    Ok(Json(ApiResponse::success(user)))
}

