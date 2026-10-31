//! Endpoints IA : registre d'outils, exécution déterministe, statut du modèle
//! et boucle agentique `/ai/chat` (LLM → appels d'outils → réponse).

use std::{collections::BTreeMap, time::Instant};

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::{
    ai::{self, ToolDefinition, get_available_tools},
    content_types::{self, ContentTypeInput},
    entries::{self, EntryQuery},
    llm::{ChatMessage, LlmClient, Role},
    models::{Entry, EntryInput, EntryStatus},
    settings, terms,
};

/// Nombre maximal d'allers-retours modèle ⇄ outils par requête.
const MAX_AGENT_STEPS: usize = 6;

#[derive(Debug, Deserialize)]
pub struct ExecuteToolRequest {
    pub tool_name: String,
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteToolResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

pub async fn list_tools(_auth: AuthUser) -> Json<Vec<ToolDefinition>> {
    Json(get_available_tools())
}

pub async fn execute_tool(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ExecuteToolRequest>,
) -> ApiResult<Json<ExecuteToolResponse>> {
    Ok(Json(
        propose_or_read(&state, &auth, &req.tool_name, req.arguments).await?,
    ))
}

async fn propose_or_read(
    state: &AppState,
    auth: &AuthUser,
    name: &str,
    mut args: Value,
) -> ApiResult<ExecuteToolResponse> {
    rustforge_core::security::require_writer(auth.user.role)?;
    if matches!(
        name,
        "search_content" | "audit_geo_and_seo" | "generate_block"
    ) {
        return run_tool(state, auth, name, args).await;
    }
    if !matches!(
        name,
        "create_or_update_content"
            | "update_design_system"
            | "create_content_type"
            | "diff_and_patch_blocks"
    ) {
        return Err(ApiError::BadRequest("Outil inconnu".into()));
    }
    if name != "create_or_update_content"
        && name != "diff_and_patch_blocks"
        && !auth.user.role.is_admin()
    {
        return Err(ApiError::Forbidden);
    }
    if !args.is_object() {
        return Err(ApiError::BadRequest("Arguments invalides".into()));
    }
    if name == "create_or_update_content"
        && let Some(id) = str_arg(&args, "id")
    {
        let entry = entries::get(&state.db, parse_uuid(id)?).await?;
        rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &entry)?;
        args["expected_updated_at"] = json!(entry.updated_at);
    }
    if name == "diff_and_patch_blocks" {
        let id = str_arg(&args, "entry_id")
            .ok_or_else(|| ApiError::BadRequest("entry_id requis".into()))?;
        let entry = entries::get(&state.db, parse_uuid(id)?).await?;
        rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &entry)?;
        args["expected_updated_at"] = json!(entry.updated_at);
        ai::patch_native_blocks(&entry.blocks, &args)?;
    }
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO ai_action_plans(actor_id,tool,arguments) VALUES($1,$2,$3) RETURNING id",
    )
    .bind(auth.user.id)
    .bind(name)
    .bind(&args)
    .fetch_one(&state.db)
    .await
    .map_err(rustforge_core::CoreError::from)?;
    Ok(ExecuteToolResponse{success:true,message:"Proposition enregistrée ; aucune modification appliquée. Vérifiez les arguments puis validez.".into(),data:Some(json!({"approval_required":true,"plan_id":id,"arguments":args}))})
}

pub async fn list_plans(
    State(state): State<AppState>,
    auth: AuthUser,
) -> ApiResult<Json<Vec<Value>>> {
    let rows=sqlx::query_scalar("SELECT to_jsonb(p) FROM ai_action_plans p WHERE actor_id=$1 ORDER BY created_at DESC LIMIT 50")
        .bind(auth.user.id).fetch_all(&state.db).await.map_err(rustforge_core::CoreError::from)?;
    Ok(Json(rows))
}

