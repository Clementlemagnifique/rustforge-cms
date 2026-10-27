//! # RustForge API
//!
//! Routeur HTTP REST Axum, middleware d'authentification et exposition des endpoints admin et publics.

pub mod auth;
pub mod error;
pub mod routes;
pub mod ssr;
pub mod state;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{Method, header},
    routing::{delete, get, post, put},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

pub use state::AppState;

/// Construit le routeur HTTP complet pour RustForge.
pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    let auth_routes = Router::new()
        .route("/login", post(routes::auth::login))
        .route("/me", get(routes::auth::me));

    let entries_routes = Router::new()
        .route("/", get(routes::entries::list).post(routes::entries::create))
        .route("/{id}", get(routes::entries::get).put(routes::entries::update).delete(routes::entries::delete))
        .route("/{id}/revisions", get(routes::entries::list_revisions))
        .route("/{id}/revisions/{rev_id}/restore", post(routes::entries::restore_revision));

    let content_types_routes = Router::new()
        .route("/", get(routes::content_types::list).post(routes::content_types::create))
        .route("/{id}", get(routes::content_types::get).put(routes::content_types::update).delete(routes::content_types::delete));

    let taxonomies_routes = Router::new()
        .route("/", get(routes::taxonomies::list_taxonomies).post(routes::taxonomies::create_taxonomy))
        .route("/{id}", delete(routes::taxonomies::delete_taxonomy));

    let terms_routes = Router::new()
        .route("/", get(routes::taxonomies::list_terms).post(routes::taxonomies::create_term))
        .route("/{id}", put(routes::taxonomies::update_term).delete(routes::taxonomies::delete_term));

    let media_routes = Router::new()
        .route("/", get(routes::media::list).post(routes::media::upload))
        .route("/folders", get(routes::media::list_folders).post(routes::media::create_folder))
        .route("/folders/{id}", delete(routes::media::delete_folder))
        .route("/{id}", put(routes::media::update).delete(routes::media::delete));

    let forms_routes = Router::new()
        .route("/", get(routes::forms::list).post(routes::forms::create))
        .route("/{id}", get(routes::forms::get).put(routes::forms::update).delete(routes::forms::delete))
        .route("/{id}/submissions", get(routes::forms::list_submissions))
        .route("/submissions/{sub_id}/read", post(routes::forms::mark_read));

    let galleries_routes = Router::new()
        .route("/", get(routes::galleries::list).post(routes::galleries::create))
        .route("/{id}", get(routes::galleries::get).delete(routes::galleries::delete))
        .route("/{id}/items", post(routes::galleries::add_item))
        .route("/{gallery_id}/items/{item_id}", delete(routes::galleries::delete_item));

    let ecommerce_routes = Router::new()
        .route("/products", get(routes::ecommerce::list_products).post(routes::ecommerce::create_product))
        .route("/products/{id}", get(routes::ecommerce::get_product).put(routes::ecommerce::update_product).delete(routes::ecommerce::delete_product))
        .route("/discounts", get(routes::ecommerce::list_discounts).post(routes::ecommerce::create_discount))
        .route("/orders", get(routes::ecommerce::list_orders))
        .route("/orders/{id}", get(routes::ecommerce::get_order))
        .route("/orders/{id}/complete", post(routes::ecommerce::complete_order))
        .route("/orders/items/{item_id}/token", post(routes::ecommerce::generate_download_link));

    let lms_routes = Router::new()
        .route("/courses", get(routes::lms::list_courses).post(routes::lms::create_course))
        .route("/courses/{id}", get(routes::lms::get_course).put(routes::lms::update_course).delete(routes::lms::delete_course))
        .route("/courses/{id}/modules", get(routes::lms::list_modules).post(routes::lms::create_module))
        .route("/courses/{id}/lessons", get(routes::lms::list_lessons).post(routes::lms::create_lesson))
        .route("/my-courses", get(routes::lms::my_courses))
        .route("/courses/{id}/progress", post(routes::lms::update_progress));

    let users_routes = Router::new()
        .route("/", get(routes::users::list).post(routes::users::create))
        .route("/{id}", put(routes::users::update).delete(routes::users::delete));

    let settings_routes = Router::new()
        .route("/", get(routes::settings::get_all).put(routes::settings::update));

    let plugins_routes = Router::new()
        .route("/", get(routes::plugins::list))
        .route("/rescan", post(routes::plugins::rescan))
        .route("/{id}/enable", post(routes::plugins::enable))
        .route("/{id}/disable", post(routes::plugins::disable));

    let stats_routes = Router::new()
        .route("/dashboard", get(routes::stats::get_dashboard));

    let ai_routes = Router::new()
        .route("/tools", get(routes::ai::list_tools))
        .route("/execute", post(routes::ai::execute_tool))
        .route("/status", get(routes::ai::status))
        .route("/chat", post(routes::ai::chat));

    let public_routes = Router::new()
        .route("/entries", get(routes::public::list_entries))
        .route("/entries/{type}/{slug}", get(routes::public::get_entry))
        .route("/settings", get(routes::public::get_settings))
        .route("/terms", get(routes::public::list_terms))
        .route("/forms/{id}/submit", post(routes::forms::public_submit))
        .route("/galleries/{slug}", get(routes::galleries::public_get_by_slug))
        .route("/ecommerce/products", get(routes::ecommerce::public_list_products))
        .route("/ecommerce/products/{slug}", get(routes::ecommerce::public_get_product_by_slug))
        .route("/ecommerce/checkout", post(routes::ecommerce::public_checkout))
        .route("/ecommerce/download/{token}", get(routes::ecommerce::public_download))
        .route("/lms/courses", get(routes::lms::public_list_courses))
        .route("/lms/courses/{slug}", get(routes::lms::public_get_course_by_slug));

    let converter_routes = Router::new()
        .route("/analyze", post(routes::converter::analyze))
        .route("/convert-theme", post(routes::converter::convert_theme))
        .route("/forms-plan", post(routes::converter::forms_plan))
        .route("/apply-plan", post(routes::converter::apply_plan))
        .route("/import-wxr", post(routes::converter::import_wxr));

    let setup_routes = Router::new()
        .route("/status", get(routes::setup::status))
        .route("/test-db", post(routes::setup::test_db))
        .route("/install", post(routes::setup::install))
        .route("/restore-capsule", post(routes::setup::restore_capsule));

    let capsules_routes = Router::new()
        .route("/export", get(routes::capsules::export_capsule))
        .route("/inspect", post(routes::capsules::inspect_capsule))
        .route("/restore", post(routes::capsules::restore_capsule));

    let api_v1 = Router::new()
        .nest("/setup", setup_routes)
        .nest("/capsules", capsules_routes)
        .nest("/auth", auth_routes)
        .nest("/entries", entries_routes)
        .nest("/content-types", content_types_routes)
        .nest("/taxonomies", taxonomies_routes)
        .nest("/terms", terms_routes)
        .nest("/media", media_routes)
        .nest("/forms", forms_routes)
        .nest("/galleries", galleries_routes)
        .nest("/ecommerce", ecommerce_routes)
        .nest("/lms", lms_routes)
        .nest("/users", users_routes)
        .nest("/settings", settings_routes)
        .nest("/plugins", plugins_routes)
        .nest("/stats", stats_routes)
        .nest("/ai", ai_routes)
        .nest("/converter", converter_routes)
        .nest("/public", public_routes);

    let mut router = Router::new()
        // Routes publiques SSR (< 1ms via cache Moka L1)
        .route("/", get(routes::public::root_redirect))
        .route("/{locale}", get(routes::public::ssr_home))
        .route("/{locale}/{content_type}/{slug}", get(routes::public::ssr_single))
        .route("/{locale}/{content_type}/{slug}/zen", get(routes::public::ssr_zen))
        // API REST v1
        .nest("/api/v1", api_v1)
        .nest_service("/uploads", ServeDir::new(&state.uploads_dir))
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    if let Some(admin_dir) = &state.admin_dist_dir {
        if admin_dir.exists() {
            router = router.nest_service("/admin", ServeDir::new(admin_dir));
        }
    }

    router
}
