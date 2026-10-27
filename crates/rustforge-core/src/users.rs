//! Utilisateurs, mots de passe (Argon2id) et jetons de rafraîchissement.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    CoreError, CoreResult, Page, PageParams,
    models::{Role, User},
};

const USER_COLUMNS: &str =
    "id, email, username, display_name, role, bio, created_at, updated_at, last_login_at, password_hash";

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: Role,
}

fn default_role() -> Role {
    Role::Author
}

#[derive(Debug, Default, Deserialize)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub role: Option<Role>,
    pub password: Option<String>,
}

pub fn hash_password(password: &str) -> CoreResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| CoreError::Internal(anyhow::anyhow!("hachage du mot de passe : {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}

fn validate_password(password: &str) -> CoreResult<()> {
    if password.chars().count() < 10 {
        return Err(CoreError::validation("le mot de passe doit contenir au moins 10 caractères"));
    }
    Ok(())
}

fn validate_email(email: &str) -> CoreResult<String> {
    let email = email.trim().to_lowercase();
    let valid = email.len() <= 254
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        });
    if !valid {
        return Err(CoreError::validation("adresse e-mail invalide"));
    }
    Ok(email)
}

fn validate_username(username: &str) -> CoreResult<String> {
    let username = username.trim().to_lowercase();
    let valid = (3..=32).contains(&username.len())
        && username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !valid {
        return Err(CoreError::validation(
            "nom d'utilisateur invalide (3 à 32 caractères : lettres, chiffres, . _ -)",
        ));
    }
    Ok(username)
}

pub async fn count(db: &PgPool) -> CoreResult<i64> {
    Ok(sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(db).await?)
}

pub async fn create(db: &PgPool, input: NewUser) -> CoreResult<User> {
    let email = validate_email(&input.email)?;
    let username = validate_username(&input.username)?;
    validate_password(&input.password)?;
    let display_name = input
        .display_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| username.clone());
    let hash = hash_password(&input.password)?;

    let user = sqlx::query_as::<_, User>(&format!(
        "INSERT INTO users (email, username, display_name, password_hash, role)
         VALUES ($1, $2, $3, $4, $5) RETURNING {USER_COLUMNS}"
    ))
    .bind(email)
    .bind(username)
    .bind(display_name)
    .bind(hash)
    .bind(input.role)
    .fetch_one(db)
    .await?;
    Ok(user)
}

pub async fn get(db: &PgPool, id: Uuid) -> CoreResult<User> {
    Ok(sqlx::query_as::<_, User>(&format!("SELECT {USER_COLUMNS} FROM users WHERE id = $1"))
        .bind(id)
        .fetch_one(db)
        .await?)
}

pub async fn list(db: &PgPool, params: PageParams, search: Option<&str>) -> CoreResult<Page<User>> {
    let params = params.normalized();
    let pattern = search.map(|s| format!("%{}%", s.trim()));
    let filter = "($1::text IS NULL OR email ILIKE $1 OR username ILIKE $1 OR display_name ILIKE $1)";
    let total: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM users WHERE {filter}"))
        .bind(&pattern)
        .fetch_one(db)
        .await?;
    let items = sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users WHERE {filter} ORDER BY created_at ASC LIMIT $2 OFFSET $3"
    ))
    .bind(&pattern)
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(db)
    .await?;
    Ok(Page::new(items, total, params))
}

pub async fn update(db: &PgPool, id: Uuid, input: UserUpdate) -> CoreResult<User> {
    let current = get(db, id).await?;
    let email = match input.email {
        Some(e) => validate_email(&e)?,
        None => current.email,
    };
    let display_name = input
        .display_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(current.display_name);
    let bio = input.bio.or(current.bio);
    let role = input.role.unwrap_or(current.role);
    let password_hash = match input.password.filter(|p| !p.is_empty()) {
        Some(p) => {
            validate_password(&p)?;
            hash_password(&p)?
        }
        None => current.password_hash,
    };

    // Garde-fou : le site doit toujours conserver au moins un administrateur.
    if current.role == Role::Admin && role != Role::Admin {
        ensure_other_admin(db, id).await?;
    }

    Ok(sqlx::query_as::<_, User>(&format!(
        "UPDATE users SET email = $2, display_name = $3, bio = $4, role = $5, password_hash = $6, updated_at = now()
         WHERE id = $1 RETURNING {USER_COLUMNS}"
    ))
    .bind(id)
    .bind(email)
    .bind(display_name)
    .bind(bio)
    .bind(role)
    .bind(password_hash)
    .fetch_one(db)
    .await?)
}

