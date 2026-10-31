//! Champs personnalisés : schéma déclaratif par type de contenu et validation.
//!
//! Équivalent natif d'ACF : chaque type de contenu déclare une liste de
//! [`FieldDef`] ; les valeurs sont stockées dans `entries.fields` (JSONB)
//! après validation stricte.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    Text,
    Textarea,
    Number,
    Boolean,
    Date,
    Url,
    Select,
    Media,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: FieldKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    /// Choix possibles (type `select`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

/// Vérifie la cohérence d'un schéma de champs.
pub fn validate_schema(defs: &[FieldDef]) -> CoreResult<()> {
    let mut seen = std::collections::HashSet::new();
    for def in defs {
        let valid_key = !def.key.is_empty()
            && def.key.len() <= 64
            && def
                .key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && def
                .key
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase());
        if !valid_key {
            return Err(CoreError::validation(format!(
                "clé de champ invalide « {} » (minuscules, chiffres et _ uniquement)",
                def.key
            )));
        }
        if !seen.insert(def.key.as_str()) {
            return Err(CoreError::validation(format!(
                "clé de champ dupliquée « {} »",
                def.key
            )));
        }
        if def.label.trim().is_empty() {
            return Err(CoreError::validation(format!(
                "le champ « {} » doit avoir un libellé",
                def.key
            )));
        }
        if def.kind == FieldKind::Select && def.options.is_empty() {
            return Err(CoreError::validation(format!(
                "le champ « {} » de type liste doit proposer des options",
                def.key
            )));
        }
    }
    Ok(())
}

/// Valide les champs déclarés sans détruire les métadonnées inconnues conservées.
pub fn validate_values(defs: &[FieldDef], input: &Value) -> CoreResult<Value> {
    let empty = Map::new();
    let obj = match input {
        Value::Object(m) => m,
        Value::Null => &empty,
        _ => {
            return Err(CoreError::validation(
                "les champs personnalisés doivent être un objet",
            ));
        }
    };

    let mut out = obj.clone();
    for def in defs {
        let value = obj.get(&def.key).cloned().unwrap_or(Value::Null);
        let is_empty = match &value {
            Value::Null => true,
            Value::String(s) => s.trim().is_empty(),
            _ => false,
        };
        if is_empty {
            if def.required {
                return Err(CoreError::validation(format!(
                    "le champ « {} » est obligatoire",
                    def.label
                )));
            }
            continue;
        }

        let err = |what: &str| {
            CoreError::validation(format!("le champ « {} » doit être {what}", def.label))
        };
        let normalized = match def.kind {
            FieldKind::Text | FieldKind::Textarea => {
                Value::String(value.as_str().ok_or_else(|| err("un texte"))?.to_string())
            }
            FieldKind::Number => match &value {
                Value::Number(_) => value.clone(),
                Value::String(s) => s
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(Value::Number)
                    .ok_or_else(|| err("un nombre"))?,
                _ => return Err(err("un nombre")),
            },
            FieldKind::Boolean => Value::Bool(value.as_bool().ok_or_else(|| err("un booléen"))?),
            FieldKind::Date => {
                let s = value.as_str().ok_or_else(|| err("une date"))?;
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|_| err("une date AAAA-MM-JJ"))?;
                Value::String(s.to_string())
            }
            FieldKind::Url => {
                let s = value.as_str().ok_or_else(|| err("une URL"))?.trim();
                if !(s.starts_with("https://") || s.starts_with("http://") || s.starts_with('/')) {
                    return Err(err("une URL http(s)"));
                }
                Value::String(s.to_string())
            }
            FieldKind::Select => {
                let s = value.as_str().ok_or_else(|| err("une option"))?;
                if !def.options.iter().any(|o| o == s) {
                    return Err(err("une des options proposées"));
                }
                Value::String(s.to_string())
            }
            FieldKind::Media => {
                let s = value.as_str().ok_or_else(|| err("un média"))?;
                uuid::Uuid::parse_str(s).map_err(|_| err("un identifiant de média"))?;
                Value::String(s.to_string())
            }
        };
        out.insert(def.key.clone(), normalized);
    }
    Ok(Value::Object(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn def(key: &str, kind: FieldKind, required: bool) -> FieldDef {
        FieldDef {
            key: key.into(),
            label: key.into(),
            kind,
            required,
            help: None,
            options: vec![],
        }
    }

    #[test]
    fn validates_and_normalizes() {
        let defs = vec![
            def("price", FieldKind::Number, true),
            def("featured", FieldKind::Boolean, false),
            def("date", FieldKind::Date, false),
        ];
        let out = validate_values(
            &defs,
            &json!({"price": "12.5", "featured": true, "junk": 1}),
        )
        .unwrap();
        assert_eq!(out, json!({"price": 12.5, "featured": true, "junk": 1}));
    }

    #[test]
    fn rejects_missing_required_and_bad_types() {
        let defs = vec![def("price", FieldKind::Number, true)];
        assert!(validate_values(&defs, &json!({})).is_err());
        assert!(validate_values(&defs, &json!({"price": "abc"})).is_err());
    }

    #[test]
    fn schema_rules() {
        assert!(validate_schema(&[def("Bad Key", FieldKind::Text, false)]).is_err());
        assert!(
            validate_schema(&[
                def("a", FieldKind::Text, false),
                def("a", FieldKind::Text, false)
            ])
            .is_err()
        );
        assert!(validate_schema(&[def("select_me", FieldKind::Select, false)]).is_err());
        assert!(validate_schema(&[def("subtitle", FieldKind::Text, false)]).is_ok());
    }
}
