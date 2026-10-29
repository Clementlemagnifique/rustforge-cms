//! Données exclusivement jetables. Ne jamais pointer ce test sur une base personnelle.
use chrono::{Duration, Utc};
use rustforge_core::{
    jobs::{self, NewJob},
    migration::native_backup,
    models::Role,
    users::{self, NewUser},
};
use serde_json::json;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::str::FromStr;
use uuid::Uuid;

fn spec(key: &str) -> NewJob {
    NewJob {
        kind: "rf.test".into(),
        payload_version: 1,
        payload: json!({"record":"test"}),
        dedup_key: key.into(),
        max_attempts: 3,
        priority: 0,
        available_at: Utc::now(),
    }
}
async fn database(server: &PgPool, base: &str) -> (String, PgPool) {
    let name = format!("rf_jobs_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(server)
        .await
        .unwrap();
    let options = PgConnectOptions::from_str(base).unwrap().database(&name);
    let db = PgPoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .unwrap();
    rustforge_core::migrate(&db).await.unwrap();
    (name, db)
}
async fn expire(db: &PgPool, id: Uuid) {
    sqlx::query("UPDATE rf_jobs SET lease_until=now()-interval '1 second' WHERE id=$1")
        .bind(id)
        .execute(db)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires RUSTFORGE_TEST_DATABASE_URL to a disposable PostgreSQL server"]
async fn durable_jobs_transactions_leases_and_restore() {
    let base =
        std::env::var("RUSTFORGE_TEST_DATABASE_URL").expect("disposable PostgreSQL URL required");
    let server = PgPoolOptions::new()
        .max_connections(2)
        .connect(&base)
        .await
        .unwrap();
    let (name, db) = database(&server, &base).await;
    let (target_name, target) = database(&server, &base).await;
    let kinds = vec!["rf.test".to_string()];

    let first = jobs::enqueue(&db, &spec("dedup:one")).await.unwrap();
    assert_eq!(
        jobs::enqueue(&db, &spec("dedup:one")).await.unwrap().id,
        first.id
    );
    let mut changed = spec("dedup:one");
    changed.payload = json!({"record":"different"});
    assert!(jobs::enqueue(&db, &changed).await.is_err());
    let race = spec("dedup:race");
    let (dedup_a, dedup_b) = tokio::join!(jobs::enqueue(&db, &race), jobs::enqueue(&db, &race));
    assert_eq!(dedup_a.unwrap().id, dedup_b.unwrap().id);

    let mut tx = db.begin().await.unwrap();
    jobs::enqueue_in(&mut tx, &spec("transaction:rollback"))
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    let rolled_back: i64 =
        sqlx::query_scalar("SELECT count(*) FROM rf_jobs WHERE dedup_key='transaction:rollback'")
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(rolled_back, 0);

    let (a, b) = tokio::join!(jobs::claim(&db, &kinds, 60), jobs::claim(&db, &kinds, 60));
    let a = a.unwrap().unwrap();
    let b = b.unwrap().unwrap();
    assert_ne!(a.id, b.id);
    jobs::heartbeat(&db, a.id, a.lease_token.unwrap(), 60)
        .await
        .unwrap();
    assert!(jobs::complete(&db, a.id, Uuid::new_v4()).await.is_err());
    jobs::complete(&db, a.id, a.lease_token.unwrap())
        .await
        .unwrap();
    assert!(
        jobs::complete(&db, a.id, a.lease_token.unwrap())
            .await
            .is_err()
    );
    expire(&db, b.id).await;
    let replacement = jobs::claim(&db, &kinds, 60).await.unwrap().unwrap();
    assert_eq!(replacement.id, b.id);
    assert_eq!(replacement.attempts, 2);
    assert_ne!(replacement.lease_token, b.lease_token);
    assert!(
        jobs::heartbeat(&db, b.id, b.lease_token.unwrap(), 60)
            .await
            .is_err()
    );
    assert!(
        jobs::complete(&db, b.id, b.lease_token.unwrap())
            .await
            .is_err()
    );
    jobs::fail(
        &db,
        replacement.id,
        replacement.lease_token.unwrap(),
        "transport_unavailable",
        true,
    )
    .await
    .unwrap();
    assert!(jobs::claim(&db, &kinds, 60).await.unwrap().is_none());
    sqlx::query("UPDATE rf_jobs SET available_at=now() WHERE id=$1")
        .bind(b.id)
        .execute(&db)
        .await
        .unwrap();
    let last = jobs::claim(&db, &kinds, 60).await.unwrap().unwrap();
    assert_eq!(last.attempts, 3);
    expire(&db, last.id).await;
    assert!(jobs::claim(&db, &kinds, 60).await.unwrap().is_none());
    let state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(last.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(state, "failed");
    jobs::retry(&db, last.id).await.unwrap();
    let retried = jobs::claim(&db, &kinds, 60).await.unwrap().unwrap();
    assert_eq!(retried.attempts, 1);
    jobs::cancel(&db, retried.id).await.unwrap();
    assert!(
        jobs::complete(&db, retried.id, retried.lease_token.unwrap())
            .await
            .is_err()
    );

    let mut delayed = spec("future:job");
    delayed.available_at = Utc::now() + Duration::days(1);
    jobs::enqueue(&db, &delayed).await.unwrap();
    let mut unsupported = spec("version:unknown");
    unsupported.kind = "rf.unknown".into();
    unsupported.payload_version = 9;
    jobs::enqueue(&db, &unsupported).await.unwrap();
    assert!(jobs::claim(&db, &kinds, 60).await.unwrap().is_none());
    assert!(jobs::list(&db, 101).await.is_err());
    let mut failed_spec = spec("permanent:failure");
    failed_spec.kind = "rf.permanent".into();
    jobs::enqueue(&db, &failed_spec).await.unwrap();
    let failed = jobs::claim(&db, &["rf.permanent".into()], 60)
        .await
        .unwrap()
        .unwrap();
    jobs::fail(
        &db,
        failed.id,
        failed.lease_token.unwrap(),
        "unsupported_version",
        false,
    )
    .await
    .unwrap();
    let failed_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(failed.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(failed_state, "failed");
    sqlx::query("UPDATE rf_jobs SET finished_at=now()-interval '40 days' WHERE id=$1")
        .bind(failed.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(jobs::prune(&db, 30).await.unwrap(), 1);

    // Un pool recréé reprend les données persistées, aucun état dépend d'une mémoire de processus.
    jobs::enqueue(&db, &spec("restart:job")).await.unwrap();
    let running = jobs::claim(&db, &kinds, 60).await.unwrap().unwrap();
    let second_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(PgConnectOptions::from_str(&base).unwrap().database(&name))
        .await
        .unwrap();
    expire(&second_pool, running.id).await;
    let recovered = jobs::claim(&second_pool, &kinds, 60)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(recovered.id, running.id);
    assert_ne!(recovered.lease_token, running.lease_token);
    second_pool.close().await;

    let admin = users::create(
        &db,
        NewUser {
            email: "jobs@example.test".into(),
            username: "jobs_admin".into(),
            display_name: None,
            password: "test-password-long-123!".into(),
            role: Role::Admin,
        },
    )
    .await
    .unwrap();
    // Transport de capture : un résultat privé persisté, aucun appel réseau.
    use rustforge_core::mail::{self, CaptureTransport, MailRequest};
    let message = MailRequest {
        recipient_id: admin.id,
        subject: "Notification de recette".into(),
        text: "Message local uniquement".into(),
    };
    let mut outbox = db.begin().await.unwrap();
    let queued = mail::enqueue_in(&mut outbox, &message, "mail:capture")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    assert_eq!(
        mail::run_batch(&db, &CaptureTransport, 10).await.unwrap(),
        1
    );
    assert_eq!(
        mail::run_batch(&db, &CaptureTransport, 10).await.unwrap(),
        0
    );
    let captured = jobs::list(&db, 100)
        .await
        .unwrap()
        .into_iter()
        .find(|j| j.id == queued.id)
        .unwrap();
    assert_eq!(captured.state, "succeeded");
    let output = captured.output.unwrap();
    assert_eq!(output["transport"], "capture");
    assert_eq!(output["capture"]["recipient"], "jobs@example.test");
    assert_eq!(output["capture"]["text"], "Message local uniquement");
    struct RetryTransport;
    #[async_trait::async_trait]
    impl mail::Transport for RetryTransport {
        async fn deliver(
            &self,
            _message: mail::Message,
            _key: Uuid,
        ) -> Result<mail::Receipt, mail::DeliveryError> {
            Err(mail::DeliveryError {
                code: "fixture_unavailable",
                retryable: true,
            })
        }
    }
    let mut outbox = db.begin().await.unwrap();
    let retry_mail = mail::enqueue_in(&mut outbox, &message, "mail:retry")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    assert_eq!(mail::run_batch(&db, &RetryTransport, 1).await.unwrap(), 1);
    let retry_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(retry_mail.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(retry_state, "retry");
    sqlx::query("UPDATE rf_jobs SET available_at=now() WHERE id=$1")
        .bind(retry_mail.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(mail::run_batch(&db, &CaptureTransport, 1).await.unwrap(), 1);
    let mut unknown = spec("mail:unknown_version");
    unknown.kind = mail::KIND.into();
    unknown.payload_version = 2;
    let unknown = jobs::enqueue(&db, &unknown).await.unwrap();
    assert_eq!(mail::run_batch(&db, &CaptureTransport, 1).await.unwrap(), 1);
    let code: String = sqlx::query_scalar("SELECT error_code FROM rf_jobs WHERE id=$1")
        .bind(unknown.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(code, "unsupported_payload_version");

    // Transport simulé : qualification en panne transitoire, permanente et succès (M1-C)
    let mut outbox = db.begin().await.unwrap();
    let sim_transient_job = mail::enqueue_in(&mut outbox, &message, "mail:sim_transient")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    let transient_transport = mail::SimulatedTransport::new(mail::SimulatedOutcome::TransientError);
    assert_eq!(mail::run_batch(&db, &transient_transport, 1).await.unwrap(), 1);
    let transient_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(sim_transient_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(transient_state, "retry");

    let mut outbox = db.begin().await.unwrap();
    let sim_perm_job = mail::enqueue_in(&mut outbox, &message, "mail:sim_permanent")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    let permanent_transport = mail::SimulatedTransport::new(mail::SimulatedOutcome::PermanentError);
    assert_eq!(mail::run_batch(&db, &permanent_transport, 1).await.unwrap(), 1);
    let perm_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(sim_perm_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(perm_state, "failed");
    let perm_code: String = sqlx::query_scalar("SELECT error_code FROM rf_jobs WHERE id=$1")
        .bind(sim_perm_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(perm_code, "simulated_permanent_error");

    // Transport non configuré : rejet explicite non rejouable
    let mut outbox = db.begin().await.unwrap();
    let unconf_job = mail::enqueue_in(&mut outbox, &message, "mail:unconfigured")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    let unconf_transport = mail::UnconfiguredTransport::new("mail_not_configured");
    assert_eq!(mail::run_batch(&db, &unconf_transport, 1).await.unwrap(), 1);
    let unconf_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(unconf_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(unconf_state, "failed");
    let unconf_code: String = sqlx::query_scalar("SELECT error_code FROM rf_jobs WHERE id=$1")
        .bind(unconf_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(unconf_code, "mail_not_configured");

    // Résolution dynamique du transport selon les réglages
    let (t_default, mode_default) = mail::load_transport(&db).await;
    assert_eq!(mode_default, "capture_only");
    let _ = t_default;

    rustforge_core::settings::update(&db, serde_json::from_value(json!({
        "mail.mode": "simulated",
        "mail.simulated_outcome": "success",
    })).unwrap()).await.unwrap();
    let (t_sim, mode_sim) = mail::load_transport(&db).await;
    assert_eq!(mode_sim, "simulated_success");
    let mut outbox = db.begin().await.unwrap();
    let sim_ok_job = mail::enqueue_in(&mut outbox, &message, "mail:sim_ok")
        .await
        .unwrap();
    outbox.commit().await.unwrap();
    assert_eq!(mail::run_batch(&db, t_sim.as_ref(), 1).await.unwrap(), 1);
    let ok_state: String = sqlx::query_scalar("SELECT state FROM rf_jobs WHERE id=$1")
        .bind(sim_ok_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(ok_state, "succeeded");
    let ok_output: serde_json::Value = sqlx::query_scalar("SELECT output FROM rf_jobs WHERE id=$1")
        .bind(sim_ok_job.id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(ok_output["transport"], "simulated");
    assert!(ok_output.get("capture").is_none());

    // Protection des secrets : mot de passe SMTP exclu des sauvegardes et effacé à la restauration
    rustforge_core::settings::update(&db, serde_json::from_value(json!({
        "mail.mode": "smtp",
        "mail.smtp_host": "smtp.example.test",
        "mail.smtp_port": 587,
        "mail.from_address": "robot@example.test",
        "mail.smtp_username": "robot_user",
        "mail.smtp_password": "super_secret_smtp_password_never_leak",
    })).unwrap()).await.unwrap();

    let snapshot = native_backup::dump(&db).await.unwrap();
    assert_eq!(snapshot.format, "rustforge-native-3");
    let dumped_str = serde_json::to_string(&snapshot).unwrap();
    assert!(!dumped_str.contains("super_secret_smtp_password_never_leak"));
    // Ancien schéma sans jobs : upgrade explicitement connu, aucune table métier ignorée.
    let mut legacy: native_backup::NativeSnapshot =
        serde_json::from_value(serde_json::to_value(&snapshot).unwrap()).unwrap();
    legacy.format = "rustforge-native-2".into();
    legacy.tables.remove("rf_jobs");
    legacy.migrations.retain(|v| *v != 20261029000007);
    let mut restore = target.begin().await.unwrap();
    native_backup::restore(&mut restore, &legacy).await.unwrap();
    let old_count: i64 = sqlx::query_scalar("SELECT count(*) FROM rf_jobs")
        .fetch_one(&mut *restore)
        .await
        .unwrap();
    assert_eq!(old_count, 0);
    restore.rollback().await.unwrap();
    let mut restore = target.begin().await.unwrap();
    native_backup::restore(&mut restore, &snapshot)
        .await
        .unwrap();
    restore.commit().await.unwrap();
    let target_pw: Option<String> = sqlx::query_scalar("SELECT value::text FROM settings WHERE key='mail.smtp_password'")
        .fetch_optional(&target)
        .await
        .unwrap();
    assert!(target_pw.is_none());
    assert!(jobs::claim(&target, &kinds, 60).await.unwrap().is_none());
    let paused = jobs::list(&target, 100).await.unwrap();
    assert!(paused.iter().any(|j| j.state == "paused_restore"));
    assert!(paused.iter().all(|j| j.lease_token.is_none()));
    let restored_capture = paused.iter().find(|j| j.id == queued.id).unwrap();
    assert_eq!(restored_capture.output.as_ref(), Some(&output));
    let pending = paused
        .iter()
        .find(|j| j.dedup_key == "restart:job")
        .unwrap();
    assert_eq!(pending.state, "paused_restore");
    jobs::resume_restored(&target, pending.id).await.unwrap();
    let resumed = jobs::claim(&target, &kinds, 60).await.unwrap().unwrap();
    assert_eq!(resumed.id, pending.id);
    jobs::complete(&target, resumed.id, resumed.lease_token.unwrap())
        .await
        .unwrap();

    db.close().await;
    target.close().await;
    for database in [name, target_name] {
        assert!(database.starts_with("rf_jobs_test_"));
        sqlx::query(&format!("DROP DATABASE {database} WITH (FORCE)"))
            .execute(&server)
            .await
            .unwrap();
    }
    server.close().await;
}

async fn notification_user(db: &PgPool, name: &str, role: Role) -> rustforge_core::models::User {
    users::create(
        db,
        NewUser {
            username: name.into(),
            email: format!("{name}@example.test"),
            display_name: None,
            password: "test-password-long-123!".into(),
            role,
        },
    )
    .await
    .unwrap()
}
async fn notification_settings(db: &PgPool, values: serde_json::Value) {
    rustforge_core::settings::update(db, serde_json::from_value(values).unwrap())
        .await
        .unwrap();
}
async fn notices(db: &PgPool, id: Uuid) -> Vec<jobs::Job> {
    sqlx::query_as(
        "SELECT * FROM rf_jobs WHERE kind=$1 AND payload->>'comment_id'=$2 ORDER BY created_at,id",
    )
    .bind(rustforge_core::comment_notifications::KIND)
    .bind(id.to_string())
    .fetch_all(db)
    .await
    .unwrap()
}
async fn guest(db: &PgPool, entry: Uuid) -> rustforge_core::models::Comment {
    rustforge_core::comments::create_comment(db,serde_json::from_value(json!({
        "entry_id":entry,"author_name":"Visiteur","author_email":"visitor@example.test","content":"Texte confidentiel en attente"
    })).unwrap()).await.unwrap()
}
#[tokio::test]
#[ignore = "requires disposable PostgreSQL"]
async fn comment_notification_outbox_atomicity_and_dispatch_policy() {
    use rustforge_core::{
        NoopHooks, comment_notifications, comments, entries, mail, models::CommentStatus,
    };
    let base =
        std::env::var("RUSTFORGE_TEST_DATABASE_URL").expect("disposable PostgreSQL URL required");
    let server = PgPoolOptions::new()
        .max_connections(2)
        .connect(&base)
        .await
        .unwrap();
    let (name, db) = database(&server, &base).await;
    let admin = notification_user(&db, "notice_admin", Role::Admin).await;
    let editor = notification_user(&db, "notice_editor", Role::Editor).await;
    let author = notification_user(&db, "notice_author", Role::Author).await;
    let entry = entries::create(
        &db,
        &NoopHooks,
        "post",
        author.id,
        serde_json::from_value(json!({
            "title":"Publication de recette","body":"Texte public","status":"published"
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    notification_settings(
        &db,
        json!({"discussion.moderation_notify":true,"discussion.comments_notify":true,
        "discussion.comment_moderation":true}),
    )
    .await;
    let pending = guest(&db, entry.id).await;
    let initial = notices(&db, pending.id).await;
    assert_eq!(initial.len(), 2);
    for job in &initial {
        let payload = job.payload.to_string();
        assert!(!payload.contains("visitor@example.test"));
        assert!(!payload.contains("Texte confidentiel"));
        assert!(job.output.is_none());
    }
    let mut duplicate = db.begin().await.unwrap();
    comment_notifications::enqueue_in(&mut duplicate, &pending, None)
        .await
        .unwrap();
    duplicate.commit().await.unwrap();
    assert_eq!(notices(&db, pending.id).await.len(), 2);
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    let captured = notices(&db, pending.id).await;
    assert!(captured.iter().all(|job| job.state == "succeeded"));
    let recipients: Vec<_> = captured
        .iter()
        .map(|job| {
            job.output.as_ref().unwrap()["capture"]["recipient"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert!(recipients.contains(&admin.email.as_str()));
    assert!(recipients.contains(&editor.email.as_str()));
    assert!(!recipients.contains(&author.email.as_str()));
    let approved = comments::update_comment_status_as(
        &db,
        pending.id,
        CommentStatus::Approved,
        Some(editor.id),
    )
    .await
    .unwrap();
    let repeat = comments::update_comment_status_as(
        &db,
        pending.id,
        CommentStatus::Approved,
        Some(editor.id),
    )
    .await
    .unwrap();
    assert_eq!(approved.updated_at, repeat.updated_at);
    assert_eq!(notices(&db, pending.id).await.len(), 3);
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        1
    );
    assert!(notices(&db, pending.id).await.iter().any(|job| {
        job.output
            .as_ref()
            .is_some_and(|v| v["capture"]["recipient"] == author.email)
    }));

    let cancelled = guest(&db, entry.id).await;
    notification_settings(&db, json!({"discussion.moderation_notify":false})).await;
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    assert!(
        notices(&db, cancelled.id)
            .await
            .iter()
            .all(|j| j.state == "cancelled" && j.output.is_none())
    );
    notification_settings(&db, json!({"discussion.moderation_notify":true})).await;
    let role_change = guest(&db, entry.id).await;
    sqlx::query("UPDATE users SET role='subscriber' WHERE id=$1")
        .bind(editor.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    let results = notices(&db, role_change.id).await;
    assert_eq!(results.iter().filter(|j| j.state == "succeeded").count(), 1);
    assert!(
        results
            .iter()
            .any(|j| j.payload["recipient_id"] == editor.id.to_string() && j.state == "cancelled")
    );
    sqlx::query("UPDATE users SET role='editor' WHERE id=$1")
        .bind(editor.id)
        .execute(&db)
        .await
        .unwrap();

    let changed = guest(&db, entry.id).await;
    let updated = comments::update_comment_content_as(
        &db,
        changed.id,
        "Texte révisé",
        None,
        None,
        None,
        Some(editor.id),
    )
    .await
    .unwrap();
    let no_change = comments::update_comment_content_as(
        &db,
        changed.id,
        "Texte révisé",
        None,
        None,
        None,
        Some(editor.id),
    )
    .await
    .unwrap();
    assert_eq!(updated.updated_at, no_change.updated_at);
    assert_eq!(notices(&db, changed.id).await.len(), 3);
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        3
    );
    let results = notices(&db, changed.id).await;
    assert_eq!(results.iter().filter(|j| j.state == "cancelled").count(), 2);
    assert_eq!(results.iter().filter(|j| j.state == "succeeded").count(), 1);
    assert!(results.iter().filter_map(|j| j.output.as_ref()).all(|v| {
        v["capture"]["text"]
            .as_str()
            .unwrap()
            .contains("Texte révisé")
    }));

    let hidden = guest(&db, entry.id).await;
    sqlx::query("UPDATE entries SET status='draft' WHERE id=$1")
        .bind(entry.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    assert!(
        notices(&db, hidden.id)
            .await
            .iter()
            .all(|j| j.state == "cancelled")
    );
    sqlx::query("UPDATE entries SET status='published' WHERE id=$1")
        .bind(entry.id)
        .execute(&db)
        .await
        .unwrap();
    let deleted = guest(&db, entry.id).await;
    comments::delete_comment(&db, deleted.id).await.unwrap();
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    assert!(
        notices(&db, deleted.id)
            .await
            .iter()
            .all(|j| j.state == "cancelled")
    );

    notification_settings(&db,json!({"discussion.comment_moderation":false,"discussion.comment_previously_approved":false})).await;
    let own=comments::create_comment(&db,serde_json::from_value(json!({
        "entry_id":entry.id,"author_id":author.id,"author_name":"Auteur","author_email":author.email,"content":"Mon propre commentaire"
    })).unwrap()).await.unwrap();
    assert_eq!(own.status, "approved");
    assert!(notices(&db, own.id).await.is_empty());

    // Une réponse du modérateur notifie l'auteur, pas le modérateur lui-même.
    let reply = comments::reply_to_comment(
        &db,
        pending.id,
        editor.id,
        &editor.display_name,
        &editor.email,
        "Réponse de modération",
    )
    .await
    .unwrap();
    assert_eq!(notices(&db, reply.id).await.len(), 1);
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        notices(&db, reply.id).await[0].output.as_ref().unwrap()["capture"]["recipient"],
        author.email
    );

    // Le spam et la corbeille ne créent pas d'événement et rendent l'ancien obsolète.
    notification_settings(
        &db,
        json!({"discussion.comment_moderation":true,"discussion.moderation_notify":true}),
    )
    .await;
    for status in [CommentStatus::Spam, CommentStatus::Trash] {
        let suppressed = guest(&db, entry.id).await;
        comments::update_comment_status_as(&db, suppressed.id, status, Some(editor.id))
            .await
            .unwrap();
        assert_eq!(
            mail::run_batch(&db, &mail::CaptureTransport, 100)
                .await
                .unwrap(),
            2
        );
        assert!(
            notices(&db, suppressed.id)
                .await
                .iter()
                .all(|job| job.state == "cancelled")
        );
    }
    let reassigned = guest(&db, entry.id).await;
    // Annuler d'abord les attentes de modération, puis créer un événement d'approbation.
    comments::update_comment_status_as(
        &db,
        reassigned.id,
        CommentStatus::Approved,
        Some(editor.id),
    )
    .await
    .unwrap();
    sqlx::query("UPDATE entries SET author_id=$2 WHERE id=$1")
        .bind(entry.id)
        .bind(admin.id)
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        3
    );
    assert!(
        notices(&db, reassigned.id)
            .await
            .iter()
            .all(|job| job.state == "cancelled")
    );
    sqlx::query("UPDATE entries SET author_id=$2 WHERE id=$1")
        .bind(entry.id)
        .bind(author.id)
        .execute(&db)
        .await
        .unwrap();

    // Un échec d'outbox annule aussi l'INSERT du commentaire.
    notification_settings(&db, json!({"discussion.comment_moderation":true})).await;
    sqlx::query("CREATE FUNCTION fixture_reject_notice() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture rejected notice'; END; $$").execute(&db).await.unwrap();
    sqlx::query("CREATE TRIGGER fixture_reject_notice BEFORE INSERT ON rf_jobs FOR EACH ROW WHEN (NEW.kind='rf.mail.comment') EXECUTE FUNCTION fixture_reject_notice()")
        .execute(&db).await.unwrap();
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM comments")
        .fetch_one(&db)
        .await
        .unwrap();
    let failed=comments::create_comment(&db,serde_json::from_value(json!({
        "entry_id":entry.id,"author_name":"Visiteur","author_email":"rollback@example.test","content":"Ne doit pas exister"
    })).unwrap()).await;
    assert!(failed.is_err());
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM comments")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(before, after);
    sqlx::query("DROP TRIGGER fixture_reject_notice ON rf_jobs")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION fixture_reject_notice()")
        .execute(&db)
        .await
        .unwrap();

    // Une erreur sur la dernière ligne annule tout le lot de modération.
    notification_settings(&db, json!({"discussion.moderation_notify":false})).await;
    let batch_a = guest(&db, entry.id).await;
    let batch_b = guest(&db, entry.id).await;
    let reject_id = batch_a.id.max(batch_b.id);
    sqlx::query(&format!("CREATE FUNCTION fixture_reject_batch() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.payload->>'comment_id'='{reject_id}' THEN RAISE EXCEPTION 'fixture last row'; END IF; RETURN NEW; END; $$"))
        .execute(&db).await.unwrap();
    sqlx::query("CREATE TRIGGER fixture_reject_batch BEFORE INSERT ON rf_jobs FOR EACH ROW WHEN (NEW.kind='rf.mail.comment') EXECUTE FUNCTION fixture_reject_batch()")
        .execute(&db).await.unwrap();
    assert!(
        comments::batch_update_status_as(
            &db,
            &[batch_a.id, batch_b.id],
            CommentStatus::Approved,
            Some(editor.id)
        )
        .await
        .is_err()
    );
    assert_eq!(
        comments::get_comment(&db, batch_a.id).await.unwrap().status,
        "pending"
    );
    assert_eq!(
        comments::get_comment(&db, batch_b.id).await.unwrap().status,
        "pending"
    );
    assert!(notices(&db, batch_a.id).await.is_empty());
    assert!(notices(&db, batch_b.id).await.is_empty());
    sqlx::query("DROP TRIGGER fixture_reject_batch ON rf_jobs")
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION fixture_reject_batch()")
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(
        comments::batch_update_status_as(
            &db,
            &[batch_a.id, batch_b.id, batch_a.id],
            CommentStatus::Approved,
            Some(editor.id)
        )
        .await
        .unwrap(),
        2
    );
    assert_eq!(
        mail::run_batch(&db, &mail::CaptureTransport, 100)
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        comments::batch_update_status_as(
            &db,
            &[batch_a.id, batch_b.id],
            CommentStatus::Approved,
            Some(editor.id)
        )
        .await
        .unwrap(),
        0
    );

    db.close().await;
    assert!(name.starts_with("rf_jobs_test_"));
    sqlx::query(&format!("DROP DATABASE {name} WITH (FORCE)"))
        .execute(&server)
        .await
        .unwrap();
    server.close().await;
}
