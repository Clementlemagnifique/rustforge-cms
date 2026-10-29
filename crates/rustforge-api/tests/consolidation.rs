//! Regression scenarios use ONLY an explicitly supplied, disposable PostgreSQL server.

use axum::{

    Router,

    body::{Body, to_bytes},

    http::Request,

};

use rustforge_core::{

    self as core, NoopHooks, entries,

    models::{EntryInput, Role},

    users::{self, NewUser},

};

use serde_json::{Value, json};

use sqlx::{PgPool, postgres::PgPoolOptions};

use std::{

    io::Cursor,

    path::{Path, PathBuf},

};

use tower::ServiceExt;

use uuid::Uuid;
use chrono::Datelike;



const SECRET: &str = "regression-only-secret-0123456789-abcdefghijklmnopqrstuvwxyz";

async fn migration_request(

    app: &Router,

    url: &str,

    token: &str,

    xml: &str,

    fields: &[(&str, &str)],

) -> (u16, Value) {

    let boundary = "trustpress-regression-boundary";

    let mut body = format!(

        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"export.xml\"\r\nContent-Type: text/xml\r\n\r\n{xml}\r\n"

    );

    for (key, value) in fields {

        body.push_str(&format!(

            "--{boundary}\r\nContent-Disposition: form-data; name=\"{key}\"\r\n\r\n{value}\r\n"

        ));

    }

    body.push_str(&format!("--{boundary}--\r\n"));

    let response = app

        .clone()

        .oneshot(

            Request::builder()

                .method("POST")

                .uri(url)

                .header("authorization", format!("Bearer {token}"))

                .header(

                    "content-type",

                    format!("multipart/form-data; boundary={boundary}"),

                )

                .body(Body::from(body))

                .unwrap(),

        )

        .await

        .unwrap();

    let code = response.status().as_u16();

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();

    (code, serde_json::from_slice(&bytes).unwrap())

}

async fn request(

    app: &Router,

    method: &str,

    url: &str,

    token: Option<&str>,

    body: Value,

) -> (u16, Value) {

    let mut req = Request::builder()

        .method(method)

        .uri(url)

        .header("content-type", "application/json");

    if let Some(token) = token {

        req = req.header("authorization", format!("Bearer {token}"));

    }

    let response = app

        .clone()

        .oneshot(req.body(Body::from(body.to_string())).unwrap())

        .await

        .unwrap();

    let status = response.status().as_u16();

    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024)

        .await

        .unwrap();

    (

        status,

        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),

    )

}

async fn database(server: &PgPool, base: &str) -> (String, PgPool) {

    let name = format!("rf_regression_{}", Uuid::new_v4().simple());

    sqlx::query(&format!("CREATE DATABASE {name}"))

        .execute(server)

        .await

        .unwrap();

    let url = format!("{}/{name}", base.rsplit_once('/').unwrap().0);

    let db = PgPoolOptions::new()

        .max_connections(6)

        .connect(&url)

        .await

        .unwrap();

    core::migrate(&db).await.unwrap();

    (name, db)

}

async fn account(db: &PgPool, name: &str, role: Role) -> core::models::User {

    users::create(

        db,

        NewUser {

            email: format!("{name}@example.test"),

            username: name.into(),

            display_name: None,

            password: "test-password-long-123!".into(),

            role,

        },

    )

    .await

    .unwrap()

}

async fn app(db: PgPool, root: &Path) -> Router {

    for dir in ["uploads", "plugins", "themes", "protected"] {

        std::fs::create_dir_all(root.join(dir)).unwrap();

    }

    let plugins = rustforge_plugins::PluginManager::new(db.clone(), root.join("plugins"))

        .await

        .unwrap();

    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

    let mut state = rustforge_api::AppState::new(

        db,

        plugins,

        SECRET.into(),

        root.join("uploads"),

        Some(workspace.join("admin/dist")),

        workspace.join("themes"),

        "default".into(),

    );

    state.protected_downloads_dir = root.join("protected");

    state.setup_token = Some(SECRET.into());

    rustforge_api::build_router(state)

}

fn input(v: Value) -> EntryInput {

    serde_json::from_value(v).unwrap()

}



#[test]

fn malformed_php_and_form_constraints_are_rejected() {

    use core::migration::php_serializer::PhpDeserializer;

    for value in ["é", "s:1:\"é\";", "a:999999999:{}", "b:7;", "N;junk"] {

        assert!(PhpDeserializer::deserialize(value).is_err(), "{value}");

    }

    let schema = json!({"type":"object","properties":{"email":{"type":"string","format":"email"}},"required":["email"]});

    assert!(core::form_validation::validate(&schema, &json!({"email":"ok@example.test"})).is_ok());

    for v in [

        json!({}),

        json!({"email":"wrong"}),

        json!({"email":4}),

        json!({"email":"ok@example.test","extra":true}),

    ] {

        assert!(core::form_validation::validate(&schema, &v).is_err());

    }

    assert!(core::form_validation::schema(&json!({"type":"string","pattern":".*"}), 0).is_err());

}



#[test]

fn rf_xml_preserves_complete_envelope_and_cdata() {

    use core::migration::{

        exporter::DataMorphExporter, tmir::TMIRDocument, wxr_importer::WxrImporter,

    };

    let mut doc = TMIRDocument {

        source_site_url: Some("https://source.example.test".into()),

        ..Default::default()

    };

    doc.metadata.insert(

        "nested".into(),

        json!({"xml":"]]> & <tag>","bool":false,"array":[1,2]}),

    );

    let native = json!({"rf_schema":"rf.document/1","rf_blocks":[{"rf_id":"p","rf_type":"paragraph","rf_text":"Source ]]> & <tag>"}]});

    doc.metadata

        .insert("entry_blocks:native-test".into(), native.clone());

    let xml = DataMorphExporter::to_rf_xml(&doc).unwrap();

    let restored = WxrImporter::parse(Cursor::new(xml)).unwrap();

    assert_eq!(restored.metadata["entry_blocks:native-test"], native);

    assert_eq!(

        serde_json::to_value(doc).unwrap(),

        serde_json::to_value(restored).unwrap()

    );

}



#[tokio::test]

async fn upload_cannot_disguise_active_html() {

    for (name, mime) in [

        ("page.html", "text/plain"),

        ("photo.png", "image/png"),

        ("script.svg", "application/pdf"),

    ] {

        assert!(

            core::media_processor::process_upload(b"<script>alert(1)</script>", name, mime)

                .await

                .is_err()

        );

    }

}



#[tokio::test]

#[ignore = "requires RUSTFORGE_TEST_DATABASE_URL pointing to a disposable PostgreSQL server"]

