//! Endpoints IA : registre d'outils, exécution déterministe, statut du modèle
//! et boucle agentique `/ai/chat` (LLM → appels d'outils → réponse).

use std::{collections::BTreeMap, time::Instant};

use axum::{Json, extract::State};
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

#[derive(Debug, Serialize)]
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
    Ok(Json(run_tool(&state, &auth, &req.tool_name, req.arguments).await?))
}

// ---------------------------------------------------------------------------
// Exécution des outils
// ---------------------------------------------------------------------------

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty())
}

fn parse_uuid(s: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(s).map_err(|_| ApiError::BadRequest(format!("UUID invalide « {s} »")))
}

/// Exécute un outil déclaratif avec les droits de l'utilisateur courant.
pub async fn run_tool(state: &AppState, auth: &AuthUser, name: &str, args: Value) -> ApiResult<ExecuteToolResponse> {
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
            let mut tokens = settings::get(&state.db, "site.design_tokens").await?.unwrap_or_else(|| json!({}));
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
            let id = parse_uuid(str_arg(&args, "entry_id").ok_or_else(|| ApiError::BadRequest("entry_id requis".into()))?)?;
            let entry = entries::get(&state.db, id).await?;
            let report = audit_entry(&entry);
            Ok(ExecuteToolResponse {
                success: true,
                message: format!("Audit SEO & GEO de « {} » : score {}/100", entry.title, report["score"]),
                data: Some(report),
            })
        }
        "search_content" => {
            let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(5).clamp(1, 20);
            let query = EntryQuery {
                q: str_arg(&args, "query").map(str::to_string),
                content_type: str_arg(&args, "content_type").map(str::to_string),
                per_page: Some(limit),
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
                    })
                })
                .collect();
            Ok(ExecuteToolResponse {
                success: true,
                message: format!("{} contenu(s) trouvé(s)", items.len()),
                data: Some(json!(items)),
            })
        }
        unknown => Err(ApiError::BadRequest(format!("outil inconnu « {unknown} »"))),
    }
}

