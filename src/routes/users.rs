use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
    body::Body,
    http::Request,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::api::ApiResponse,
    state::AppState,
};


pub fn routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_user))
}
/*
async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {
    let user = state.users_service.get_user_by_id(user_id).await?;
    Ok(Json(ApiResponse::success(user)))
}*/

/*async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    req: Request<Body>,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {

    let requester_id: Uuid = req
        .headers()
        .get("x-altair-user-id")
        .ok_or(AppError::Unauthorized("Missing user id".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid user id".to_string()))?
        .parse()
        .map_err(|_| AppError::Unauthorized("Invalid user id".to_string()))?;

    let roles_header = req
        .headers()
        .get("x-altair-roles")
        .ok_or(AppError::Unauthorized("Missing roles".to_string()))?
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid roles".to_string()))?;

    let is_admin = roles_header
        .split(',')
        .any(|r| r.trim() == "admin");

    if requester_id != user_id && !is_admin {
        return Err(AppError::Forbidden(
            "You can only access your own user".to_string(),
        ));
    }

    let user = state.users_service.get_user_by_id(user_id).await?;
    Ok(Json(ApiResponse::success(user)))
}*/

use axum::http::HeaderMap;
use crate::services::extractor::extract_caller;


async fn get_user(
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<crate::models::User>>, AppError> {

    let caller = extract_caller(&headers)?;

    let is_admin = caller.roles.iter().any(|r| r == "admin");
    let is_self = caller.user_id == target_user_id;

    if !is_admin && !is_self {
        return Err(AppError::Forbidden("You are not allowed to access this user".to_string(),
    ));

    }

    let user = state.users_service.get_user_by_id(target_user_id).await?;
    Ok(Json(ApiResponse::success(user)))
}
