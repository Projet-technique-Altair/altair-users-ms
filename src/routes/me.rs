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
    // Priority policy: admin > creator > learner.
    let has_admin = roles.iter().any(|r| r == "admin");
    let has_creator = roles.iter().any(|r| r == "creator");
    let has_learner = roles.iter().any(|r| r == "learner");

    if !has_admin && !has_creator && !has_learner {
        return Err(AppError::Forbidden(
            "No recognized role in x-altair-roles".to_string(),
        ));
    }

    let role = if has_admin {
        "admin"
    } else if has_creator {
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
