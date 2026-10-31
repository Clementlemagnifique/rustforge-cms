-- Migration 009 : Motifs synchronisés et blocs réutilisables (WP-34, STAB-07)
CREATE TABLE IF NOT EXISTS synced_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug VARCHAR(96) NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9_-]+$'),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    category VARCHAR(64) NOT NULL DEFAULT 'content',
    document JSONB NOT NULL CHECK (jsonb_typeof(document) = 'object'),
    is_synced BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_synced_patterns_category ON synced_patterns(category);
CREATE INDEX IF NOT EXISTS idx_synced_patterns_slug ON synced_patterns(slug);
