use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};
use rustforge_core::models::{Role, User};

const JWT_EXPIRATION_MINUTES: i64 = 60 * 24; // 24h en dev

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub sub: Uuid,
    pub role: Role,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_token(
    user_id: Uuid,
    role: Role,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    issue_token(user_id, role, secret, None, JWT_EXPIRATION_MINUTES)
}

pub fn create_comment_token(
    user_id: Uuid,
    role: Role,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    issue_token(user_id, role, secret, Some("comments".into()), 30)
}

fn issue_token(
    user_id: Uuid,
    role: Role,
    secret: &str,
    scope: Option<String>,
    minutes: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::minutes(minutes)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims {
        scope,
        sub: user_id,
        role,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok(data.claims)
}

/// Utilisateur authentifié injecté dans les requêtes protégées.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user: User,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let user = bearer_identity(&parts.headers, &app_state, false).await?;

        Ok(AuthUser { user })
    }
}

/// Relit toujours le compte ; un jeton public ne peut servir d'identité au Studio.
pub async fn bearer_identity(
    headers: &axum::http::HeaderMap,
    state: &AppState,
    comments: bool,
) -> Result<User, ApiError> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(ApiError::Unauthorized)?;
    let claims = verify_token(token, &state.jwt_secret).map_err(|_| ApiError::Unauthorized)?;
    if claims
        .scope
        .as_deref()
        .is_some_and(|scope| !comments || scope != "comments")
    {
        return Err(ApiError::Unauthorized);
    }
    rustforge_core::users::get(&state.db, claims.sub)
        .await
        .map_err(|_| ApiError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_session_is_short_and_scoped() {
        let secret = "scoped-test-secret-long-enough";
        let token = create_comment_token(Uuid::nil(), Role::Admin, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.scope.as_deref(), Some("comments"));
        assert_eq!(claims.exp - claims.iat, 1800);
        let legacy = create_token(Uuid::nil(), Role::Admin, secret).unwrap();
        assert!(verify_token(&legacy, secret).unwrap().scope.is_none());
    }
    #[test]
    fn expired_public_session_is_rejected() {
        let secret = "scoped-test-secret-long-enough";
        let claims = Claims {
            scope: Some("comments".into()),
            sub: Uuid::nil(),
            role: Role::Subscriber,
            exp: 1,
            iat: 0,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();
        assert!(verify_token(&token, secret).is_err());
    }
}
