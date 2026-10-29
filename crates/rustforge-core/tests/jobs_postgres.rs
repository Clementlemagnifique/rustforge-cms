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

    let snapshot = native_backup::dump(&db).await.unwrap();
    assert_eq!(snapshot.format, "rustforge-native-3");
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
