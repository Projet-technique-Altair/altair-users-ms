use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{api::ApiResponse, auth::AuthUser},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {
    match claims.role.as_str() {
        "admin" | "creator" => {}
        _ => return Err(AppError::Forbidden("Insufficient role".into())),
    }

    let user = state.users_service.get_user_by_id(user_id).await?;
    Ok(Json(ApiResponse::success(user)))
}
