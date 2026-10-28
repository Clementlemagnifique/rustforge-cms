//! Deliberately supported form schema subset; unknown constraints are rejected.
use crate::{CoreError, CoreResult};
use serde_json::Value;
pub fn schema(schema: &Value, depth: usize) -> CoreResult<()> {
    if depth > 16 {
        return Err(CoreError::validation("schéma trop profond"));
    }
    let object = schema
        .as_object()
        .ok_or_else(|| CoreError::validation("schéma de formulaire objet requis"))?;
    const ALLOWED: &[&str] = &[
        "type",
        "properties",
        "required",
        "additionalProperties",
        "title",
        "description",
        "format",
        "minLength",
        "maxLength",
        "minimum",
        "maximum",
        "enum",
    ];
    if object.keys().any(|k| !ALLOWED.contains(&k.as_str())) {
        return Err(CoreError::validation(
            "contrainte de formulaire non prise en charge",
        ));
    }
    if !matches!(
        schema["type"].as_str(),
        Some("object" | "string" | "number" | "integer" | "boolean")
    ) {
        return Err(CoreError::validation("type de champ non pris en charge"));
    }
    for key in ["minLength", "maxLength"] {
        if schema.get(key).is_some_and(|v| v.as_u64().is_none()) {
            return Err(CoreError::validation("limite de longueur invalide"));
        }
    }
    for key in ["minimum", "maximum"] {
        if schema.get(key).is_some_and(|v| v.as_f64().is_none()) {
            return Err(CoreError::validation("limite numérique invalide"));
        }
    }
    if schema
        .get("additionalProperties")
        .is_some_and(|v| !v.is_boolean())
        || schema
            .get("enum")
            .is_some_and(|v| v.as_array().is_none_or(|a| a.is_empty()))
    {
        return Err(CoreError::validation("contrainte invalide"));
    }
    if let Some(required) = schema.get("required") {
        let keys = required
            .as_array()
            .ok_or_else(|| CoreError::validation("required doit être une liste"))?;
        for key in keys {
            let key = key
                .as_str()
                .ok_or_else(|| CoreError::validation("nom obligatoire invalide"))?;
            if schema["properties"].get(key).is_none() {
                return Err(CoreError::validation("champ obligatoire non déclaré"));
            }
        }
    }
    if schema.get("format").is_some_and(|v| !v.is_string()) {
        return Err(CoreError::validation("format invalide"));
    }
    if let Some(properties) = schema.get("properties") {
        for child in properties
            .as_object()
            .ok_or_else(|| CoreError::validation("properties invalide"))?
            .values()
        {
            self::schema(child, depth + 1)?;
        }
    }
    if let Some(format) = schema.get("format").and_then(Value::as_str)
        && !matches!(format, "email" | "date" | "uri")
    {
        return Err(CoreError::validation("format non pris en charge"));
    }
    Ok(())
}
pub fn validate(schema: &Value, value: &Value) -> CoreResult<()> {
    self::schema(schema, 0)?;
    fn check(s: &Value, v: &Value) -> CoreResult<()> {
        let valid = match s["type"].as_str() {
            Some("object") => v.is_object(),
            Some("string") => v.is_string(),
            Some("number") => v.is_number(),
            Some("integer") => v.as_i64().is_some(),
            Some("boolean") => v.is_boolean(),
            _ => false,
        };
        if !valid {
            return Err(CoreError::validation("type de champ invalide"));
        }
        if let Some(choices) = s.get("enum").and_then(Value::as_array)
            && !choices.contains(v)
        {
            return Err(CoreError::validation("valeur hors des choix autorisés"));
        }
        if let Some(obj) = v.as_object() {
            if let Some(required) = s["required"].as_array() {
                for key in required {
                    let key = key
                        .as_str()
                        .ok_or_else(|| CoreError::validation("required invalide"))?;
                    if obj.get(key).is_none_or(|v| {
                        v.is_null() || v.as_str().is_some_and(|s| s.trim().is_empty())
                    }) {
                        return Err(CoreError::validation(format!("champ obligatoire : {key}")));
                    }
                }
            }
            for (key, value) in obj {
                if let Some(child) = s["properties"].get(key) {
                    check(child, value)?;
                } else if s["additionalProperties"] != true {
                    return Err(CoreError::validation(format!("champ inconnu : {key}")));
                }
            }
        }
        if let Some(text) = v.as_str() {
            let len = text.chars().count() as u64;
            if s["minLength"].as_u64().is_some_and(|n| len < n)
                || s["maxLength"].as_u64().is_some_and(|n| len > n)
            {
                return Err(CoreError::validation("longueur de champ invalide"));
            }
            let valid = match s["format"].as_str() {
                Some("email") => text.split_once('@').is_some_and(|(a, b)| {
                    !a.is_empty() && b.contains('.') && !text.contains(char::is_whitespace)
                }),
                Some("date") => chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").is_ok(),
                Some("uri") => {
                    reqwest::Url::parse(text).is_ok_and(|u| matches!(u.scheme(), "http" | "https"))
                }
                _ => true,
            };
            if !valid {
                return Err(CoreError::validation("format de champ invalide"));
            }
        }
        if let Some(n) = v.as_f64()
            && (s["minimum"].as_f64().is_some_and(|min| n < min)
                || s["maximum"].as_f64().is_some_and(|max| n > max))
        {
            return Err(CoreError::validation("valeur numérique hors limites"));
        }
        Ok(())
    }
    check(schema, value)
}
