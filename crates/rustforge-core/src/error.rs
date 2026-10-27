use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

/// Erreur métier unifiée, convertie en réponse HTTP par la couche API.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("ressource introuvable")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("accès refusé")]
    Forbidden,
    #[error("identifiants invalides")]
    Unauthorized,
    #[error("erreur de base de données")]
    Database(#[source] sqlx::Error),
    #[error("erreur interne")]
    Internal(#[from] anyhow::Error),
}

impl CoreError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

impl From<sqlx::Error> for CoreError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::RowNotFound => CoreError::NotFound,
            sqlx::Error::Database(db) => match db.code().as_deref() {
                // unique_violation
                Some("23505") => CoreError::Conflict(match db.constraint() {
                    Some(c) if c.contains("email") => "cette adresse e-mail est déjà utilisée".into(),
                    Some(c) if c.contains("username") => "ce nom d'utilisateur est déjà pris".into(),
                    Some(c) if c.contains("slug") => "ce slug est déjà utilisé".into(),
                    _ => "cette ressource existe déjà".into(),
                }),
                // foreign_key_violation
                Some("23503") => CoreError::Conflict(
                    "cette ressource est référencée ailleurs ou une référence est invalide".into(),
                ),
                _ => CoreError::Database(err),
            },
            _ => CoreError::Database(err),
        }
    }
}
