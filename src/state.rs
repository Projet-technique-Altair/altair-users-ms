use crate::services::users_service::UsersService;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub users_service: UsersService,
}

impl AppState {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        let db = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let users_service = UsersService::new(db);

        Self { users_service }
    }
}

#[cfg(test)]
impl AppState {
    /// Test-only constructor.
    /// Uses a lazy SQLx pool so tests can run without a live DB connection.
    pub fn test() -> Self {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            // Dummy fallback: connect_lazy won't connect until first query.
            "postgresql://user:password@localhost:5432/altair_users".to_string()
        });

        let db = PgPool::connect_lazy(&database_url)
            .expect("Failed to create lazy PgPool for tests");

        let users_service = UsersService::new(db);

        Self { users_service }
    }
}
