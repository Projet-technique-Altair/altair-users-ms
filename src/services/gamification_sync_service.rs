use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::AppError, services::users_service::UserLoginResolution};

#[derive(Clone)]
pub struct GamificationSyncService {
    base_url: String,
    internal_token: Option<String>,
    http: reqwest::Client,
}

#[derive(Serialize)]
struct ProgressSyncPayload {
    user_id: Uuid,
    created_at: DateTime<Utc>,
    current_login_at: DateTime<Utc>,
    previous_last_login: Option<DateTime<Utc>>,
    is_new_user: bool,
}

impl GamificationSyncService {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("GAMIFICATION_MS_URL").ok()?;

        Some(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            internal_token: std::env::var("INTERNAL_SERVICE_TOKEN").ok(),
            http: reqwest::Client::new(),
        })
    }

    pub async fn sync_login_progress(
        &self,
        resolution: &UserLoginResolution,
    ) -> Result<(), AppError> {
        let current_login_at = resolution
            .user
            .last_login
            .ok_or_else(|| AppError::Internal("Missing current last_login for sync".into()))?;

        let payload = ProgressSyncPayload {
            user_id: resolution.user.user_id,
            created_at: naive_utc_to_datetime(resolution.user.created_at),
            current_login_at: naive_utc_to_datetime(current_login_at),
            previous_last_login: resolution.previous_last_login.map(naive_utc_to_datetime),
            is_new_user: resolution.is_new_user,
        };

        let mut request = self
            .http
            .post(format!("{}/internal/progression/sync", self.base_url))
            .json(&payload);

        if let Some(token) = &self.internal_token {
            request = request.header("x-altair-internal-token", token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| AppError::Internal(format!("Gamification sync failed: {err}")))?;

        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Gamification sync returned {}: {}",
                status, body
            )))
        }
    }
}

fn naive_utc_to_datetime(value: chrono::NaiveDateTime) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(value, Utc)
}
