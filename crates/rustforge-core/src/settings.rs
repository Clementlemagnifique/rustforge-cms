//! Réglages clé/valeur typés en JSON (≈ `wp_options`).

use std::collections::BTreeMap;

use serde_json::Value;
use sqlx::PgPool;

use crate::{CoreError, CoreResult};

/// Préfixe des réglages exposés publiquement.
pub const PUBLIC_PREFIX: &str = "site.";

fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.len() <= 100
        && key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.')
}

pub async fn all(db: &PgPool) -> CoreResult<BTreeMap<String, Value>> {
    let rows: Vec<(String, Value)> = sqlx::query_as("SELECT key, value FROM settings ORDER BY key")
        .fetch_all(db)
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn public(db: &PgPool) -> CoreResult<BTreeMap<String, Value>> {
    let rows: Vec<(String, Value)> =
        sqlx::query_as("SELECT key, value FROM settings WHERE starts_with(key, $1) ORDER BY key")
            .bind(PUBLIC_PREFIX)
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().map(|(k, v)| (k.trim_start_matches(PUBLIC_PREFIX).to_string(), v)).collect())
}

pub async fn get(db: &PgPool, key: &str) -> CoreResult<Option<Value>> {
    Ok(sqlx::query_scalar("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await?)
}

/// Met à jour plusieurs réglages ; une valeur `null` supprime la clé.
pub async fn update(db: &PgPool, values: BTreeMap<String, Value>) -> CoreResult<BTreeMap<String, Value>> {
    if let Some(bad) = values.keys().find(|k| !valid_key(k)) {
        return Err(CoreError::validation(format!("clé de réglage invalide « {bad} »")));
    }
    let mut tx = db.begin().await?;
    for (key, value) in values {
        if value.is_null() {
            sqlx::query("DELETE FROM settings WHERE key = $1").bind(&key).execute(&mut *tx).await?;
        } else {
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES ($1, $2)
                 ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
            )
            .bind(&key)
            .bind(&value)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    all(db).await
}

/// Retourne l'état d'activation des modules natifs de RustForge
pub async fn get_active_modules(db: &PgPool) -> CoreResult<BTreeMap<String, bool>> {
    let raw = get(db, "modules.active").await?;
    if let Some(val) = raw {
        if let Ok(map) = serde_json::from_value::<BTreeMap<String, bool>>(val) {
            return Ok(map);
        }
    }
    let mut default_map = BTreeMap::new();
    default_map.insert("forms".to_string(), true);
    default_map.insert("galleries".to_string(), true);
    default_map.insert("zen_mode".to_string(), true);
    default_map.insert("ecommerce".to_string(), true);
    default_map.insert("lms".to_string(), true);
    Ok(default_map)
}

/// Vérifie si un module spécifique est activé
pub async fn is_module_active(db: &PgPool, module: &str) -> bool {
    let modules = get_active_modules(db).await.unwrap_or_default();
    *modules.get(module).unwrap_or(&false)
}
