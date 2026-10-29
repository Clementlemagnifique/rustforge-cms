//! # Module E-Commerce Numérique (Type Easy Digital Downloads)
//!
//! Conçu pour la vente de produits dématérialisés :
//! - Fichiers protégés téléchargeables via liens signés cryptographiquement (HMAC-SHA256)
//! - Limite de téléchargements par commande
//! - Liaison directe et automatique avec le module LMS (Sensei-like)
//! - Gestion des codes de réduction (pourcentage ou montant fixe)
//! - Paiement manuel ; connecteurs bancaires à implémenter séparément

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{CoreError, CoreResult, slugify};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ProductStatus {
    Draft,
    #[default]
    Published,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "discount_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum DiscountType {
    #[default]
    Percentage,
    FixedAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum OrderStatus {
    #[default]
    Pending,
    Completed,
    Refunded,
    Failed,
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
    #[serde(skip_serializing)]
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
    validate_product(&input)?;
    let slug = input
        .slug
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
        "#,
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
    validate_product(&input)?;
    let current = get_product(db, id).await?;
    let slug = input
        .slug
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
        "#,
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
    Ok(
        sqlx::query_as::<_, Discount>("SELECT * FROM ecommerce_discounts ORDER BY created_at DESC")
            .fetch_all(db)
            .await?,
    )
}

pub async fn create_discount(db: &PgPool, input: DiscountInput) -> CoreResult<Discount> {
    let code = input.code.trim().to_uppercase();
    let dtype = input.discount_type.unwrap_or(DiscountType::Percentage);
    if input.amount < 0
        || (dtype == DiscountType::Percentage && input.amount > 100)
        || input.max_uses.is_some_and(|n| n <= 0)
        || code.is_empty()
    {
        return Err(CoreError::validation("réduction invalide"));
    }
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

pub async fn delete_discount(db: &PgPool, id: Uuid) -> CoreResult<()> {
    sqlx::query("DELETE FROM ecommerce_discounts WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn validate_discount(
    db: &PgPool,
    code: &str,
    subtotal_cents: i32,
) -> CoreResult<(Discount, i32)> {
    let clean = code.trim().to_uppercase();
    let discount = sqlx::query_as::<_, Discount>(
        "SELECT * FROM ecommerce_discounts WHERE code = $1 AND is_active = true",
    )
    .bind(clean)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| CoreError::validation("code de réduction invalide ou inactif"))?;

    if let Some(exp) = discount.expires_at
        && Utc::now() > exp
    {
        return Err(CoreError::validation("ce code de réduction a expiré"));
    }

    if let Some(max_u) = discount.max_uses
        && discount.uses_count >= max_u
    {
        return Err(CoreError::validation(
            "ce code de réduction a atteint sa limite d'utilisation",
        ));
    }

    let discount_cents = match discount.discount_type {
        DiscountType::Percentage if (0..=100).contains(&discount.amount) => {
            (subtotal_cents as i64 * discount.amount as i64 / 100) as i32
        }
        DiscountType::Percentage => {
            return Err(CoreError::validation("pourcentage de réduction invalide"));
        }
        DiscountType::FixedAmount => discount.amount.min(subtotal_cents),
    };

    if discount_cents < 0 {
        return Err(CoreError::validation("réduction négative invalide"));
    }
    Ok((discount, discount_cents))
}

// ---------------------------------------------------------------------------
// Commandes & Tunnel de vente
// ---------------------------------------------------------------------------

pub async fn list_orders(db: &PgPool, limit: i64, offset: i64) -> CoreResult<Vec<Order>> {
    Ok(sqlx::query_as::<_, Order>(
        "SELECT * FROM ecommerce_orders ORDER BY created_at DESC LIMIT $1 OFFSET $2",
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
        "SELECT * FROM ecommerce_order_items WHERE order_id = $1 ORDER BY created_at ASC",
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    Ok(OrderWithItems { order, items })
}

pub async fn checkout(db: &PgPool, payload: CheckoutPayload) -> CoreResult<OrderWithItems> {
    if !matches!(payload.gateway.as_deref(), None | Some("manual")) {
        return Err(CoreError::validation(
            "passerelle non autorisée ; les paiements de test ne sont pas publics",
        ));
    }
    if payload.user_id.is_some() {
        return Err(CoreError::validation(
            "le compte acheteur est déterminé par le serveur",
        ));
    }
    if payload.items.is_empty() || payload.items.len() > 100 {
        return Err(CoreError::validation("le panier ne peut pas être vide"));
    }

    let mut tx = db.begin().await?;
    let mut resolved_products = Vec::new();
    let mut subtotal_cents = 0i32;

    for item in &payload.items {
        let p = get_product(db, item.product_id).await?;
        if p.status != ProductStatus::Published {
            return Err(CoreError::validation(format!(
                "le produit '{}' n'est plus en vente",
                p.title
            )));
        }
        if p.price_cents < 0 {
            return Err(CoreError::validation("prix invalide"));
        }
        subtotal_cents = subtotal_cents
            .checked_add(p.price_cents)
            .ok_or_else(|| CoreError::validation("montant du panier trop élevé"))?;
        resolved_products.push(p);
    }

    let mut discount_id = None;
    let mut discount_cents = 0;
    if let Some(ref code) = payload.discount_code
        && !code.trim().is_empty()
    {
        let (discount, amt) = validate_discount(db, code, subtotal_cents).await?;
        discount_id = Some(discount.id);
        discount_cents = amt;

        let consumed = sqlx::query("UPDATE ecommerce_discounts SET uses_count = uses_count + 1 WHERE id = $1 AND (max_uses IS NULL OR uses_count < max_uses)")
                .bind(discount.id)
                .execute(&mut *tx)
                .await?;
        if consumed.rows_affected() != 1 {
            return Err(CoreError::Conflict("Code promotionnel épuisé".into()));
        }
    }

    let total_cents = (subtotal_cents - discount_cents).max(0);
    let order_number = format!(
        "TP-{}-{}",
        Utc::now().format("%Y%m%d"),
        Uuid::new_v4().to_string()[..6].to_uppercase()
    );
    let gateway = "manual".to_string();
    let customer_name = payload.customer_name.unwrap_or_default();

    let initial_status = "pending";

    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO ecommerce_orders
            (order_number, customer_email, customer_name, user_id,
             subtotal_cents, discount_cents, total_cents, gateway, discount_id, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::order_status)
        RETURNING *
        "#,
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
    .fetch_one(&mut *tx)
    .await?;

    let mut order_items = Vec::new();
    for prod in &resolved_products {
        let item = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO ecommerce_order_items (order_id, product_id, title, price_cents)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(order.id)
        .bind(prod.id)
        .bind(&prod.title)
        .bind(prod.price_cents)
        .fetch_one(&mut *tx)
        .await?;

        order_items.push(item);
    }

    tx.commit().await?;
    if total_cents == 0 {
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
    let mut tx = db.begin().await?;
    let status: OrderStatus =
        sqlx::query_scalar("SELECT status FROM ecommerce_orders WHERE id=$1 FOR UPDATE")
            .bind(order_id)
            .fetch_one(&mut *tx)
            .await?;
    if status == OrderStatus::Completed {
        tx.rollback().await?;
        return get_order(db, order_id).await;
    }
    if status != OrderStatus::Pending {
        return Err(CoreError::Conflict(
            "Seule une commande en attente peut être confirmée.".into(),
        ));
    }
    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE ecommerce_orders
        SET status = 'completed'::order_status, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(order_id)
    .fetch_one(&mut *tx)
    .await?;

    let items =
        sqlx::query_as::<_, OrderItem>("SELECT * FROM ecommerce_order_items WHERE order_id = $1")
            .bind(order_id)
            .fetch_all(&mut *tx)
            .await?;

    let mut target_user_id = order.user_id;
    if target_user_id.is_none() {
        let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
            .bind(&order.customer_email)
            .fetch_optional(&mut *tx)
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
                    "#,
                )
                .bind(&order.customer_email)
                .bind(&username)
                .bind(&display)
                .bind(&hash)
                .fetch_one(&mut *tx)
                .await?;

                new_id
            }
        };

        target_user_id = Some(uid);
        sqlx::query("UPDATE ecommerce_orders SET user_id = $1 WHERE id = $2")
            .bind(uid)
            .bind(order.id)
            .execute(&mut *tx)
            .await?;
    }

    for item in &items {
        sqlx::query("UPDATE ecommerce_products SET sales_count = sales_count + 1 WHERE id = $1")
            .bind(item.product_id)
            .execute(&mut *tx)
            .await?;

        let course_id: Option<Uuid> =
            sqlx::query_scalar("SELECT course_id FROM ecommerce_products WHERE id = $1")
                .bind(item.product_id)
                .fetch_optional(&mut *tx)
                .await?
                .flatten();

        if let (Some(cid), Some(uid)) = (course_id, target_user_id) {
            sqlx::query("INSERT INTO lms_enrollments(course_id,user_id,progress_pct) VALUES($1,$2,0) ON CONFLICT(course_id,user_id) DO NOTHING")
                .bind(cid).bind(uid).execute(&mut *tx).await?;
            tracing::info!(
                course_id = %cid,
                user_id = %uid,
                order_id = %order.id,
                "Liaison E-Commerce ➔ LMS : inscription automatique confirmée"
            );
        }
    }

    tx.commit().await?;
    get_order(db, order.id).await
}

pub async fn refund_order(db: &PgPool, order_id: Uuid) -> CoreResult<OrderWithItems> {
    let mut tx = db.begin().await?;
    let status: OrderStatus =
        sqlx::query_scalar("SELECT status FROM ecommerce_orders WHERE id = $1 FOR UPDATE")
            .bind(order_id)
            .fetch_one(&mut *tx)
            .await?;
    if status == OrderStatus::Refunded {
        tx.rollback().await?;
        return get_order(db, order_id).await;
    }
    if status != OrderStatus::Completed {
        return Err(CoreError::Conflict(
            "Seule une commande terminée peut être remboursée.".into(),
        ));
    }
    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE ecommerce_orders
        SET status = 'refunded'::order_status, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(order_id)
    .fetch_one(&mut *tx)
    .await?;

    let items =
        sqlx::query_as::<_, OrderItem>("SELECT * FROM ecommerce_order_items WHERE order_id = $1")
            .bind(order_id)
            .fetch_all(&mut *tx)
            .await?;

    for item in &items {
        sqlx::query("UPDATE ecommerce_products SET sales_count = GREATEST(sales_count - 1, 0) WHERE id = $1")
            .bind(item.product_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    get_order(db, order.id).await
}

pub async fn cancel_order(db: &PgPool, order_id: Uuid) -> CoreResult<OrderWithItems> {
    let mut tx = db.begin().await?;
    let status: OrderStatus =
        sqlx::query_scalar("SELECT status FROM ecommerce_orders WHERE id = $1 FOR UPDATE")
            .bind(order_id)
            .fetch_one(&mut *tx)
            .await?;
    if status == OrderStatus::Failed {
        tx.rollback().await?;
        return get_order(db, order_id).await;
    }
    if status != OrderStatus::Pending {
        return Err(CoreError::Conflict(
            "Seule une commande en attente peut être annulée.".into(),
        ));
    }
    let order = sqlx::query_as::<_, Order>(
        r#"
        UPDATE ecommerce_orders
        SET status = 'failed'::order_status, updated_at = now()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(order_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    get_order(db, order.id).await
}

// ---------------------------------------------------------------------------
// Liens de téléchargement sécurisés (Tokens HMAC signés)
// ---------------------------------------------------------------------------

pub fn generate_download_token(order_item_id: Uuid, secret: &str, expires_in_hours: i64) -> String {
    let expires_at = Utc::now().timestamp() + (expires_in_hours * 3600);
    let raw = format!("rustforge-download-v1:{order_item_id}:{expires_at}");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC supports arbitrary keys");
    mac.update(raw.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{order_item_id}.{expires_at}.{signature}")
}

pub async fn verify_and_consume_download(
    db: &PgPool,
    token: &str,
    secret: &str,
) -> CoreResult<(String, String)> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CoreError::validation(
            "format de jeton de téléchargement invalide",
        ));
    }

    let order_item_id: Uuid = parts[0]
        .parse()
        .map_err(|_| CoreError::validation("identifiant de commande invalide dans le jeton"))?;
    let expires_at: i64 = parts[1]
        .parse()
        .map_err(|_| CoreError::validation("date d'expiration invalide dans le jeton"))?;
    let provided_sig = parts[2];

    if Utc::now().timestamp() > expires_at {
        return Err(CoreError::validation("ce lien de téléchargement a expiré"));
    }

    let expected_raw = format!("rustforge-download-v1:{order_item_id}:{expires_at}");
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC supports arbitrary keys");
    mac.update(expected_raw.as_bytes());
    let signature = hex::decode(provided_sig).map_err(|_| CoreError::Forbidden)?;
    mac.verify_slice(&signature)
        .map_err(|_| CoreError::Forbidden)?;

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
        return Err(CoreError::validation(
            "la commande n'est pas encore finalisée",
        ));
    }

    if row.download_count >= row.download_limit {
        return Err(CoreError::validation(
            "la limite de téléchargements autorisés a été atteinte",
        ));
    }

    let path = row.file_path.ok_or(CoreError::NotFound)?;
    let name = row.file_name.unwrap_or_else(|| "download".to_string());

    let consumed = sqlx::query("UPDATE ecommerce_order_items SET download_count = download_count + 1 WHERE id = $1 AND download_count < $2")
        .bind(order_item_id)
        .bind(row.download_limit)
        .execute(db)
        .await?;
    if consumed.rows_affected() != 1 {
        return Err(CoreError::Forbidden);
    }

    Ok((path, name))
}

fn validate_product(input: &ProductInput) -> CoreResult<()> {
    if input.price_cents < 0 || input.download_limit.is_some_and(|n| n <= 0) {
        return Err(CoreError::validation("prix ou quota invalide"));
    }
    if let Some(path) = &input.file_path {
        crate::security::relative_path(path)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Réglages Boutique
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommerceSettings {
    pub currency: String,
    pub currency_symbol: String,
    pub gateway_mode: String,
    pub stripe_public_key: String,
    pub stripe_secret_key: String,
    pub tax_rate_percent: f64,
    pub require_login: bool,
    pub default_download_limit: i32,
}

impl Default for CommerceSettings {
    fn default() -> Self {
        Self {
            currency: "EUR".to_string(),
            currency_symbol: "€".to_string(),
            gateway_mode: "test".to_string(),
            stripe_public_key: String::new(),
            stripe_secret_key: String::new(),
            tax_rate_percent: 20.0,
            require_login: false,
            default_download_limit: 5,
        }
    }
}

pub async fn get_commerce_settings(db: &PgPool) -> CoreResult<CommerceSettings> {
    let val = crate::settings::get(db, "ecommerce.settings").await?;
    if let Some(v) = val
        && let Ok(settings) = serde_json::from_value::<CommerceSettings>(v)
    {
        return Ok(settings);
    }
    Ok(CommerceSettings::default())
}

pub async fn update_commerce_settings(
    db: &PgPool,
    settings: CommerceSettings,
) -> CoreResult<CommerceSettings> {
    let val = serde_json::to_value(&settings)
        .map_err(|e| CoreError::validation(format!("réglages boutique invalides: {e}")))?;
    let mut map = std::collections::BTreeMap::new();
    map.insert("ecommerce.settings".to_string(), val);
    crate::settings::update(db, map).await?;
    Ok(settings)
}