pub async fn approve_plan(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ExecuteToolResponse>> {
    rustforge_core::security::require_writer(auth.user.role)?;
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(rustforge_core::CoreError::from)?;
    let row:Option<(String,Value,String,Option<Value>,bool)>=sqlx::query_as("SELECT tool,arguments,status,result,expires_at>now() FROM ai_action_plans WHERE id=$1 AND actor_id=$2 FOR UPDATE")
        .bind(id).bind(auth.user.id).fetch_optional(&mut *tx).await.map_err(rustforge_core::CoreError::from)?;
    let (tool, args, status, result, valid) = row.ok_or(ApiError::Forbidden)?;
    if status == "applied" {
        return Ok(Json(
            serde_json::from_value(result.ok_or(ApiError::NotFound)?)
                .map_err(|e| ApiError::Internal(e.into()))?,
        ));
    }
    if status != "pending" || !valid {
        return Err(ApiError::BadRequest("Cette proposition ne peut pas être rejouée. Consultez son état et créez un nouveau plan si nécessaire.".into()));
    }
    sqlx::query("UPDATE ai_action_plans SET status='executing' WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(rustforge_core::CoreError::from)?;
    tx.commit().await.map_err(rustforge_core::CoreError::from)?;
    // Executing is durable BEFORE the write: an interrupted request is never blindly replayed.
    let result = match run_tool(&state, &auth, &tool, args).await {
        Ok(value) => value,
        Err(err) => ExecuteToolResponse {
            success: false,
            message: err.to_string(),
            data: None,
        },
    };
    let status = if result.success { "applied" } else { "failed" };
    sqlx::query("UPDATE ai_action_plans SET status=$2,result=$3 WHERE id=$1")
        .bind(id)
        .bind(status)
        .bind(serde_json::to_value(&result).map_err(|e| ApiError::Internal(e.into()))?)
        .execute(&state.db)
        .await
        .map_err(rustforge_core::CoreError::from)?;
    Ok(Json(result))
}

// ---------------------------------------------------------------------------
// Exécution des outils
// ---------------------------------------------------------------------------

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn parse_uuid(s: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(s).map_err(|_| ApiError::BadRequest(format!("UUID invalide « {s} »")))
}

/// Exécute un outil déclaratif avec les droits de l'utilisateur courant.
pub async fn run_tool(
    state: &AppState,
    auth: &AuthUser,
    name: &str,
    args: Value,
) -> ApiResult<ExecuteToolResponse> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let args = if args.is_object() { args } else { json!({}) };

    match name {
        "create_or_update_content" => create_or_update_content(state, auth, &args).await,
        "update_design_system" => {
            if !auth.user.role.is_admin() {
                return Err(ApiError::Forbidden);
            }
            // Fusion avec les tokens existants : seules les clés fournies changent.
            let mut tokens = settings::get(&state.db, "site.design_tokens")
                .await?
                .unwrap_or_else(|| json!({}));
            if let (Some(current), Some(new)) = (tokens.as_object_mut(), args.as_object()) {
                for (k, v) in new {
                    current.insert(k.clone(), v.clone());
                }
            } else {
                tokens = args.clone();
            }
            let mut map = BTreeMap::new();
            map.insert("site.design_tokens".to_string(), tokens.clone());
            settings::update(&state.db, map).await?;
            Ok(ExecuteToolResponse {
                success: true,
                message: "Tokens du Design System mis à jour".into(),
                data: Some(tokens),
            })
        }
        "create_content_type" => {
            if !auth.user.role.is_admin() {
                return Err(ApiError::Forbidden);
            }
            let mut args = args;
            if str_arg(&args, "slug").is_none() {
                let name = str_arg(&args, "name").unwrap_or("type").to_string();
                args["slug"] = json!(rustforge_core::slugify(&name));
            }
            let input: ContentTypeInput = serde_json::from_value(args)
                .map_err(|e| ApiError::BadRequest(format!("arguments invalides : {e}")))?;
            let ct = content_types::create(&state.db, input).await?;
            Ok(ExecuteToolResponse {
                success: true,
                message: format!("Type de contenu « {} » créé (slug : {})", ct.name, ct.slug),
                data: Some(serde_json::to_value(ct).unwrap_or_default()),
            })
        }
        "audit_geo_and_seo" => {
            let id = parse_uuid(
                str_arg(&args, "entry_id")
                    .ok_or_else(|| ApiError::BadRequest("entry_id requis".into()))?,
            )?;
            let entry = entries::get(&state.db, id).await?;
            rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &entry)?;
            let report = audit_entry(&entry);
            Ok(ExecuteToolResponse {
                success: true,
                message: format!(
                    "Audit SEO & GEO de « {} » : score {}/100",
                    entry.title, report["score"]
                ),
                data: Some(report),
            })
        }
        "search_content" => {
            let limit = args
                .get("limit")
                .and_then(|v| v.as_i64())
                .unwrap_or(5)
                .clamp(1, 20);
            let query = EntryQuery {
                q: str_arg(&args, "query").map(str::to_string),
                content_type: str_arg(&args, "content_type").map(str::to_string),
                per_page: Some(limit),
                author_id: if auth.user.role.can_edit_others() {
                    None
                } else {
                    Some(auth.user.id)
                },
                ..Default::default()
            };
            let page = entries::list(&state.db, &query).await?;
            let items: Vec<Value> = page
                .items
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id, "title": e.title, "slug": e.slug,
                        "content_type": e.content_type, "status": e.status,
                        "excerpt": e.excerpt, "updated_at": e.updated_at,
                        "body": e.body,
                    })
                })
                .collect();
            Ok(ExecuteToolResponse {
                success: true,
                message: format!("{} contenu(s) trouvé(s)", items.len()),
                data: Some(json!(items)),
            })
        }
        "generate_block" => {
            let block = ai::generate_native_block(&args)?;
            Ok(ExecuteToolResponse {
                success: true,
                message: "Bloc natif validé".into(),
                data: Some(block),
            })
        }
        "diff_and_patch_blocks" => diff_and_patch_blocks(state, auth, &args).await,
        unknown => Err(ApiError::BadRequest(format!("outil inconnu « {unknown} »"))),
    }
}

