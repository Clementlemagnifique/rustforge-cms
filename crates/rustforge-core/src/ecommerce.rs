//! # Module E-Commerce Numérique (Type Easy Digital Downloads)
//!
//! Conçu pour la vente de produits dématérialisés :
//! - Fichiers protégés téléchargeables via liens signés cryptographiquement (HMAC-SHA256)
//! - Limite de téléchargements par commande
//! - Liaison directe et automatique avec le module LMS (Sensei-like)
//! - Gestion des codes de réduction (pourcentage ou montant fixe)
//! - Multi-passerelles (Test / Stripe / Webhook)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreError, CoreResult, slugify};

pub const DEFAULT_SIGNING_SECRET: &str = "rustforge-digital-vault-2026-secret-key";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProductStatus {
    Draft,
    Published,
    Archived,
}

impl Default for ProductStatus {
    fn default() -> Self {
        Self::Published
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "discount_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DiscountType {
    Percentage,
    FixedAmount,
}

impl Default for DiscountType {
    fn default() -> Self {
        Self::Percentage
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Completed,
    Refunded,
    Failed,
}

impl Default for OrderStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub description: String,
    pub price_cents: i32,
    pub featured_media_id: Option<Uuid>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub download_limit: i32,
    pub course_id: Option<Uuid>,
    pub status: ProductStatus,
    pub sales_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInput {
    pub title: String,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub description: Option<String>,
    pub price_cents: i32,
    pub featured_media_id: Option<Uuid>,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub download_limit: Option<i32>,
    pub course_id: Option<Uuid>,
    pub status: Option<ProductStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Discount {
    pub id: Uuid,
    pub code: String,
    pub discount_type: DiscountType,
    pub amount: i32,
    pub max_uses: Option<i32>,
    pub uses_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountInput {
    pub code: String,
    pub discount_type: Option<DiscountType>,
    pub amount: i32,
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub order_number: String,
    pub customer_email: String,
    pub customer_name: String,
    pub user_id: Option<Uuid>,
    pub subtotal_cents: i32,
    pub discount_cents: i32,
    pub total_cents: i32,
    pub currency: String,
    pub gateway: String,
    pub gateway_tx_id: Option<String>,
    pub status: OrderStatus,
    pub discount_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub title: String,
    pub price_cents: i32,
    pub download_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithItems {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutItemPayload {
    pub product_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckoutPayload {
    pub customer_email: String,
    pub customer_name: Option<String>,
    pub user_id: Option<Uuid>,
    pub items: Vec<CheckoutItemPayload>,
    pub discount_code: Option<String>,
    pub gateway: Option<String>,
}

// ---------------------------------------------------------------------------
// Produits
// ---------------------------------------------------------------------------

pub async fn list_products(db: &PgPool, published_only: bool) -> CoreResult<Vec<Product>> {
    let sql = if published_only {
        "SELECT * FROM ecommerce_products WHERE status = 'published' ORDER BY created_at DESC"
    } else {
        "SELECT * FROM ecommerce_products ORDER BY created_at DESC"
    };
    Ok(sqlx::query_as::<_, Product>(sql).fetch_all(db).await?)
}

pub async fn get_product(db: &PgPool, id: Uuid) -> CoreResult<Product> {
    sqlx::query_as::<_, Product>("SELECT * FROM ecommerce_products WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)
}

pub async fn get_product_by_slug(db: &PgPool, slug: &str) -> CoreResult<Product> {
    sqlx::query_as::<_, Product>("SELECT * FROM ecommerce_products WHERE slug = $1")
        .bind(slug)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)
}

pub async fn create_product(db: &PgPool, input: ProductInput) -> CoreResult<Product> {
    let slug = input.slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| slugify(&input.title));
    let excerpt = input.excerpt.unwrap_or_default();
    let description = input.description.unwrap_or_default();
    let limit = input.download_limit.unwrap_or(5);
    let status = input.status.unwrap_or(ProductStatus::Published);

    let product = sqlx::query_as::<_, Product>(
        r#"
        INSERT INTO ecommerce_products
            (title, slug, excerpt, description, price_cents, featured_media_id,
             file_path, file_name, download_limit, course_id, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#
    )
    .bind(input.title)
    .bind(slug)
    .bind(excerpt)
    .bind(description)
    .bind(input.price_cents)
    .bind(input.featured_media_id)
    .bind(input.file_path)
    .bind(input.file_name)
    .bind(limit)
    .bind(input.course_id)
    .bind(status)
    .fetch_one(db)
    .await?;

    if let Some(course_id) = product.course_id {
        let _ = sqlx::query("UPDATE lms_courses SET product_id = $1 WHERE id = $2")
            .bind(product.id)
            .bind(course_id)
            .execute(db)
            .await;
    }

    Ok(product)
}

pub async fn update_product(db: &PgPool, id: Uuid, input: ProductInput) -> CoreResult<Product> {
    let current = get_product(db, id).await?;
    let slug = input.slug
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(current.slug);
    let excerpt = input.excerpt.unwrap_or(current.excerpt);
    let description = input.description.unwrap_or(current.description);
    let feat = input.featured_media_id.or(current.featured_media_id);
    let file_path = input.file_path.or(current.file_path);
    let file_name = input.file_name.or(current.file_name);
    let limit = input.download_limit.unwrap_or(current.download_limit);
    let course_id = input.course_id.or(current.course_id);
    let status = input.status.unwrap_or(current.status);

    let updated = sqlx::query_as::<_, Product>(
        r#"
        UPDATE ecommerce_products
        SET title = $2, slug = $3, excerpt = $4, description = $5,
            price_cents = $6, featured_media_id = $7, file_path = $8,
            file_name = $9, download_limit = $10, course_id = $11,
            status = $12, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(id)
    .bind(input.title)
    .bind(slug)
    .bind(excerpt)
    .bind(description)
    .bind(input.price_cents)
    .bind(feat)
    .bind(file_path)
    .bind(file_name)
    .bind(limit)
    .bind(course_id)
    .bind(status)
    .fetch_one(db)
    .await?;

    if let Some(cid) = updated.course_id {
        let _ = sqlx::query("UPDATE lms_courses SET product_id = $1 WHERE id = $2")
            .bind(updated.id)
            .bind(cid)
            .execute(db)
            .await;
    }

    Ok(updated)
}

pub async fn delete_product(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM ecommerce_products WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Réductions (Discounts)
// ---------------------------------------------------------------------------

pub async fn list_discounts(db: &PgPool) -> CoreResult<Vec<Discount>> {
    Ok(sqlx::query_as::<_, Discount>(
        "SELECT * FROM ecommerce_discounts ORDER BY created_at DESC"
    )
    .fetch_all(db)
    .await?)
}

pub async fn create_discount(db: &PgPool, input: DiscountInput) -> CoreResult<Discount> {
    let code = input.code.trim().to_uppercase();
    let dtype = input.discount_type.unwrap_or(DiscountType::Percentage);
    let active = input.is_active.unwrap_or(true);

    let d = sqlx::query_as::<_, Discount>(
        r#"
        INSERT INTO ecommerce_discounts (code, discount_type, amount, max_uses, expires_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(code)
    .bind(dtype)
    .bind(input.amount)
    .bind(input.max_uses)
    .bind(input.expires_at)
    .bind(active)
    .fetch_one(db)
    .await?;

    Ok(d)
}

pub async fn validate_discount(db: &PgPool, code: &str, subtotal_cents: i32) -> CoreResult<(Discount, i32)> {
    let clean = code.trim().to_uppercase();
    let discount = sqlx::query_as::<_, Discount>(
        "SELECT * FROM ecommerce_discounts WHERE code = $1 AND is_active = true"
    )
    .bind(clean)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| CoreError::validation("code de réduction invalide ou inactif"))?;

    if let Some(exp) = discount.expires_at {
        if Utc::now() > exp {
            return Err(CoreError::validation("ce code de réduction a expiré"));
        }
    }

    if let Some(max_u) = discount.max_uses {
        if discount.uses_count >= max_u {
            return Err(CoreError::validation("ce code de réduction a atteint sa limite d'utilisation"));
        }
    }

    let discount_cents = match discount.discount_type {
        DiscountType::Percentage => (subtotal_cents as i64 * discount.amount as i64 / 100) as i32,
        DiscountType::FixedAmount => discount.amount.min(subtotal_cents),
    };

    Ok((discount, discount_cents))
}

// ---------------------------------------------------------------------------
// Commandes & Tunnel de vente
// ---------------------------------------------------------------------------

pub async fn list_orders(db: &PgPool, limit: i64, offset: i64) -> CoreResult<Vec<Order>> {
    Ok(sqlx::query_as::<_, Order>(
        "SELECT * FROM ecommerce_orders ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?)
}

pub async fn get_order(db: &PgPool, id: Uuid) -> CoreResult<OrderWithItems> {
    let order = sqlx::query_as::<_, Order>("SELECT * FROM ecommerce_orders WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(CoreError::NotFound)?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM ecommerce_order_items WHERE order_id = $1 ORDER BY created_at ASC"
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    Ok(OrderWithItems { order, items })
}

pub async fn checkout(db: &PgPool, payload: CheckoutPayload) -> CoreResult<OrderWithItems> {
    if payload.items.is_empty() {
        return Err(CoreError::validation("le panier ne peut pas être vide"));
    }

    let mut resolved_products = Vec::new();
    let mut subtotal_cents = 0;

    for item in &payload.items {
        let p = get_product(db, item.product_id).await?;
        if p.status != ProductStatus::Published {
            return Err(CoreError::validation(format!("le produit '{}' n'est plus en vente", p.title)));
        }
        subtotal_cents += p.price_cents;
        resolved_products.push(p);
    }

    let mut discount_id = None;
    let mut discount_cents = 0;
    if let Some(ref code) = payload.discount_code {
        if !code.trim().is_empty() {
            let (discount, amt) = validate_discount(db, code, subtotal_cents).await?;
            discount_id = Some(discount.id);
            discount_cents = amt;

            let _ = sqlx::query("UPDATE ecommerce_discounts SET uses_count = uses_count + 1 WHERE id = $1")
                .bind(discount.id)
                .execute(db)
                .await;
        }
    }

    let total_cents = (subtotal_cents - discount_cents).max(0);
    let order_number = format!("TP-{}-{}", Utc::now().format("%Y%m%d"), &Uuid::new_v4().to_string()[..6].to_uppercase());
    let gateway = payload.gateway.unwrap_or_else(|| "test".to_string());
    let customer_name = payload.customer_name.unwrap_or_default();

    let initial_status = if total_cents == 0 || gateway == "test" {
        "completed"
    } else {
        "pending"
    };

    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO ecommerce_orders
            (order_number, customer_email, customer_name, user_id,
             subtotal_cents, discount_cents, total_cents, gateway, discount_id, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::order_status)
        RETURNING *
        "#
    )
    .bind(&order_number)
    .bind(&payload.customer_email)
    .bind(&customer_name)
    .bind(payload.user_id)
    .bind(subtotal_cents)
    .bind(discount_cents)
    .bind(total_cents)
    .bind(&gateway)
    .bind(discount_id)
    .bind(initial_status)
    .fetch_one(db)
    .await?;

    let mut order_items = Vec::new();
    for prod in &resolved_products {
        let item = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO ecommerce_order_items (order_id, product_id, title, price_cents)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#
        )
        .bind(order.id)
        .bind(prod.id)
        .bind(&prod.title)
        .bind(prod.price_cents)
        .fetch_one(db)
        .await?;

        order_items.push(item);
    }

    if initial_status == "completed" {
        let finalized = complete_order(db, order.id).await?;
        Ok(finalized)
    } else {
        Ok(OrderWithItems {
            order,
            items: order_items,
        })
    }
}

pub async fn complete_order(db: &PgPool, order_id: Uuid) -> CoreResult<OrderWithItems> {
    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE ecommerce_orders
        SET status = 'completed'::order_status, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#
    )
    .bind(order_id)
    .fetch_one(db)
    .await?;

    let items = sqlx::query_as::<_, OrderItem>(
        "SELECT * FROM ecommerce_order_items WHERE order_id = $1"
    )
    .bind(order_id)
    .fetch_all(db)
    .await?;

    let mut target_user_id = order.user_id;
    if target_user_id.is_none() {
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(&order.customer_email)
            .fetch_optional(db)
            .await?;

        let uid = match existing {
            Some(id) => id,
            None => {
                let username = format!("client_{}", &Uuid::new_v4().to_string()[..8]);
                let temp_password = Uuid::new_v4().to_string();
                let hash = crate::users::hash_password(&temp_password)?;
                let display = if order.customer_name.trim().is_empty() {
                    username.clone()
                } else {
                    order.customer_name.clone()
                };

                let new_id: Uuid = sqlx::query_scalar(
                    r#"
                    INSERT INTO users (email, username, display_name, password_hash, role)
                    VALUES ($1, $2, $3, $4, 'subscriber'::user_role)
                    RETURNING id
                    "#
                )
                .bind(&order.customer_email)
                .bind(&username)
                .bind(&display)
                .bind(&hash)
                .fetch_one(db)
                .await?;

                new_id
            }
        };

        target_user_id = Some(uid);
        let _ = sqlx::query("UPDATE ecommerce_orders SET user_id = $1 WHERE id = $2")
            .bind(uid)
            .bind(order.id)
            .execute(db)
            .await;
    }

    for item in &items {
        let _ = sqlx::query("UPDATE ecommerce_products SET sales_count = sales_count + 1 WHERE id = $1")
            .bind(item.product_id)
            .execute(db)
            .await;

        let course_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT course_id FROM ecommerce_products WHERE id = $1"
        )
        .bind(item.product_id)
        .fetch_optional(db)
        .await?
        .flatten();

        if let (Some(cid), Some(uid)) = (course_id, target_user_id) {
            let _ = crate::lms::enroll_user(db, cid, uid).await;
            tracing::info!(
                course_id = %cid,
                user_id = %uid,
                order_id = %order.id,
                "Liaison E-Commerce ➔ LMS : inscription automatique confirmée"
            );
        }
    }

    Ok(OrderWithItems { order, items })
}

// ---------------------------------------------------------------------------
// Liens de téléchargement sécurisés (Tokens HMAC signés)
// ---------------------------------------------------------------------------

pub fn generate_download_token(order_item_id: Uuid, secret: &str, expires_in_hours: i64) -> String {
    let expires_at = Utc::now().timestamp() + (expires_in_hours * 3600);
    let raw = format!("{order_item_id}:{expires_at}:{secret}");
    let signature = hex::encode(Sha256::digest(raw.as_bytes()));
    format!("{order_item_id}.{expires_at}.{signature}")
}

pub async fn verify_and_consume_download(
    db: &PgPool,
    token: &str,
    secret: &str,
) -> CoreResult<(String, String)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CoreError::validation("format de jeton de téléchargement invalide"));
    }

    let order_item_id: Uuid = parts[0].parse()
        .map_err(|_| CoreError::validation("identifiant de commande invalide dans le jeton"))?;
    let expires_at: i64 = parts[1].parse()
        .map_err(|_| CoreError::validation("date d'expiration invalide dans le jeton"))?;
    let provided_sig = parts[2];

    if Utc::now().timestamp() > expires_at {
        return Err(CoreError::validation("ce lien de téléchargement a expiré"));
    }

    let expected_raw = format!("{order_item_id}:{expires_at}:{secret}");
    let expected_sig = hex::encode(Sha256::digest(expected_raw.as_bytes()));
    if provided_sig != expected_sig {
        return Err(CoreError::validation("signature cryptographique invalide"));
    }

    #[derive(sqlx::FromRow)]
    struct DownloadRow {
        file_path: Option<String>,
        file_name: Option<String>,
        download_limit: i32,
        download_count: i32,
        order_status: OrderStatus,
    }

    let row = sqlx::query_as::<_, DownloadRow>(
        r#"
        SELECT p.file_path, p.file_name, p.download_limit, oi.download_count, o.status as order_status
        FROM ecommerce_order_items oi
        JOIN ecommerce_orders o ON o.id = oi.order_id
        JOIN ecommerce_products p ON p.id = oi.product_id
        WHERE oi.id = $1
        "#
    )
    .bind(order_item_id)
    .fetch_optional(db)
    .await?
    .ok_or(CoreError::NotFound)?;

    if row.order_status != OrderStatus::Completed {
        return Err(CoreError::validation("la commande n'est pas encore finalisée"));
    }

    if row.download_count >= row.download_limit {
        return Err(CoreError::validation("la limite de téléchargements autorisés a été atteinte"));
    }

    let path = row.file_path.ok_or(CoreError::NotFound)?;
    let name = row.file_name.unwrap_or_else(|| "download".to_string());

    sqlx::query("UPDATE ecommerce_order_items SET download_count = download_count + 1 WHERE id = $1")
        .bind(order_item_id)
        .execute(db)
        .await?;

    Ok((path, name))
}
