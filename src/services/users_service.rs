use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    models::user::{User, UserRow},
    error::AppError,
};

#[derive(Clone)]
pub struct UsersService {
    db: PgPool,
}

impl UsersService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_user_by_id(
        &self,
        user_id: Uuid,
    ) -> Result<User, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                user_id,
                name,
                pseudo,
                mail,
                post::text   AS post,
                status::text AS status
            FROM users
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        let user = User::try_from(row)?;
        Ok(user)
    }

    
}
