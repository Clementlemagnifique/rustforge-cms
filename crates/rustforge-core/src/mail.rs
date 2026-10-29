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
}