async fn diff_and_patch_blocks(
    state: &AppState,
    auth: &AuthUser,
    args: &Value,
) -> ApiResult<ExecuteToolResponse> {
    let entry_id = parse_uuid(
        str_arg(args, "entry_id").ok_or_else(|| ApiError::BadRequest("entry_id requis".into()))?,
    )?;
    let entry = entries::get(&state.db, entry_id).await?;
    rustforge_core::security::require_entry_access(auth.user.role, auth.user.id, &entry)?;

    let expected: chrono::DateTime<chrono::Utc> =
        serde_json::from_value(args["expected_updated_at"].clone()).map_err(|_| {
            ApiError::BadRequest("Version attendue du plan manquante ou invalide".into())
        })?;
    if expected != entry.updated_at {
        return Err(rustforge_core::CoreError::Conflict(
            "Le contenu a changé depuis la proposition ; créez un nouveau plan".into(),
        )
        .into());
    }
    let final_blocks_val = ai::patch_native_blocks(&entry.blocks, args)?;
    let before_count = entry.blocks["rf_blocks"].as_array().map_or(0, Vec::len);
    let after_count = final_blocks_val["rf_blocks"].as_array().map_or(0, Vec::len);
    let operation = args["operation"].as_str().unwrap_or("");
    let (clean_body, _) = rustforge_core::document::project(&entry.body, &final_blocks_val)?;

    let input = EntryInput {
        expected_updated_at: Some(expected),
        title: entry.title.clone(),
        slug: Some(entry.slug.clone()),
        excerpt: entry.excerpt.clone(),
        body: clean_body,
        blocks: final_blocks_val,
        intent: entry.intent.clone(),
        design_tokens: entry.design_tokens.clone(),
        status: entry.status,
        fields: entry.fields.clone(),
        featured_media_id: entry.featured_media_id,
        parent_id: entry.parent_id,
        published_at: entry.published_at,
        scheduled_unpublish_at: entry.scheduled_unpublish_at,
        locale: entry.locale.clone(),
        translation_group_id: entry.translation_group_id,
        term_ids: Some(entry.terms.iter().map(|t| t.id).collect()),
    };

    let updated = entries::update(
        &state.db,
        state.plugins.as_ref(),
        entry_id,
        auth.user.id,
        input,
    )
    .await?;

    Ok(ExecuteToolResponse {
        success: true,
        message: format!(
            "Diff de blocs appliqué sur « {} » ({} -> {} blocs, opération {})",
            updated.title, before_count, after_count, operation
        ),
        data: Some(json!({
            "entry_id": updated.id,
            "before_count": before_count,
            "after_count": after_count,
            "operation": operation,
            "updated_at": updated.updated_at
        })),
    })
}

