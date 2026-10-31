//! Local form notices reuse durable mail jobs, with references only and no copied answers.
use crate::{
    CoreError, CoreResult,
    forms::{Form, FormSubmission},
    jobs, mail, settings, users,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;
pub const KIND: &str = "rf.mail.form_capture";
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    pub submission_id: Uuid,
    pub form_id: Uuid,
    pub recipient_id: Uuid,
}
pub fn validate_settings(value: Option<&Value>) -> CoreResult<()> {
    let Some(notice) = value.and_then(|v| v.get("notifications")) else {
        return Ok(());
    };
    let Some(obj) = notice.as_object() else {
        return Err(CoreError::validation(
            "Notifications de formulaire invalides",
        ));
    };
    if obj.len() != 1 || obj.get("capture_owner").and_then(Value::as_bool).is_none() {
        return Err(CoreError::validation(
            "Option notifications attendue : capture_owner booléen",
        ));
    }
    Ok(())
}
pub fn enabled(form: &Form) -> bool {
    form.settings
        .pointer("/notifications/capture_owner")
        .and_then(Value::as_bool)
        == Some(true)
}
pub async fn enqueue_in(
    connection: &mut PgConnection,
    form: &Form,
    sub: &FormSubmission,
) -> CoreResult<()> {
    if sub.is_spam || !enabled(form) {
        return Ok(());
    }
    let Some(owner) = form.created_by else {
        return Ok(());
    };
    jobs::enqueue_in(connection,&jobs::NewJob{kind:KIND.into(),payload_version:1,payload:serde_json::json!({"submission_id":sub.id,"form_id":form.id,"recipient_id":owner}),dedup_key:format!("form.notice:{}:{owner}",sub.id),priority:0,max_attempts:5,available_at:chrono::Utc::now()}).await?;
    Ok(())
}
/// Recheck current ownership, module/configuration and trash at execution time.
pub async fn resolve(db: &PgPool, event: &Notification) -> CoreResult<Option<mail::MailRequest>> {
    if !settings::is_module_active(db, "forms").await {
        return Ok(None);
    }
    let form = match crate::forms::get_form(db, event.form_id).await {
        Ok(form) => form,
        Err(CoreError::NotFound) => return Ok(None),
        Err(e) => return Err(e),
    };
    if !form.is_active || !enabled(&form) || form.created_by != Some(event.recipient_id) {
        return Ok(None);
    }
    let eligible:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM form_submissions WHERE id=$1 AND form_id=$2 AND deleted_at IS NULL AND NOT is_spam)").bind(event.submission_id).bind(event.form_id).fetch_one(db).await?;
    if !eligible {
        return Ok(None);
    }
    let owner = match users::get(db, event.recipient_id).await {
        Ok(user) => user,
        Err(CoreError::NotFound) => return Ok(None),
        Err(e) => return Err(e),
    };
    if !owner.role.can_write() {
        return Ok(None);
    }
    Ok(Some(mail::MailRequest{recipient_id:owner.id,subject:format!("Nouvelle réponse — {}",form.name),text:"Une nouvelle réponse est disponible dans l’administration, rubrique Formulaires > Soumissions. Les réponses ne sont pas recopiées dans cette notification.".into()}))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_are_explicit_and_cannot_request_remote_delivery() {
        assert!(validate_settings(None).is_ok());
        assert!(validate_settings(Some(&serde_json::json!({"notify_email":"legacy@example.test","notifications":{"capture_owner":true}}))).is_ok());
        assert!(
            validate_settings(Some(
                &serde_json::json!({"notifications":{"capture_owner":true,"smtp":true}})
            ))
            .is_err()
        );
        assert!(
            validate_settings(Some(
                &serde_json::json!({"notifications":{"capture_owner":"true"}})
            ))
            .is_err()
        );
    }
}
