use crate::services::users_service::UsersService;

#[derive(Clone)]
pub struct AppState {
    pub users_service: UsersService,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users_service: UsersService::new(),
        }
    }
}