async fn create_or_update_content(
    state: &AppState,
    auth: &AuthUser,
    args: &Value,
) -> ApiResult<ExecuteToolResponse> {
    let existing: Option<Entry> = match str_arg(args, "id") {
        Some(id) => Some(entries::get(&state.db, parse_uuid(id)?).await?),
        None => None,
    };
    if let Some(current) = &existing {
        rustforge_core::document::validate_source_patch(&current.blocks, &current.body, args)?;
        EntryInput::from_patch(
            current,
            json!({"expected_updated_at":args["expected_updated_at"]}),
        )?;
    }

    let title = str_arg(args, "title")
        .map(str::to_string)
        .or_else(|| existing.as_ref().map(|e| e.title.clone()))
        .ok_or_else(|| ApiError::BadRequest("titre manquant".into()))?;

    // Champs texte : valeur fournie, sinon valeur existante (mise à jour partielle sûre).
    let text = |key: &str, current: Option<&String>| -> String {
        args.get(key)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| current.cloned())
            .unwrap_or_default()
    };
    let value = |key: &str, current: Option<&Value>, default: Value| -> Value {
        args.get(key)
            .filter(|v| !v.is_null())
            .cloned()
            .or_else(|| current.cloned())
            .unwrap_or(default)
    };

    let status = match str_arg(args, "status") {
        Some("published") => EntryStatus::Published,
        Some("archived") => EntryStatus::Archived,
        Some("draft") => EntryStatus::Draft,
        Some(_) => return Err(ApiError::BadRequest("Statut de contenu inconnu".into())),
        None => existing
            .as_ref()
            .map(|e| e.status)
            .unwrap_or(EntryStatus::Draft),
    };

    // Termes : slugs (ou noms) → identifiants ; sinon termes existants.
    let term_ids = match args.get("terms").and_then(|v| v.as_array()) {
        Some(wanted) => {
            let all = terms::list_terms(&state.db, None).await?;
            let mut selected = Vec::new();
            for wanted in wanted {
                let wanted = wanted
                    .as_str()
                    .ok_or_else(|| ApiError::BadRequest("Terme invalide".into()))?;
                let slug = rustforge_core::slugify(wanted);
                let term = all
                    .iter()
                    .find(|t| t.slug == slug || t.name.eq_ignore_ascii_case(wanted))
                    .ok_or_else(|| ApiError::BadRequest(format!("Terme inconnu : {wanted}")))?;
                selected.push(term.id);
            }
            selected
        }
        None => existing
            .as_ref()
            .map(|e| e.terms.iter().map(|t| t.id).collect())
            .unwrap_or_default(),
    };

    let e = existing.as_ref();
    let input = EntryInput {
        expected_updated_at: e.map(|entry| entry.updated_at),
        title,
        slug: str_arg(args, "slug")
            .map(str::to_string)
            .or_else(|| e.map(|e| e.slug.clone())),
        excerpt: text("excerpt", e.map(|e| &e.excerpt)),
        body: text("body", e.map(|e| &e.body)),
        blocks: value("blocks", e.map(|e| &e.blocks), json!([])),
        intent: value("intent", e.map(|e| &e.intent), json!({})),
        design_tokens: value("design_tokens", e.map(|e| &e.design_tokens), json!({})),
        status,
        fields: value("fields", e.map(|e| &e.fields), json!({})),
        featured_media_id: e.and_then(|e| e.featured_media_id),
        parent_id: e.and_then(|e| e.parent_id),
        published_at: e.and_then(|e| e.published_at),
        scheduled_unpublish_at: e.and_then(|e| e.scheduled_unpublish_at),
        locale: str_arg(args, "locale")
            .map(str::to_string)
            .or_else(|| e.map(|e| e.locale.clone()))
            .unwrap_or_else(|| "fr".to_string()),
        translation_group_id: e.and_then(|e| e.translation_group_id),
        term_ids: Some(term_ids),
    };

    let (entry, verb) = match existing {
        Some(current) => (
            entries::update(
                &state.db,
                state.plugins.as_ref(),
                current.id,
                auth.user.id,
                input,
            )
            .await?,
            "mis à jour",
        ),
        None => {
            let content_type = str_arg(args, "content_type").unwrap_or("post");
            (
                entries::create(
                    &state.db,
                    state.plugins.as_ref(),
                    content_type,
                    auth.user.id,
                    input,
                )
                .await?,
                "créé",
            )
        }
    };

    Ok(ExecuteToolResponse {
        success: true,
        message: format!(
            "Contenu « {} » {verb} ({})",
            entry.title,
            status_label(entry.status)
        ),
        data: Some(json!({
            "id": entry.id, "title": entry.title, "slug": entry.slug,
            "content_type": entry.content_type, "status": entry.status,
        })),
    })
}

