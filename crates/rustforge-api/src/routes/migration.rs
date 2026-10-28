//! Read-only inspection followed by an explicitly confirmed editorial import.
use crate::{
    AppState,
    auth::AuthUser,
    error::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Multipart, State},
};
use hmac::{Hmac, Mac};
use rustforge_core::migration::{DataMorphImporter, RFIRDocument, WxrImporter};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MAX_FILE: usize = 16 * 1024 * 1024;
#[derive(Default)]
struct Upload {
    bytes: Vec<u8>,
    source: Option<String>,
    seal: String,
    acknowledge: bool,
    content_only: bool,
}
async fn upload(mut body: Multipart) -> ApiResult<Upload> {
    let mut result = Upload::default();
    let mut seen = std::collections::HashSet::new();
    while let Some(field) = body
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("Envoi du fichier interrompu".into()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if !seen.insert(name.clone()) {
            return Err(ApiError::BadRequest("Champ dupliqué".into()));
        }
        if name == "file" {
            let bytes = field
                .bytes()
                .await
                .map_err(|_| ApiError::BadRequest("Fichier trop volumineux ou incomplet".into()))?;
            if bytes.len() > MAX_FILE {
                return Err(ApiError::BadRequest(
                    "Export limité à 16 Mio pour cette version testable".into(),
                ));
            }
            result.bytes = bytes.to_vec();
        } else {
            let value = field
                .text()
                .await
                .map_err(|_| ApiError::BadRequest("Champ invalide".into()))?;
            if value.len() > 2048 {
                return Err(ApiError::BadRequest("Champ trop long".into()));
            }
            match name.as_str() {
                "source_url" => result.source = Some(value),
                "seal" => result.seal = value,
                "acknowledge" => result.acknowledge = value == "true",
                "content_only" => result.content_only = value == "true",
                _ => return Err(ApiError::BadRequest("Champ inconnu".into())),
            }
        }
    }
    if result.bytes.is_empty() {
        return Err(ApiError::BadRequest(
            "Choisissez un export XML WordPress ou JSON RFIR".into(),
        ));
    }
    Ok(result)
}
fn document(input: &Upload) -> ApiResult<RFIRDocument> {
    let mut doc: RFIRDocument =
        if input.bytes.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{') {
            serde_json::from_slice(&input.bytes)
                .map_err(|_| ApiError::BadRequest("Document JSON RFIR invalide".into()))?
        } else {
            WxrImporter::parse(std::io::Cursor::new(&input.bytes))?
        };
    if doc.tmir_version != rustforge_core::migration::tmir::RFIR_CURRENT_VERSION
        || doc.entries.len() > 20_000
        || doc.terms.len() > 20_000
        || doc.media.len() > 20_000
    {
        return Err(ApiError::BadRequest(
            "Version ou taille du document non prise en charge".into(),
        ));
    }
    let source = input
        .source
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(doc.source_site_url.as_deref())
        .ok_or_else(|| {
            ApiError::BadRequest("Indiquez l’adresse du site WordPress source".into())
        })?;
    let url = source
        .trim()
        .parse::<reqwest::Url>()
        .map_err(|_| ApiError::BadRequest("Adresse source invalide".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(ApiError::BadRequest(
            "Utilisez une adresse HTTP(S) sans identifiants, paramètres ou fragment".into(),
        ));
    }
    doc.source_site_url = Some(url.as_str().trim_end_matches('/').into());
    let mut ids = std::collections::HashSet::new();
    for entry in &doc.entries {
        if entry.source_id.is_empty()
            || entry.slug.is_empty()
            || !ids.insert((&entry.content_type, &entry.source_id))
        {
            return Err(ApiError::BadRequest(
                "Identifiant source vide ou dupliqué, ou URL de contenu vide".into(),
            ));
        }
    }
    Ok(doc)
}
fn signature(secret: &str, hash: &str, source: &str, time: i64) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC key");
    mac.update(format!("trustpress-import-preview-v1\n{hash}\n{source}\n{time}").as_bytes());
    mac
}
fn seal(secret: &str, hash: &str, source: &str, time: i64) -> String {
    format!(
        "{time}.{}",
        hex::encode(
            signature(secret, hash, source, time)
                .finalize()
                .into_bytes()
        )
    )
}
fn verify(secret: &str, hash: &str, source: &str, seal: &str, now: i64) -> bool {
    let Some((time, tag)) = seal.split_once('.') else {
        return false;
    };
    let Ok(time) = time.parse::<i64>() else {
        return false;
    };
    if time > now || now.saturating_sub(time) > 1800 {
        return false;
    }
    let Ok(tag) = hex::decode(tag) else {
        return false;
    };
    signature(secret, hash, source, time)
        .verify_slice(&tag)
        .is_ok()
}

