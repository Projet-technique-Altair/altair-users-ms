use sqlx::PgPool;
use crate::services::users_service::UsersService;

#[derive(Clone)]
pub struct AppState {
    pub users_service: UsersService,
}

impl AppState {
    pub async fn new() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

        let db = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        let users_service = UsersService::new(db);

        Self { users_service }
    }
}
