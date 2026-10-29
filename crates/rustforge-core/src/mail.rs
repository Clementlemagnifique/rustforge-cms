//! Contrat mail transactionnel. Transport de capture seul, aucun envoi réseau implicite.
use crate::{CoreError, CoreResult, jobs, users};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub const KIND: &str = "rf.mail.transactional";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailRequest {
    pub recipient_id: Uuid,
    pub subject: String,
    pub text: String,
}
impl MailRequest {
    pub fn validate(&self) -> CoreResult<()> {
        if self.subject.trim().is_empty()
            || self.subject.len() > 512
            || self.subject.chars().any(char::is_control)
            || self.text.trim().is_empty()
            || self.text.len() > 32 * 1024
        {
            return Err(CoreError::validation("Message transactionnel invalide"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub recipient: String,
    pub subject: String,
    pub text: String,
}
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub transport: &'static str,
    pub receipt_id: String,
    /// Uniquement pour la capture locale ; résultat privé, jamais une réponse publique.
    pub capture: Option<Message>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryError {
    pub code: &'static str,
    pub retryable: bool,
}
#[async_trait]
pub trait Transport: Send + Sync {
    async fn deliver(
        &self,
        message: Message,
        idempotency_key: Uuid,
    ) -> Result<Receipt, DeliveryError>;
}
pub struct CaptureTransport;
#[async_trait]
impl Transport for CaptureTransport {
    async fn deliver(
        &self,
        message: Message,
        idempotency_key: Uuid,
    ) -> Result<Receipt, DeliveryError> {
        Ok(Receipt {
            transport: "capture",
            receipt_id: idempotency_key.to_string(),
            capture: Some(message),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MailMode {
    #[default]
    Capture,
    Simulated,
    Smtp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulatedOutcome {
    Success,
    TransientError,
    PermanentError,
}

pub struct SimulatedTransport {
    pub outcome: SimulatedOutcome,
}

impl SimulatedTransport {
    pub fn new(outcome: SimulatedOutcome) -> Self {
        Self { outcome }
    }
}

#[async_trait]
impl Transport for SimulatedTransport {
    async fn deliver(
        &self,
        _message: Message,
        idempotency_key: Uuid,
    ) -> Result<Receipt, DeliveryError> {
        match self.outcome {
            SimulatedOutcome::Success => Ok(Receipt {
                transport: "simulated",
                receipt_id: format!("sim-{idempotency_key}"),
                capture: None,
            }),
            SimulatedOutcome::TransientError => Err(DeliveryError {
                code: "simulated_transient_error",
                retryable: true,
            }),
            SimulatedOutcome::PermanentError => Err(DeliveryError {
                code: "simulated_permanent_error",
                retryable: false,
            }),
        }
    }
}

pub struct UnconfiguredTransport {
    pub reason: &'static str,
}

impl UnconfiguredTransport {
    pub fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

#[async_trait]
impl Transport for UnconfiguredTransport {
    async fn deliver(
        &self,
        _message: Message,
        _idempotency_key: Uuid,
    ) -> Result<Receipt, DeliveryError> {
        Err(DeliveryError {
            code: self.reason,
            retryable: false,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SmtpEncryption {
    None,
    #[default]
    Starttls,
    Tls,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    #[serde(default, skip_serializing)]
    pub password: Option<String>,
    pub encryption: SmtpEncryption,
    pub from_address: String,
    pub from_name: Option<String>,
}

impl SmtpConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.host.trim().is_empty()
            || self.host.len() > 255
            || self.host.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err("Hôte SMTP invalide");
        }
        if self.port == 0 {
            return Err("Port SMTP invalide");
        }
        if self.from_address.trim().is_empty()
            || self.from_address.len() > 254
            || !self.from_address.contains('@')
            || self.from_address.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err("Adresse d'expéditeur SMTP invalide");
        }
        if let Some(ref name) = self.from_name
            && (name.len() > 100 || name.chars().any(char::is_control))
        {
            return Err("Nom d'expéditeur SMTP invalide");
        }
        if let Some(ref user) = self.username
            && (user.len() > 255 || user.chars().any(char::is_control))
        {
            return Err("Identifiant SMTP invalide");
        }
        if let Some(ref pass) = self.password
            && pass.len() > 255
        {
            return Err("Mot de passe SMTP trop long");
        }
        Ok(())
    }
}

pub struct SmtpTransport {
    pub config: SmtpConfig,
}

impl SmtpTransport {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Transport for SmtpTransport {
    async fn deliver(
        &self,
        message: Message,
        idempotency_key: Uuid,
    ) -> Result<Receipt, DeliveryError> {
        if self.config.validate().is_err() {
            return Err(DeliveryError {
                code: "mail_config_invalid",
                retryable: false,
            });
        }
        if message.recipient.chars().any(|c| c.is_control() || c.is_whitespace())
            || message.subject.chars().any(char::is_control)
        {
            return Err(DeliveryError {
                code: "header_injection_rejected",
                retryable: false,
            });
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let stream = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(_)) => {
                return Err(DeliveryError {
                    code: "smtp_connection_failed",
                    retryable: true,
                });
            }
            Err(_) => {
                return Err(DeliveryError {
                    code: "smtp_timeout",
                    retryable: true,
                });
            }
        };

        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
            Ok(Ok(n)) if n > 0 && line.starts_with("220") => {}
            _ => {
                return Err(DeliveryError {
                    code: "smtp_handshake_failed",
                    retryable: true,
                });
            }
        }

        if writer.write_all(b"EHLO rustforge.local\r\n").await.is_err() || writer.flush().await.is_err() {
            return Err(DeliveryError {
                code: "smtp_connection_lost",
                retryable: true,
            });
        }

        line.clear();
        loop {
            match tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
                Ok(Ok(n)) if n > 0 => {
                    if line.starts_with("250 ") {
                        break;
                    } else if !line.starts_with("250-") {
                        let code = if line.starts_with('4') {
                            "smtp_transient_error"
                        } else {
                            "smtp_permanent_error"
                        };
                        return Err(DeliveryError {
                            code,
                            retryable: code == "smtp_transient_error",
                        });
                    }
                    line.clear();
                }
                _ => {
                    return Err(DeliveryError {
                        code: "smtp_handshake_failed",
                        retryable: true,
                    });
                }
            }
        }

        let mail_from = format!("MAIL FROM:<{}>\r\n", self.config.from_address);
        if writer.write_all(mail_from.as_bytes()).await.is_err() || writer.flush().await.is_err() {
            return Err(DeliveryError {
                code: "smtp_connection_lost",
                retryable: true,
            });
        }
        line.clear();
        if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
            if n == 0 || !line.starts_with("250") {
                let retryable = line.starts_with('4');
                return Err(DeliveryError {
                    code: if retryable { "smtp_transient_error" } else { "smtp_permanent_error" },
                    retryable,
                });
            }
        } else {
            return Err(DeliveryError { code: "smtp_timeout", retryable: true });
        }

        let rcpt_to = format!("RCPT TO:<{}>\r\n", message.recipient);
        if writer.write_all(rcpt_to.as_bytes()).await.is_err() || writer.flush().await.is_err() {
            return Err(DeliveryError {
                code: "smtp_connection_lost",
                retryable: true,
            });
        }
        line.clear();
        if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
            if n == 0 || !line.starts_with("250") {
                let retryable = line.starts_with('4');
                return Err(DeliveryError {
                    code: if retryable { "smtp_transient_error" } else { "smtp_permanent_error" },
                    retryable,
                });
            }
        } else {
            return Err(DeliveryError { code: "smtp_timeout", retryable: true });
        }

        if writer.write_all(b"DATA\r\n").await.is_err() || writer.flush().await.is_err() {
            return Err(DeliveryError { code: "smtp_connection_lost", retryable: true });
        }
        line.clear();
        if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(5), reader.read_line(&mut line)).await {
            if n == 0 || !line.starts_with("354") {
                let retryable = line.starts_with('4');
                return Err(DeliveryError {
                    code: if retryable { "smtp_transient_error" } else { "smtp_permanent_error" },
                    retryable,
                });
            }
        } else {
            return Err(DeliveryError { code: "smtp_timeout", retryable: true });
        }

        let from_header = match &self.config.from_name {
            Some(name) => format!("From: \"{name}\" <{}>\r\n", self.config.from_address),
            None => format!("From: <{}>\r\n", self.config.from_address),
        };
        let to_header = format!("To: <{}>\r\n", message.recipient);
        let subject_header = format!("Subject: {}\r\n", message.subject);
        let id_header = format!("Message-ID: <{idempotency_key}@rustforge.local>\r\n");
        let mut payload = format!("{from_header}{to_header}{subject_header}{id_header}Content-Type: text/plain; charset=utf-8\r\n\r\n");

        for line_str in message.text.lines() {
            if line_str.starts_with('.') {
                payload.push('.');
            }
            payload.push_str(line_str);
            payload.push_str("\r\n");
        }
        payload.push_str(".\r\n");

        if writer.write_all(payload.as_bytes()).await.is_err() || writer.flush().await.is_err() {
            return Err(DeliveryError { code: "smtp_connection_lost", retryable: true });
        }
        line.clear();
        if let Ok(Ok(n)) = tokio::time::timeout(std::time::Duration::from_secs(10), reader.read_line(&mut line)).await {
            if n == 0 || !line.starts_with("250") {
                let retryable = line.starts_with('4');
                return Err(DeliveryError {
                    code: if retryable { "smtp_transient_error" } else { "smtp_permanent_error" },
                    retryable,
                });
            }
        } else {
            return Err(DeliveryError { code: "smtp_timeout", retryable: true });
        }

        let _ = writer.write_all(b"QUIT\r\n").await;
        let _ = writer.flush().await;

        Ok(Receipt {
            transport: "smtp",
            receipt_id: format!("smtp-{idempotency_key}"),
            capture: None,
        })
    }
}

pub async fn load_transport(db: &PgPool) -> (Box<dyn Transport>, &'static str) {
    let mode: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.mode'")
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.trim_matches('"').to_string());

    match mode.as_deref() {
        Some("simulated") => {
            let outcome: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.simulated_outcome'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string());
            match outcome.as_deref() {
                Some("transient_error") => (
                    Box::new(SimulatedTransport::new(SimulatedOutcome::TransientError)),
                    "simulated_transient_error",
                ),
                Some("permanent_error") => (
                    Box::new(SimulatedTransport::new(SimulatedOutcome::PermanentError)),
                    "simulated_permanent_error",
                ),
                _ => (
                    Box::new(SimulatedTransport::new(SimulatedOutcome::Success)),
                    "simulated_success",
                ),
            }
        }
        Some("smtp") => {
            let host: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.smtp_host'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string());
            let from_addr: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.from_address'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string());

            let Some(host) = host else {
                return (Box::new(UnconfiguredTransport::new("mail_not_configured")), "smtp_unconfigured");
            };
            let Some(from_address) = from_addr else {
                return (Box::new(UnconfiguredTransport::new("mail_not_configured")), "smtp_unconfigured");
            };
            if host.trim().is_empty() || from_address.trim().is_empty() {
                return (Box::new(UnconfiguredTransport::new("mail_not_configured")), "smtp_unconfigured");
            }

            let port: u16 = sqlx::query_scalar::<_, i32>("SELECT (value->>0)::int FROM settings WHERE key='mail.smtp_port'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .and_then(|p: i32| u16::try_from(p).ok())
                .unwrap_or(587);

            let from_name: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.from_name'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string())
                .filter(|s| !s.is_empty());

            let username: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.smtp_username'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string())
                .filter(|s| !s.is_empty());

            let password: Option<String> = sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.smtp_password'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .map(|s| s.trim_matches('"').to_string())
                .filter(|s| !s.is_empty());

            let encryption = match sqlx::query_scalar::<_, String>("SELECT value::text FROM settings WHERE key='mail.smtp_encryption'")
                .fetch_optional(db)
                .await
                .ok()
                .flatten()
                .as_deref()
            {
                Some("\"tls\"") | Some("tls") => SmtpEncryption::Tls,
                Some("\"none\"") | Some("none") => SmtpEncryption::None,
                _ => SmtpEncryption::Starttls,
            };

            let config = SmtpConfig {
                host,
                port,
                username,
                password,
                encryption,
                from_address,
                from_name,
            };

            if config.validate().is_err() {
                (Box::new(UnconfiguredTransport::new("mail_config_invalid")), "smtp_unconfigured")
            } else {
                (Box::new(SmtpTransport::new(config)), "smtp")
            }
        }
        _ => (Box::new(CaptureTransport), "capture"),
    }
}

pub async fn enqueue_in(
    connection: &mut PgConnection,
    request: &MailRequest,
    key: &str,
) -> CoreResult<jobs::Job> {
    request.validate()?;
    let spec = jobs::NewJob {
        kind: KIND.into(),
        payload_version: 1,
        payload: serde_json::to_value(request).map_err(|e| CoreError::Internal(e.into()))?,
        dedup_key: key.into(),
        max_attempts: 5,
        priority: 0,
        available_at: chrono::Utc::now(),
    };
    jobs::enqueue_in(connection, &spec).await
}

/// Un lot explicite, sans boucle cachée. Le dispatch réseau reste hors de la transaction de claim.
pub async fn run_batch(db: &PgPool, transport: &dyn Transport, limit: usize) -> CoreResult<usize> {
    if !(1..=100).contains(&limit) {
        return Err(CoreError::validation("Lot mail limité à 1..100 jobs"));
    }
    let mut handled = 0;
    for _ in 0..limit {
        let Some(job) = jobs::claim(
            db,
            &[KIND.into(), crate::comment_notifications::KIND.into()],
            60,
        )
        .await?
        else {
            break;
        };
        let token = job
            .lease_token
            .ok_or_else(|| CoreError::Conflict("Job sans bail".into()))?;
        if job.payload_version != 1 {
            jobs::fail(db, job.id, token, "unsupported_payload_version", false).await?;
            handled += 1;
            continue;
        }
        let request: MailRequest = if job.kind == crate::comment_notifications::KIND {
            let event = match serde_json::from_value(job.payload.clone()) {
                Ok(event) => event,
                Err(_) => {
                    jobs::fail(db, job.id, token, "invalid_notification_payload", false).await?;
                    handled += 1;
                    continue;
                }
            };
            match crate::comment_notifications::resolve(db, &event).await {
                Ok(Some(request)) => request,
                Ok(None) => {
                    jobs::skip(db, job.id, token, "notification_obsolete").await?;
                    handled += 1;
                    continue;
                }
                Err(CoreError::Validation(_)) => {
                    jobs::fail(db, job.id, token, "notification_render_invalid", false).await?;
                    handled += 1;
                    continue;
                }
                Err(_) => {
                    jobs::fail(db, job.id, token, "notification_lookup_failed", true).await?;
                    handled += 1;
                    continue;
                }
            }
        } else {
            match serde_json::from_value(job.payload.clone()) {
                Ok(request) => request,
                Err(_) => {
                    jobs::fail(db, job.id, token, "invalid_mail_payload", false).await?;
                    handled += 1;
                    continue;
                }
            }
        };
        if request.validate().is_err() {
            jobs::fail(db, job.id, token, "invalid_mail_payload", false).await?;
            handled += 1;
            continue;
        }
        let user = match users::get(db, request.recipient_id).await {
            Ok(user) => user,
            Err(CoreError::NotFound) => {
                jobs::fail(db, job.id, token, "recipient_missing", false).await?;
                handled += 1;
                continue;
            }
            Err(_) => {
                jobs::fail(db, job.id, token, "recipient_lookup_failed", true).await?;
                handled += 1;
                continue;
            }
        };
        if user.email.len() > 254
            || user
                .email
                .chars()
                .any(|c| c.is_control() || c.is_whitespace())
            || !user.email.contains('@')
        {
            jobs::fail(db, job.id, token, "invalid_recipient", false).await?;
            handled += 1;
            continue;
        }
        let message = Message {
            recipient: user.email,
            subject: request.subject,
            text: request.text,
        };
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            transport.deliver(message, job.id),
        )
        .await
        {
            Ok(Ok(receipt)) => {
                let output =
                    serde_json::to_value(receipt).map_err(|e| CoreError::Internal(e.into()))?;
                jobs::complete_with_output(db, job.id, token, &output).await?;
            }
            Ok(Err(error)) => jobs::fail(db, job.id, token, error.code, error.retryable).await?,
            Err(_) => jobs::fail(db, job.id, token, "transport_timeout", true).await?,
        }
        handled += 1;
    }
    Ok(handled)
}

pub fn prune_captures_days_valid(days: i32) -> bool {
    (0..=3650).contains(&days)
}

/// Efface les captures de courriels des jobs terminés (`succeeded`, `failed`, `cancelled`) plus anciens que `days` jours.
/// Pour `days == 0`, efface immédiatement les captures de tous les jobs terminés.
/// Préserve l'entrée rf_jobs, le reçu et les compteurs de livraison tout en anonymisant et purgeant
/// le corps, destinataire et sujet enregistrés conformément aux règles RGPD et de rétention.
pub async fn prune_captures(db: &PgPool, days: i32) -> CoreResult<u64> {
    if !prune_captures_days_valid(days) {
        return Err(CoreError::validation(
            "Période de rétention de capture invalide (0 à 3650 jours)",
        ));
    }
    let rows = if days == 0 {
        sqlx::query(
            "WITH targets AS (
                SELECT id FROM rf_jobs
                WHERE output->>'transport' = 'capture'
                  AND jsonb_typeof(output->'capture') = 'object'
                  AND state IN ('succeeded', 'failed', 'cancelled')
                LIMIT 100 FOR UPDATE SKIP LOCKED
            ) UPDATE rf_jobs j
              SET output = jsonb_set(output, '{capture}', 'null'::jsonb), updated_at = now()
              FROM targets t WHERE j.id = t.id",
        )
        .execute(db)
        .await?
        .rows_affected()
    } else {
        sqlx::query(
            "WITH targets AS (
                SELECT id FROM rf_jobs
                WHERE output->>'transport' = 'capture'
                  AND jsonb_typeof(output->'capture') = 'object'
                  AND state IN ('succeeded', 'failed', 'cancelled')
                  AND finished_at < now() - ($1::int * interval '1 day')
                LIMIT 100 FOR UPDATE SKIP LOCKED
            ) UPDATE rf_jobs j
              SET output = jsonb_set(output, '{capture}', 'null'::jsonb), updated_at = now()
              FROM targets t WHERE j.id = t.id",
        )
        .bind(days)
        .execute(db)
        .await?
        .rows_affected()
    };
    Ok(rows)
}

/// Supprime ou purge les captures de notification rattachées à un commentaire spécifique (ex: lors de sa suppression).
pub async fn purge_captures_for_comment(db: &PgPool, comment_id: Uuid) -> CoreResult<u64> {
    let rows = sqlx::query(
        "UPDATE rf_jobs
         SET output = jsonb_set(output, '{capture}', 'null'::jsonb), updated_at = now()
         WHERE kind = $1
           AND payload->>'comment_id' = $2
           AND output->>'transport' = 'capture'
           AND jsonb_typeof(output->'capture') = 'object'",
    )
    .bind(crate::comment_notifications::KIND)
    .bind(comment_id.to_string())
    .execute(db)
    .await?
    .rows_affected();
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_header_injection_and_large_messages() {
        let mut request = MailRequest {
            recipient_id: Uuid::nil(),
            subject: "Titre".into(),
            text: "Texte".into(),
        };
        assert!(request.validate().is_ok());
        request.subject = "Titre\r\nBcc: attacker@example.test".into();
        assert!(request.validate().is_err());
        request.subject = "Titre".into();
        request.text = "x".repeat(33000);
        assert!(request.validate().is_err());
    }

    #[tokio::test]
    async fn simulated_transport_outcomes() {
        let key = Uuid::new_v4();
        let msg = Message {
            recipient: "user@example.test".into(),
            subject: "Test".into(),
            text: "Corps".into(),
        };

        // Succès
        let success_transport = SimulatedTransport::new(SimulatedOutcome::Success);
        let receipt = success_transport.deliver(msg.clone(), key).await.unwrap();
        assert_eq!(receipt.transport, "simulated");
        assert_eq!(receipt.receipt_id, format!("sim-{key}"));
        assert!(receipt.capture.is_none());

        // Erreur temporaire (rejouable)
        let transient_transport =
            SimulatedTransport::new(SimulatedOutcome::TransientError);
        let err = transient_transport.deliver(msg.clone(), key).await.unwrap_err();
        assert_eq!(err.code, "simulated_transient_error");
        assert!(err.retryable);

        // Erreur permanente (non rejouable)
        let permanent_transport =
            SimulatedTransport::new(SimulatedOutcome::PermanentError);
        let err = permanent_transport.deliver(msg, key).await.unwrap_err();
        assert_eq!(err.code, "simulated_permanent_error");
        assert!(!err.retryable);
    }

    #[tokio::test]
    async fn unconfigured_transport_returns_explicit_failure() {
        let unconfigured = UnconfiguredTransport::new("mail_not_configured");
        let msg = Message {
            recipient: "admin@example.test".into(),
            subject: "Important".into(),
            text: "Contenu".into(),
        };
        let err = unconfigured.deliver(msg, Uuid::new_v4()).await.unwrap_err();
        assert_eq!(err.code, "mail_not_configured");
        assert!(!err.retryable);
    }

    #[test]
    fn smtp_config_validation() {
        let valid = SmtpConfig {
            host: "smtp.example.test".into(),
            port: 587,
            username: Some("user".into()),
            password: Some("secret123".into()),
            encryption: SmtpEncryption::Starttls,
            from_address: "notifications@example.test".into(),
            from_name: Some("RustForge CMS".into()),
        };
        assert!(valid.validate().is_ok());

        // Rejet hôte invalide ou vide
        let mut bad = valid.clone();
        bad.host = "   ".into();
        assert!(bad.validate().is_err());

        // Rejet port 0
        let mut bad_port = valid.clone();
        bad_port.port = 0;
        assert!(bad_port.validate().is_err());

        // Rejet injection CRLF dans from_address
        let mut bad_addr = valid.clone();
        bad_addr.from_address = "admin@example.test\r\nBcc: evil@example.test".into();
        assert!(bad_addr.validate().is_err());

        // Rejet adresse sans @
        let mut no_at = valid.clone();
        no_at.from_address = "invalid-address".into();
        assert!(no_at.validate().is_err());

        // Rejet CRLF dans from_name
        let mut bad_name = valid.clone();
        bad_name.from_name = Some("Nom\nInvalide".into());
        assert!(bad_name.validate().is_err());
    }

    #[tokio::test]
    async fn smtp_transport_rejects_header_injection_in_recipient() {
        let config = SmtpConfig {
            host: "127.0.0.1".into(),
            port: 2525,
            username: None,
            password: None,
            encryption: SmtpEncryption::None,
            from_address: "noreply@example.test".into(),
            from_name: None,
        };
        let transport = SmtpTransport::new(config);
        let injected = Message {
            recipient: "target@example.test\r\nRCPT TO:<attacker@example.test>".into(),
            subject: "Sujet".into(),
            text: "Texte".into(),
        };
        let err = transport.deliver(injected, Uuid::new_v4()).await.unwrap_err();
        assert_eq!(err.code, "header_injection_rejected");
        assert!(!err.retryable);
    }

    #[tokio::test]
    async fn at_least_once_idempotency_key_preservation() {
        let key = Uuid::new_v4();
        let msg = Message {
            recipient: "idempotent@example.test".into(),
            subject: "Idempotence test".into(),
            text: "Message".into(),
        };

        // CaptureTransport conserve la clé dans le receipt_id et capture
        let capture_transport = CaptureTransport;
        let receipt1 = capture_transport.deliver(msg.clone(), key).await.unwrap();
        let receipt2 = capture_transport.deliver(msg.clone(), key).await.unwrap();
        assert_eq!(receipt1.receipt_id, receipt2.receipt_id);
        assert_eq!(receipt1.receipt_id, key.to_string());

        // SimulatedTransport conserve la clé
        let sim_transport = SimulatedTransport::new(SimulatedOutcome::Success);
        let sim_receipt1 = sim_transport.deliver(msg.clone(), key).await.unwrap();
        let sim_receipt2 = sim_transport.deliver(msg.clone(), key).await.unwrap();
        assert_eq!(sim_receipt1.receipt_id, sim_receipt2.receipt_id);
        assert_eq!(sim_receipt1.receipt_id, format!("sim-{key}"));
    }

    #[test]
    fn prune_captures_validation() {
        assert!(prune_captures_days_valid(0));
        assert!(prune_captures_days_valid(30));
        assert!(prune_captures_days_valid(3650));
        assert!(!prune_captures_days_valid(-1));
        assert!(!prune_captures_days_valid(3651));
    }
}
