//! Notifications de commentaires : références versionnées, jamais un texte privé dupliqué en file.
use crate::{CoreError, CoreResult, jobs, mail, models::Comment};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

pub const KIND: &str = "rf.mail.comment";
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Notice {
    Pending,
    Approved,
}
impl Notice {
    fn status(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
        }
    }
    fn preference(self) -> &'static str {
        match self {
            Self::Pending => "discussion.moderation_notify",
            Self::Approved => "discussion.comments_notify",
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    pub comment_id: Uuid,
    pub recipient_id: Uuid,
    pub event_status: Notice,
    pub event_at: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
}

/// Doit partager la transaction qui vient de créer/modifier le commentaire.
pub async fn enqueue_in(
    connection: &mut PgConnection,
    comment: &Comment,
    actor_id: Option<Uuid>,
) -> CoreResult<usize> {
    let notice = match comment.status.as_str() {
        "pending" => Notice::Pending,
        "approved" => Notice::Approved,
        _ => return Ok(0),
    };
    let enabled: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key=$1")
            .bind(notice.preference())
            .fetch_optional(&mut *connection)
            .await?;
    if enabled.and_then(|v| v.as_bool()) != Some(true) {
        return Ok(0);
    }
    let recipients: Vec<Uuid> = match notice {
        Notice::Pending => {
            sqlx::query_scalar(
                "SELECT id FROM users WHERE role IN ('admin','editor') ORDER BY id LIMIT 101",
            )
            .fetch_all(&mut *connection)
            .await?
        }
        Notice::Approved => {
            sqlx::query_scalar(
                "SELECT author_id FROM entries WHERE id=$1 AND author_id IS NOT NULL",
            )
            .bind(comment.entry_id)
            .fetch_all(&mut *connection)
            .await?
        }
    };
    if recipients.len() > 100 {
        return Err(CoreError::validation(
            "Notification limitée à 100 destinataires par commentaire",
        ));
    }
    let mut count = 0;
    for recipient_id in recipients {
        if Some(recipient_id) == actor_id || Some(recipient_id) == comment.author_id {
            continue;
        }
        let event = Notification {
            comment_id: comment.id,
            recipient_id,
            event_status: notice,
            event_at: comment.updated_at,
            actor_id,
        };
        let spec = jobs::NewJob {
            kind: KIND.into(),
            payload_version: 1,
            payload: serde_json::to_value(event).map_err(|e| CoreError::Internal(e.into()))?,
            dedup_key: format!(
                "comment:{}:{}:{}:{}",
                comment.id,
                comment.updated_at.timestamp_micros(),
                notice.status(),
                recipient_id
            ),
            max_attempts: 5,
            priority: 0,
            available_at: Utc::now(),
        };
        jobs::enqueue_in(connection, &spec).await?;
        count += 1;
    }
    Ok(count)
}

/// None = événement devenu obsolète, à annuler ; erreur DB = réessayer sans envoyer.
pub async fn resolve(db: &PgPool, event: &Notification) -> CoreResult<Option<mail::MailRequest>> {
    let settings = crate::settings::all(db).await?;
    if settings
        .get(event.event_status.preference())
        .and_then(|v| v.as_bool())
        != Some(true)
    {
        return Ok(None);
    }
    let comment = match crate::comments::get_comment(db, event.comment_id).await {
        Ok(comment) => comment,
        Err(CoreError::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    };
    if comment.status != event.event_status.status()
        || comment.updated_at != event.event_at
        || comment.author_id == Some(event.recipient_id)
        || event.actor_id == Some(event.recipient_id)
    {
        return Ok(None);
    }
    let entry = match crate::entries::get_published_by_id(db, comment.entry_id).await {
        Ok(entry) => entry,
        Err(CoreError::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    };
    let recipient = match crate::users::get(db, event.recipient_id).await {
        Ok(user) => user,
        Err(CoreError::NotFound) => return Ok(None),
        Err(error) => return Err(error),
    };
    let allowed = match event.event_status {
        Notice::Pending => recipient.role.can_edit_others(),
        Notice::Approved => entry.author_id == Some(recipient.id) && recipient.role.can_write(),
    };
    if !allowed {
        return Ok(None);
    }
    let title = entry
        .title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect::<String>();
    let headline = match event.event_status {
        Notice::Pending => "Commentaire à modérer",
        Notice::Approved => "Commentaire publié",
    };
    let excerpt = comment.content.chars().take(4000).collect::<String>();
    let request = mail::MailRequest {
        recipient_id: event.recipient_id,
        subject: format!("{headline} : {title}"),
        text: format!(
            "{headline}\nPublication : {title}\nAuteur du commentaire : {}\n\n{excerpt}\n\nConsultez les commentaires dans le Studio du site.",
            comment.author_name
        ),
    };
    request.validate()?;
    Ok(Some(request))
}
