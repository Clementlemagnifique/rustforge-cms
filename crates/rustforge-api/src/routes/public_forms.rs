//! Server-rendered public forms: no client framework or arbitrary embed code.
use crate::{
    error::{ApiError, ApiResult},
    state::AppState,
};
use axum::{
    extract::{Form, Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use rustforge_core::{CoreError, forms, settings};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
#[derive(Serialize, Deserialize)]
struct Claims {
    form_id: uuid::Uuid,
    scope: String,
    exp: usize,
}
pub(crate) fn fields(form: &forms::Form) -> ApiResult<Vec<Value>> {
    let properties = form.fields["properties"].as_object().ok_or_else(|| {
        ApiError::BadRequest("Ce formulaire ne peut pas encore être présenté sur cette page".into())
    })?;
    if properties.len() > 100 {
        return Err(ApiError::BadRequest("Trop de champs".into()));
    }
    properties.iter().map(|(key,schema)|{
  if key.starts_with('_')||key.len()>64||!key.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')||!matches!(schema["type"].as_str(),Some("string"|"integer"|"number"|"boolean")){return Err(ApiError::BadRequest("Un champ de ce formulaire nécessite un affichage avancé".into()));}
  Ok(json!({"key":key,"label":schema["title"].as_str().unwrap_or(key),"type":schema["type"],"input_type":match schema["type"].as_str(){Some("integer"|"number")=>"number",_=>match schema["format"].as_str(){Some("email")=>"email",Some("uri")=>"url",Some("date")=>"date",_=>"text"}},"choices":schema["enum"].as_array().map(|items|items.iter().map(|value|value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string())).collect::<Vec<_>>()),"required":form.fields["required"].as_array().is_some_and(|items|items.contains(&json!(key))),"min":schema["minimum"],"max":schema["maximum"],"maxlength":schema["maxLength"]}))
 }).collect()
}
fn render_html(
    state: &AppState,
    form: &forms::Form,
    sent: bool,
    error: Option<&str>,
    values: Option<&HashMap<String, String>>,
    embedded: bool,
    prefix: &str,
) -> ApiResult<String> {
    let claims = Claims {
        form_id: form.id,
        scope: "public_form".into(),
        exp: (chrono::Utc::now() + chrono::Duration::minutes(30)).timestamp() as usize,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    let mut env = minijinja::Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    env.add_template("form.html", include_str!("../../templates/form.html"))
        .map_err(|error| ApiError::Internal(error.into()))?;
    env.add_template(
        "form-fields.html",
        include_str!("../../templates/form-fields.html"),
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    env.add_template("form-embed.html","<section class=\"rf-form-embed\"><h2>{{ form.name }}</h2><p>{{ form.description }}</p>{% include \"form-fields.html\" %}</section>").map_err(|error|ApiError::Internal(error.into()))?;
    let html=env.get_template(if embedded{"form-embed.html"}else{"form.html"}).map_err(|error|ApiError::Internal(error.into()))?.render(json!({"form":{"id":form.id,"name":form.name,"description":form.description},"prefix":prefix,"fields":fields(form)?,"token":token,"sent":sent,"error":error,"values":values})).map_err(|error|ApiError::Internal(error.into()))?;
    Ok(html)
}
fn render(
    state: &AppState,
    form: &forms::Form,
    sent: bool,
    error: Option<&str>,
    values: Option<&HashMap<String, String>>,
) -> ApiResult<Response> {
    let html = render_html(state, form, sent, error, values, false, "field")?;
    Ok((
        if error.is_some() {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::OK
        },
        [(header::CACHE_CONTROL, "no-store")],
        Html(html),
    )
        .into_response())
}

/// Resolve markers after the page cache: form availability and expiring tokens stay current.
pub async fn expand(state: &AppState, html: String) -> ApiResult<(String, bool)> {
    const PREFIX: &str = "<div data-rf-form=\"";
    const SUFFIX: &str = "\">Formulaire intégré</div>";
    if !html.contains(PREFIX) {
        return Ok((html, false));
    }
    let enabled = settings::is_module_active(&state.db, "forms").await;
    let mut rest = html.as_str();
    let mut out = String::with_capacity(html.len());
    let mut index = 0;
    while let Some(offset) = rest.find(PREFIX) {
        out.push_str(&rest[..offset]);
        rest = &rest[offset + PREFIX.len()..];
        let Some(end) = rest.find(SUFFIX) else {
            out.push_str(PREFIX);
            out.push_str(rest);
            rest = "";
            break;
        };
        let raw = &rest[..end];
        index += 1;
        let form = if enabled && index <= 8 {
            if let Ok(id) = uuid::Uuid::parse_str(raw) {
                match forms::get_form(&state.db, id).await {
                    Ok(form) if form.is_active => Some(form),
                    Ok(_) | Err(CoreError::NotFound) => None,
                    Err(error) => return Err(error.into()),
                }
            } else {
                None
            }
        } else {
            None
        };
        if let Some(form) = form {
            match render_html(
                state,
                &form,
                false,
                None,
                None,
                true,
                &format!("rf-form-{index}"),
            ) {
                Ok(rendered) => out.push_str(&rendered),
                Err(ApiError::BadRequest(_)) => {
                    out.push_str("<p>Ce formulaire nécessite un affichage avancé.</p>")
                }
                Err(error) => return Err(error),
            }
        } else {
            out.push_str("<p class=\"rf-form-unavailable\">Formulaire indisponible.</p>");
        }
        rest = &rest[end + SUFFIX.len()..];
    }
    out.push_str(rest);
    Ok((out, true))
}
pub async fn page_response(state: &AppState, html: String) -> ApiResult<Response> {
    let (html, dynamic) = expand(state, html).await?;
    let mut response = Html(html).into_response();
    if dynamic {
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-store"),
        );
    }
    Ok(response)
}

async fn active(state: &AppState, slug: &str) -> ApiResult<forms::Form> {
    if !settings::is_module_active(&state.db, "forms").await {
        return Err(ApiError::NotFound);
    }
    let form = if let Ok(id) = uuid::Uuid::parse_str(slug) {
        forms::get_form(&state.db, id).await?
    } else {
        forms::get_form_by_slug(&state.db, slug).await?
    };
    if !form.is_active {
        return Err(ApiError::NotFound);
    }
    Ok(form)
}
pub async fn page(State(state): State<AppState>, Path(slug): Path<String>) -> ApiResult<Response> {
    let form = active(&state, &slug).await?;
    render(&state, &form, false, None, None)
}
pub async fn submit(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Form(values): Form<HashMap<String, String>>,
) -> ApiResult<Response> {
    let form = active(&state, &slug).await?;
    let encoded = values
        .get("_rf_token")
        .ok_or_else(|| ApiError::BadRequest("Formulaire expiré ; rechargez la page".into()))?;
    let token = jsonwebtoken::decode::<Claims>(
        encoded,
        &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| ApiError::BadRequest("Formulaire expiré ; rechargez la page".into()))?;
    if token.claims.form_id != form.id || token.claims.scope != "public_form" {
        return Err(ApiError::Forbidden);
    }
    let mut data = serde_json::Map::new();
    for field in fields(&form)? {
        let key = field["key"].as_str().unwrap_or_default();
        let raw = values.get(key);
        let value = match field["type"].as_str() {
            Some("boolean") => json!(raw.is_some_and(|value| value == "true")),
            Some("integer") => match raw.filter(|value| !value.is_empty()) {
                Some(value) => match value.parse::<i64>() {
                    Ok(value) => json!(value),
                    Err(_) => {
                        return render(
                            &state,
                            &form,
                            false,
                            Some("Un nombre entier est attendu"),
                            Some(&values),
                        );
                    }
                },
                None => continue,
            },
            Some("number") => match raw.filter(|value| !value.is_empty()) {
                Some(value) => match value.parse::<f64>() {
                    Ok(value) if value.is_finite() => json!(value),
                    _ => {
                        return render(
                            &state,
                            &form,
                            false,
                            Some("Un nombre est attendu"),
                            Some(&values),
                        );
                    }
                },
                None => continue,
            },
            _ => match raw {
                Some(value) => json!(value),
                None => continue,
            },
        };
        data.insert(key.to_owned(), value);
    }
    let payload = forms::SubmitFormPayload {
        data: Value::Object(data),
        _website: values.get("_website").cloned(),
        user_agent: None,
        client_ip: None,
    };
    match forms::submit(&state.db, form.id, payload).await {
        Ok(_) => render(&state, &form, true, None, None),
        Err(CoreError::Validation(error)) => {
            render(&state, &form, false, Some(&error), Some(&values))
        }
        Err(error) => Err(error.into()),
    }
}
