use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

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
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
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
    }

    /// MVP : retourne le premier user en DB
    /// (sera remplacé plus tard par l'auth)
    pub async fn get_first_user(
        &self,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT
                user_id,
                name,
                pseudo,
                mail,
                post::text   AS post,
                status::text AS status
            FROM users
            ORDER BY date_of_creation ASC
            LIMIT 1
            "#
        )
        .fetch_one(&self.db)
        .await
    }
}
