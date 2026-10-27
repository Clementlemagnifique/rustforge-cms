//! Désérialiseur Rust natif pour les données PHP sérialisées de WordPress (`serialize()` / `unserialize()`).
//! Convertit les structures `a:N:{...}`, `s:N:"..."`, `i:N;`, `d:N;`, `b:N;`, `O:N:"...":...`
//! en `serde_json::Value` (JSONB PostgreSQL) sans exécuter de PHP.

use serde_json::{Map, Number, Value};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum PhpParseError {
    UnexpectedEnd,
    InvalidFormat(String),
    RecursionLimitExceeded,
}

impl std::fmt::Display for PhpParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "Fin inattendue des données PHP"),
            Self::InvalidFormat(msg) => write!(f, "Format PHP sérialisé invalide: {}", msg),
            Self::RecursionLimitExceeded => write!(f, "Limite de récursion PHP dépassée"),
        }
    }
}

impl std::error::Error for PhpParseError {}

/// Parseur récursif sécurisé pour données PHP sérialisées.
pub struct PhpDeserializer<'a> {
    input: &'a str,
    pos: usize,
    depth: usize,
}

const MAX_DEPTH: usize = 64;

impl<'a> PhpDeserializer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            depth: 0,
        }
    }

    /// Désérialise une chaîne PHP en `serde_json::Value`.
    pub fn deserialize(input: &str) -> Result<Value, PhpParseError> {
        let trimmed = input.trim();
        let mut de = PhpDeserializer::new(trimmed);
        let val = de.parse_value()?;
        Ok(val)
    }

    fn remaining(&self) -> &'a str {
        if self.pos < self.input.len() {
            &self.input[self.pos..]
        } else {
            ""
        }
    }

    fn next_char(&mut self) -> Result<char, PhpParseError> {
        let c = self.remaining().chars().next().ok_or(PhpParseError::UnexpectedEnd)?;
        self.pos += c.len_utf8();
        Ok(c)
    }

    #[allow(dead_code)]
    fn peek_char(&self) -> Result<char, PhpParseError> {
        self.remaining().chars().next().ok_or(PhpParseError::UnexpectedEnd)
    }

    fn expect_char(&mut self, expected: char) -> Result<(), PhpParseError> {
        let c = self.next_char()?;
        if c == expected {
            Ok(())
        } else {
            Err(PhpParseError::InvalidFormat(format!(
                "Caractère attendu '{}', trouvé '{}' à l'index {}",
                expected, c, self.pos
            )))
        }
    }

    fn read_until(&mut self, delimiter: char) -> Result<&'a str, PhpParseError> {
        let rem = self.remaining();
        if let Some(idx) = rem.find(delimiter) {
            let res = &rem[..idx];
            self.pos += idx + delimiter.len_utf8();
            Ok(res)
        } else {
            Err(PhpParseError::UnexpectedEnd)
        }
    }

    fn parse_value(&mut self) -> Result<Value, PhpParseError> {
        if self.depth > MAX_DEPTH {
            return Err(PhpParseError::RecursionLimitExceeded);
        }

        let type_char = self.next_char()?;
        match type_char {
            // Null : N;
            'N' => {
                self.expect_char(';')?;
                Ok(Value::Null)
            }
            // Booléen : b:1; ou b:0;
            'b' => {
                self.expect_char(':')?;
                let val_str = self.read_until(';')?;
                let b = val_str == "1";
                Ok(Value::Bool(b))
            }
            // Entier : i:123;
            'i' => {
                self.expect_char(':')?;
                let val_str = self.read_until(';')?;
                let n = i64::from_str(val_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Entier PHP invalide '{}': {}", val_str, e))
                })?;
                Ok(Value::Number(Number::from(n)))
            }
            // Décimal / Float : d:12.34;
            'd' => {
                self.expect_char(':')?;
                let val_str = self.read_until(';')?;
                let f = f64::from_str(val_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Float PHP invalide '{}': {}", val_str, e))
                })?;
                if let Some(num) = Number::from_f64(f) {
                    Ok(Value::Number(num))
                } else {
                    Ok(Value::String(val_str.to_string()))
                }
            }
            // Chaîne : s:5:"hello";
            's' => {
                self.expect_char(':')?;
                let len_str = self.read_until(':')?;
                let byte_len = usize::from_str(len_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Longueur de chaîne invalide '{}': {}", len_str, e))
                })?;
                self.expect_char('"')?;

                let rem = self.remaining();
                if rem.len() < byte_len {
                    return Err(PhpParseError::UnexpectedEnd);
                }

                // PHP sérialise la longueur en octets
                let str_slice = &rem[..byte_len];
                self.pos += byte_len;

                self.expect_char('"')?;
                self.expect_char(';')?;
                Ok(Value::String(str_slice.to_string()))
            }
            // Tableau : a:2:{i:0;s:3:"foo";s:3:"bar";s:3:"baz";}
            'a' => {
                self.expect_char(':')?;
                let len_str = self.read_until(':')?;
                let count = usize::from_str(len_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Taille de tableau invalide '{}': {}", len_str, e))
                })?;
                self.expect_char('{')?;

                self.depth += 1;
                let mut map = Map::new();
                let mut is_indexed_array = true;
                let mut array_items = Vec::with_capacity(count);

                for i in 0..count {
                    let key = self.parse_value()?;
                    let val = self.parse_value()?;

                    match key {
                        Value::Number(n) if n.as_u64() == Some(i as u64) && is_indexed_array => {
                            array_items.push(val.clone());
                            map.insert(i.to_string(), val);
                        }
                        Value::Number(n) => {
                            is_indexed_array = false;
                            map.insert(n.to_string(), val);
                        }
                        Value::String(s) => {
                            is_indexed_array = false;
                            map.insert(s, val);
                        }
                        _ => {
                            is_indexed_array = false;
                            map.insert(format!("{:?}", key), val);
                        }
                    }
                }
                self.depth -= 1;
                self.expect_char('}')?;

                if is_indexed_array && array_items.len() == count {
                    Ok(Value::Array(array_items))
                } else {
                    Ok(Value::Object(map))
                }
            }
            // Objet PHP : O:8:"stdClass":1:{s:3:"foo";s:3:"bar";}
            'O' => {
                self.expect_char(':')?;
                let class_name_len_str = self.read_until(':')?;
                let class_name_len = usize::from_str(class_name_len_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Longueur nom classe invalide: {}", e))
                })?;
                self.expect_char('"')?;

                let rem = self.remaining();
                if rem.len() < class_name_len {
                    return Err(PhpParseError::UnexpectedEnd);
                }
                let class_name = &rem[..class_name_len];
                self.pos += class_name_len;
                self.expect_char('"')?;
                self.expect_char(':')?;

                let props_count_str = self.read_until(':')?;
                let props_count = usize::from_str(props_count_str).map_err(|e| {
                    PhpParseError::InvalidFormat(format!("Nombre propriétés invalide: {}", e))
                })?;
                self.expect_char('{')?;

                self.depth += 1;
                let mut obj = Map::new();
                obj.insert("__php_class".to_string(), Value::String(class_name.to_string()));

                for _ in 0..props_count {
                    let prop_name = match self.parse_value()? {
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        other => format!("{:?}", other),
                    };
                    let prop_val = self.parse_value()?;
                    obj.insert(prop_name, prop_val);
                }
                self.depth -= 1;
                self.expect_char('}')?;

                Ok(Value::Object(obj))
            }
            other => Err(PhpParseError::InvalidFormat(format!(
                "Type PHP sérialisé inconnu '{}' à l'index {}",
                other, self.pos
            ))),
        }
    }
}

