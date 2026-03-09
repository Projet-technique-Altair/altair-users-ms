use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::user::{User, UserRow},
};

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
    // GET /users/search?q=
    // ==========================
    pub async fn search_users(
        &self,
        query: String,
    ) -> Result<Vec<User>, AppError> {

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
    ) -> Result<User, AppError> {
        // 1) Existing user: touch last_login and return fresh row.
        if self.get_user_by_keycloak_id(keycloak_id).await.is_ok() {
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

            return self.get_user_by_keycloak_id(keycloak_id).await;
        }

        // 2) New user
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

        // 3) Always fetch final row.
        self.get_user_by_keycloak_id(keycloak_id).await
    }


    pub async fn pseudo_exists_for_other_user(
        &self,
        keycloak_id: &str,
        pseudo: &str,
    ) -> Result<bool, AppError> {
        // Excludes current user to allow keeping the same pseudo.
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE pseudo = $1
                  AND keycloak_id <> $2
            )
            "#,
        )
        .bind(pseudo)
        .bind(keycloak_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(exists)
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

    async fn pseudo_exists(&self, pseudo: &str) -> Result<bool, AppError> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM users
                WHERE pseudo = $1
            )
            "#,
        )
        .bind(pseudo)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(exists)
    }

    async fn generate_unique_pseudo(&self, name: &str) -> Result<String, AppError> {
        let base = name.trim().to_lowercase();
        let base = if base.is_empty() {
            "user".to_string()
        } else {
            base
        };

        if !self.pseudo_exists(&base).await? {
            return Ok(base);
        }

        for idx in 2..=10_000 {
            let candidate = format!("{base}-{idx}");
            if !self.pseudo_exists(&candidate).await? {
                return Ok(candidate);
            }
        }

        Err(AppError::Internal(
            "Failed to allocate unique pseudo".to_string(),
        ))
    }
}
