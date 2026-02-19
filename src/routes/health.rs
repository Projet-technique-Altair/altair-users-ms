use crate::state::AppState;
use axum::{extract::State, Json};
use serde_json::json;

pub async fn health(State(_): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;

    #[tokio::test]
    async fn health_is_ok() {
        // AppState minimal, lazy DB (aucune connexion réelle)
        let state = AppState::test();

        let response = health(State(state)).await;

        assert_eq!(response.0["status"], "ok");
    }
}
