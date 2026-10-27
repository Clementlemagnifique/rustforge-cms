-- ===========================================================================
-- TrustPress — Migration 03 : E-Commerce Numérique (EDD-Style) Lié au LMS
-- Produits immatériels, téléchargements signés, commandes, réductions et passerelles.
-- ===========================================================================

-- ---------------------------------------------------------------------------
-- 1. Produits Numériques & Téléchargements
-- ---------------------------------------------------------------------------
CREATE TYPE product_status AS ENUM ('draft', 'published', 'archived');

CREATE TABLE ecommerce_products (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title              TEXT NOT NULL,
    slug               TEXT NOT NULL UNIQUE,
    excerpt            TEXT NOT NULL DEFAULT '',
    description        TEXT NOT NULL DEFAULT '',
    price_cents        INTEGER NOT NULL DEFAULT 0, -- ex: 4900 = 49.00 €
    featured_media_id  UUID REFERENCES media(id) ON DELETE SET NULL,
    -- Fichier payant protégé (stocké hors accès direct du web)
    file_path          TEXT,
    file_name          TEXT,
    download_limit     INTEGER NOT NULL DEFAULT 5, -- nombre max de téléchargements autorisés
    -- Liaison directe avec une formation LMS
    course_id          UUID REFERENCES lms_courses(id) ON DELETE SET NULL,
    status             product_status NOT NULL DEFAULT 'draft',
    sales_count        INTEGER NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_products_slug ON ecommerce_products(slug);
CREATE INDEX idx_products_course ON ecommerce_products(course_id);

-- Liaison inverse sur le cours LMS pour affichage bidirectionnel
ALTER TABLE lms_courses ADD COLUMN product_id UUID REFERENCES ecommerce_products(id) ON DELETE SET NULL;

-- ---------------------------------------------------------------------------
-- 2. Codes Promo & Réductions
-- ---------------------------------------------------------------------------
CREATE TYPE discount_type AS ENUM ('percentage', 'fixed_amount');

CREATE TABLE ecommerce_discounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code            TEXT NOT NULL UNIQUE,
    discount_type   discount_type NOT NULL DEFAULT 'percentage',
    amount          INTEGER NOT NULL, -- ex: 20 pour 20%, ou 1000 pour 10.00 €
    max_uses        INTEGER, -- NULL = illimité
    uses_count      INTEGER NOT NULL DEFAULT 0,
    expires_at      TIMESTAMPTZ,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 3. Commandes & Transactions
-- ---------------------------------------------------------------------------
CREATE TYPE order_status AS ENUM ('pending', 'completed', 'refunded', 'failed');

CREATE TABLE ecommerce_orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_number    TEXT NOT NULL UNIQUE, -- ex: "TP-2026-0001"
    customer_email  TEXT NOT NULL,
    customer_name   TEXT NOT NULL DEFAULT '',
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    subtotal_cents  INTEGER NOT NULL,
    discount_cents  INTEGER NOT NULL DEFAULT 0,
    total_cents     INTEGER NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    gateway         TEXT NOT NULL DEFAULT 'test', -- 'stripe', 'paypal', 'test'
    gateway_tx_id   TEXT,
    status          order_status NOT NULL DEFAULT 'pending',
    discount_id     UUID REFERENCES ecommerce_discounts(id) ON DELETE SET NULL,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_orders_customer ON ecommerce_orders(customer_email);
CREATE INDEX idx_orders_status ON ecommerce_orders(status);

CREATE TABLE ecommerce_order_items (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id        UUID NOT NULL REFERENCES ecommerce_orders(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES ecommerce_products(id) ON DELETE RESTRICT,
    title           TEXT NOT NULL,
    price_cents     INTEGER NOT NULL,
    -- Nombre de fois que le client a téléchargé le fichier
    download_count  INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_order_items_order ON ecommerce_order_items(order_id);
