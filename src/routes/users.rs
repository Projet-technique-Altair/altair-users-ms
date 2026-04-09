use axum::{
    extract::{Path, State, Query},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use uuid::Uuid;
use serde::Deserialize;

use crate::{
    error::AppError,
    models::{api::ApiResponse, User},
    services::extractor::extract_caller,
    state::AppState,
};


#[derive(Deserialize)]
pub struct SearchUsersQuery {
    pub q: String,
}

use serde::Serialize;

#[derive(Serialize)]
struct UserPseudo {
    user_id: Uuid,
    pseudo: String,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/:id", get(get_user))
    .route("/:id/pseudo", get(get_user_pseudo))
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


// ==========================
// GET /user/pseudo
// ==========================
async fn get_user_pseudo(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserPseudo>>, AppError> {

    let (id, pseudo) = state
        .users_service
        .get_user_pseudo_by_id(user_id)
        .await?;

    Ok(Json(ApiResponse::success(UserPseudo {
        user_id: id,
        pseudo,
    })))
}



// ==========================
// GET /users/search?q=
// ==========================
pub async fn search_users(
    State(state): State<AppState>,
    Query(params): Query<SearchUsersQuery>,
) -> Result<Json<ApiResponse<Vec<User>>>, AppError> {

    let users = state
        .users_service
        .search_users(params.q)
        .await?;

    Ok(Json(ApiResponse::success(users)))
}