async fn postgres_http_and_native_backup_regressions() {

    use core::migration::{

        capsule::{CapsuleManager, StoragePaths},

        exporter::DataMorphExporter,

        importer::DataMorphImporter,

        native_backup,

    };

    let base = std::env::var("RUSTFORGE_TEST_DATABASE_URL")

        .expect("disposable test server URL required; never use a production URL");

    let server = PgPoolOptions::new()

        .max_connections(2)

        .connect(&base)

        .await

        .unwrap();

    let (name, db) = database(&server, &base).await;

    let (target_name, target) = database(&server, &base).await;

    let root = std::env::temp_dir().join(format!("rf_regression_{}", Uuid::new_v4()));

    let app = app(db.clone(), &root).await;

    // A public installer cannot be claimed without the operator token; install is atomic.

    let setup = json!({"setup_token":"wrong","site_title":"Regression","admin_email":"admin@example.test","admin_username":"admin","admin_password":"test-password-long-123!"});

    assert_eq!(

        request(&app, "POST", "/api/v1/setup/install", None, setup.clone())

            .await

            .0,

        403

    );

    let mut setup = setup;

    setup["setup_token"] = json!(SECRET);

    let installed = request(&app, "POST", "/api/v1/setup/install", None, setup.clone()).await;

    assert_eq!(installed.0, 200, "{installed:?}");

    assert_eq!(

        request(&app, "POST", "/api/v1/setup/install", None, setup)

            .await

            .0,

        403

    );

    let admin_id = Uuid::parse_str(installed.1["user"]["id"].as_str().unwrap()).unwrap();

    let admin = installed.1["token"].as_str().unwrap().to_string();

    let author = account(&db, "author", Role::Author).await;

    let other = account(&db, "other", Role::Author).await;

    let subscriber = account(&db, "reader", Role::Subscriber).await;

    let token = rustforge_api::auth::create_token(author.id, author.role, SECRET).unwrap();

    let other_token = rustforge_api::auth::create_token(other.id, other.role, SECRET).unwrap();

    let reader_token =

        rustforge_api::auth::create_token(subscriber.id, subscriber.role, SECRET).unwrap();

    let preview = request(

        &app,

        "POST",

        "/api/v1/entries/preview",

        Some(&token),

        json!({"body":"# Titre\n<script>alert(1)</script>"}),

    )

    .await;

    assert_eq!(preview.0, 200);

    assert!(

        preview.1["body_html"]

            .as_str()

            .unwrap()

            .contains("<h1>Titre</h1>")

    );

    assert!(!preview.1["body_html"].as_str().unwrap().contains("<script"));

    assert_eq!(

        request(

            &app,

            "POST",

            "/api/v1/entries/preview",

            Some(&reader_token),

            json!({"body":"private"})

        )

        .await

        .0,

        403

    );

    let fixture = include_str!("../../../tests/fixtures/wordpress-editorial.xml");

    assert_eq!(

        migration_request(&app, "/api/v1/converter/inspect-wxr", &token, fixture, &[])

            .await

            .0,

        403

    );

    let before_inspection: i64 = sqlx::query_scalar("SELECT count(*) FROM entries")

        .fetch_one(&db)

        .await

        .unwrap();

    let inspection =

        migration_request(&app, "/api/v1/converter/inspect-wxr", &admin, fixture, &[]).await;

    assert_eq!(inspection.0, 200, "{inspection:?}");

    assert_eq!(inspection.1["entries"], 2);

    let after_inspection: i64 = sqlx::query_scalar("SELECT count(*) FROM entries")

        .fetch_one(&db)

        .await

        .unwrap();

    assert_eq!(before_inspection, after_inspection);

    let seal = inspection.1["seal"].as_str().unwrap();

    assert_eq!(

        migration_request(

            &app,

            "/api/v1/converter/import-wxr",

            &admin,

            fixture,

            &[("seal", seal)]

        )

        .await

        .0,

        400

    );

    assert_eq!(

        migration_request(

            &app,

            "/api/v1/converter/import-wxr",

            &admin,

            &fixture.replace("Article de démonstration", "Fichier changé"),

            &[("seal", seal), ("acknowledge", "true")]

        )

        .await

        .0,

        400

    );

    for _ in 0..2 {

        let imported = migration_request(

            &app,

            "/api/v1/converter/import-wxr",

            &admin,

            fixture,

            &[("seal", seal), ("acknowledge", "true")],

        )

        .await;

        assert_eq!(imported.0, 200, "{imported:?}");

        assert_eq!(imported.1["report"]["entries_imported"], 2);

    }

    let page_status: String =

        sqlx::query_scalar("SELECT status::text FROM entries WHERE slug='presentation'")

            .fetch_one(&db)

            .await

            .unwrap();

    assert_eq!(page_status, "draft");

    for _ in 0..2 {

        core::scheduler::run_scheduler_tick(&db).await.unwrap();

    }

    let private_state: (String, Option<chrono::DateTime<chrono::Utc>>) =

        sqlx::query_as("SELECT status::text,published_at FROM entries WHERE slug='presentation'")

            .fetch_one(&db)

            .await

            .unwrap();

    assert_eq!(private_state, ("draft".into(), None));

    assert_eq!(

        request(

            &app,

            "GET",

            "/api/v1/public/entries/page/presentation",

            None,

            Value::Null

        )

        .await

        .0,

        404

    );

    let media:Uuid=sqlx::query_scalar("INSERT INTO media(path,original_name,mime_type,size_bytes) VALUES('example.txt','example.txt','text/plain',14) RETURNING id").fetch_one(&db).await.unwrap();

    let term: Uuid = sqlx::query_scalar("SELECT id FROM terms LIMIT 1")

        .fetch_one(&db)

        .await

        .unwrap();

    let parent = entries::create(

        &db,

        &NoopHooks,

        "page",

        author.id,

        input(json!({"title":"Parent"})),

    )

    .await

    .unwrap();

    let entry=entries::create(&db,&NoopHooks,"page",author.id,input(json!({"title":"Original","body":"Bonjour","status":"published","fields":{"_rf_vault":{"private":"kept"}},"blocks":[{"type":"paragraph","content":"kept"}],"intent":{"goal":"kept"},"parent_id":parent.id,"featured_media_id":media,"term_ids":[term]}))).await.unwrap();

    let endpoint = format!("/api/v1/entries/{}", entry.id);

    let updated = request(

        &app,

        "PUT",

        &endpoint,

        Some(&token),

        json!({"title":"Modified"}),

    )

    .await;

    assert_eq!(updated.0, 200, "{updated:?}");

    assert_eq!(updated.1["blocks"], entry.blocks);

    assert_eq!(updated.1["fields"], entry.fields);

    assert_eq!(updated.1["body"], "Bonjour");

    assert_eq!(updated.1["parent_id"], json!(parent.id));

    assert_eq!(updated.1["featured_media_id"], json!(media));

    assert_eq!(updated.1["terms"][0]["id"], json!(term));

    let stale_input = EntryInput::from_patch(&entry, json!({"title":"Outdated client"})).unwrap();

    assert!(

        entries::update(&db, &NoopHooks, entry.id, author.id, stale_input)

            .await

            .is_err()

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            &endpoint,

            Some(&other_token),

            json!({"title":"Forbidden"})

        )

        .await

        .0,

        403

    );

    assert_eq!(

        request(&app, "GET", &endpoint, Some(&reader_token), Value::Null)

            .await

            .0,

        403

    );

    let public = request(

        &app,

        "GET",

        &format!("/api/v1/public/entries/page/{}", entry.slug),

        None,

        Value::Null,

    )

    .await;

    assert_eq!(public.0, 200, "{public:?}");

    assert!(public.1.get("fields").is_none());

    assert!(public.1.get("body").is_none());

    let revisions = request(

        &app,

        "GET",

        &format!("{endpoint}/revisions"),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(revisions.0, 200);

    let revision = revisions

        .1

        .as_array()

        .unwrap()

        .iter()

        .find(|r| r["title"] == "Original")

        .unwrap()["id"]

        .as_str()

        .unwrap();

    let restore = format!("{endpoint}/revisions/{revision}/restore");

    assert_eq!(

        request(&app, "POST", &restore, Some(&other_token), json!({}))

            .await

            .0,

        403

    );

    let restored = request(&app, "POST", &restore, Some(&token), json!({})).await;

    assert_eq!(restored.0, 200, "{restored:?}");

    assert_eq!(restored.1["blocks"], entry.blocks);

    // Cached public HTML must disappear after unpublishing.

    let page = format!("/fr/page/{}", entry.slug);

    assert_eq!(request(&app, "GET", &page, None, Value::Null).await.0, 200);

    assert_eq!(

        request(

            &app,

            "PUT",

            &endpoint,

            Some(&token),

            json!({"status":"draft"})

        )

        .await

        .0,

        200

    );

    assert_eq!(request(&app, "GET", &page, None, Value::Null).await.0, 404);

    // Proposal -> explicit approval -> idempotent repeated approval, without a model call.

    // Also verify G2 native document lifecycle: creation -> publication -> preview/public HTML ->

    // reopening -> concurrent edit conflict -> text-only patch guard -> modern & legacy revision restore.

    let native_doc_v1 = json!({

        "rf_schema": "rf.document/1",

        "rf_blocks": [

            {"rf_id": "h1_intro", "rf_type": "heading", "rf_text": "Titre natif V1", "rf_level": 2},

            {"rf_id": "p_body", "rf_type": "paragraph", "rf_text": "Paragraphe <script>alert(1)</script> **littéral** & sûr"},

            {"rf_id": "q_cite", "rf_type": "quote", "rf_text": "Citation première ligne\nSeconde ligne"},

            {"rf_id": "c_code", "rf_type": "code", "rf_text": "fn main() { println!(\"rf\"); }"},

            {"rf_id": "l_items", "rf_type": "list", "rf_ordered": true, "rf_items": ["Étape A", "Étape B"]},

            {"rf_id": "d_sep", "rf_type": "divider"}

        ]

    });

    let native_preview = request(

        &app,

        "POST",

        "/api/v1/entries/preview",

        Some(&token),

        json!({"body": "", "blocks": native_doc_v1}),

    )

    .await;

    assert_eq!(native_preview.0, 200, "{native_preview:?}");

    let preview_body = native_preview.1["body"].as_str().unwrap().to_string();

    let preview_html = native_preview.1["body_html"].as_str().unwrap().to_string();

    assert!(preview_html.contains("<h2>Titre natif V1</h2>"));

    assert!(preview_html.contains("<ol>"));

    assert!(preview_html.contains("<hr"));

    assert!(!preview_html.contains("<script"));

    assert!(preview_html.contains("<strong>littéral</strong>"));
    assert!(preview_html.contains("&lt;script&gt;"));



    let created_native = request(

        &app,

        "POST",

        "/api/v1/entries?content_type=post",

        Some(&token),

        json!({

            "title": "Article Natif G2",

            "slug": "article-natif-g2",

            "excerpt": "Résumé natif",

            "body": "ignoré par la projection native",

            "blocks": native_doc_v1,

            "status": "published",

            "fields": {

                "_rf_vault": {"origin_key": "preserve_me", "nested": {"count": 42}}

            }

        }),

    )

    .await;

    assert_eq!(created_native.0, 200, "{created_native:?}");

    let native_id = Uuid::parse_str(created_native.1["id"].as_str().unwrap()).unwrap();

    let native_endpoint = format!("/api/v1/entries/{native_id}");

    assert_eq!(created_native.1["blocks"], native_doc_v1);

    assert_eq!(created_native.1["body"], preview_body);

    assert_eq!(created_native.1["body_html"], preview_html);

    assert_eq!(

        created_native.1["fields"]["_rf_vault"]["origin_key"],

        "preserve_me"

    );



    // Public SSR and headless API serve the exact projected HTML without exposing raw body or vault.

    let ssr_post = request(&app, "GET", "/fr/post/article-natif-g2", None, Value::Null).await;

    assert_eq!(ssr_post.0, 200);

    assert!(

        ssr_post

            .1

            .as_str()

            .unwrap()

            .contains("<h2>Titre natif V1</h2>")

    );

    assert!(!ssr_post.1.as_str().unwrap().contains("<script>alert(1)"));

    let pub_json = request(

        &app,

        "GET",

        "/api/v1/public/entries/post/article-natif-g2",

        None,

        Value::Null,

    )

    .await;

    assert_eq!(pub_json.0, 200);

    assert_eq!(pub_json.1["body_html"], preview_html);

    assert!(pub_json.1.get("body").is_none());

    assert!(pub_json.1.get("fields").is_none());



    // Reopening returns identical blocks, IDs, projected body and vault.

    let reopened = request(&app, "GET", &native_endpoint, Some(&token), Value::Null).await;

    assert_eq!(reopened.0, 200);

    assert_eq!(reopened.1["blocks"], native_doc_v1);

    assert_eq!(reopened.1["body"], preview_body);

    assert_eq!(reopened.1["body_html"], preview_html);

    let v1_updated_at = reopened.1["updated_at"].as_str().unwrap().to_string();



    // Text-only patch attempting to change `body` without updating `blocks` or converting must be rejected (409).

    assert_eq!(

        request(

            &app,

            "PUT",

            &native_endpoint,

            Some(&token),

            json!({"body": "Tentative d'écrasement texte seul"})

        )

        .await

        .0,

        409

    );

    // AI tool attempting to overwrite `body` only on a native document must also fail on approval.

    let bad_ai_plan = request(

        &app,

        "POST",

        "/api/v1/ai/execute",

        Some(&token),

        json!({

            "tool_name": "create_or_update_content",

            "arguments": {"id": native_id, "body": "Texte IA sans blocs"}

        }),

    )

    .await;

    assert_eq!(bad_ai_plan.0, 200);

    let bad_ai_approve = format!(

        "/api/v1/ai/plans/{}/approve",

        bad_ai_plan.1["data"]["plan_id"].as_str().unwrap()

    );

    let bad_ai_res = request(&app, "POST", &bad_ai_approve, Some(&token), json!({})).await;

    assert_eq!(bad_ai_res.1["success"], false);



    // Update blocks to V2, then verify concurrent edit with stale expected_updated_at is rejected (409).

    let native_doc_v2 = json!({

        "rf_schema": "rf.document/1",

        "rf_blocks": [

            {"rf_id": "h1_intro", "rf_type": "heading", "rf_text": "Titre natif V2", "rf_level": 2},

            {"rf_id": "p_body", "rf_type": "paragraph", "rf_text": "Contenu mis à jour en V2"}

        ]

    });

    let updated_native = request(

        &app,

        "PUT",

        &native_endpoint,

        Some(&token),

        json!({

            "title": "Article Natif G2 V2",

            "blocks": native_doc_v2,

            "expected_updated_at": v1_updated_at

        }),

    )

    .await;

    assert_eq!(updated_native.0, 200, "{updated_native:?}");

    assert_eq!(updated_native.1["blocks"], native_doc_v2);

    assert!(

        updated_native.1["body_html"]

            .as_str()

            .unwrap()

            .contains("<h2>Titre natif V2</h2>")

    );

    assert_eq!(

        updated_native.1["fields"]["_rf_vault"]["origin_key"],

        "preserve_me"

    );

    // Stale concurrent modification is rejected.

    assert_eq!(

        request(

            &app,

            "PUT",

            &native_endpoint,

            Some(&token),

            json!({

                "title": "Conflit concurrent",

                "blocks": native_doc_v1,

                "expected_updated_at": v1_updated_at

            })

        )

        .await

        .0,

        409

    );



    // Legacy revision without snapshot cannot overwrite a native document (409).

    let legacy_rev_id: Uuid = sqlx::query_scalar(

        "INSERT INTO revisions(entry_id, author_id, title, excerpt, body, fields, snapshot)

         VALUES ($1, $2, 'Ancienne sans snapshot', '', 'Corps ancien', '{}'::jsonb, NULL)

         RETURNING id",

    )

    .bind(native_id)

    .bind(author.id)

    .fetch_one(&db)

    .await

    .unwrap();

    assert_eq!(

        request(

            &app,

            "POST",

            &format!("{native_endpoint}/revisions/{legacy_rev_id}/restore"),

            Some(&token),

            json!({})

        )

        .await

        .0,

        409

    );



    // Modern revision with full snapshot restores V1 blocks, IDs, projected body/HTML and vault.

    let native_revs = request(

        &app,

        "GET",

        &format!("{native_endpoint}/revisions"),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(native_revs.0, 200);

    let rev_v1_id = native_revs

        .1

        .as_array()

        .unwrap()

        .iter()

        .find(|r| r["title"] == "Article Natif G2")

        .unwrap()["id"]

        .as_str()

        .unwrap();

    let restored_native = request(

        &app,

        "POST",

        &format!("{native_endpoint}/revisions/{rev_v1_id}/restore"),

        Some(&token),

        json!({}),

    )

    .await;

    assert_eq!(restored_native.0, 200, "{restored_native:?}");

    assert_eq!(restored_native.1["blocks"], native_doc_v1);

    assert_eq!(restored_native.1["body"], preview_body);

    assert_eq!(restored_native.1["body_html"], preview_html);

    assert_eq!(

        restored_native.1["fields"]["_rf_vault"]["origin_key"],

        "preserve_me"

    );



    // 9b. Conversion explicite Markdown -> rf.document/1, blocs image/button sûrs et hiérarchie de pages sans cycle

    let converted_md = request(

        &app,

        "POST",

        "/api/v1/entries/convert-blocks",

        Some(&token),

        json!({

            "body": "# Titre converti\n\nParagraphe converti.\n\n![Visuel](/uploads/2026/10/visuel.webp)\n*Légende convertie*\n\n[Découvrir](/contact)"

        }),

    )

    .await;

    assert_eq!(converted_md.0, 200, "{converted_md:?}");

    assert_eq!(converted_md.1["blocks"]["rf_schema"], "rf.document/1");

    assert_eq!(

        converted_md.1["blocks"]["rf_blocks"]

            .as_array()

            .unwrap()

            .len(),

        4

    );

    assert!(

        converted_md.1["body_html"]

            .as_str()

            .unwrap()

            .contains("<img src=\"/uploads/2026/10/visuel.webp\" alt=\"Visuel\"")

    );

    assert_eq!(

        request(

            &app,

            "POST",

            "/api/v1/entries/preview",

            Some(&token),

            json!({

                "body": "",

                "blocks": {

                    "rf_schema": "rf.document/1",

                    "rf_blocks": [{"rf_id": "bad_img", "rf_type": "image", "rf_url": "javascript:alert(1)"}]

                }

            }),

        )

        .await

        .0,

        422

    );

    let page_parent = request(

        &app,

        "POST",

        "/api/v1/entries?content_type=page",

        Some(&token),

        json!({"title": "Page Parente", "slug": "page-parente", "status": "published"}),

    )

    .await;

    assert_eq!(page_parent.0, 200);

    let page_parent_id = page_parent.1["id"].as_str().unwrap();

    let page_child = request(

        &app,

        "POST",

        "/api/v1/entries?content_type=page",

        Some(&token),

        json!({"title": "Page Enfant", "slug": "page-enfant", "parent_id": page_parent_id, "status": "published"}),

    )

    .await;

    assert_eq!(page_child.0, 200);

    let page_child_id = page_child.1["id"].as_str().unwrap();

    // Refuser un cycle A -> B -> A

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/entries/{page_parent_id}"),

            Some(&token),

            json!({"parent_id": page_child_id}),

        )

        .await

        .0,

        422

    );

    // Refuser un parent d'un autre type de contenu (post au lieu de page)

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/entries/{page_child_id}"),

            Some(&token),

            json!({"parent_id": native_id}),

        )

        .await

        .0,

        422

    );

    // 9b. Permaliens & redirection canonique sans chaîne (WP-21) + garde expected_updated_at sur restauration de révision (WP-28).

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/entries/{page_child_id}"),

            Some(&token),

            json!({"slug": "page-enfant-v2"}),

        )

        .await

        .0,

        200

    );

    let renamed_v3 = request(

        &app,

        "PUT",

        &format!("/api/v1/entries/{page_child_id}"),

        Some(&token),

        json!({"slug": "page-enfant-v3"}),

    )

    .await;

    assert_eq!(renamed_v3.0, 200);

    assert_eq!(

        renamed_v3.1["fields"]["_rf_vault"]["rf_old_slugs"],

        json!(["page-enfant", "page-enfant-v2"])

    );

    // L'ancien slug initial et le slug intermédiaire redirigent tous deux en 308 Permanent directement vers la cible finale.

    assert_eq!(

        request(&app, "GET", "/fr/page/page-enfant", None, Value::Null)

            .await

            .0,

        308

    );

    assert_eq!(

        request(&app, "GET", "/fr/page/page-enfant-v2", None, Value::Null)

            .await

            .0,

        308

    );

    let child_revs = request(

        &app,

        "GET",

        &format!("/api/v1/entries/{page_child_id}/revisions"),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(child_revs.0, 200);

    let oldest_child_rev = child_revs.1.as_array().unwrap().last().unwrap()["id"]

        .as_str()

        .unwrap();

    assert_eq!(

        request(

            &app,

            "POST",

            &format!("/api/v1/entries/{page_child_id}/revisions/{oldest_child_rev}/restore"),

            Some(&token),

            json!({"expected_updated_at": "2020-01-01T00:00:00Z"}),

        )

        .await

        .0,

        409

    );

    // 10. Quick Edit (WP-18) & filtres combinés (WP-16/WP-17) : associer une catégorie et modifier le titre sans renvoyer blocks ni body conserve l'enveloppe native et le coffre _rf_vault.

    let created_term = request(

        &app,

        "POST",

        "/api/v1/terms",

        Some(&token),

        json!({"taxonomy": "category", "name": "Guide technique"}),

    )

    .await;

    assert_eq!(created_term.0, 200, "{created_term:?}");

    let term_id = Uuid::parse_str(created_term.1["id"].as_str().unwrap()).unwrap();

    let quick_edited = request(

        &app,

        "PUT",

        &native_endpoint,

        Some(&token),

        json!({

            "title": "Article natif G2",

            "slug": "article-natif-g2",

            "status": "published",

            "term_ids": [term_id]

        }),

    )

    .await;

    assert_eq!(quick_edited.0, 200, "{quick_edited:?}");

    assert_eq!(quick_edited.1["blocks"], native_doc_v1);

    assert_eq!(

        quick_edited.1["fields"]["_rf_vault"]["origin_key"],

        "preserve_me"

    );

    assert_eq!(quick_edited.1["terms"][0]["slug"], "guide-technique");

    let filtered_list = request(

        &app,

        "GET",

        "/api/v1/entries?content_type=post&taxonomy=category&term=guide-technique&order=title",

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(filtered_list.0, 200, "{filtered_list:?}");

    assert_eq!(filtered_list.1["total"], 1);

    assert_eq!(filtered_list.1["items"][0]["id"], native_id.to_string());



    // 11. Catégories hiérarchiques & étiquettes (WP-19) : hiérarchie, anti-cycle, fusion et suppression avec réaffectation.

    let child_term = request(

        &app,

        "POST",

        "/api/v1/terms",

        Some(&token),

        json!({"taxonomy": "category", "name": "Sous-guide Rust", "parent_id": term_id}),

    )

    .await;

    assert_eq!(child_term.0, 200, "{child_term:?}");

    let child_term_id = Uuid::parse_str(child_term.1["id"].as_str().unwrap()).unwrap();

    // Refuser un cycle de catégories parent -> enfant -> parent (422)

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/terms/{term_id}"),

            Some(&token),

            json!({"parent_id": child_term_id}),

        )

        .await

        .0,

        422

    );

    // Créer une étiquette (tag) et refuser de l'utiliser comme parent d'une catégorie (422)

    let created_tag = request(

        &app,

        "POST",

        "/api/v1/terms",

        Some(&token),

        json!({"taxonomy": "tag", "name": "rust-cms"}),

    )

    .await;

    assert_eq!(created_tag.0, 200, "{created_tag:?}");

    let tag_id = Uuid::parse_str(created_tag.1["id"].as_str().unwrap()).unwrap();

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/terms/{child_term_id}"),

            Some(&token),

            json!({"parent_id": tag_id}),

        )

        .await

        .0,

        422

    );

    // Refuser la fusion d'une catégorie parente vers son propre sous-terme (422 anti-cycle)

    assert_eq!(

        request(

            &app,

            "POST",

            &format!("/api/v1/terms/{term_id}/merge"),

            Some(&token),

            json!({"target_id": child_term_id}),

        )

        .await

        .0,

        422

    );

    // Créer une catégorie cible `final_cat` et fusionner `term_id` vers `final_cat` :

    // l'entrée `native_id` et le sous-terme `child_term_id` sont rattachés à `final_cat`.

    let final_cat = request(

        &app,

        "POST",

        "/api/v1/terms",

        Some(&token),

        json!({"taxonomy": "category", "name": "Documentation officielle"}),

    )

    .await;

    assert_eq!(final_cat.0, 200, "{final_cat:?}");

    let final_cat_id = Uuid::parse_str(final_cat.1["id"].as_str().unwrap()).unwrap();

    let merged_term = request(

        &app,

        "POST",

        &format!("/api/v1/terms/{term_id}/merge"),

        Some(&token),

        json!({"target_id": final_cat_id}),

    )

    .await;

    assert_eq!(merged_term.0, 200, "{merged_term:?}");

    assert_eq!(merged_term.1["entry_count"], 1);

    // Supprimer `child_term_id` avec réaffectation vers `final_cat_id`.

    let deleted_with_reassign = request(

        &app,

        "DELETE",

        &format!("/api/v1/terms/{child_term_id}?reassign_to={final_cat_id}"),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(deleted_with_reassign.0, 200, "{deleted_with_reassign:?}");

    let after_reassign = request(

        &app,

        "GET",

        "/api/v1/entries?content_type=post&taxonomy=category&term=documentation-officielle",

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(after_reassign.0, 200, "{after_reassign:?}");

    assert_eq!(after_reassign.1["total"], 1);



    // 12. G3 Lot 1 : Protection de suppression des médias utilisés (WP-40), Menus publics (WP-46) & CSS personnalisé borné (WP-48/WP-50).

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/entries/{page_child_id}"),

            Some(&token),

            json!({"featured_media_id": media}),

        )

        .await

        .0,

        200

    );

    // Refuser la suppression non forcée d'un média utilisé dans un contenu actif (409 Conflict)

    assert_eq!(

        request(

            &app,

            "DELETE",

            &format!("/api/v1/media/{media}"),

            Some(&token),

            Value::Null,

        )

        .await

        .0,

        409

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            &format!("/api/v1/entries/{page_child_id}"),

            Some(&token),

            json!({"featured_media_id": null}),

        )

        .await

        .0,

        200

    );

    // Refuser une URL javascript: dans site.menus (422) et du HTML/@import dans site.design_tokens.custom_css (422)

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({

                "site.menus": {

                    "header": [{"id": "bad", "label": "Piège", "url": "javascript:alert(1)"}]

                }

            }),

        )

        .await

        .0,

        422

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({

                "site.design_tokens": {

                    "primary_color": "#4f46e5",

                    "custom_css": "</style><script>alert(1)</script>"

                }

            }),

        )

        .await

        .0,

        422

    );

    // Enregistrer un menu public et un CSS personnalisé valides et vérifier leur rendu SSR immédiat

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({

                "site.menus": {

                    "header": [{"id": "m1", "label": "Documentation G3", "url": "/fr/post/article-natif-g2", "target": "_self"}],

                    "footer": [{"id": "m2", "label": "Mentions légales", "url": "/fr/page/page-parente", "target": "_self"}]

                },

                "site.design_tokens": {

                    "primary_color": "#4f46e5",

                    "accent_color": "#06b6d4",

                    "radius": "14px",

                    "container_width": "1280px",

                    "custom_css": ".custom-g3-rule { letter-spacing: 0.01em; }"

                }

            }),

        )

        .await

        .0,

        200

    );

    let home_g3 = request(&app, "GET", "/fr", None, Value::Null).await;

    assert_eq!(home_g3.0, 200);

    let home_html = home_g3.1.as_str().unwrap();

    assert!(home_html.contains("Documentation G3"));

    assert!(home_html.contains("Mentions légales"));

    assert!(home_html.contains(".custom-g3-rule"));

    // Validation hiérarchique QuadMenu (NAV-A, NAV-C, WP-46, TH-15, TH-17)
    // Rejet d'un cycle A -> B -> A (422)
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "site.menus": {
                    "header": [
                        {"id": "c1", "parent_id": "c2", "label": "Cycle 1", "url": "/fr"},
                        {"id": "c2", "parent_id": "c1", "label": "Cycle 2", "url": "/fr"}
                    ]
                }
            }),
        )
        .await
        .0,
        422
    );

    // Rejet d'auto-référence parent_id == id (422)
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "site.menus": {
                    "header": [
                        {"id": "loop_me", "parent_id": "loop_me", "label": "Boucle", "url": "/fr"}
                    ]
                }
            }),
        )
        .await
        .0,
        422
    );

    // Rejet de profondeur excédant 3 niveaux (422)
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "site.menus": {
                    "header": [
                        {"id": "d1", "label": "N1", "url": "/1"},
                        {"id": "d2", "parent_id": "d1", "label": "N2", "url": "/2"},
                        {"id": "d3", "parent_id": "d2", "label": "N3", "url": "/3"},
                        {"id": "d4", "parent_id": "d3", "label": "N4", "url": "/4"}
                    ]
                }
            }),
        )
        .await
        .0,
        422
    );

    // Enregistrement valide d'un menu arborescent avec sous-menu, badge, sous-titre et contrôle SSR
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "site.menus": {
                    "header": [
                        {"id": "m_root", "label": "Solutions Pro", "url": "/fr/solutions"},
                        {"id": "m_sub", "parent_id": "m_root", "label": "Audit Rust", "url": "/fr/audit", "badge": "VIP", "subtitle": "Expertise souveraine"}
                    ],
                    "footer": [
                        {"id": "m_foot", "label": "Confidentialité", "url": "/fr/privacy", "badge": "RGPD"}
                    ]
                }
            }),
        )
        .await
        .0,
        200
    );

    let home_nav = request(&app, "GET", "/fr", None, Value::Null).await;
    assert_eq!(home_nav.0, 200);
    let nav_html = home_nav.1.as_str().unwrap();
    assert!(nav_html.contains("Solutions Pro"));
    assert!(nav_html.contains("Audit Rust"));
    assert!(nav_html.contains("has-children"));
    assert!(nav_html.contains("rf-sub-menu"));
    assert!(nav_html.contains("VIP"));
    assert!(nav_html.contains("Expertise souveraine"));
    assert!(nav_html.contains("Confidentialité"));
    assert!(nav_html.contains("RGPD"));



    // 13. G3 Lot 2 : Utilisateurs, rôles, réattribution et sessions (WP-52 à WP-55) + Réglages Lecture/Général (WP-69 à WP-81).

    let filtered_authors = request(

        &app,

        "GET",

        "/api/v1/users?role=author",

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(filtered_authors.0, 200, "{filtered_authors:?}");

    assert_eq!(filtered_authors.1["total"], 2);

    // Un auteur non-admin ne peut pas s'octroyer le rôle admin (WP-52/WP-53)

    let self_updated = request(

        &app,

        "PUT",

        &format!("/api/v1/users/{}", author.id),

        Some(&token),

        json!({"display_name": "Auteur Vérifié", "bio": "Bio test", "role": "admin"}),

    )

    .await;

    assert_eq!(self_updated.0, 200, "{self_updated:?}");

    assert_eq!(self_updated.1["role"], "author");

    assert_eq!(self_updated.1["display_name"], "Auteur Vérifié");

    // Sessions actives (WP-55) : issue_refresh_token -> list_sessions -> revoke_sessions

    let _ = users::issue_refresh_token(&db, author.id, chrono::Duration::days(1))

        .await

        .unwrap();

    let sessions_before = request(

        &app,

        "GET",

        &format!("/api/v1/users/{}/sessions", author.id),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(sessions_before.0, 200, "{sessions_before:?}");

    assert_eq!(sessions_before.1.as_array().unwrap().len(), 1);

    let revoked_sessions = request(

        &app,

        "DELETE",

        &format!("/api/v1/users/{}/sessions", author.id),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(revoked_sessions.0, 200, "{revoked_sessions:?}");

    assert_eq!(revoked_sessions.1["revoked"], 1);

    // Créer un auteur éphémère, lui attribuer une page, puis supprimer le compte avec réattribution vers `author.id` (WP-54)

    let temp_author = account(&db, "temp_writer", Role::Author).await;

    let temp_token =

        rustforge_api::auth::create_token(temp_author.id, temp_author.role, SECRET).unwrap();

    let temp_entry = request(

        &app,

        "POST",

        "/api/v1/entries?content_type=post",

        Some(&temp_token),

        json!({"title": "Article à réattribuer", "body": "Texte", "status": "published"}),

    )

    .await;

    assert_eq!(temp_entry.0, 200, "{temp_entry:?}");

    let temp_entry_id = Uuid::parse_str(temp_entry.1["id"].as_str().unwrap()).unwrap();

    let deleted_user = request(

        &app,

        "DELETE",

        &format!("/api/v1/users/{}?reassign_to={}", temp_author.id, author.id),

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(deleted_user.0, 200, "{deleted_user:?}");

    let reassigned_entry = request(

        &app,

        "GET",

        &format!("/api/v1/entries/{temp_entry_id}"),

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(reassigned_entry.0, 200, "{reassigned_entry:?}");

    assert_eq!(

        reassigned_entry.1["author_id"].as_str().unwrap(),

        author.id.to_string()

    );

    // Réglages Lecture & Général (WP-72, WP-78, WP-79, WP-81) : refuser users.default_role = admin (422) et reading.per_page = 0 (422)

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"users.default_role": "admin"}),

        )

        .await

        .0,

        422

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.per_page": 0}),

        )

        .await

        .0,

        422

    );

    // Configurer une page d'accueil statique + blog_public = false (noindex) et vérifier le SSR de `/fr`

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({

                "reading.show_on_front": "page",

                "reading.page_on_front": page_parent_id.to_string(),

                "reading.blog_public": false

            }),

        )

        .await

        .0,

        200

    );

    let static_home = request(&app, "GET", "/fr", None, Value::Null).await;

    assert_eq!(static_home.0, 200);

    let static_home_html = static_home.1.as_str().unwrap();

    assert!(static_home_html.contains("Page Parente"));

    assert!(static_home_html.contains("noindex, nofollow"));



    // G3.3-A : la page d'accueil suit les règles publiques, même sans scheduler.

    let home_page_id = Uuid::parse_str(page_parent_id).unwrap();

    for mutation in [

        "status = 'draft'",

        "status = 'published', published_at = now() + interval '1 day'",

        "status = 'published', published_at = now() - interval '1 day', scheduled_unpublish_at = now() - interval '1 second'",

    ] {

        sqlx::query(&format!("UPDATE entries SET {mutation} WHERE id = $1"))

            .bind(home_page_id)

            .execute(&db)

            .await

            .unwrap();

        assert!(matches!(

            entries::get_published_page(&db, home_page_id).await,

            Err(core::CoreError::NotFound)

        ));

        let page_slug = entries::get(&db, home_page_id).await.unwrap().slug;

        assert!(matches!(

            entries::get_published(&db, "page", &page_slug).await,

            Err(core::CoreError::NotFound)

        ));

        let home = request(&app, "GET", "/fr", None, Value::Null).await;

        assert_eq!(home.0, 200);

        assert!(!home.1.as_str().unwrap().contains("Page Parente"));

        let public = entries::list(

            &db,

            &core::entries::EntryQuery {

                public_only: true,

                ..Default::default()

            },

        )

        .await

        .unwrap();

        assert!(!public.items.iter().any(|e| e.id == home_page_id));

    }

    sqlx::query("UPDATE entries SET status='published', published_at=now() - interval '1 day', scheduled_unpublish_at=now() + interval '2 seconds' WHERE id=$1")

        .bind(home_page_id).execute(&db).await.unwrap();

    assert!(

        request(&app, "GET", "/fr", None, Value::Null)

            .await

            .1

            .as_str()

            .unwrap()

            .contains("Page Parente")

    );

    tokio::time::sleep(std::time::Duration::from_millis(2200)).await;

    // Pas d'écriture ni de tick entre les deux requêtes : le cache chaud doit expirer.

    assert!(

        !request(&app, "GET", "/fr", None, Value::Null)

            .await

            .1

            .as_str()

            .unwrap()

            .contains("Page Parente")

    );

    sqlx::query("UPDATE entries SET scheduled_unpublish_at=NULL WHERE id=$1")

        .bind(home_page_id)

        .execute(&db)

        .await

        .unwrap();



    // G3.3-B : parcours publics, pagination, cache et frontières JSON/HTML.

    sqlx::query("INSERT INTO terms(taxonomy_id,name,slug) SELECT id,'Vide','g33-empty' FROM taxonomies WHERE slug='tag'").execute(&db).await.unwrap();

    let archive_term: Uuid = sqlx::query_scalar("INSERT INTO terms(taxonomy_id,name,slug) SELECT id,'Archive publique','g33-public' FROM taxonomies WHERE slug='category' RETURNING id")

        .fetch_one(&db).await.unwrap();

    for n in 1..=3 {

        entries::create(&db, &NoopHooks, "post", author.id,

            input(json!({"title":format!("G33Needle article {n}"),"status":"published","body":"Texte public", "term_ids":[archive_term]}))).await.unwrap();

    }

    entries::create(

        &db,

        &NoopHooks,

        "post",

        author.id,

        input(json!({"title":"G33Needle SECRET","status":"draft","term_ids":[archive_term]})),

    )

    .await

    .unwrap();

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.show_on_front":"posts","reading.per_page":1})

        )

        .await

        .0,

        200

    );

    for (name, timing) in [

        ("FUTURE", "published_at=now()+interval '1 day'"),

        (

            "EXPIRED",

            "scheduled_unpublish_at=now()-interval '1 second'",

        ),

    ] {

        let hidden = entries::create(&db,&NoopHooks,"post",author.id,input(json!({"title":format!("G33Needle {name}"),"status":"published","term_ids":[archive_term]}))).await.unwrap();

        sqlx::query(&format!("UPDATE entries SET {timing} WHERE id=$1"))

            .bind(hidden.id)

            .execute(&db)

            .await

            .unwrap();

    }

    assert_eq!(

        request(&app, "GET", "/fr/archive/tag/g33-empty", None, Value::Null)

            .await

            .0,

        200

    );

    assert_eq!(

        request(

            &app,

            "GET",

            "/fr/archive/tag/g33-empty?page=2",

            None,

            Value::Null

        )

        .await

        .0,

        404

    );

    let results = request(&app, "GET", "/fr/search?q=G33Needle", None, Value::Null).await;

    assert_eq!(results.0, 200, "{results:?}");

    let html = results.1.as_str().unwrap();

    assert!(html.contains("3 résultat(s)"));

    assert!(!html.contains("SECRET"));

    assert!(html.contains("q=G33Needle&amp;page=2"));

    let second = request(

        &app,

        "GET",

        "/fr/search?q=G33Needle&page=2",

        None,

        Value::Null,

    )

    .await;

    assert_eq!(second.0, 200);

    assert_ne!(results.1, second.1);

    assert_eq!(

        request(

            &app,

            "GET",

            "/fr/search?q=G33Needle&page=4",

            None,

            Value::Null

        )

        .await

        .0,

        404

    );

    assert_eq!(

        request(&app, "GET", "/fr/search?q=inexistantG33", None, Value::Null)

            .await

            .0,

        200

    );

    assert_eq!(

        request(&app, "GET", "/fr/search", None, Value::Null)

            .await

            .0,

        200

    );

    for url in ["/fr/search?page=0", "/fr/search?page=abc", "/fr?page=-1"] {

        assert_eq!(request(&app, "GET", url, None, Value::Null).await.0, 400);

    }

    let escaped = request(

        &app,

        "GET",

        "/fr/search?q=%3Cscript%3Ealert%281%29%3C%2Fscript%3E",

        None,

        Value::Null,

    )

    .await;

    assert_eq!(escaped.0, 200);

    assert!(

        !escaped

            .1

            .as_str()

            .unwrap()

            .contains("<script>alert(1)</script>")

    );

    assert_eq!(

        request(

            &app,

            "GET",

            "/fr/archive/category/g33-public",

            None,

            Value::Null

        )

        .await

        .0,

        200

    );

    assert_eq!(

        request(

            &app,

            "GET",

            "/fr/archive/category/g33-public?page=2",

            None,

            Value::Null

        )

        .await

        .0,

        200

    );

    for url in [

        "/fr/archive/category/missing-g33",

        "/fr/archive/unknown/g33-public",

        "/fr/post/missing-g33",

        "/fr/unknown/extra/path/404",

    ] {

        let missing = request(&app, "GET", url, None, Value::Null).await;

        assert_eq!(missing.0, 404);

        assert!(missing.1.as_str().unwrap().contains("Page introuvable"));

    }

    assert!(

        request(&app, "GET", "/api/v1/missing-g33", None, Value::Null)

            .await

            .1

            .is_object()

    );

    assert_eq!(

        request(&app, "GET", "/fr?page=2", None, Value::Null)

            .await

            .0,

        200

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.show_on_front":"page","reading.per_page":10})

        )

        .await

        .0,

        200

    );



    // G3.3-C : choix distincts et index public effectif.

    let blog_page=entries::create(&db,&NoopHooks,"page",author.id,input(json!({"title":"Index du journal","slug":"g33-blog","body":"Ce texte ne doit pas devenir la liste","status":"published"}))).await.unwrap();

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.page_for_posts":home_page_id.to_string()})

        )

        .await

        .0,

        422

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.page_for_posts":"not-a-uuid"})

        )

        .await

        .0,

        422

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.page_for_posts":blog_page.id.to_string(),"reading.per_page":1})

        )

        .await

        .0,

        200

    );

    let index = request(&app, "GET", "/fr/page/g33-blog", None, Value::Null).await;

    assert_eq!(index.0, 200);

    assert!(index.1.as_str().unwrap().contains("G33Needle article"));

    assert!(

        !index

            .1

            .as_str()

            .unwrap()

            .contains("Ce texte ne doit pas devenir la liste")

    );

    assert_eq!(

        request(&app, "GET", "/fr/page/g33-blog?page=2", None, Value::Null)

            .await

            .0,

        200

    );

    assert!(

        request(&app, "GET", "/fr", None, Value::Null)

            .await

            .1

            .as_str()

            .unwrap()

            .contains("Page Parente")

    );

    sqlx::query("UPDATE entries SET status='draft' WHERE id=$1")

        .bind(blog_page.id)

        .execute(&db)

        .await

        .unwrap();

    assert_eq!(

        request(&app, "GET", "/fr/page/g33-blog", None, Value::Null)

            .await

            .0,

        404

    );



    // G3.3-D : Flux RSS 2.0 et Atom 1.0 effectifs avec limitation et extrait/complet

    let feed_rss = request(&app, "GET", "/fr/feed", None, Value::Null).await;

    assert_eq!(feed_rss.0, 200);

    let rss_body = feed_rss.1.as_str().unwrap();

    assert!(rss_body.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));

    assert!(rss_body.contains("<rss version=\"2.0\""));

    assert!(rss_body.contains("<item>"));

    assert!(!rss_body.contains("G33Needle SECRET"));



    let feed_atom = request(&app, "GET", "/fr/feed/atom", None, Value::Null).await;

    assert_eq!(feed_atom.0, 200);

    let atom_body = feed_atom.1.as_str().unwrap();

    assert!(atom_body.contains("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));

    assert!(atom_body.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"));

    assert!(atom_body.contains("<entry>"));

    assert!(!atom_body.contains("G33Needle SECRET"));



    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.posts_per_rss": 2, "reading.rss_use_excerpt": true})

        )

        .await

        .0,

        200

    );

    let feed_rss_excerpt = request(&app, "GET", "/feed", None, Value::Null).await;

    assert_eq!(feed_rss_excerpt.0, 200);

    let rss_excerpt_body = feed_rss_excerpt.1.as_str().unwrap();

    assert!(!rss_excerpt_body.contains("<content:encoded>"));

    assert_eq!(rss_excerpt_body.matches("<item>").count(), 2);

    // G3.3-E : SEO structurel, robots.txt, sitemap.xml et canonical
    // 1. ?tat blog_public == false (configur? ? l'?tape ant?rieure)
    let robots_private = request(&app, "GET", "/robots.txt", None, Value::Null).await;
    assert_eq!(robots_private.0, 200);
    assert_eq!(robots_private.1.as_str().unwrap(), "User-agent: *\nDisallow: /\n");

    let sitemap_private = request(&app, "GET", "/sitemap.xml", None, Value::Null).await;
    assert_eq!(sitemap_private.0, 404);

    let home_private = request(&app, "GET", "/fr", None, Value::Null).await;
    assert_eq!(home_private.0, 200);
    assert!(home_private.1.as_str().unwrap().contains("<meta name=\"robots\" content=\"noindex, nofollow\">"));

    // 2. Passage ? blog_public == true : robots.txt permissif avec sitemap et disallow admin/api
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({"reading.blog_public": true})
        )
        .await
        .0,
        200
    );

    let robots = request(&app, "GET", "/robots.txt", None, Value::Null).await;
    assert_eq!(robots.0, 200);
    let robots_txt = robots.1.as_str().unwrap();
    assert!(robots_txt.contains("User-agent: *"));
    assert!(robots_txt.contains("Disallow: /admin/"));
    assert!(robots_txt.contains("Allow: /"));
    assert!(robots_txt.contains("Sitemap: http://localhost:8080/sitemap.xml"));

    let sitemap = request(&app, "GET", "/sitemap.xml", None, Value::Null).await;
    assert_eq!(sitemap.0, 200);
    let sitemap_xml = sitemap.1.as_str().unwrap();
    assert!(sitemap_xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert!(sitemap_xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
    assert!(sitemap_xml.contains("<loc>http://localhost:8080/fr</loc>"));
    assert!(!sitemap_xml.contains("G33Needle SECRET"));

    let home_html = request(&app, "GET", "/fr", None, Value::Null).await;
    assert_eq!(home_html.0, 200);
    assert!(home_html.1.as_str().unwrap().contains("<link rel=\"canonical\" href=\"http://localhost:8080/fr\" />"));
    assert!(!home_html.1.as_str().unwrap().contains("<meta name=\"robots\" content=\"noindex, nofollow\">"));

    // G3.3-F : Structure des permaliens (WP-90) et bases de taxonomies (WP-91)
    let permalink_post = entries::create(
        &db,
        &NoopHooks,
        "post",
        author.id,
        input(json!({
            "title": "Article Permalien G33F",
            "slug": "article-permalien-g33f",
            "status": "published",
            "body": "Contenu du test de permaliens",
            "term_ids": [archive_term]
        })),
    )
    .await
    .unwrap();

    // 1. Validation des réglages de permaliens (collisions, tokens invalides)
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({"permalink.structure": "/%year%/%monthnum%/"})
        )
        .await
        .0,
        422
    );

    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({"permalink.category_base": "admin"})
        )
        .await
        .0,
        422
    );

    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "permalink.category_base": "rubrique",
                "permalink.tag_base": "rubrique"
            })
        )
        .await
        .0,
        422
    );

    // 2. Configuration des bases personnalisées (WP-91)
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "permalink.category_base": "rubrique",
                "permalink.tag_base": "mot-cle"
            })
        )
        .await
        .0,
        200
    );

    let old_cat_archive = request(&app, "GET", "/fr/archive/category/g33-public", None, Value::Null).await;
    assert_eq!(old_cat_archive.0, 308);

    let new_cat_archive = request(&app, "GET", "/fr/archive/rubrique/g33-public", None, Value::Null).await;
    assert_eq!(new_cat_archive.0, 200);
    assert!(new_cat_archive.1.as_str().unwrap().contains("Article Permalien G33F"));

    // 3. Permalien par défaut plain (/fr/post/{slug})
    let plain_resp = request(
        &app,
        "GET",
        "/fr/post/article-permalien-g33f",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(plain_resp.0, 200);

    // 4. Passage en day_and_name : /fr/post/{slug} redirige 308 vers /fr/{year}/{month}/{day}/{slug}
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({"permalink.structure": "day_and_name"})
        )
        .await
        .0,
        200
    );

    let old_url_resp = request(
        &app,
        "GET",
        "/fr/post/article-permalien-g33f",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(old_url_resp.0, 308);

    let pub_date = permalink_post.published_at.unwrap_or(permalink_post.created_at);
    let expected_day_path = format!(
        "/fr/{:04}/{:02}/{:02}/article-permalien-g33f",
        pub_date.year(),
        pub_date.month(),
        pub_date.day()
    );

    let day_name_resp = request(&app, "GET", &expected_day_path, None, Value::Null).await;
    assert_eq!(day_name_resp.0, 200);
    assert!(day_name_resp.1.as_str().unwrap().contains("Article Permalien G33F"));
    assert!(day_name_resp.1.as_str().unwrap().contains(&expected_day_path));

    // 5. Permalien numérique : archives/{id}
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({"permalink.structure": "numeric"})
        )
        .await
        .0,
        200
    );

    let numeric_path = format!("/fr/archives/{}", permalink_post.id);
    let numeric_resp = request(&app, "GET", &numeric_path, None, Value::Null).await;
    assert_eq!(numeric_resp.0, 200);
    assert!(numeric_resp.1.as_str().unwrap().contains("Article Permalien G33F"));

    let redirect_to_numeric = request(&app, "GET", &expected_day_path, None, Value::Null).await;
    assert_eq!(redirect_to_numeric.0, 308);

    // 6. Rétablissement de la structure plain et réinitialisation des bases
    assert_eq!(
        request(
            &app,
            "PUT",
            "/api/v1/settings",
            Some(&admin),
            json!({
                "permalink.structure": "plain",
                "permalink.category_base": "",
                "permalink.tag_base": ""
            })
        )
        .await
        .0,
        200
    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/settings",

            Some(&admin),

            json!({"reading.page_for_posts":null,"reading.per_page":10})

        )

        .await

        .0,

        200

    );



    let proposal=request(&app,"POST","/api/v1/ai/execute",Some(&token),json!({"tool_name":"create_or_update_content","arguments":{"id":entry.id,"title":"AI approved"}})).await;

    assert_eq!(proposal.0, 200, "{proposal:?}");

    assert_eq!(entries::get(&db, entry.id).await.unwrap().title, "Original");

    let approve = format!(

        "/api/v1/ai/plans/{}/approve",

        proposal.1["data"]["plan_id"].as_str().unwrap()

    );

    assert_eq!(

        request(&app, "POST", &approve, Some(&other_token), json!({}))

            .await

            .0,

        403

    );

    let approved = request(&app, "POST", &approve, Some(&token), json!({})).await;

    assert_eq!(approved.1["success"], true, "{approved:?}");

    assert_eq!(

        request(&app, "POST", &approve, Some(&token), json!({}))

            .await

            .1,

        approved.1

    );

    let proposal = request(

        &app,

        "POST",

        "/api/v1/ai/execute",

        Some(&token),

        json!({"tool_name":"create_or_update_content","arguments":{"id":entry.id,"title":"Stale"}}),

    )

    .await;

    request(

        &app,

        "PUT",

        &endpoint,

        Some(&token),

        json!({"title":"Human edit"}),

    )

    .await;

    let stale = format!(

        "/api/v1/ai/plans/{}/approve",

        proposal.1["data"]["plan_id"].as_str().unwrap()

    );

    assert_eq!(

        request(&app, "POST", &stale, Some(&token), json!({}))

            .await

            .1["success"],

        false

    );

    assert_eq!(

        entries::get(&db, entry.id).await.unwrap().title,

        "Human edit"

    );

    // Checkout never accepts a test payment; manual completion is privileged and idempotent.

    let expensive = core::ecommerce::create_product(

        &db,

        serde_json::from_value(

            json!({"title":"Limit","price_cents":2147483647,"status":"published"}),

        )

        .unwrap(),

    )

    .await

    .unwrap();

    assert!(core::ecommerce::checkout(&db,serde_json::from_value(json!({"customer_email":"buyer@example.test","items":[{"product_id":expensive.id},{"product_id":expensive.id}]})).unwrap()).await.is_err());

    assert!(

        core::ecommerce::create_discount(

            &db,

            serde_json::from_value(

                json!({"code":"INVALID","amount":101,"discount_type":"percentage"})

            )

            .unwrap()

        )

        .await

        .is_err()

    );

    let product=core::ecommerce::create_product(&db,serde_json::from_value(json!({"title":"File","price_cents":500,"file_path":"file.pdf","download_limit":1,"status":"published"})).unwrap()).await.unwrap();

    assert!(

        core::ecommerce::create_product(

            &db,

            serde_json::from_value(

                json!({"title":"Bad","price_cents":500,"file_path":"../secret"})

            )

            .unwrap()

        )

        .await

        .is_err()

    );

    let checkout = json!({"customer_email":"buyer@example.test","items":[{"product_id":product.id}],"gateway":"test"});

    assert!(

        core::ecommerce::checkout(&db, serde_json::from_value(checkout.clone()).unwrap())

            .await

            .is_err()

    );

    let mut checkout = checkout;

    checkout["gateway"] = json!("manual");

    let order = core::ecommerce::checkout(&db, serde_json::from_value(checkout).unwrap())

        .await

        .unwrap();

    assert_eq!(

        serde_json::to_value(&order.order).unwrap()["status"],

        "pending"

    );

    let complete = format!("/api/v1/ecommerce/orders/{}/complete", order.order.id);

    assert_eq!(

        request(&app, "POST", &complete, Some(&token), json!({}))

            .await

            .0,

        403

    );

    assert_eq!(

        request(&app, "POST", &complete, Some(&admin), json!({}))

            .await

            .0,

        200

    );

    assert_eq!(

        request(&app, "POST", &complete, Some(&admin), json!({}))

            .await

            .0,

        200

    );

    let sales: i32 = sqlx::query_scalar("SELECT sales_count FROM ecommerce_products WHERE id=$1")

        .bind(product.id)

        .fetch_one(&db)

        .await

        .unwrap();

    assert_eq!(sales, 1);

    let download = core::ecommerce::generate_download_token(order.items[0].id, SECRET, 1);

    let (a, b) = tokio::join!(

        core::ecommerce::verify_and_consume_download(&db, &download, SECRET),

        core::ecommerce::verify_and_consume_download(&db, &download, SECRET)

    );

    assert_ne!(a.is_ok(), b.is_ok());

    assert!(

        core::ecommerce::verify_and_consume_download(&db, &download, "wrong")

            .await

            .is_err()

    );

    assert_eq!(

        request(

            &app,

            "POST",

            &format!("/api/v1/ecommerce/orders/items/{}/token", order.items[0].id),

            Some(&other_token),

            json!({})

        )

        .await

        .0,

        403

    );

    // Import source mapping detects a changed target instead of overwriting human work.

    // Also verify RFIR JSON and RF-XML SQL export/import roundtrip for native documents (blocks, IDs, body, HTML, vault).

    let doc = DataMorphExporter::dump_database(&db).await.unwrap();

    let exported_blocks = &doc.metadata[&format!("entry_blocks:{native_id}")];

    assert_eq!(exported_blocks, &native_doc_v1);

    let rfir_json = DataMorphExporter::to_json(&doc).unwrap();

    let mut parsed_json: core::migration::RFIRDocument = serde_json::from_str(&rfir_json).unwrap();

    parsed_json.source_site_url = Some("https://wp.example.test".into());

    for e in &mut parsed_json.entries {

        e.slug = format!("import-{}", e.slug);

    }

    DataMorphImporter::apply(&db, &parsed_json, admin_id)

        .await

        .unwrap();

    DataMorphImporter::apply(&db, &parsed_json, admin_id)

        .await

        .unwrap();

    let imported_native_id: Uuid =

        sqlx::query_scalar("SELECT id FROM entries WHERE slug='import-article-natif-g2'")

            .fetch_one(&db)

            .await

            .unwrap();

    let imported_native = entries::get(&db, imported_native_id).await.unwrap();

    assert_eq!(imported_native.blocks, native_doc_v1);

    assert_eq!(imported_native.body, preview_body);

    assert_eq!(imported_native.body_html, preview_html);

    assert_eq!(

        imported_native.fields["_rf_vault"]["origin_key"],

        "preserve_me"

    );



    // Also verify RF-XML export -> WxrImporter::parse -> DataMorphImporter::apply SQL roundtrip.

    let mut xml_source_doc = DataMorphExporter::dump_database(&db).await.unwrap();

    xml_source_doc.source_site_url = Some("https://rfxml.example.test".into());

    xml_source_doc

        .entries

        .retain(|e| e.source_id == native_id.to_string());

    xml_source_doc.entries[0].slug = "rfxml-article-natif-g2".into();

    let rf_xml_str = DataMorphExporter::to_rf_xml(&xml_source_doc).unwrap();

    let parsed_xml_doc =

        core::migration::wxr_importer::WxrImporter::parse(Cursor::new(rf_xml_str)).unwrap();

    DataMorphImporter::apply(&db, &parsed_xml_doc, admin_id)

        .await

        .unwrap();

    let xml_imported_id: Uuid =

        sqlx::query_scalar("SELECT id FROM entries WHERE slug='rfxml-article-natif-g2'")

            .fetch_one(&db)

            .await

            .unwrap();

    let xml_imported = entries::get(&db, xml_imported_id).await.unwrap();

    assert_eq!(xml_imported.blocks, native_doc_v1);

    assert_eq!(xml_imported.body, preview_body);

    assert_eq!(xml_imported.body_html, preview_html);

    assert_eq!(

        xml_imported.fields["_rf_vault"]["origin_key"],

        "preserve_me"

    );



    let imported: Uuid = sqlx::query_scalar(

        "SELECT entry_id FROM import_origins WHERE source='https://wp.example.test' LIMIT 1",

    )

    .fetch_one(&db)

    .await

    .unwrap();

    sqlx::query("UPDATE entries SET updated_at=now(),title='Keep human edit' WHERE id=$1")

        .bind(imported)

        .execute(&db)

        .await

        .unwrap();

    assert!(

        DataMorphImporter::apply(&db, &parsed_json, admin_id)

            .await

            .is_err()

    );

    assert_eq!(

        entries::get(&db, imported).await.unwrap().title,

        "Keep human edit"

    );

    // Native capsule includes module data and physical roots, and rejects overwrite.

    let form:Uuid=sqlx::query_scalar("INSERT INTO forms(name,slug,fields,created_by) VALUES('Contact','contact',$1,$2) RETURNING id").bind(json!({"type":"object","properties":{}})).bind(admin_id).fetch_one(&db).await.unwrap();

    core::forms::submit(

        &db,

        form,

        serde_json::from_value(json!({"data":{}})).unwrap(),

    )

    .await

    .unwrap();

    let gallery: Uuid = sqlx::query_scalar(

        "INSERT INTO galleries(name,slug) VALUES('Gallery','gallery') RETURNING id",

    )

    .fetch_one(&db)

    .await

    .unwrap();

    sqlx::query("INSERT INTO gallery_items(gallery_id,media_id) VALUES($1,$2)")

        .bind(gallery)

        .bind(media)

        .execute(&db)

        .await

        .unwrap();

    let course: Uuid = sqlx::query_scalar(

        "INSERT INTO lms_courses(title,slug,product_id) VALUES('Course','course',$1) RETURNING id",

    )

    .bind(product.id)

    .fetch_one(&db)

    .await

    .unwrap();

    sqlx::query("UPDATE ecommerce_products SET course_id=$1 WHERE id=$2")

        .bind(course)

        .bind(product.id)

        .execute(&db)

        .await

        .unwrap();

    let module: Uuid = sqlx::query_scalar(

        "INSERT INTO lms_modules(course_id,title) VALUES($1,'Module') RETURNING id",

    )

    .bind(course)

    .fetch_one(&db)

    .await

    .unwrap();

    sqlx::query("INSERT INTO lms_lessons(course_id,module_id,title,slug,content) VALUES($1,$2,'Lesson','lesson','Private lesson')").bind(course).bind(module).execute(&db).await.unwrap();

    assert_eq!(

        request(

            &app,

            "GET",

            &format!("/api/v1/lms/courses/{course}/lessons"),

            Some(&reader_token),

            Value::Null

        )

        .await

        .0,

        403

    );



    // G1 Modules & Navigation HTTP regressions:

    // 1. Author and Subscriber cannot PUT /api/v1/admin/modules (403) and do not receive full admin catalog.

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/admin/modules",

            Some(&token),

            json!({"lms": false})

        )

        .await

        .0,

        403

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/admin/modules",

            Some(&reader_token),

            json!({"lms": false})

        )

        .await

        .0,

        403

    );

    assert_eq!(

        request(

            &app,

            "PUT",

            "/api/v1/admin/modules",

            Some(&admin),

            json!({"unknown_module": true})

        )

        .await

        .0,

        400

    );

    let author_nav = request(

        &app,

        "GET",

        "/api/v1/admin/navigation",

        Some(&token),

        Value::Null,

    )

    .await;

    assert_eq!(author_nav.0, 200);

    assert!(author_nav.1["catalog"].as_array().unwrap().is_empty());

    assert!(

        !author_nav.1["items"]

            .as_array()

            .unwrap()

            .iter()

            .any(|n| n["id"] == "rf_lms" || n["id"] == "rf_settings")

    );



    // 2. Zen mode cache invalidation & module guard: warm SSR cache while zen_mode is active, then disable zen_mode.

    let zen_url = "/fr/post/article-natif-g2/zen";

    let zen_active = request(&app, "GET", zen_url, None, Value::Null).await;

    assert_eq!(zen_active.0, 200);

    assert!(zen_active.1.as_str().unwrap().contains("Titre natif V1"));

    let disable_zen = request(

        &app,

        "PUT",

        "/api/v1/admin/modules",

        Some(&admin),

        json!({"zen_mode": false}),

    )

    .await;

    assert_eq!(disable_zen.0, 200);

    assert_eq!(disable_zen.1["modules"]["zen_mode"], false);

    assert_eq!(

        request(&app, "GET", zen_url, None, Value::Null).await.0,

        404

    );



    // 3. Disabling LMS removes it from navigation and closes both admin and public LMS APIs (404),

    // while preserving underlying course/module/lesson data upon reactivation.

    let disable_lms = request(

        &app,

        "PUT",

        "/api/v1/admin/modules",

        Some(&admin),

        json!({"lms": false}),

    )

    .await;

    assert_eq!(disable_lms.0, 200);

    assert_eq!(disable_lms.1["modules"]["lms"], false);

    assert!(

        !disable_lms.1["items"]

            .as_array()

            .unwrap()

            .iter()

            .any(|n| n["module"] == "lms")

    );

    assert!(

        disable_lms.1["catalog"]

            .as_array()

            .unwrap()

            .iter()

            .any(|n| n["id"] == "rf_lms")

    );

    assert_eq!(

        request(

            &app,

            "GET",

            "/api/v1/lms/courses",

            Some(&admin),

            Value::Null

        )

        .await

        .0,

        404

    );

    assert_eq!(

        request(&app, "GET", "/api/v1/public/lms/courses", None, Value::Null)

            .await

            .0,

        404

    );

    // Persisted in DB across new router/state instantiation ("restart").

    let restarted_app = crate::app(db.clone(), &root).await;

    let nav_after_restart = request(

        &restarted_app,

        "GET",

        "/api/v1/admin/navigation",

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(nav_after_restart.0, 200);

    assert_eq!(nav_after_restart.1["modules"]["lms"], false);

    assert_eq!(nav_after_restart.1["modules"]["zen_mode"], false);



    // 4. Atomic concurrent updates to distinct module flags merge both changes without clobbering.

    let (mod_a, mod_b) = tokio::join!(

        request(

            &app,

            "PUT",

            "/api/v1/admin/modules",

            Some(&admin),

            json!({"forms": false})

        ),

        request(

            &app,

            "PUT",

            "/api/v1/admin/modules",

            Some(&admin),

            json!({"lms": true, "zen_mode": true})

        )

    );

    assert_eq!(mod_a.0, 200);

    assert_eq!(mod_b.0, 200);

    let merged_nav = request(

        &app,

        "GET",

        "/api/v1/admin/navigation",

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(merged_nav.0, 200);

    assert_eq!(merged_nav.1["modules"]["forms"], false);

    assert_eq!(merged_nav.1["modules"]["lms"], true);

    assert_eq!(merged_nav.1["modules"]["zen_mode"], true);

    assert_eq!(merged_nav.1["modules"]["galleries"], true);

    assert_eq!(merged_nav.1["modules"]["ecommerce"], true);

    // Reactivated LMS still has its course and lessons intact.

    let courses_after = request(

        &app,

        "GET",

        "/api/v1/lms/courses",

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(courses_after.0, 200);

    assert_eq!(courses_after.1.as_array().unwrap().len(), 1);

    let lessons_after = request(

        &app,

        "GET",

        &format!("/api/v1/lms/courses/{course}/lessons"),

        Some(&admin),

        Value::Null,

    )

    .await;

    assert_eq!(lessons_after.0, 200);

    assert_eq!(lessons_after.1.as_array().unwrap().len(), 1);



    std::fs::write(root.join("uploads/example.txt"), b"physical-media").unwrap();

    std::fs::write(root.join("protected/file.pdf"), b"private-product").unwrap();

    assert_eq!(

        request(&app, "GET", "/api/v1/forms", Some(&admin), Value::Null)

            .await

            .0,

        404

    );

    // ========================================================================
    // WP-42 à WP-45, WP-82 à WP-87 : COMMENTAIRES, MODÉRATION & RGPD
    // ========================================================================
    // 1. Rejet honeypot anti-bot (WP-45)
    let hp_res = request(
        &app,
        "POST",
        &format!("/api/v1/public/entries/{}/comments", permalink_post.id),
        None,
        json!({
            "author_name": "Robot Spammer",
            "author_email": "bot@spam.test",
            "content": "Achetez des pilules magiques",
            "website_url_hp": "http://spammer.bot"
        }),
    )
    .await;
    assert_eq!(hp_res.0, 400);

    // 2. Soumission d'un commentaire légitime (mis en modération automatique WP-42, WP-84)
    let comment_res = request(
        &app,
        "POST",
        &format!("/api/v1/public/entries/{}/comments", permalink_post.id),
        None,
        json!({
            "author_name": "Jean Lecteur",
            "author_email": "jean.lecteur@example.test",
            "author_url": "https://jean.test",
            "content": "Superbe article sur Rust et la souveraineté !",
        }),
    )
    .await;
    assert_eq!(comment_res.0, 200);
    let created_comment = &comment_res.1;
    let comment_id = created_comment["id"].as_str().unwrap();
    // RGPD / WP-44 : l'e-mail et l'IP ne doivent pas être renvoyés dans la vue publique
    assert!(created_comment.get("author_email").is_none());
    assert!(created_comment.get("author_ip").is_none());
    assert_eq!(created_comment["author_name"], "Jean Lecteur");

    // 3. Vérification de la liste publique : le commentaire en attente ne doit PAS apparaître
    let public_comments = request(
        &app,
        "GET",
        &format!("/api/v1/public/entries/{}/comments", permalink_post.id),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(public_comments.0, 200);
    assert_eq!(public_comments.1.as_array().unwrap().len(), 0);

    // 4. Modération Admin : consultation et approbation (WP-43)
    let admin_comments = request(
        &app,
        "GET",
        "/api/v1/comments?status=pending",
        Some(&admin),
        Value::Null,
    )
    .await;
    assert_eq!(admin_comments.0, 200);
    assert_eq!(admin_comments.1["total"], 1);

    let approve_res = request(
        &app,
        "PUT",
        &format!("/api/v1/comments/{comment_id}/status"),
        Some(&admin),
        json!({ "status": "approved" }),
    )
    .await;
    assert_eq!(approve_res.0, 200);
    assert_eq!(approve_res.1["status"], "approved");

    // 5. Réponse admin en cascade (WP-43, WP-83)
    let reply_res = request(
        &app,
        "POST",
        &format!("/api/v1/comments/{comment_id}/reply"),
        Some(&admin),
        json!({ "content": "Merci Jean pour votre lecture attentive !" }),
    )
    .await;
    assert_eq!(reply_res.0, 200);
    assert_eq!(reply_res.1["parent_id"], comment_id);
    assert_eq!(reply_res.1["status"], "approved");

    // 6. Rendu public SSR : intégration dans single.html et protection RGPD (WP-44)
    let ssr_article = request(
        &app,
        "GET",
        "/fr/post/article-permalien-g33f",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(ssr_article.0, 200);
    let ssr_html = ssr_article.1.as_str().unwrap();
    assert!(ssr_html.contains("Jean Lecteur"));
    assert!(ssr_html.contains("Superbe article sur Rust"));
    assert!(ssr_html.contains("Merci Jean pour votre lecture"));
    // Sécurité RGPD : pas d'e-mail dans le HTML public rendu
    assert!(!ssr_html.contains("jean.lecteur@example.test"));

    // 7. Compteurs, action groupée et vidage de corbeille (WP-45)
    // STAB: public comment links and entry visibility share the public contract.
    let comments_url = format!("/api/v1/public/entries/{}/comments", permalink_post.id);
    let invalid_link = request(&app, "POST", &comments_url, None, json!({
        "author_name":"Test", "author_email":"test@example.test",
        "author_url":"javascript:alert(1)", "content":"Un lien invalide"
    })).await;
    assert_eq!(invalid_link.0, 422, "{invalid_link:?}");
    sqlx::query("UPDATE comments SET author_url='javascript:alert(1)' WHERE id=$1")
        .bind(Uuid::parse_str(comment_id).unwrap()).execute(&db).await.unwrap();
    let sanitized = request(&app, "GET", &comments_url, None, Value::Null).await;
    assert_eq!(sanitized.0, 200);
    assert!(!sanitized.1.to_string().contains("javascript:"));
    sqlx::query("UPDATE comments SET author_url=NULL WHERE id=$1")
        .bind(Uuid::parse_str(comment_id).unwrap()).execute(&db).await.unwrap();
    let previous = entries::get(&db, permalink_post.id).await.unwrap();
    for condition in [
        "status='draft'",
        "status='published', published_at=now()+interval '1 day'",
        "status='published', published_at=now()-interval '1 day', scheduled_unpublish_at=now()-interval '1 hour'",
    ] {
        sqlx::query(&format!("UPDATE entries SET {condition} WHERE id=$1"))
            .bind(permalink_post.id).execute(&db).await.unwrap();
        assert_eq!(request(&app, "GET", &comments_url, None, Value::Null).await.0, 404);
        let denied = request(&app, "POST", &comments_url, None, json!({
            "author_name":"Test", "author_email":"test@example.test", "content":"Publication inaccessible"
        })).await;
        assert_eq!(denied.0, 404, "{denied:?}");
    }
    sqlx::query("UPDATE entries SET status='published', published_at=$2, scheduled_unpublish_at=$3 WHERE id=$1")
        .bind(permalink_post.id).bind(previous.published_at).bind(previous.scheduled_unpublish_at)
        .execute(&db).await.unwrap();

    // Discussion rules must affect HTTP, not just saved settings.
    let before_count: i64 = sqlx::query_scalar("SELECT count(*) FROM comments").fetch_one(&db).await.unwrap();
    let update_setting = |key: &str, value: Value| {
        let mut map = serde_json::Map::new(); map.insert(key.to_string(), value); Value::Object(map)
    };
    let closed = request(&app, "PUT", "/api/v1/settings", Some(&admin),
        update_setting("discussion.default_comment_status", json!(false))).await;
    assert_eq!(closed.0, 200);
    let submit = json!({"author_name":"Invité", "author_email":"invite@example.test", "content":"Commentaire de recette"});
    assert_eq!(request(&app, "POST", &comments_url, None, submit.clone()).await.0, 422);
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.default_comment_status":true,"discussion.comment_registration":true})).await.0, 200);
    assert_eq!(request(&app, "POST", &comments_url, None, submit.clone()).await.0, 401);
    let authenticated = request(&app, "POST", &comments_url, Some(&admin),
        json!({"author_name":"Usurpation", "author_email":"fake@example.test", "content":"Compte authentifié"})).await;
    assert_eq!(authenticated.0, 200, "{authenticated:?}");
    assert_ne!(authenticated.1["author_name"], "Usurpation");
    let authenticated_id=Uuid::parse_str(authenticated.1["id"].as_str().unwrap()).unwrap();
    sqlx::query("DELETE FROM comments WHERE id=$1").bind(authenticated_id).execute(&db).await.unwrap();
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.comment_registration":false,"discussion.require_name_email":false})).await.0, 200);
    let anonymous = request(&app, "POST", &comments_url, None, json!({"content":"Sans identité obligatoire"})).await;
    assert_eq!(anonymous.0, 200, "{anonymous:?}");
    assert_eq!(anonymous.1["author_name"], "Visiteur");
    sqlx::query("DELETE FROM comments WHERE id=$1")
        .bind(Uuid::parse_str(anonymous.1["id"].as_str().unwrap()).unwrap()).execute(&db).await.unwrap();
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.require_name_email":true,"discussion.thread_comments":false})).await.0, 200);
    let no_thread = request(&app, "POST", &comments_url, None,
        json!({"author_name":"Invité", "author_email":"invite@example.test", "content":"Réponse", "parent_id":comment_id})).await;
    assert_eq!(no_thread.0, 422, "{no_thread:?}");
    let flattened=request(&app, "GET", &comments_url, None, Value::Null).await;
    assert!(flattened.1.as_array().unwrap().iter().all(|n| n["children"].as_array().unwrap().is_empty()));
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.thread_comments":true})).await.0, 200);
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.show_avatars":false,"discussion.default_comment_status":false})).await.0, 200);
    let closed_html=request(&app,"GET","/fr/post/article-permalien-g33f",None,Value::Null).await;
    assert_eq!(closed_html.0,200);
    let closed_html=closed_html.1.as_str().unwrap();
    assert!(!closed_html.contains("id=\"comment-form\""));
    assert!(!closed_html.contains("rf-comment-avatar"));
    assert!(closed_html.contains("commentaires sont fermés"));
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.show_avatars":true,"discussion.default_comment_status":true,"discussion.thread_comments_depth":1})).await.0, 200);
    let depth_one=request(&app,"POST",&comments_url,None,
        json!({"author_name":"Invité","author_email":"invite@example.test","content":"Trop profond","parent_id":comment_id})).await;
    assert_eq!(depth_one.0,422);
    let flattened_depth=request(&app,"GET",&comments_url,None,Value::Null).await;
    assert_eq!(flattened_depth.1.as_array().unwrap().len(),2);
    assert_eq!(request(&app, "PUT", "/api/v1/settings", Some(&admin),
        json!({"discussion.thread_comments_depth":3})).await.0, 200);

    // Une page contient un fil complet ; le compteur inclut ses réponses.
    let second_thread = rustforge_core::comments::create_comment(&db, serde_json::from_value(json!({
        "entry_id":permalink_post.id,"author_name":"Deuxième lecteur",
        "author_email":"second@example.test","content":"Second fil de pagination"
    })).unwrap()).await.unwrap();
    sqlx::query("UPDATE comments SET status='approved' WHERE id=$1")
        .bind(second_thread.id).execute(&db).await.unwrap();
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({
        "discussion.page_comments":true,"discussion.comments_per_page":1,
        "discussion.default_comments_page":"oldest","discussion.comment_order":"asc"
    })).await.0,200);
    let first=rustforge_core::comments::get_entry_comment_page(&db,permalink_post.id,Some(1)).await.unwrap();
    assert_eq!((first.root_total,first.total,first.total_pages),(2,3,2));
    assert_eq!(first.items.len(),1);
    assert_eq!(first.items[0].children.len(),1);
    let second=request(&app,"GET",&format!("{comments_url}?comment_page=2"),None,Value::Null).await;
    assert_eq!(second.0,200);
    assert_eq!(second.1[0]["id"],second_thread.id.to_string());
    assert_eq!(request(&app,"GET",&format!("{comments_url}?comment_page=3"),None,Value::Null).await.0,404);
    assert_eq!(request(&app,"GET",&format!("{comments_url}?comment_page=0"),None,Value::Null).await.0,400);
    let second_html=request(&app,"GET","/fr/post/article-permalien-g33f?comment_page=2",None,Value::Null).await;
    assert_eq!(second_html.0,200);
    let second_html=second_html.1.as_str().unwrap();
    assert!(second_html.contains("Second fil de pagination"));
    assert!(!second_html.contains("Superbe article sur Rust"));
    assert!(second_html.contains("Pagination des commentaires"));
    assert!(second_html.contains("(3)"));
    assert_eq!(request(&app,"GET","/fr/post/article-permalien-g33f?comment_page=3",None,Value::Null).await.0,404);
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({"discussion.default_comments_page":"newest"})).await.0,200);
    assert_eq!(request(&app,"GET",&comments_url,None,Value::Null).await.1[0]["id"],second_thread.id.to_string());
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({
        "discussion.default_comments_page":"oldest","discussion.comment_order":"desc"
    })).await.0,200);
    assert_eq!(request(&app,"GET",&comments_url,None,Value::Null).await.1[0]["id"],second_thread.id.to_string());
    sqlx::query("DELETE FROM comments WHERE id=$1").bind(second_thread.id).execute(&db).await.unwrap();
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({
        "discussion.page_comments":false,"discussion.comment_order":"asc",
        "discussion.close_comments_for_old_posts":true,"discussion.close_comments_days":1
    })).await.0,200);
    sqlx::query("UPDATE entries SET published_at=now()-interval '1 day'+interval '5 seconds' WHERE id=$1")
        .bind(permalink_post.id).execute(&db).await.unwrap();
    let warm=request(&app,"GET","/fr/post/article-permalien-g33f",None,Value::Null).await;
    assert_eq!(warm.0,200);
    assert!(warm.1.as_str().unwrap().contains("id=\"comment-form\""));
    tokio::time::sleep(std::time::Duration::from_millis(5500)).await;
    let expired_form=request(&app,"GET","/fr/post/article-permalien-g33f",None,Value::Null).await;
    assert_eq!(expired_form.0,200);
    assert!(!expired_form.1.as_str().unwrap().contains("id=\"comment-form\""));
    assert_eq!(request(&app,"POST",&comments_url,None,submit.clone()).await.0,422);
    sqlx::query("UPDATE entries SET published_at=$2 WHERE id=$1")
        .bind(permalink_post.id).bind(previous.published_at).execute(&db).await.unwrap();
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({"discussion.close_comments_for_old_posts":false})).await.0,200);

    // Session créée depuis l'article : portée commentaires, jamais Studio.
    let public_login = "/api/v1/public/session/login";
    let registered_account = users::get(&db, admin_id).await.unwrap();
    assert_eq!(request(&app,"POST",public_login,None,json!({"login":registered_account.username,"password":"incorrect"})).await.0,401);
    let session=request(&app,"POST",public_login,None,json!({
        "login":registered_account.username,"password":"test-password-long-123!"
    })).await;
    assert_eq!(session.0,200,"{session:?}");
    assert!(session.1.get("user").is_none());
    assert!(session.1.get("email").is_none());
    assert!(session.1.get("role").is_none());
    assert_eq!(session.1["expires_in"],1800);
    let comment_token=session.1["token"].as_str().unwrap();
    assert_eq!(request(&app,"GET","/api/v1/auth/me",Some(comment_token),Value::Null).await.0,401);
    assert_eq!(request(&app,"GET","/api/v1/comments",Some(comment_token),Value::Null).await.0,401);
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({"discussion.comment_registration":true})).await.0,200);
    let account_html=request(&app,"GET","/fr/post/article-permalien-g33f",None,Value::Null).await;
    assert_eq!(account_html.0,200);
    let account_html=account_html.1.as_str().unwrap();
    assert!(account_html.contains("id=\"comment-login\""));
    assert!(account_html.contains("id=\"comment-fields\" disabled"));
    assert!(!account_html.contains(comment_token));
    let scoped=request(&app,"POST",&comments_url,Some(comment_token),json!({"content":"Session limitée", "author_name":"Faux nom"})).await;
    assert_eq!(scoped.0,200,"{scoped:?}");
    assert_eq!(scoped.1["author_name"],registered_account.display_name);
    sqlx::query("DELETE FROM comments WHERE id=$1").bind(Uuid::parse_str(scoped.1["id"].as_str().unwrap()).unwrap()).execute(&db).await.unwrap();
    assert_eq!(request(&app,"PUT","/api/v1/settings",Some(&admin),json!({"discussion.comment_registration":false})).await.0,200);

    let after_count: i64 = sqlx::query_scalar("SELECT count(*) FROM comments").fetch_one(&db).await.unwrap();
    assert_eq!(before_count,after_count);

    let counts_res = request(&app, "GET", "/api/v1/comments/counts", Some(&admin), Value::Null).await;
    assert_eq!(counts_res.0, 200);
    assert_eq!(counts_res.1["approved"], 2);

    let reply_id = reply_res.1["id"].as_str().unwrap();
    let batch_res = request(
        &app,
        "POST",
        "/api/v1/comments/batch",
        Some(&admin),
        json!({ "ids": [comment_id, reply_id], "status": "trash" }),
    )
    .await;
    assert_eq!(batch_res.0, 200);
    assert_eq!(batch_res.1["updated"], 2);

    let empty_trash_res = request(&app, "DELETE", "/api/v1/comments/empty-trash", Some(&admin), Value::Null).await;
    assert_eq!(empty_trash_res.0, 200);
    assert_eq!(empty_trash_res.1["deleted"], 2);

    let paths = StoragePaths {

        uploads: root.join("uploads"),

        themes: root.join("themes"),

        plugins: root.join("plugins"),

        protected: root.join("protected"),

    };

    let archive = root.join("test.rfcapsule");

    // Missing source media must fail explicitly, without leaving a half-written capsule.

    std::fs::rename(root.join("uploads/example.txt"), root.join("example.saved")).unwrap();

    assert!(

        CapsuleManager::pack(&db, &paths, &archive, None)

            .await

            .is_err()

    );

    assert!(!archive.exists());

    std::fs::rename(root.join("example.saved"), root.join("uploads/example.txt")).unwrap();

    CapsuleManager::pack(&db, &paths, &archive, Some("https://source.test".into()))

        .await

        .unwrap();

    let bytes = std::fs::read(&archive).unwrap();

    // Recompute ZIP CRC after tampering, while keeping the original SHA inventory.

    // This proves the application inventory detects changes beyond container integrity.

    let mut source_zip = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();

    let mut corrupt = Cursor::new(Vec::new());

    {

        let mut writer = zip::ZipWriter::new(&mut corrupt);

        for i in 0..source_zip.len() {

            let mut file = source_zip.by_index(i).unwrap();

            let name = file.name().to_string();

            let mut data = Vec::new();

            std::io::Read::read_to_end(&mut file, &mut data).unwrap();

            if name == "uploads/example.txt" {

                data = b"tampered".to_vec();

            }

            writer

                .start_file(name, zip::write::SimpleFileOptions::default())

                .unwrap();

            std::io::Write::write_all(&mut writer, &data).unwrap();

        }

        writer.finish().unwrap();

    }

    assert!(CapsuleManager::inspect(Cursor::new(corrupt.into_inner())).is_err());

    assert_eq!(

        CapsuleManager::inspect(Cursor::new(&bytes))

            .unwrap()

            .version,

        "2.0"

    );

    let dest = root.join("restored");

    let dest_paths = StoragePaths {

        uploads: dest.join("uploads"),

        themes: dest.join("themes"),

        plugins: dest.join("plugins"),

        protected: dest.join("protected"),

    };

    CapsuleManager::unpack(&target, &dest_paths, Cursor::new(&bytes), None, Uuid::nil())

        .await

        .unwrap();

    let before = native_backup::dump(&db).await.unwrap();

    let after = native_backup::dump(&target).await.unwrap();

    for table in native_backup::TABLES {

        assert_eq!(

            before.tables[*table].len(),

            after.tables[*table].len(),

            "{table}"

        );

    }

    assert_eq!(

        entries::get(&target, entry.id).await.unwrap().fields,

        entry.fields

    );

    let restored_native_target = entries::get(&target, native_id).await.unwrap();

    assert_eq!(restored_native_target.blocks, native_doc_v1);

    assert_eq!(restored_native_target.body, preview_body);

    assert_eq!(restored_native_target.body_html, preview_html);

    assert_eq!(

        restored_native_target.fields["_rf_vault"]["origin_key"],

        "preserve_me"

    );

    assert!(users::verify_password(

        "test-password-long-123!",

        &users::get(&target, admin_id).await.unwrap().password_hash

    ));

    assert_eq!(

        std::fs::read(dest.join("protected/file.pdf")).unwrap(),

        b"private-product"

    );

    let restored_course: Option<Uuid> =

        sqlx::query_scalar("SELECT course_id FROM ecommerce_products WHERE id=$1")

            .bind(product.id)

            .fetch_one(&target)

            .await

            .unwrap();

    assert_eq!(restored_course, Some(course));

    assert!(

        CapsuleManager::unpack(&target, &dest_paths, Cursor::new(&bytes), None, Uuid::nil())

            .await

            .is_err()

    );

    // Release HTML must reference an asset actually served under /admin/.

    let index = request(&app, "GET", "/admin/", None, Value::Null).await;

    assert_eq!(index.0, 200);

    let html = index.1.as_str().unwrap();

    let asset = html

        .split("src=\"")

        .nth(1)

        .unwrap()

        .split('"')

        .next()

        .unwrap();

    assert!(asset.starts_with("/admin/assets/"), "{asset}");

    assert_eq!(request(&app, "GET", asset, None, Value::Null).await.0, 200);

    // Explicitly close clients before dropping only the databases created by this test.

    drop(app);

    db.close().await;

    target.close().await;

    for db_name in [name, target_name] {

        assert!(db_name.starts_with("rf_regression_"));

        sqlx::query(&format!("DROP DATABASE {db_name} WITH (FORCE)"))

            .execute(&server)

            .await

            .unwrap();

    }

    server.close().await;

    // Leave temporary test artifacts available for diagnosis; no recursive deletion.

}