fn status_label(status: EntryStatus) -> &'static str {
    match status {
        EntryStatus::Draft => "brouillon",
        EntryStatus::Published => "publié",
        EntryStatus::Archived => "archivé",
        EntryStatus::Trash => "corbeille",
    }
}

// ---------------------------------------------------------------------------
// Audit SEO / GEO (heuristiques déterministes)
// ---------------------------------------------------------------------------

fn audit_entry(entry: &Entry) -> Value {
    let body = &entry.body;
    let words = body.split_whitespace().count();
    let lines: Vec<&str> = body.lines().map(str::trim).collect();
    let headings: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| l.starts_with('#'))
        .collect();
    let question_headings = headings
        .iter()
        .filter(|h| h.trim_end().ends_with('?'))
        .count();
    let list_items = lines
        .iter()
        .filter(|l| {
            l.starts_with("- ")
                || l.starts_with("* ")
                || l.chars().next().is_some_and(|c| c.is_ascii_digit()) && l.contains(". ")
        })
        .count();
    let links = body.matches("](").count();
    let title_len = entry.title.chars().count();
    let excerpt_len = entry.excerpt.chars().count();

    let mut score: i32 = 0;
    let mut recos: Vec<String> = Vec::new();
    let mut check = |ok: bool, points: i32, reco: &str| {
        if ok {
            score += points
        } else {
            recos.push(reco.to_string())
        }
    };

    check(
        (30..=65).contains(&title_len),
        15,
        "Titre : viser 30 à 65 caractères pour l'affichage dans les SERP.",
    );
    check(
        (70..=160).contains(&excerpt_len),
        15,
        "Extrait / meta description : viser 70 à 160 caractères.",
    );
    check(
        words >= 300,
        20,
        "Contenu : au moins 300 mots pour être considéré comme une source substantielle.",
    );
    check(
        headings.len() >= 2,
        15,
        "Structure : ajouter des intertitres (##) pour segmenter le contenu.",
    );
    check(
        question_headings >= 1,
        10,
        "GEO : formuler au moins un intertitre sous forme de question (les LLM citent les réponses directes).",
    );
    check(
        list_items >= 3,
        10,
        "GEO : ajouter une liste à puces de points clés, facilement extractible par les moteurs génératifs.",
    );
    check(
        links >= 1,
        5,
        "Maillage : ajouter au moins un lien interne ou une source externe.",
    );
    check(
        entry.intent.as_object().is_some_and(|o| !o.is_empty()),
        10,
        "Intention : renseigner l'audience cible et le ton pour guider les futures réécritures IA.",
    );

    json!({
        "entry_id": entry.id,
        "title": entry.title,
        "score": score,
        "metrics": {
            "words": words, "title_length": title_len, "excerpt_length": excerpt_len,
            "headings": headings.len(), "question_headings": question_headings,
            "list_items": list_items, "links": links,
        },
        "recommendations": recos,
        "schema_org": {
            "@context": "https://schema.org",
            "@type": if entry.content_type == "post" { "BlogPosting" } else { "WebPage" },
            "headline": entry.title,
            "description": entry.excerpt,
            "datePublished": entry.published_at,
            "dateModified": entry.updated_at,
            "wordCount": words,
            "author": { "@type": "Person", "name": entry.author_name.clone().unwrap_or_else(|| "RustForge".into()) }
        }
    })
}

