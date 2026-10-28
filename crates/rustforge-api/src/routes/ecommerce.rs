use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};
use rustforge_core::ecommerce::{
    self, CheckoutPayload, Discount, DiscountInput, Order, OrderWithItems, Product, ProductInput,
};

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Administration : Produits
// ---------------------------------------------------------------------------

pub async fn list_products(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Product>>> {
    rustforge_core::security::require_editor(_auth.user.role)?;
    let products = ecommerce::list_products(&state.db, false).await?;
    Ok(Json(products))
}

pub async fn get_product(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Product>> {
    rustforge_core::security::require_editor(_auth.user.role)?;
    let product = ecommerce::get_product(&state.db, id).await?;
    Ok(Json(product))
}

pub async fn create_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<ProductInput>,
) -> ApiResult<Json<Product>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let product = ecommerce::create_product(&state.db, input).await?;
    Ok(Json(product))
}

pub async fn update_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ProductInput>,
) -> ApiResult<Json<Product>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let product = ecommerce::update_product(&state.db, id, input).await?;
    Ok(Json(product))
}

pub async fn delete_product(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<()>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    ecommerce::delete_product(&state.db, id).await?;
    Ok(Json(()))
}

// ---------------------------------------------------------------------------
// Administration : Réductions (Discounts)
// ---------------------------------------------------------------------------

pub async fn list_discounts(
    State(state): State<AppState>,
    _auth: AuthUser,
) -> ApiResult<Json<Vec<Discount>>> {
    rustforge_core::security::require_editor(_auth.user.role)?;
    let discounts = ecommerce::list_discounts(&state.db).await?;
    Ok(Json(discounts))
}

pub async fn create_discount(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<DiscountInput>,
) -> ApiResult<Json<Discount>> {
    if !auth.user.role.can_edit_others() {
        return Err(ApiError::Forbidden);
    }
    let discount = ecommerce::create_discount(&state.db, input).await?;
    Ok(Json(discount))
}

// ---------------------------------------------------------------------------
// Administration : Commandes
// ---------------------------------------------------------------------------

pub async fn list_orders(
    State(state): State<AppState>,
    _auth: AuthUser,
    Query(query): Query<ListOrdersQuery>,
) -> ApiResult<Json<Vec<Order>>> {
    rustforge_core::security::require_editor(_auth.user.role)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);
    let orders = ecommerce::list_orders(&state.db, limit, offset).await?;
    Ok(Json(orders))
}

pub async fn get_order(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<OrderWithItems>> {
    let order = ecommerce::get_order(&state.db, id).await?;
    if !_auth.user.role.can_edit_others() && order.order.user_id != Some(_auth.user.id) {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(order))
}

pub async fn complete_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<OrderWithItems>> {
    if !auth.user.role.is_admin() {
        return Err(ApiError::Forbidden);
    }
    let order = ecommerce::complete_order(&state.db, id).await?;
    Ok(Json(order))
}

#[derive(Debug, Serialize)]
pub struct DownloadLinkResponse {
    pub download_url: String,
    pub token: String,
}

pub async fn generate_download_link(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(item_id): Path<Uuid>,
) -> ApiResult<Json<DownloadLinkResponse>> {
    let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ecommerce_order_items i JOIN ecommerce_orders o ON o.id = i.order_id WHERE i.id = $1 AND o.status = 'completed' AND ($2 OR o.user_id = $3))")
        .bind(item_id).bind(_auth.user.role.can_edit_others()).bind(_auth.user.id).fetch_one(&state.db).await.map_err(rustforge_core::CoreError::from)?;
    if !allowed {
        return Err(ApiError::Forbidden);
    }
    let token = ecommerce::generate_download_token(item_id, &state.jwt_secret, 48);
    let download_url = format!("/api/v1/public/ecommerce/download/{token}");
    Ok(Json(DownloadLinkResponse {
        download_url,
        token,
    }))
}

// ---------------------------------------------------------------------------
// Endpoints publics E-Commerce
// ---------------------------------------------------------------------------

pub async fn public_list_products(State(state): State<AppState>) -> ApiResult<Json<Vec<Product>>> {
    let products = ecommerce::list_products(&state.db, true).await?;
    Ok(Json(products))
}

pub async fn public_get_product_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Product>> {
    let product = ecommerce::get_product_by_slug(&state.db, &slug).await?;
    if product.status != rustforge_core::ecommerce::ProductStatus::Published {
        return Err(ApiError::NotFound);
    }
    Ok(Json(product))
}

pub async fn public_checkout(
    State(state): State<AppState>,
    Json(payload): Json<CheckoutPayload>,
) -> ApiResult<Json<OrderWithItems>> {
    let order = ecommerce::checkout(&state.db, payload).await?;
    Ok(Json(order))
}

pub async fn public_download(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, ApiError> {
    let (file_path, file_name) =
        ecommerce::verify_and_consume_download(&state.db, &token, &state.jwt_secret).await?;

    let path = rustforge_core::security::confined_path(&state.protected_downloads_dir, &file_path)?;
    if !path.exists() {
        return Err(ApiError::NotFound);
    }

    let bytes = tokio::fs::read(&path).await.map_err(|e| {
        tracing::error!("Erreur lecture fichier sécurisé : {e}");
        ApiError::Internal(e.into())
    })?;

    let mime = mime_guess::from_path(&path).first_or_octet_stream();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{file_name}\""))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );

    Ok((headers, Body::from(bytes)).into_response())
}
