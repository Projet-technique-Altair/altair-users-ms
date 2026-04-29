use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{api::ApiResponse, User},
    services::extractor::extract_caller,
    services::users_service::{UserAuditLog, UserSanction},
    state::AppState,
};

#[derive(Deserialize)]
pub struct SearchUsersQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct AdminUsersQuery {
    pub q: Option<String>,
    pub role: Option<String>,
    pub account_status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateSanctionRequest {
    pub action: String,
    pub reason: String,
    pub duration_days: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateAccountStatusRequest {
    pub account_status: String,
    pub reason: Option<String>,
}

#[derive(Serialize)]
struct UserPseudo {
    user_id: Uuid,
    pseudo: String,
}

#[derive(Serialize)]
pub struct PaginatedUsers {
    items: Vec<User>,
    total: i64,
    limit: i64,
    offset: i64,
}

#[derive(Serialize)]
pub struct AdminUserDetail {
    user: User,
    sanctions: Vec<UserSanction>,
    audit_logs: Vec<UserAuditLog>,
}

#[derive(Serialize)]
pub struct AdminSanctionResponse {
    user: User,
    sanction: UserSanction,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_user))
        .route("/:id/pseudo", get(get_user_pseudo))
}

fn ensure_admin(headers: &HeaderMap) -> Result<crate::services::extractor::Caller, AppError> {
    let caller = extract_caller(headers)?;
    let is_admin = caller.roles.iter().any(|r| r == "admin");

    if !is_admin {
        return Err(AppError::Forbidden("Admin role is required".to_string()));
    }

    Ok(caller)
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
    let (id, pseudo) = state.users_service.get_user_pseudo_by_id(user_id).await?;

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
    let users = state.users_service.search_users(params.q).await?;

    Ok(Json(ApiResponse::success(users)))
}

// ==========================
// GET /admin/users
// ==========================
pub async fn list_users_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AdminUsersQuery>,
) -> Result<Json<ApiResponse<PaginatedUsers>>, AppError> {
    ensure_admin(&headers)?;

    let limit = params.limit.unwrap_or(200).clamp(1, 500);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = state
        .users_service
        .list_users_admin(params.q, params.role, params.account_status, limit, offset)
        .await?;

    Ok(Json(ApiResponse::success(PaginatedUsers {
        items,
        total,
        limit,
        offset,
    })))
}

pub async fn get_admin_user_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<AdminUserDetail>>, AppError> {
    ensure_admin(&headers)?;

    let user = state.users_service.get_user_by_id(user_id).await?;
    let sanctions = state.users_service.list_user_sanctions(user_id).await?;
    let audit_logs = state.users_service.list_user_audit_logs(user_id).await?;

    Ok(Json(ApiResponse::success(AdminUserDetail {
        user,
        sanctions,
        audit_logs,
    })))
}

pub async fn create_admin_user_sanction(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(body): Json<CreateSanctionRequest>,
) -> Result<Json<ApiResponse<AdminSanctionResponse>>, AppError> {
    let caller = ensure_admin(&headers)?;
    let action = body.action.trim().to_lowercase();
    let keycloak_id = if matches!(action.as_str(), "suspend" | "ban") {
        Some(
            state
                .users_service
                .get_keycloak_id_by_user_id(user_id)
                .await?,
        )
    } else {
        None
    };

    let (user, sanction) = state
        .users_service
        .apply_sanction_admin(
            caller.user_id,
            user_id,
            action.as_str(),
            body.reason.trim(),
            body.duration_days,
        )
        .await?;

    if let (Some(keycloak), Some(keycloak_id)) = (&state.keycloak_admin_service, keycloak_id) {
        if let Err(err) = keycloak.set_user_enabled(&keycloak_id, false).await {
            eprintln!("keycloak sync failed after {action} sanction for user {user_id}: {err}");
        } else if let Err(err) = keycloak.logout_user_sessions(&keycloak_id).await {
            eprintln!("keycloak logout failed after {action} sanction for user {user_id}: {err}");
        }
    }

    Ok(Json(ApiResponse::success(AdminSanctionResponse {
        user,
        sanction,
    })))
}

pub async fn update_admin_account_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateAccountStatusRequest>,
) -> Result<Json<ApiResponse<User>>, AppError> {
    let caller = ensure_admin(&headers)?;
    let account_status = body.account_status.trim().to_lowercase();
    let reason = body
        .reason
        .unwrap_or_else(|| "Manual admin status update".into());

    let keycloak_id = state
        .users_service
        .get_keycloak_id_by_user_id(user_id)
        .await?;

    let user = state
        .users_service
        .update_account_status_admin(
            caller.user_id,
            user_id,
            account_status.as_str(),
            reason.trim(),
        )
        .await?;

    if let Some(keycloak) = &state.keycloak_admin_service {
        let enabled = account_status == "active";
        if enabled {
            keycloak.set_user_enabled(&keycloak_id, true).await?;
        } else if let Err(err) = keycloak.set_user_enabled(&keycloak_id, false).await {
            eprintln!("keycloak sync failed after blocking user {user_id}: {err}");
        } else {
            if let Err(err) = keycloak.logout_user_sessions(&keycloak_id).await {
                eprintln!("keycloak logout failed after blocking user {user_id}: {err}");
            }
        }
    }

    Ok(Json(ApiResponse::success(user)))
}
