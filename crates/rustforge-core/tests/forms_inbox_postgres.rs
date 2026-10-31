//! Disposable databases only; exercise inbox transitions, retention and backup compatibility.
use rustforge_core::{
    forms::{self, FormInput, SubmissionAction, SubmissionChange, SubmissionFilter},
    migration::native_backup,
    models::Role,
    settings,
    users::{self, NewUser},
};
use serde_json::json;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::{collections::BTreeMap, str::FromStr};
use uuid::Uuid;
async fn database(server: &PgPool, base: &str) -> (String, PgPool) {
    let name = format!("rf_inbox_test_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE DATABASE {name}"))
        .execute(server)
        .await
        .unwrap();
    let db = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(PgConnectOptions::from_str(base).unwrap().database(&name))
        .await
        .unwrap();
    rustforge_core::migrate(&db).await.unwrap();
    (name, db)
}
#[tokio::test]
#[ignore = "requires isolated local PostgreSQL on 55496"]
async fn inbox_retention_and_legacy_backup() {
    let base = std::env::var("RUSTFORGE_TEST_DATABASE_URL").unwrap();
    let url = reqwest::Url::parse(&base).unwrap();
    assert_eq!(url.host_str(), Some("127.0.0.1"));
    assert_eq!(url.port(), Some(55496));
    let server = PgPoolOptions::new()
        .max_connections(2)
        .connect(&base)
        .await
        .unwrap();
    let (name, db) = database(&server, &base).await;
    let (target_name, target) = database(&server, &base).await;
    let admin = users::create(
        &db,
        NewUser {
            email: "inbox@example.test".into(),
            username: "inbox_admin".into(),
            password: "Test-only-password-2026!".into(),
            role: Role::Admin,
            display_name: None,
        },
    )
    .await
    .unwrap();
    let form = forms::create_form(
        &db,
        admin.id,
        FormInput {
            name: "Recette boîte de réception".into(),
            slug: None,
            description: None,
            fields: json!({"type":"object","properties":{"message":{"type":"string"}}}),
            settings: None,
            is_active: Some(true),
        },
    )
    .await
    .unwrap();
    let row: forms::FormSubmission = sqlx::query_as(
        "INSERT INTO form_submissions(form_id,data,ip_hash) VALUES($1,$2,'') RETURNING *",
    )
    .bind(form.id)
    .bind(json!({"message":"Été 50%_\\ exact"}))
    .fetch_one(&db)
    .await
    .unwrap();
    let query = SubmissionFilter {
        q: Some("50%_\\".into()),
        ..Default::default()
    };
    let inbox = forms::submission_inbox(&db, form.id, &query).await.unwrap();
    assert_eq!(inbox.page.total, 1);
    assert_eq!(inbox.counts.unread, 1);
    assert_eq!(
        forms::submission_inbox(
            &db,
            form.id,
            &SubmissionFilter {
                q: Some("50_".into()),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .page
        .total,
        0
    );
    let change = |action, is_read, deleted_at| SubmissionChange {
        action,
        expected_is_read: is_read,
        expected_deleted_at: deleted_at,
    };
    let read = forms::change_submission(
        &db,
        form.id,
        row.id,
        &change(SubmissionAction::Read, false, None),
    )
    .await
    .unwrap();
    assert!(read.is_read);
    assert!(
        forms::change_submission(
            &db,
            form.id,
            row.id,
            &change(SubmissionAction::Trash, false, None)
        )
        .await
        .is_err()
    );
    let trashed = forms::change_submission(
        &db,
        form.id,
        row.id,
        &change(SubmissionAction::Trash, true, None),
    )
    .await
    .unwrap();
    assert!(trashed.deleted_at.is_some());
    assert_eq!(
        forms::list_submissions(&db, form.id, None, None)
            .await
            .unwrap()
            .total,
        0
    );
    let inbox = forms::submission_inbox(
        &db,
        form.id,
        &SubmissionFilter {
            trash: true,
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(inbox.page.total, 1);
    assert_eq!(inbox.counts.trash, 1);
    assert_eq!(inbox.counts.inbox, 0);
    assert!(
        forms::change_submission(
            &db,
            form.id,
            row.id,
            &change(SubmissionAction::Read, true, trashed.deleted_at)
        )
        .await
        .is_err()
    );
    let restored = forms::change_submission(
        &db,
        form.id,
        row.id,
        &change(SubmissionAction::Restore, true, trashed.deleted_at),
    )
    .await
    .unwrap();
    assert_eq!(restored.data, row.data);
    assert!(restored.deleted_at.is_none());
    assert!(
        forms::change_submission(
            &db,
            form.id,
            row.id,
            &change(SubmissionAction::Restore, true, trashed.deleted_at)
        )
        .await
        .is_err()
    );
    assert!(
        !forms::change_submission(
            &db,
            form.id,
            row.id,
            &change(SubmissionAction::Unread, true, None)
        )
        .await
        .unwrap()
        .is_read
    );
    sqlx::query("INSERT INTO form_submissions(form_id,data,ip_hash,created_at,deleted_at) SELECT $1,'{}','',now()-d*interval '1 day',CASE WHEN d=100 THEN now() ELSE NULL END FROM unnest(ARRAY[10,50,100]) d").bind(form.id).execute(&db).await.unwrap();
    assert_eq!(forms::purge_expired_submissions(&db).await.unwrap(), 0);
    for invalid in [json!(30), json!("60_days"), json!(true)] {
        assert!(
            settings::update(&db, BTreeMap::from([("forms.retention".into(), invalid)]))
                .await
                .is_err()
        );
    }
    settings::update(
        &db,
        BTreeMap::from([("forms.retention".into(), json!("90_days"))]),
    )
    .await
    .unwrap();
    assert_eq!(forms::purge_expired_submissions(&db).await.unwrap(), 1);
    settings::update(
        &db,
        BTreeMap::from([("forms.retention".into(), json!("30_days"))]),
    )
    .await
    .unwrap();
    assert_eq!(
        rustforge_core::scheduler::run_scheduler_tick(&db)
            .await
            .unwrap()
            .purged_submissions_count,
        1
    );
    assert_eq!(
        forms::list_submissions(&db, form.id, None, None)
            .await
            .unwrap()
            .total,
        2
    );
    let mut snapshot = native_backup::dump(&db).await.unwrap();
    // Snapshot from immediately before the nullable inbox migration.
    assert_eq!(snapshot.migrations.pop(), Some(20261031000011));
    for row in snapshot.tables.get_mut("form_submissions").unwrap() {
        row.as_object_mut().unwrap().remove("deleted_at");
    }
    let mut tx = target.begin().await.unwrap();
    native_backup::restore(&mut tx, &snapshot).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        forms::list_submissions(&target, form.id, None, None)
            .await
            .unwrap()
            .total,
        2
    );
    let policy: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key='forms.retention'")
            .fetch_optional(&target)
            .await
            .unwrap();
    assert!(policy.is_none());
    assert_eq!(forms::purge_expired_submissions(&target).await.unwrap(), 0);
    db.close().await;
    target.close().await;
    for name in [name, target_name] {
        sqlx::query(&format!("DROP DATABASE {name}"))
            .execute(&server)
            .await
            .unwrap();
    }
}

struct NeverRemote;
#[async_trait::async_trait]
impl rustforge_core::mail::Transport for NeverRemote {
    async fn deliver(
        &self,
        _message: rustforge_core::mail::Message,
        _key: Uuid,
    ) -> Result<rustforge_core::mail::Receipt, rustforge_core::mail::DeliveryError> {
        panic!("form notices must never call the supplied remote transport")
    }
}
#[tokio::test]
#[ignore = "requires isolated local PostgreSQL on 55496"]
async fn atomic_batches_capture_notices_and_custom_taxonomies() {
    use rustforge_core::{mail, terms};
    let base = std::env::var("RUSTFORGE_TEST_DATABASE_URL").unwrap();
    let url = reqwest::Url::parse(&base).unwrap();
    assert_eq!(url.host_str(), Some("127.0.0.1"));
    assert_eq!(url.port(), Some(55496));
    let server = PgPoolOptions::new()
        .max_connections(2)
        .connect(&base)
        .await
        .unwrap();
    let (name, db) = database(&server, &base).await;
    let admin = users::create(
        &db,
        NewUser {
            email: "batch@example.test".into(),
            username: "batch_admin".into(),
            password: "Test-only-password-2026!".into(),
            role: Role::Admin,
            display_name: None,
        },
    )
    .await
    .unwrap();
    let form=forms::create_form(&db,admin.id,FormInput{name:"Avis de recette".into(),slug:None,description:None,fields:json!({"type":"object","properties":{"message":{"type":"string"}}}),settings:Some(json!({"notify_email":"legacy@example.test","notifications":{"capture_owner":true}})),is_active:Some(true)}).await.unwrap();
    let submit = |spam| forms::SubmitFormPayload {
        data: json!({"message":"SECRET_ANSWER_NOT_IN_QUEUE"}),
        _website: if spam { Some("bot".into()) } else { None },
        user_agent: None,
        client_ip: None,
    };
    let first = forms::submit(&db, form.id, submit(false)).await.unwrap();
    forms::submit(&db, form.id, submit(true)).await.unwrap();
    let payloads: Vec<serde_json::Value> = sqlx::query_scalar("SELECT payload FROM rf_jobs")
        .fetch_all(&db)
        .await
        .unwrap();
    assert_eq!(payloads.len(), 1);
    assert!(!payloads[0].to_string().contains("SECRET_ANSWER"));
    assert_eq!(mail::run_batch(&db, &NeverRemote, 10).await.unwrap(), 1);
    let output: serde_json::Value =
        sqlx::query_scalar("SELECT output FROM rf_jobs WHERE payload->>'submission_id'=$1")
            .bind(first.id.to_string())
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(output["transport"], "capture");
    assert!(!output.to_string().contains("SECRET_ANSWER"));
    let second = forms::submit(&db, form.id, submit(false)).await.unwrap();
    let expected = |row: &forms::FormSubmission| json!({"id":row.id,"expected_is_read":row.is_read,"expected_deleted_at":row.deleted_at});
    let batch = |action: &str, items: Vec<serde_json::Value>, confirm| {
        serde_json::from_value::<forms::SubmissionBatch>(
            json!({"action":action,"items":items,"confirm_delete":confirm}),
        )
        .unwrap()
    };
    let mut stale = expected(&second);
    stale["expected_is_read"] = json!(true);
    assert!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("read", vec![expected(&first), stale], false)
        )
        .await
        .is_err()
    );
    assert_eq!(
        forms::submission_inbox(
            &db,
            form.id,
            &SubmissionFilter {
                is_read: Some(true),
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .page
        .total,
        0
    );
    assert!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("read", vec![expected(&first), expected(&first)], false)
        )
        .await
        .is_err()
    );
    assert!(
        forms::batch_submissions(
            &db,
            Uuid::new_v4(),
            &batch("read", vec![expected(&first)], false)
        )
        .await
        .is_err()
    );
    assert_eq!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("trash", vec![expected(&first), expected(&second)], false)
        )
        .await
        .unwrap(),
        2
    );
    assert_eq!(mail::run_batch(&db, &NeverRemote, 10).await.unwrap(), 1);
    let code: Option<String> =
        sqlx::query_scalar("SELECT error_code FROM rf_jobs WHERE payload->>'submission_id'=$1")
            .bind(second.id.to_string())
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(code.as_deref(), Some("form_notice_obsolete"));
    let trashed = forms::submission_inbox(
        &db,
        form.id,
        &SubmissionFilter {
            trash: true,
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .page
    .items;
    assert!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("delete", trashed.iter().map(expected).collect(), false)
        )
        .await
        .is_err()
    );
    assert_eq!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("restore", trashed.iter().map(expected).collect(), false)
        )
        .await
        .unwrap(),
        2
    );
    let rows = forms::submission_inbox(
        &db,
        form.id,
        &SubmissionFilter {
            is_spam: Some(false),
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .page
    .items;
    assert_eq!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("trash", rows.iter().map(expected).collect(), false)
        )
        .await
        .unwrap(),
        2
    );
    let rows = forms::submission_inbox(
        &db,
        form.id,
        &SubmissionFilter {
            trash: true,
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .page
    .items;
    assert_eq!(
        forms::batch_submissions(
            &db,
            form.id,
            &batch("delete", rows.iter().map(expected).collect(), true)
        )
        .await
        .unwrap(),
        2
    );
    assert_eq!(
        forms::list_submissions(&db, form.id, None, None)
            .await
            .unwrap()
            .total,
        1
    );
    let tx = terms::create_taxonomy(
        &db,
        terms::TaxonomyInput {
            name: "Publics".into(),
            slug: Some("publics".into()),
            hierarchical: true,
        },
    )
    .await
    .unwrap();
    let term = terms::create_term(
        &db,
        terms::TermInput {
            taxonomy: Some(tx.slug.clone()),
            name: "Équipe".into(),
            slug: None,
            description: "".into(),
            parent_id: None,
        },
    )
    .await
    .unwrap();
    assert!(terms::delete_taxonomy(&db, tx.id).await.is_err());
    terms::rename_taxonomy(
        &db,
        tx.id,
        terms::TaxonomyRename {
            name: "Nos publics".into(),
            expected_name: tx.name.clone(),
        },
    )
    .await
    .unwrap();
    assert!(
        terms::rename_taxonomy(
            &db,
            tx.id,
            terms::TaxonomyRename {
                name: "Écrasement".into(),
                expected_name: tx.name
            }
        )
        .await
        .is_err()
    );
    terms::delete_term_with_reassignment(&db, term.id, None)
        .await
        .unwrap();
    terms::delete_taxonomy(&db, tx.id).await.unwrap();
    let system = terms::list_taxonomies(&db)
        .await
        .unwrap()
        .into_iter()
        .find(|t| t.is_system)
        .unwrap();
    assert!(terms::delete_taxonomy(&db, system.id).await.is_err());
    db.close().await;
    sqlx::query(&format!("DROP DATABASE {name}"))
        .execute(&server)
        .await
        .unwrap();
}
