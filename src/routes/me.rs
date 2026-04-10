use axum::{extract::State, Json};

use crate::{
    error::AppError,
    extractors::auth_user::AuthUser,
    features::profile_update::{build_update_input, UpdateMePayload},
    models::api::ApiResponse,
    state::AppState,
};

use crate::models::User;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdatePasswordPayload {
    pub new_password: String,
}

pub(crate) async fn me(
    State(state): State<AppState>,
    AuthUser {
        keycloak_id,
        name,
        pseudo,
        email,
        roles,
    }: AuthUser,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let role = resolve_effective_role(&roles)?;

    let user = state
        .users_service
        .get_or_create_user_from_keycloak(&keycloak_id, role, &name, &pseudo, &email)
        .await?;

    Ok(Json(ApiResponse::success(user)))
}

pub(crate) async fn update_me(
    State(state): State<AppState>,
    AuthUser {
        keycloak_id, roles, ..
    }: AuthUser,
    Json(payload): Json<UpdateMePayload>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    // Normalize and validate requested changes before any side-effect.
    let caller_role = resolve_effective_role(&roles)?;
    let update_input = build_update_input(payload)?;

    // Role escalation is restricted to admin callers.
    if update_input.role.is_some() && caller_role != "admin" {
        return Err(AppError::Forbidden(
            "Only admin can update role".to_string(),
        ));
    }

    if update_input.pseudo.is_some() {
        return Err(AppError::Forbidden(
            "Username cannot be changed".to_string(),
        ));
    }

    // Reject local pseudo/email collisions early.
    /*if let Some(ref pseudo) = update_input.pseudo {
        let exists = state
            .users_service
            .pseudo_exists_for_other_user(&keycloak_id, pseudo)
            .await?;
        if exists {
            return Err(AppError::Conflict(
                "Pseudo already used by another user".to_string(),
            ));
        }
    }*/

    if let Some(ref email) = update_input.email {
        let exists = state
            .users_service
            .email_exists_for_other_user(&keycloak_id, email)
            .await?;
        if exists {
            return Err(AppError::Conflict(
                "Email already used by another user".to_string(),
            ));
        }
    }

    // Keycloak is synchronized first to avoid local-only divergence.
    let keycloak_service = state.keycloak_admin_service.as_ref().ok_or_else(|| {
        AppError::Internal(
            "Keycloak admin sync is not configured. Set KEYCLOAK_URL, KEYCLOAK_REALM, KEYCLOAK_ADMIN_USERNAME and KEYCLOAK_ADMIN_PASSWORD.".to_string(),
        )
    })?;

    keycloak_service
        .sync_profile(
            &keycloak_id,
            None,
            update_input.email.as_deref(),
            update_input.role.as_deref(),
        )
        .await?;

    // Persist the same changes in users DB once Keycloak sync succeeded.
    let updated = state
        .users_service
        .update_user_profile_by_keycloak_id(
            &keycloak_id,
            None,
            update_input.email.as_deref(),
            update_input.role.as_deref(),
        )
        .await?;

    Ok(Json(ApiResponse::success(updated)))
}

fn resolve_effective_role(roles: &[String]) -> Result<&'static str, AppError> {
    // Priority policy is deterministic: admin > creator > learner.
    let has_admin = roles.iter().any(|r| r == "admin");
    let has_creator = roles.iter().any(|r| r == "creator");
    let has_learner = roles.iter().any(|r| r == "learner");

    if !has_admin && !has_creator && !has_learner {
        return Err(AppError::Forbidden(
            "No recognized role in x-altair-roles".to_string(),
        ));
    }

    if has_admin {
        Ok("admin")
    } else if has_creator {
        Ok("creator")
    } else {
        Ok("learner")
    }
}

pub(crate) async fn toggle_my_role(
    State(state): State<AppState>,
    AuthUser {
        keycloak_id, roles, ..
    }: AuthUser,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let (_, new_role) = resolve_toggle_roles(&roles)?;

    let keycloak_service = state
        .keycloak_admin_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("Keycloak admin not configured".to_string()))?;

    keycloak_service
        .toggle_realm_role(&keycloak_id, new_role)
        .await?;

    let updated = state
        .users_service
        .update_user_profile_by_keycloak_id(&keycloak_id, None, None, Some(new_role))
        .await?;

    Ok(Json(ApiResponse::success(updated)))
}

fn resolve_toggle_roles(roles: &[String]) -> Result<(&'static str, &'static str), AppError> {
    let has_creator = roles.iter().any(|r| r == "creator");
    let has_learner = roles.iter().any(|r| r == "learner");

    match (has_creator, has_learner) {
        (false, true) => Ok(("learner", "creator")),
        (true, false) => Ok(("creator", "learner")),
        (true, true) => Ok(("creator", "learner")),
        (false, false) => Err(AppError::Forbidden("User has no valid role".into())),
    }
}

pub(crate) async fn update_password(
    State(state): State<AppState>,
    AuthUser { keycloak_id, .. }: AuthUser,
    Json(payload): Json<UpdatePasswordPayload>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let keycloak_service = state
        .keycloak_admin_service
        .as_ref()
        .ok_or_else(|| AppError::Internal("Keycloak admin not configured".to_string()))?;

    keycloak_service
        .update_password(&keycloak_id, &payload.new_password)
        .await?;

    Ok(Json(ApiResponse::success(())))
}
