use axum::{extract::State, routing::get, Json, Router};

use crate::{
    error::AppError, extractors::auth_user::AuthUser, models::api::ApiResponse, state::AppState,
};

use crate::models::User;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(me))
}

async fn me(
    State(state): State<AppState>,
    AuthUser {
        keycloak_id,
        name,
        email,
        roles,
    }: AuthUser,
) -> Result<Json<ApiResponse<User>>, AppError> {
    // 🎯 Choix du rôle MVP (1 seul rôle stocké en DB)
    let role = if roles.iter().any(|r| r == "admin") {
        "admin"
    } else if roles.iter().any(|r| r == "creator") {
        "creator"
    } else {
        "learner"
    };

    let user = state
        .users_service
        .get_or_create_user_from_keycloak(&keycloak_id, role, &name, &email)
        .await?;

    Ok(Json(ApiResponse::success(user)))
}