async fn create_or_update_content(state: &AppState, auth: &AuthUser, args: &Value) -> ApiResult<ExecuteToolResponse> {
    let existing: Option<Entry> = match str_arg(args, "id") {
        Some(id) => Some(entries::get(&state.db, parse_uuid(id)?).await?),
        None => None,
    };

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
        args.get(key).filter(|v| !v.is_null()).cloned().or_else(|| current.cloned()).unwrap_or(default)
    };

    let status = match str_arg(args, "status") {
        Some("published") => EntryStatus::Published,
        Some("archived") => EntryStatus::Archived,
        Some(_) => EntryStatus::Draft,
        None => existing.as_ref().map(|e| e.status).unwrap_or(EntryStatus::Draft),
    };

    // Termes : slugs (ou noms) → identifiants ; sinon termes existants.
    let term_ids = match args.get("terms").and_then(|v| v.as_array()) {
        Some(wanted) => {
            let all = terms::list_terms(&state.db, None).await?;
            wanted
                .iter()
                .filter_map(|w| w.as_str())
                .filter_map(|w| {
                    let slug = rustforge_core::slugify(w);
                    all.iter().find(|t| t.slug == slug || t.name.eq_ignore_ascii_case(w)).map(|t| t.id)
                })
                .collect()
        }
        None => existing.as_ref().map(|e| e.terms.iter().map(|t| t.id).collect()).unwrap_or_default(),
    };

    let e = existing.as_ref();
    let input = EntryInput {
        title,
        slug: str_arg(args, "slug").map(str::to_string).or_else(|| e.map(|e| e.slug.clone())),
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
        term_ids: Some(term_ids),
    };

    let (entry, verb) = match existing {
        Some(current) => (
            entries::update(&state.db, state.plugins.as_ref(), current.id, auth.user.id, input).await?,
            "mis à jour",
        ),
        None => {
            let content_type = str_arg(args, "content_type").unwrap_or("post");
            (
                entries::create(&state.db, state.plugins.as_ref(), content_type, auth.user.id, input).await?,
                "créé",
            )
        }
    };

    Ok(ExecuteToolResponse {
        success: true,
        message: format!("Contenu « {} » {verb} ({})", entry.title, status_label(entry.status)),
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
    let headings: Vec<&str> = lines.iter().copied().filter(|l| l.starts_with('#')).collect();
    let question_headings = headings.iter().filter(|h| h.trim_end().ends_with('?')).count();
    let list_items = lines.iter().filter(|l| l.starts_with("- ") || l.starts_with("* ") || l.chars().next().is_some_and(|c| c.is_ascii_digit()) && l.contains(". ")).count();
    let links = body.matches("](").count();
    let title_len = entry.title.chars().count();
    let excerpt_len = entry.excerpt.chars().count();

    let mut score: i32 = 0;
    let mut recos: Vec<String> = Vec::new();
    let mut check = |ok: bool, points: i32, reco: &str| {
        if ok { score += points } else { recos.push(reco.to_string()) }
    };

    check((30..=65).contains(&title_len), 15, "Titre : viser 30 à 65 caractères pour l'affichage dans les SERP.");
    check((70..=160).contains(&excerpt_len), 15, "Extrait / meta description : viser 70 à 160 caractères.");
    check(words >= 300, 20, "Contenu : au moins 300 mots pour être considéré comme une source substantielle.");
    check(headings.len() >= 2, 15, "Structure : ajouter des intertitres (##) pour segmenter le contenu.");
    check(question_headings >= 1, 10, "GEO : formuler au moins un intertitre sous forme de question (les LLM citent les réponses directes).");
    check(list_items >= 3, 10, "GEO : ajouter une liste à puces de points clés, facilement extractible par les moteurs génératifs.");
    check(links >= 1, 5, "Maillage : ajouter au moins un lien interne ou une source externe.");
    check(entry.intent.as_object().is_some_and(|o| !o.is_empty()), 10, "Intention : renseigner l'audience cible et le ton pour guider les futures réécritures IA.");

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
    let site_title = site.get("site.title").and_then(|v| v.as_str()).unwrap_or("RustForge");
    let types = content_types::list(&state.db).await?;
    let types_desc = types
        .iter()
        .map(|t| {
            let fields = serde_json::to_string(&t.fields).unwrap_or_default();
            format!("- `{}` ({}) champs personnalisés : {}", t.slug, t.name, fields)
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
            .map(|m| ChatMessage { role: m.role, content: m.content, tool_calls: None, tool_name: None }),
    );

    let mut actions = Vec::new();
    let mut reply = String::new();

    for step in 0..MAX_AGENT_STEPS {
        let answer = client.chat(&messages, &tools).await?;
        let calls = answer.tool_calls.clone().unwrap_or_default();
        reply = answer.content.clone();
        messages.push(answer);

        if calls.is_empty() {
            break;
        }
        tracing::debug!(step, calls = calls.len(), "appels d'outils demandés par le modèle");

        for call in calls {
            let args = call.function.arguments_object();
            let name = call.function.name.clone();
            let log = match run_tool(&state, &auth, &name, args.clone()).await {
                Ok(res) => ActionLog { tool: name.clone(), arguments: args, success: res.success, message: res.message, data: res.data },
                Err(err) => ActionLog { tool: name.clone(), arguments: args, success: false, message: err.to_string(), data: None },
            };
            let feedback = json!({ "success": log.success, "message": log.message, "data": log.data });
            messages.push(ChatMessage::tool_result(name, feedback.to_string()));
            actions.push(log);
        }

        if step + 1 == MAX_AGENT_STEPS {
            reply = "J'ai atteint le nombre maximal d'étapes ; voici les actions réalisées.".into();
        }
    }

    if reply.trim().is_empty() {
        reply = if actions.is_empty() { "(aucune réponse du modèle)".into() } else { "Actions exécutées.".into() };
    }

    Ok(Json(ChatResponse { reply, actions, model, duration_ms: started.elapsed().as_millis() }))
}
