//! Shared authorization and storage boundaries used by every transport.
use crate::{
    CoreError, CoreResult,
    models::{Entry, Role},
};
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

pub fn secret_matches(expected: &str, supplied: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    if expected.len() < 32 || supplied.len() > 256 {
        return false;
    }
    let mut a = Hmac::<Sha256>::new_from_slice(expected.as_bytes()).expect("HMAC key");
    let mut b = Hmac::<Sha256>::new_from_slice(supplied.as_bytes()).expect("HMAC key");
    a.update(b"rustforge-bootstrap");
    b.update(b"rustforge-bootstrap");
    a.verify_slice(&b.finalize().into_bytes()).is_ok()
}

pub fn require_writer(role: Role) -> CoreResult<()> {
    if role.can_write() {
        Ok(())
    } else {
        Err(CoreError::Forbidden)
    }
}

pub fn require_editor(role: Role) -> CoreResult<()> {
    if role.can_edit_others() {
        Ok(())
    } else {
        Err(CoreError::Forbidden)
    }
}

pub fn require_entry_access(role: Role, actor: Uuid, entry: &Entry) -> CoreResult<()> {
    require_writer(role)?;
    if role.can_edit_others() || entry.author_id == Some(actor) {
        Ok(())
    } else {
        Err(CoreError::Forbidden)
    }
}

pub fn safe_slug(slug: &str) -> CoreResult<&str> {
    if slug.is_empty()
        || slug.len() > 80
        || !slug
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err(CoreError::validation(
            "nom invalide : minuscules, chiffres et tirets uniquement",
        ));
    }
    Ok(slug)
}

pub fn relative_path(path: &str) -> CoreResult<PathBuf> {
    if path.is_empty() || path.contains(['\\', ':', '\0']) {
        return Err(CoreError::validation("chemin relatif invalide"));
    }
    for part in path.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || ['<', '>', '"', '|', '?', '*'].contains(&c))
            || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || ((stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.len() == 4
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(CoreError::validation("chemin non portable ou nom réservé"));
        }
    }
    let path = Path::new(path);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(CoreError::validation("chemin hors du stockage autorisé"));
    }
    Ok(path.to_path_buf())
}

/// Resolves existing files AND parents to reject symlink/junction escapes.
pub fn confined_path(root: &Path, relative: &str) -> CoreResult<PathBuf> {
    let relative = relative_path(relative)?;
    let base = root
        .canonicalize()
        .map_err(|e| CoreError::Internal(e.into()))?;
    let candidate = base.join(relative);
    let mut existing = candidate.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or(CoreError::Forbidden)?;
    }
    if !existing
        .canonicalize()
        .map_err(|e| CoreError::Internal(e.into()))?
        .starts_with(&base)
    {
        return Err(CoreError::Forbidden);
    }
    Ok(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn storage_rejects_traversal_and_platform_prefixes() {
        for p in [
            "../secret",
            "/secret",
            "C:/secret",
            "a\\b",
            "a/../b",
            "",
            "a/.. /b",
            "con.txt",
            "a./b",
            "LPT1",
            "a//b",
        ] {
            assert!(relative_path(p).is_err(), "{p}");
        }
        assert!(relative_path("2026/10/photo.png").is_ok());
        for slug in ["../x", "C:/x", "", "a/b"] {
            assert!(safe_slug(slug).is_err());
        }
        assert!(safe_slug("theme-2026").is_ok());
    }
    #[test]
    fn subscribers_cannot_write_and_authors_cannot_manage_sales() {
        assert!(require_writer(Role::Subscriber).is_err());
        assert!(require_editor(Role::Author).is_err());
        assert!(require_editor(Role::Admin).is_ok());
    }
}
