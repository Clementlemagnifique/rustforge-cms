//! Limits exposed operations independently from the administrative UI.
use crate::AppState;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

pub async fn boundaries(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    if path == "/api/v1/auth/login" || path == "/api/v1/setup/install" {
        static ATTEMPTS: OnceLock<Mutex<HashMap<String, (Instant, u32)>>> = OnceLock::new();
        let ip = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip().to_string())
            .unwrap_or_else(|| "local".into());
        let limited = match ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new())).lock() {
            Ok(mut all) => {
                all.retain(|_, (t, _)| t.elapsed() < Duration::from_secs(60));
                if all.len() >= 10_000 && !all.contains_key(&ip) {
                    true
                } else {
                    let value = all.entry(ip).or_insert((Instant::now(), 0));
                    value.1 += 1;
                    value.1 > 20
                }
            }
            Err(_) => true,
        };
        if limited {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Trop de tentatives. Réessayez dans une minute.",
            )
                .into_response();
        }
    }
    let module = path.strip_prefix("/api/v1/").and_then(|path| {
        path.split('/')
            .find(|segment| matches!(*segment, "ecommerce" | "lms" | "forms" | "galleries"))
    });
    if let Some(module) = module
        && !rustforge_core::settings::is_module_active(&state.db, module).await
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    next.run(request).await
}