pub async fn delete(db: &PgPool, id: Uuid) -> CoreResult<()> {
    let user = get(db, id).await?;
    if user.role == Role::Admin {
        ensure_other_admin(db, id).await?;
    }
    sqlx::query("DELETE FROM users WHERE id = $1").bind(id).execute(db).await?;
    Ok(())
}

async fn ensure_other_admin(db: &PgPool, excluded: Uuid) -> CoreResult<()> {
    let others: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE role = 'admin' AND id <> $1")
        .bind(excluded)
        .fetch_one(db)
        .await?;
    if others == 0 {
        return Err(CoreError::Conflict("le site doit conserver au moins un administrateur".into()));
    }
    Ok(())
}

/// Vérifie les identifiants (e-mail ou nom d'utilisateur).
pub async fn authenticate(db: &PgPool, login: &str, password: &str) -> CoreResult<User> {
    let login = login.trim().to_lowercase();
    let user = sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLUMNS} FROM users WHERE email = $1 OR username = $1"
    ))
    .bind(&login)
    .fetch_optional(db)
    .await?;

    match user {
        Some(user) if verify_password(password, &user.password_hash) => {
            sqlx::query("UPDATE users SET last_login_at = now() WHERE id = $1")
                .bind(user.id)
                .execute(db)
                .await?;
            Ok(user)
        }
        Some(_) => Err(CoreError::Unauthorized),
        None => {
            // Temps constant approximatif : on hache quand même pour ne pas révéler l'existence du compte.
            let _ = hash_password(password);
            Err(CoreError::Unauthorized)
        }
    }
}

// ---------------------------------------------------------------------------
// Jetons de rafraîchissement (stockés hachés, rotation à chaque usage)
// ---------------------------------------------------------------------------

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn generate_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Émet un nouveau jeton de rafraîchissement et retourne sa valeur en clair.
pub async fn issue_refresh_token(db: &PgPool, user_id: Uuid, ttl: Duration) -> CoreResult<(String, DateTime<Utc>)> {
    let token = generate_token();
    let expires_at = Utc::now() + ttl;
    sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(hash_token(&token))
        .bind(expires_at)
        .execute(db)
        .await?;
    // Ménage opportuniste des jetons expirés.
    sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < now() - interval '1 day'")
        .execute(db)
        .await?;
    Ok((token, expires_at))
}

/// Consomme un jeton (rotation) et renvoie l'utilisateur associé.
pub async fn consume_refresh_token(db: &PgPool, token: &str) -> CoreResult<User> {
    let user_id: Option<Uuid> = sqlx::query_scalar(
        "UPDATE refresh_tokens SET revoked_at = now()
         WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
         RETURNING user_id",
    )
    .bind(hash_token(token))
    .fetch_optional(db)
    .await?;
    match user_id {
        Some(id) => get(db, id).await.map_err(|_| CoreError::Unauthorized),
        None => Err(CoreError::Unauthorized),
    }
}

pub async fn revoke_refresh_token(db: &PgPool, token: &str) -> CoreResult<()> {
    sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE token_hash = $1 AND revoked_at IS NULL")
        .bind(hash_token(token))
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip() {
        let hash = hash_password("correct horse battery").unwrap();
        assert!(verify_password("correct horse battery", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn input_validation() {
        assert!(validate_email("Foo@Example.com").is_ok_and(|e| e == "foo@example.com"));
        assert!(validate_email("nope").is_err());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("jean.dupont").is_ok());
        assert!(validate_password("court").is_err());
    }
}