/// Tente de désérialiser une chaîne. Si la chaîne n'est pas du PHP sérialisé valide,
/// retourne la chaîne d'origine enveloppée dans `Value::String`.
pub fn maybe_deserialize_php(input: &str) -> Value {
    let trimmed = input.trim();
    if is_likely_php_serialized(trimmed) {
        if let Ok(val) = PhpDeserializer::deserialize(trimmed) {
            return val;
        }
    }
    Value::String(input.to_string())
}

/// Heuristique rapide pour détecter si une chaîne commence par un marqueur PHP sérialisé.
pub fn is_likely_php_serialized(s: &str) -> bool {
    let s = s.trim();
    if s.len() < 2 {
        return false;
    }
    match &s[..2] {
        "a:" | "O:" | "s:" | "i:" | "d:" | "b:" => true,
        "N;" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_primitives() {
        assert_eq!(PhpDeserializer::deserialize("N;").unwrap(), Value::Null);
        assert_eq!(PhpDeserializer::deserialize("b:1;").unwrap(), Value::Bool(true));
        assert_eq!(PhpDeserializer::deserialize("b:0;").unwrap(), Value::Bool(false));
        assert_eq!(PhpDeserializer::deserialize("i:42;").unwrap(), serde_json::json!(42));
        assert_eq!(PhpDeserializer::deserialize("d:3.14;").unwrap(), serde_json::json!(3.14));
        assert_eq!(PhpDeserializer::deserialize("s:5:\"hello\";").unwrap(), serde_json::json!("hello"));
    }

    #[test]
    fn test_deserialize_indexed_array() {
        let input = "a:3:{i:0;s:5:\"apple\";i:1;s:6:\"banana\";i:2;s:6:\"cherry\";}";
        let res = PhpDeserializer::deserialize(input).unwrap();
        assert_eq!(res, serde_json::json!(["apple", "banana", "cherry"]));
    }

    #[test]
    fn test_deserialize_associative_array() {
        let input = "a:2:{s:4:\"name\";s:5:\"Alice\";s:3:\"age\";i:30;}";
        let res = PhpDeserializer::deserialize(input).unwrap();
        assert_eq!(res, serde_json::json!({
            "name": "Alice",
            "age": 30
        }));
    }

    #[test]
    fn test_deserialize_nested_structures() {
        let input = "a:2:{s:5:\"roles\";a:2:{i:0;s:5:\"admin\";i:1;s:6:\"editor\";}s:6:\"active\";b:1;}";
        let res = PhpDeserializer::deserialize(input).unwrap();
        assert_eq!(res, serde_json::json!({
            "roles": ["admin", "editor"],
            "active": true
        }));
    }

    #[test]
    fn test_deserialize_php_object() {
        let input = "O:8:\"stdClass\":2:{s:4:\"host\";s:9:\"localhost\";s:4:\"port\";i:5432;}";
        let res = PhpDeserializer::deserialize(input).unwrap();
        assert_eq!(res, serde_json::json!({
            "__php_class": "stdClass",
            "host": "localhost",
            "port": 5432
        }));
    }

    #[test]
    fn test_maybe_deserialize_fallback() {
        let normal_text = "Ceci est un texte brut non sérialisé.";
        assert_eq!(maybe_deserialize_php(normal_text), Value::String(normal_text.to_string()));
    }
}
