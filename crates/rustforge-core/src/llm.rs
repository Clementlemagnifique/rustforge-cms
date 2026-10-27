//! # Client LLM
//!
//! Connecteur vers les modèles de langage. Première implémentation : **Ollama**
//! (API `/api/chat` avec *tool calling* natif), exécuté en local.
//!
//! La boucle agentique (modèle → appel d'outil → résultat → modèle) vit dans la
//! couche API, qui possède le contexte d'authentification nécessaire à
//! l'exécution des outils.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    CoreError, CoreResult,
    ai::{AiProvider, AiSettings, ToolDefinition},
};

/// Rôle d'un message dans une conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Appel d'outil demandé par le modèle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    /// Ollama renvoie un objet JSON ; certains modèles renvoient une chaîne JSON.
    #[serde(default)]
    pub arguments: Value,
}

impl FunctionCall {
    /// Arguments normalisés en objet JSON (décode une éventuelle chaîne JSON).
    pub fn arguments_object(&self) -> Value {
        match &self.arguments {
            Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| json!({})),
            Value::Null => json!({}),
            other => other.clone(),
        }
    }
}

/// Message de conversation au format Ollama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Nom de l'outil dont ce message (rôle `tool`) est le résultat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into(), tool_calls: None, tool_name: None }
    }
    pub fn tool_result(tool_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: Role::Tool, content: content.into(), tool_calls: None, tool_name: Some(tool_name.into()) }
    }
}

/// Modèle disponible localement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    #[serde(default)]
    pub size: u64,
}

/// Client Ollama.
#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    settings: AiSettings,
}

impl LlmClient {
    pub fn new(settings: AiSettings) -> CoreResult<Self> {
        if settings.provider != AiProvider::Ollama {
            return Err(CoreError::validation(format!(
                "fournisseur IA {:?} pas encore pris en charge (seul Ollama l'est pour l'instant)",
                settings.provider
            )));
        }
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            // Un modèle local de 12B peut mettre du temps (chargement + génération).
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|e| CoreError::Internal(e.into()))?;
        Ok(Self { http, settings })
    }

    pub fn settings(&self) -> &AiSettings {
        &self.settings
    }

    fn endpoint(&self) -> String {
        self.settings
            .endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".into())
            .trim_end_matches('/')
            .to_string()
    }

    /// Liste les modèles installés dans Ollama.
    pub async fn list_models(&self) -> CoreResult<Vec<ModelInfo>> {
        #[derive(Deserialize)]
        struct Tags {
            models: Vec<ModelInfo>,
        }
        let res = self
            .http
            .get(format!("{}/api/tags", self.endpoint()))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(unreachable)?;
        let tags: Tags = res.error_for_status().map_err(unreachable)?.json().await.map_err(unreachable)?;
        Ok(tags.models)
    }

    /// Envoie la conversation au modèle avec les outils disponibles ; renvoie la réponse de l'assistant.
    pub async fn chat(&self, messages: &[ChatMessage], tools: &[ToolDefinition]) -> CoreResult<ChatMessage> {
        let tools: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": { "name": t.name, "description": t.description, "parameters": t.parameters }
                })
            })
            .collect();

        let mut options = json!({ "num_ctx": 16384 });
        if let Some(t) = self.settings.temperature {
            options["temperature"] = json!(t);
        }
        if let Some(n) = self.settings.max_tokens {
            options["num_predict"] = json!(n);
        }

        let body = json!({
            "model": self.settings.model,
            "messages": messages,
            "tools": tools,
            "stream": false,
            "options": options,
        });

        let res = self
            .http
            .post(format!("{}/api/chat", self.endpoint()))
            .json(&body)
            .send()
            .await
            .map_err(unreachable)?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            let detail = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
                .unwrap_or(text);
            return Err(CoreError::validation(format!("Ollama a répondu {status} : {detail}")));
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            message: ChatMessage,
        }
        let parsed: ChatResponse = res.json().await.map_err(|e| CoreError::Internal(e.into()))?;
        Ok(parsed.message)
    }
}

fn unreachable(e: reqwest::Error) -> CoreError {
    CoreError::validation(format!("Ollama injoignable : {e}. Vérifiez que « ollama serve » est lancé."))
}
