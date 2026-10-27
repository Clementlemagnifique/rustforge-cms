-- TrustPress — schéma initial
-- Inspiré du modèle WordPress (posts, terms, options, users) mais normalisé et typé.

CREATE TYPE user_role AS ENUM ('admin', 'editor', 'author', 'subscriber');
CREATE TYPE entry_status AS ENUM ('draft', 'published', 'archived', 'trash');

-- ---------------------------------------------------------------------------
-- Utilisateurs & sessions
-- ---------------------------------------------------------------------------
CREATE TABLE users (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email          TEXT NOT NULL UNIQUE,
    username       TEXT NOT NULL UNIQUE,
    display_name   TEXT NOT NULL,
    password_hash  TEXT NOT NULL,
    role           user_role NOT NULL DEFAULT 'author',
    bio            TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at  TIMESTAMPTZ
);

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at  TIMESTAMPTZ
);
CREATE INDEX refresh_tokens_user_idx ON refresh_tokens(user_id);

-- ---------------------------------------------------------------------------
-- Médias
-- ---------------------------------------------------------------------------
CREATE TABLE media (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path           TEXT NOT NULL UNIQUE,
    original_name  TEXT NOT NULL,
    mime_type      TEXT NOT NULL,
    size_bytes     BIGINT NOT NULL,
    width          INTEGER,
    height         INTEGER,
    alt            TEXT NOT NULL DEFAULT '',
    caption        TEXT NOT NULL DEFAULT '',
    uploaded_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX media_created_idx ON media(created_at DESC);

-- ---------------------------------------------------------------------------
-- Types de contenu (≈ register_post_type + ACF)
-- ---------------------------------------------------------------------------
CREATE TABLE content_types (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug           TEXT NOT NULL UNIQUE,
    name           TEXT NOT NULL,
    name_singular  TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    icon           TEXT NOT NULL DEFAULT 'file-text',
    hierarchical   BOOLEAN NOT NULL DEFAULT false,
    has_revisions  BOOLEAN NOT NULL DEFAULT true,
    is_system      BOOLEAN NOT NULL DEFAULT false,
    fields         JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Entrées (≈ wp_posts + wp_postmeta)
-- ---------------------------------------------------------------------------
CREATE TABLE entries (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_type_id    UUID NOT NULL REFERENCES content_types(id) ON DELETE RESTRICT,
    parent_id          UUID REFERENCES entries(id) ON DELETE SET NULL,
    author_id          UUID REFERENCES users(id) ON DELETE SET NULL,
    title              TEXT NOT NULL,
    slug               TEXT NOT NULL,
    excerpt            TEXT NOT NULL DEFAULT '',
    body               TEXT NOT NULL DEFAULT '',
    body_html          TEXT NOT NULL DEFAULT '',
    blocks             JSONB NOT NULL DEFAULT '[]'::jsonb,
    intent             JSONB NOT NULL DEFAULT '{}'::jsonb,
    design_tokens      JSONB NOT NULL DEFAULT '{}'::jsonb,
    status             entry_status NOT NULL DEFAULT 'draft',
    fields             JSONB NOT NULL DEFAULT '{}'::jsonb,
    featured_media_id  UUID REFERENCES media(id) ON DELETE SET NULL,
    published_at       TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    search             TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('simple'::regconfig, coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple'::regconfig, coalesce(excerpt, '')), 'B') ||
        setweight(to_tsvector('simple'::regconfig, coalesce(body, '')), 'C')
    ) STORED,
    UNIQUE (content_type_id, slug)
);
CREATE INDEX entries_type_status_idx ON entries(content_type_id, status, published_at DESC);
CREATE INDEX entries_author_idx ON entries(author_id);
CREATE INDEX entries_search_idx ON entries USING GIN(search);
CREATE INDEX entries_fields_idx ON entries USING GIN(fields jsonb_path_ops);

CREATE TABLE revisions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id    UUID NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    author_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    title       TEXT NOT NULL,
    excerpt     TEXT NOT NULL,
    body        TEXT NOT NULL,
    fields      JSONB NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX revisions_entry_idx ON revisions(entry_id, created_at DESC);

-- ---------------------------------------------------------------------------
-- Taxonomies (≈ wp_terms + wp_term_taxonomy + wp_term_relationships)
-- ---------------------------------------------------------------------------
CREATE TABLE taxonomies (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug          TEXT NOT NULL UNIQUE,
    name          TEXT NOT NULL,
    hierarchical  BOOLEAN NOT NULL DEFAULT false,
    is_system     BOOLEAN NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE terms (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    taxonomy_id  UUID NOT NULL REFERENCES taxonomies(id) ON DELETE CASCADE,
    parent_id    UUID REFERENCES terms(id) ON DELETE SET NULL,
    name         TEXT NOT NULL,
    slug         TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (taxonomy_id, slug)
);

CREATE TABLE entry_terms (
    entry_id  UUID NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    term_id   UUID NOT NULL REFERENCES terms(id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, term_id)
);
CREATE INDEX entry_terms_term_idx ON entry_terms(term_id);

-- ---------------------------------------------------------------------------
-- Réglages (≈ wp_options)
-- ---------------------------------------------------------------------------
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       JSONB NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Plugins WASM
-- ---------------------------------------------------------------------------
CREATE TABLE plugins (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    version       TEXT NOT NULL,
    description   TEXT NOT NULL DEFAULT '',
    hooks         JSONB NOT NULL DEFAULT '[]'::jsonb,
    capabilities  JSONB NOT NULL DEFAULT '[]'::jsonb,
    granted       JSONB NOT NULL DEFAULT '[]'::jsonb,
    enabled       BOOLEAN NOT NULL DEFAULT false,
    kv            JSONB NOT NULL DEFAULT '{}'::jsonb,
    installed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Données initiales
-- ---------------------------------------------------------------------------
INSERT INTO content_types (slug, name, name_singular, description, icon, hierarchical, is_system) VALUES
    ('post', 'Articles', 'Article', 'Contenus datés, classés par catégories et étiquettes.', 'newspaper', false, true),
    ('page', 'Pages', 'Page', 'Contenus statiques et hiérarchiques.', 'file-text', true, true);

INSERT INTO taxonomies (slug, name, hierarchical, is_system) VALUES
    ('category', 'Catégories', true, true),
    ('tag', 'Étiquettes', false, true);

INSERT INTO terms (taxonomy_id, name, slug)
SELECT id, 'Non classé', 'non-classe' FROM taxonomies WHERE slug = 'category';

INSERT INTO settings (key, value) VALUES
    ('site.title', '"TrustPress"'),
    ('site.tagline', '"Un CMS minimaliste, moderne et puissant"'),
    ('site.url', '"http://localhost:8080"'),
    ('site.language', '"fr"'),
    ('reading.per_page', '10');
