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
        email: &str,
    ) -> Result<User, AppError> {
        // 1️⃣ Try get (fast path)
        if let Ok(user) = self.get_user_by_keycloak_id(keycloak_id).await {
            return Ok(user);
        }

        // 2️⃣ Try insert (idempotent)
        sqlx::query(
            r#"
            INSERT INTO users (
                keycloak_id,
                role,
                name,
                pseudo,
                email
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (keycloak_id) DO NOTHING
            "#,
        )
        .bind(keycloak_id)
        .bind(role)
        .bind(name)
        .bind(name.to_lowercase())
        .bind(email)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        // 3️⃣ Always fetch (RETURN DIRECTEMENT)
        self.get_user_by_keycloak_id(keycloak_id).await
    }
}
