use super::{Activity, DashboardData, QuickDraft};
use gloo_net::http::Request;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    Network,
    Response,
    Validation,
    Unconfirmed,
}

impl ApiError {
    pub fn key(self) -> &'static str {
        match self {
            Self::Unauthorized => "dashboard.unauthorized",
            Self::Forbidden => "dashboard.forbidden",
            Self::Network => "dashboard.network",
            Self::Response => "dashboard.response",
            Self::Validation => "dashboard.validation",
            Self::Unconfirmed => "dashboard.unconfirmed",
        }
    }
}

pub async fn load(token: &str) -> Result<DashboardData, ApiError> {
    let response = Request::get("/api/v1/stats/dashboard")
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    match response.status() {
        200 => response.json().await.map_err(|_| ApiError::Response),
        401 => Err(ApiError::Unauthorized),
        403 => Err(ApiError::Forbidden),
        _ => Err(ApiError::Response),
    }
}

pub async fn save(token: &str, draft: &QuickDraft) -> Result<Activity, ApiError> {
    let response = Request::post("/api/v1/entries?content_type=post")
        .header("Authorization", &format!("Bearer {token}"))
        .json(draft)
        .map_err(|_| ApiError::Validation)?
        .send()
        .await
        .map_err(|_| ApiError::Unconfirmed)?;
    match response.status() {
        200 | 201 => {
            let entry: Activity = response.json().await.map_err(|_| ApiError::Unconfirmed)?;
            if entry.state != "draft" {
                return Err(ApiError::Unconfirmed);
            }
            Ok(entry)
        }
        401 => Err(ApiError::Unauthorized),
        403 => Err(ApiError::Forbidden),
        400 | 413 | 422 => Err(ApiError::Validation),
        _ => Err(ApiError::Unconfirmed),
    }
}
