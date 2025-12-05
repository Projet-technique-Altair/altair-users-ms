use crate::services::users_service::UsersService;

#[derive(Clone)]
pub struct AppState {
    pub users: UsersService,
}

impl AppState {
    pub fn init() -> Self {
        Self {
            users: UsersService::new(),
        }
    }
}