#[derive(Serialize, Deserialize)]
pub struct Inspection {
    pub source_url: String,
    pub entries: usize,
    pub terms: usize,
    pub media_references: usize,
    pub by_type: std::collections::BTreeMap<String, usize>,
    pub sample: Vec<Value>,
    pub warnings: Vec<String>,
    pub blocking_conflicts: i64,
    pub seal: String,
}
async fn inspect_document(
    state: &AppState,
    doc: &RFIRDocument,
    hash: &str,
) -> ApiResult<Inspection> {
    let source = doc.source_site_url.as_deref().expect("validated");
    let mut by_type = std::collections::BTreeMap::new();
    for entry in &doc.entries {
        *by_type.entry(entry.content_type.clone()).or_default() += 1;
    }
    let types: Vec<_> = doc.entries.iter().map(|e| e.content_type.clone()).collect();
    let slugs: Vec<_> = doc.entries.iter().map(|e| e.slug.clone()).collect();
    let ids: Vec<_> = doc.entries.iter().map(|e| e.source_id.clone()).collect();
    let blocking_conflicts:i64=sqlx::query_scalar("SELECT count(*) FROM unnest($1::text[],$2::text[],$3::text[]) AS x(kind,slug,source_id) LEFT JOIN content_types ct ON ct.slug=x.kind LEFT JOIN entries occupied ON occupied.content_type_id=ct.id AND occupied.slug=x.slug LEFT JOIN import_origins o ON o.source=$4 AND o.entity_type=x.kind AND o.source_id=x.source_id LEFT JOIN entries previous ON previous.id=o.entry_id WHERE (o.entry_id IS NULL AND occupied.id IS NOT NULL) OR (o.entry_id IS NOT NULL AND (previous.updated_at<>o.target_updated_at OR previous.slug<>x.slug))")
        .bind(types).bind(slugs).bind(ids).bind(source).fetch_one(&state.db).await.map_err(rustforge_core::CoreError::from)?;
    let mut warnings=vec!["Les contenus importés seront attribués à votre compte. Les auteurs source restent conservés comme provenance.".into(),"Les adresses, redirections, thèmes et fonctions d’extensions doivent être vérifiés avant la mise en ligne.".into()];
    if !doc.media.is_empty() {
        warnings.push(format!("{} médias sont référencés. Leurs fichiers ne sont pas transférés par cet import ; une sauvegarde native complète sera impossible tant qu’ils manquent.",doc.media.len()));
    }
    if by_type
        .keys()
        .any(|kind| !matches!(kind.as_str(), "post" | "page"))
    {
        warnings.push("Des types personnalisés seront conservés comme contenus ; cela ne recrée pas les fonctionnalités des extensions d’origine.".into());
    }
    if doc.entries.iter().any(|e| e.raw_content.contains('[')) {
        warnings.push("Des shortcodes peuvent nécessiter une adaptation manuelle.".into());
    }
    if doc.entries.iter().any(|e| {
        matches!(
            e.status,
            rustforge_core::migration::tmir::TMIREntryStatus::Private
                | rustforge_core::migration::tmir::TMIREntryStatus::Pending
        )
    }) {
        warnings.push("Les contenus privés et en attente restent en brouillon.".into());
    }
    if blocking_conflicts > 0 {
        warnings.push("Des contenus existants entrent en conflit. L’import est bloqué : utilisez un site de test vierge ou résolvez ces conflits.".into());
    }
    Ok(Inspection{source_url:source.into(),entries:doc.entries.len(),terms:doc.terms.len(),media_references:doc.media.len(),by_type,sample:doc.entries.iter().take(10).map(|e|json!({"title":e.title,"slug":e.slug,"content_type":e.content_type,"status":e.status})).collect(),warnings,blocking_conflicts,seal:seal(&state.jwt_secret,hash,source,chrono::Utc::now().timestamp())})
}
pub async fn inspect(
    State(state): State<AppState>,
    auth: AuthUser,
    body: Multipart,
) -> ApiResult<Json<Inspection>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let input = upload(body).await?;
    let doc = document(&input)?;
    let hash = hex::encode(Sha256::digest(&input.bytes));
    Ok(Json(inspect_document(&state, &doc, &hash).await?))
}
pub async fn import(
    State(state): State<AppState>,
    auth: AuthUser,
    body: Multipart,
) -> ApiResult<Json<Value>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let input = upload(body).await?;
    let doc = document(&input)?;
    let hash = hex::encode(Sha256::digest(&input.bytes));
    let source = doc.source_site_url.as_deref().expect("validated");
    if !input.acknowledge
        || !verify(
            &state.jwt_secret,
            &hash,
            source,
            &input.seal,
            chrono::Utc::now().timestamp(),
        )
    {
        return Err(ApiError::BadRequest("Analysez ce fichier puis confirmez les points à vérifier. Une analyse expire après 30 minutes.".into()));
    }
    if !doc.media.is_empty() && !input.content_only {
        return Err(ApiError::BadRequest(
            "Confirmez explicitement l’import des contenus sans transfert des médias".into(),
        ));
    }
    let inspection = inspect_document(&state, &doc, &hash).await?;
    if inspection.blocking_conflicts > 0 {
        return Err(ApiError::BadRequest(
            "Conflits détectés : aucun contenu importé".into(),
        ));
    }
    let report = DataMorphImporter::apply(&state.db, &doc, auth.user.id).await?;
    Ok(Json(
        json!({"status":"content_imported","report":report,"checks_remaining":inspection.warnings}),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confirmation_binds_file_source_and_expiry() {
        let token = seal("secret", "hash", "https://source.test", 100);
        assert!(verify("secret", "hash", "https://source.test", &token, 200));
        assert!(!verify(
            "secret",
            "other",
            "https://source.test",
            &token,
            200
        ));
        assert!(!verify("secret", "hash", "https://other.test", &token, 200));
        assert!(!verify(
            "secret",
            "hash",
            "https://source.test",
            &token,
            1901
        ));
        assert!(!verify("secret", "hash", "https://source.test", &token, 99));
    }
}
