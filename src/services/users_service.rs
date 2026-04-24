use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::user::{User, UserRow},
};

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

        let rows = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                keycloak_id,
                role,
                name,
                pseudo,
                email,
                avatar,
                last_login,
                created_at
            FROM users
            WHERE ($1::TEXT IS NULL OR pseudo ILIKE $1 OR email ILIKE $1 OR name ILIKE $1)
              AND ($2::TEXT IS NULL OR role = $2)
            ORDER BY created_at DESC
            LIMIT $3
            OFFSET $4
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(role_filter.as_deref())
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
            "#,
        )
        .bind(query_pattern.as_deref())
        .bind(role_filter.as_deref())
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
                name,
                pseudo,
                email,
                last_login
            )
            VALUES ($1, $2, $3, $4, $5, NOW())
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
}