// ---------------------------------------------------------------------------
// Statut du modèle
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AiStatus {
    pub provider: String,
    pub model: String,
    pub endpoint: Option<String>,
    pub reachable: bool,
    pub model_available: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}

pub async fn status(State(state): State<AppState>, _auth: AuthUser) -> ApiResult<Json<AiStatus>> {
    let cfg = ai::load_settings(&state.db).await?;
    let provider = format!("{:?}", cfg.provider).to_lowercase();
    let mut out = AiStatus {
        provider,
        model: cfg.model.clone(),
        endpoint: cfg.endpoint.clone(),
        reachable: false,
        model_available: false,
        models: vec![],
        error: None,
    };
    match LlmClient::new(cfg) {
        Ok(client) => match client.list_models().await {
            Ok(models) => {
                out.reachable = true;
                out.models = models.into_iter().map(|m| m.name).collect();
                out.model_available = out.models.iter().any(|m| m == &out.model);
            }
            Err(e) => out.error = Some(e.to_string()),
        },
        Err(e) => out.error = Some(e.to_string()),
    }
    Ok(Json(out))
}

// ---------------------------------------------------------------------------
// Boucle agentique
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ChatTurn {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatTurn>,
}

#[derive(Debug, Serialize)]
pub struct ActionLog {
    pub tool: String,
    pub arguments: Value,
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
    pub actions: Vec<ActionLog>,
    pub model: String,
    pub duration_ms: u128,
}

