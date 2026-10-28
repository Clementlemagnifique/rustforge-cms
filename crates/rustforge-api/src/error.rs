use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rustforge_core::CoreError;
use serde_json::json;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    Core(#[from] CoreError),
    #[error("{0}")]
    BadRequest(String),
    #[error("non autorisé")]
    Unauthorized,
    #[error("interdit")]
    Forbidden,
    #[error("introuvable")]
    NotFound,
    #[error("erreur interne: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Core(CoreError::NotFound) | ApiError::NotFound => {
                (StatusCode::NOT_FOUND, "ressource introuvable".to_string())
            }
            ApiError::Core(CoreError::Validation(msg)) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            ApiError::Core(CoreError::Conflict(msg)) => (StatusCode::CONFLICT, msg),
            ApiError::Core(CoreError::Unauthorized) | ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "authentification requise".to_string(),
            ),
            ApiError::Core(CoreError::Forbidden) | ApiError::Forbidden => {
                (StatusCode::FORBIDDEN, "droits insuffisants".to_string())
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Core(CoreError::Database(err)) => {
                tracing::error!(%err, "erreur de base de données");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "erreur de base de données".to_string(),
                )
            }
            ApiError::Core(CoreError::Internal(err)) | ApiError::Internal(err) => {
                tracing::error!(%err, "erreur interne");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "erreur interne".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "status": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}
