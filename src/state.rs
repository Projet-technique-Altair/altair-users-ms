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
    /// State minimal pour les tests CI
    /// - DB réelle
    /// - connexion lazy (aucune requête exécutée)
    pub fn test() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/dummy".to_string());

        let db = PgPool::connect_lazy(&database_url).expect("Invalid DATABASE_URL");

        let users_service = UsersService::new(db);

        Self { users_service }
    }
}
