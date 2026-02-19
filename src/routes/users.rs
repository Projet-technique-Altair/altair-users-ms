use axum::{
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::api::ApiResponse,
    services::extractor::extract_caller,
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_user))
}
async fn get_user(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {
    let caller = extract_caller(&headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_self = caller.user_id == target_user_id;

    if !is_admin && !is_self {
        return Err(AppError::Forbidden(
            "You are not allowed to access this user".to_string(),
        ));
    }

    let user = state.users_service.get_user_by_id(target_user_id).await?;
    Ok(Json(ApiResponse::success(user)))
}