async fn system_prompt(state: &AppState, auth: &AuthUser) -> ApiResult<String> {
    let site = settings::all(&state.db).await?;
    let site_title = site
        .get("site.title")
        .and_then(|v| v.as_str())
        .unwrap_or("RustForge");
    let types = content_types::list(&state.db).await?;
    let types_desc = types
        .iter()
        .map(|t| {
            let fields = serde_json::to_string(&t.fields).unwrap_or_default();
            format!(
                "- `{}` ({}) champs personnalisés : {}",
                t.slug, t.name, fields
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let categories = terms::list_terms(&state.db, None)
        .await?
        .iter()
        .map(|t| format!("{} ({})", t.slug, t.taxonomy))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!(
        "Tu es l'assistant éditorial de RustForge, un CMS. Site : « {site_title} ». \
Date : {date}. Utilisateur : {user} (rôle {role:?}).\n\n\
Tu pilotes le CMS UNIQUEMENT via les outils fournis. Règles :\n\
1. Pour toute action (créer, modifier, auditer, changer le design), appelle l'outil adapté ; n'invente jamais un résultat.\n\
2. Pour modifier ou auditer un contenu existant sans UUID connu, appelle d'abord `search_content`.\n\
3. Quand tu rédiges un contenu, écris un corps Markdown complet et de qualité (intertitres ##, listes, au moins 400 mots), \
un extrait de 120 à 160 caractères, et renseigne `intent` (audience, ton, objectif).\n\
4. Statut `draft` par défaut ; `published` seulement si l'utilisateur demande explicitement de publier.\n\
5. Réponds toujours en français, brièvement, en résumant ce qui a été fait.\n\n\
Types de contenu disponibles :\n{types_desc}\n\nTermes existants : {categories}",
        date = chrono::Utc::now().format("%Y-%m-%d"),
        user = auth.user.display_name,
        role = auth.user.role,
    ))
}

pub async fn chat(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChatRequest>,
) -> ApiResult<Json<ChatResponse>> {
    if !auth.user.role.can_write() {
        return Err(ApiError::Forbidden);
    }
    let started = Instant::now();
    let cfg = ai::load_settings(&state.db).await?;
    let model = cfg.model.clone();
    let client = LlmClient::new(cfg)?;
    let tools = get_available_tools();

    let mut messages = vec![ChatMessage::system(system_prompt(&state, &auth).await?)];
    // Seuls les tours utilisateur/assistant sont acceptés du client.
    messages.extend(
        req.messages
            .into_iter()
            .filter(|m| matches!(m.role, Role::User | Role::Assistant))
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
                tool_calls: None,
                tool_name: None,
            }),
    );

    let mut actions = Vec::new();
    let mut reply = String::new();

    for step in 0..MAX_AGENT_STEPS {
        let answer = match client.chat(&messages, &tools).await {
            Ok(answer) => answer,
            Err(err) => {
                reply = format!(
                    "Le modèle a interrompu la conversation : {err}. Consultez les actions ci-dessous avant de réessayer."
                );
                break;
            }
        };
        let calls = answer.tool_calls.clone().unwrap_or_default();
        reply = answer.content.clone();
        messages.push(answer);

        if calls.is_empty() {
            break;
        }
        tracing::debug!(
            step,
            calls = calls.len(),
            "appels d'outils demandés par le modèle"
        );

        for call in calls {
            let args = call.function.arguments_object();
            let name = call.function.name.clone();
            let log = match propose_or_read(&state, &auth, &name, args.clone()).await {
                Ok(res) => ActionLog {
                    tool: name.clone(),
                    arguments: args,
                    success: res.success,
                    message: res.message,
                    data: res.data,
                },
                Err(err) => ActionLog {
                    tool: name.clone(),
                    arguments: args,
                    success: false,
                    message: err.to_string(),
                    data: None,
                },
            };
            let feedback =
                json!({ "success": log.success, "message": log.message, "data": log.data });
            messages.push(ChatMessage::tool_result(name, feedback.to_string()));
            actions.push(log);
        }
        if actions.iter().any(|a| {
            a.data
                .as_ref()
                .is_some_and(|d| d["approval_required"] == true)
        }) {
            reply="Une proposition est prête. Consultez ses détails et validez-la dans le Studio pour appliquer les changements.".into();
            break;
        }

        if step + 1 == MAX_AGENT_STEPS {
            reply = "J'ai atteint le nombre maximal d'étapes ; voici les actions réalisées.".into();
        }
    }

    if reply.trim().is_empty() {
        reply = if actions.is_empty() {
            "(aucune réponse du modèle)".into()
        } else {
            "Actions exécutées.".into()
        };
    }

    Ok(Json(ChatResponse {
        reply,
        actions,
        model,
        duration_ms: started.elapsed().as_millis(),
    }))
}
