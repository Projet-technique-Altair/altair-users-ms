use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::user::{User, UserRow},
};

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct UserSanction {
    pub sanction_id: Uuid,
    pub user_id: Uuid,
    pub actor_user_id: Uuid,
    pub action: String,
    pub reason: String,
    pub status: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
    pub resolved_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct UserAuditLog {
    pub audit_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UserLoginResolution {
    pub user: User,
    pub is_new_user: bool,
    pub previous_last_login: Option<chrono::NaiveDateTime>,
}

#[derive(Clone)]
pub struct UsersService {
    db: PgPool,
}

impl UsersService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))?;

        Ok(row.into())
    }

    pub async fn get_keycloak_id_by_user_id(&self, user_id: Uuid) -> Result<String, AppError> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT keycloak_id
            FROM users
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))
    }

    // ==========================
    // GET /user/pseudo
    // ==========================
    pub async fn get_user_pseudo_by_id(&self, user_id: Uuid) -> Result<(Uuid, String), AppError> {
        let row = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            SELECT user_id, pseudo
            FROM users
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))?;

        Ok(row)
    }

    // ==========================
    // GET /users/search?q=
    // ==========================
    pub async fn search_users(&self, query: String) -> Result<Vec<User>, AppError> {
        let pattern = format!("%{}%", query);

        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE pseudo ILIKE $1
            ORDER BY pseudo
            LIMIT 10
            "#,
        )
        .bind(pattern)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_users_admin(
        &self,
        query: Option<String>,
        role: Option<String>,
        account_status: Option<String>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<User>, i64), AppError> {
        let query_pattern = query
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value));
        let role_filter = role
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let status_filter = account_status
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());

        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE ($1::TEXT IS NULL OR pseudo ILIKE $1 OR email ILIKE $1 OR name ILIKE $1)
              AND ($2::TEXT IS NULL OR role = $2)
              AND ($3::TEXT IS NULL OR account_status = $3)
            ORDER BY created_at DESC
            LIMIT $4
            OFFSET $5
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(role_filter.as_deref())
        .bind(status_filter.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM users
            WHERE ($1::TEXT IS NULL OR pseudo ILIKE $1 OR email ILIKE $1 OR name ILIKE $1)
              AND ($2::TEXT IS NULL OR role = $2)
              AND ($3::TEXT IS NULL OR account_status = $3)
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(role_filter.as_deref())
        .bind(status_filter.as_deref())
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok((rows.into_iter().map(|row| row.into()).collect(), total))
    }

    pub async fn get_user_by_keycloak_id(&self, keycloak_id: &str) -> Result<User, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE keycloak_id = $1
            "#,
        )
        .bind(keycloak_id)
        .fetch_one(&self.db)
        .await
        .map_err(|_| AppError::NotFound("User not found".into()))?;

        Ok(row.into())
    }

    pub async fn get_or_create_user_from_keycloak(
        &self,
        keycloak_id: &str,
        role: &str,
        name: &str,
        pseudo: &str,
        email: &str,
    ) -> Result<UserLoginResolution, AppError> {
        let existing_user = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE keycloak_id = $1
            "#,
        )
        .bind(keycloak_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        if let Some(existing_user) = existing_user {
            let previous_last_login = existing_user.last_login;

            sqlx::query(
                r#"
                UPDATE users
                SET last_login = NOW()
                WHERE keycloak_id = $1
                "#,
            )
            .bind(keycloak_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

            return Ok(UserLoginResolution {
                user: self.get_user_by_keycloak_id(keycloak_id).await?,
                is_new_user: false,
                previous_last_login,
            });
        }

        sqlx::query(
            r#"
            INSERT INTO users (
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                last_login
            )
            VALUES ($1, $2, 'active', $3, $4, $5, NOW())
            ON CONFLICT (keycloak_id) DO NOTHING
            "#,
        )
        .bind(keycloak_id)
        .bind(role)
        .bind(name)
        .bind(pseudo)
        .bind(email)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(UserLoginResolution {
            user: self.get_user_by_keycloak_id(keycloak_id).await?,
            is_new_user: true,
            previous_last_login: None,
        })
    }

    pub async fn email_exists_for_other_user(
        &self,
        keycloak_id: &str,
        email: &str,
    ) -> Result<bool, AppError> {
        // Excludes current user to allow keeping the same email.
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE email = $1
                  AND keycloak_id <> $2
            )
            "#,
        )
        .bind(email)
        .bind(keycloak_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(exists)
    }

    pub async fn update_user_profile_by_keycloak_id(
        &self,
        keycloak_id: &str,
        pseudo: Option<&str>,
        email: Option<&str>,
        role: Option<&str>,
    ) -> Result<User, AppError> {
        // COALESCE keeps existing DB values for fields omitted by caller.
        sqlx::query(
            r#"
            UPDATE users
            SET
                pseudo = COALESCE($2, pseudo),
                email = COALESCE($3, email),
                role = COALESCE($4, role)
            WHERE keycloak_id = $1
            "#,
        )
        .bind(keycloak_id)
        .bind(pseudo)
        .bind(email)
        .bind(role)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        self.get_user_by_keycloak_id(keycloak_id).await
    }

    pub async fn update_account_status_admin(
        &self,
        actor_user_id: Uuid,
        target_user_id: Uuid,
        account_status: &str,
        reason: &str,
    ) -> Result<User, AppError> {
        if !matches!(account_status, "active" | "suspended" | "banned") {
            return Err(AppError::BadRequest(
                "account_status must be active, suspended, or banned".into(),
            ));
        }

        let row = sqlx::query_as::<_, UserRow>(
            r#"
            UPDATE users
            SET account_status = $1
            WHERE user_id = $2
            RETURNING
                user_id,
                keycloak_id,
                role,
                account_status,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            "#,
        )
        .bind(account_status)
        .bind(target_user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

        if let Err(err) = self
            .insert_audit_log(
                Some(actor_user_id),
                Some(target_user_id),
                "user.account_status.updated",
                serde_json::json!({
                    "account_status": account_status,
                    "reason": reason.trim()
                }),
            )
            .await
        {
            eprintln!(
                "audit log write failed after account status update for user {target_user_id}: {err}"
            );
        }

        if account_status == "active" {
            sqlx::query(
                r#"
                UPDATE user_sanctions
                SET status = 'resolved',
                    resolved_at = COALESCE(resolved_at, NOW())
                WHERE user_id = $1
                  AND status = 'active'
                "#,
            )
            .bind(target_user_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        }

        Ok(row.into())
    }

    pub async fn apply_sanction_admin(
        &self,
        actor_user_id: Uuid,
        target_user_id: Uuid,
        action: &str,
        reason: &str,
        duration_days: Option<i64>,
    ) -> Result<(User, UserSanction), AppError> {
        if !matches!(action, "warn" | "suspend" | "ban") {
            return Err(AppError::BadRequest(
                "sanction action must be warn, suspend, or ban".into(),
            ));
        }

        let trimmed_reason = reason.trim();
        if trimmed_reason.is_empty() {
            return Err(AppError::BadRequest("reason is required".into()));
        }

        let expires_at = if action == "suspend" {
            duration_days
                .filter(|days| *days > 0)
                .map(|days| chrono::Utc::now().naive_utc() + chrono::Duration::days(days))
        } else {
            None
        };

        let sanction = sqlx::query_as::<_, UserSanction>(
            r#"
            INSERT INTO user_sanctions (
                user_id,
                actor_user_id,
                action,
                reason,
                status,
                expires_at
            )
            VALUES ($1, $2, $3, $4, 'active', $5)
            RETURNING
                sanction_id,
                user_id,
                actor_user_id,
                action,
                reason,
                status,
                expires_at,
                created_at,
                resolved_at
            "#,
        )
        .bind(target_user_id)
        .bind(actor_user_id)
        .bind(action)
        .bind(trimmed_reason)
        .bind(expires_at)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let user = match action {
            "warn" => self.get_user_by_id(target_user_id).await?,
            "suspend" => {
                self.update_account_status_admin(
                    actor_user_id,
                    target_user_id,
                    "suspended",
                    trimmed_reason,
                )
                .await?
            }
            "ban" => {
                self.update_account_status_admin(
                    actor_user_id,
                    target_user_id,
                    "banned",
                    trimmed_reason,
                )
                .await?
            }
            _ => unreachable!(),
        };

        if let Err(err) = self
            .insert_audit_log(
                Some(actor_user_id),
                Some(target_user_id),
                "user.sanction.created",
                serde_json::json!({
                    "sanction_id": sanction.sanction_id,
                    "action": action,
                    "reason": trimmed_reason,
                    "expires_at": expires_at
                }),
            )
            .await
        {
            eprintln!(
                "audit log write failed after {action} sanction for user {target_user_id}: {err}"
            );
        }

        Ok((user, sanction))
    }

    pub async fn list_user_sanctions(&self, user_id: Uuid) -> Result<Vec<UserSanction>, AppError> {
        sqlx::query_as::<_, UserSanction>(
            r#"
            SELECT
                sanction_id,
                user_id,
                actor_user_id,
                action,
                reason,
                status,
                expires_at,
                created_at,
                resolved_at
            FROM user_sanctions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn list_user_audit_logs(&self, user_id: Uuid) -> Result<Vec<UserAuditLog>, AppError> {
        sqlx::query_as::<_, UserAuditLog>(
            r#"
            SELECT
                audit_id,
                actor_user_id,
                target_user_id,
                action,
                metadata,
                created_at
            FROM user_audit_logs
            WHERE target_user_id = $1 OR actor_user_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    async fn insert_audit_log(
        &self,
        actor_user_id: Option<Uuid>,
        target_user_id: Option<Uuid>,
        action: &str,
        metadata: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_audit_logs (
                actor_user_id,
                target_user_id,
                action,
                metadata
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(actor_user_id)
        .bind(target_user_id)
        .bind(action)
        .bind(metadata)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}
