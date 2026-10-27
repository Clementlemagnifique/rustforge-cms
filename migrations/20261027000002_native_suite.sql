-- ===========================================================================
-- TrustPress — Migration 02 : Modules Natifs & Architecture Entreprise
-- Intègre nativement dans le cœur :
-- 1. Médias hiérarchiques (dossiers) & conversion WebP
-- 2. Formulaires Pro & Soumissions (type WPForms)
-- 3. Galeries Multi-sources (Locales, YouTube, Vimeo...)
-- 4. Calendrier éditorial & Programmation précise
-- 5. Module LMS Natif (Sensei-like : Cours, Modules, Leçons, Inscriptions)
-- 6. Automatisations (Trigger / Condition / Action style WP Automator)
-- 7. Campagnes & Abonnés Emailing (Newsletter & Séquences)
-- 8. Sécurité Zero-Trust & Journal d'Audit
-- 9. Analytics natifs respectueux de la vie privée
-- ===========================================================================

-- ---------------------------------------------------------------------------
-- 1. Organisation Médias : Dossiers & Variantes WebP
-- ---------------------------------------------------------------------------
CREATE TABLE media_folders (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_id   UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(parent_id, slug)
);

ALTER TABLE media 
    ADD COLUMN folder_id UUID REFERENCES media_folders(id) ON DELETE SET NULL,
    ADD COLUMN webp_path TEXT,
    ADD COLUMN external_provider TEXT, -- 'local', 'dropbox', 'gdrive'
    ADD COLUMN metadata JSONB NOT NULL DEFAULT '{}'::jsonb;

CREATE INDEX idx_media_folder ON media(folder_id);

-- ---------------------------------------------------------------------------
-- 2. Calendrier & Programmation éditoriale
-- ---------------------------------------------------------------------------
ALTER TABLE entries
    ADD COLUMN scheduled_unpublish_at TIMESTAMPTZ,
    ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN locale TEXT NOT NULL DEFAULT 'fr',
    ADD COLUMN translation_group_id UUID;

CREATE INDEX idx_entries_schedule ON entries(status, published_at, scheduled_unpublish_at);
CREATE INDEX idx_entries_translation ON entries(translation_group_id, locale);

-- ---------------------------------------------------------------------------
-- 3. Formulaires Professionnels & Soumissions
-- ---------------------------------------------------------------------------
CREATE TABLE forms (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name           TEXT NOT NULL,
    slug           TEXT NOT NULL UNIQUE,
    description    TEXT NOT NULL DEFAULT '',
    fields         JSONB NOT NULL DEFAULT '[]'::jsonb, -- configuration des champs
    settings       JSONB NOT NULL DEFAULT '{}'::jsonb, -- redirections, notifications email, webhooks
    is_active      BOOLEAN NOT NULL DEFAULT true,
    created_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE form_submissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    form_id     UUID NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    data        JSONB NOT NULL DEFAULT '{}'::jsonb,
    ip_hash     TEXT NOT NULL, -- anonymisé pour conformité RGPD
    user_agent  TEXT,
    is_spam     BOOLEAN NOT NULL DEFAULT false,
    is_read     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_submissions_form ON form_submissions(form_id, created_at DESC);
CREATE INDEX idx_submissions_unread ON form_submissions(form_id, is_read) WHERE is_read = false;

-- ---------------------------------------------------------------------------
-- 4. Galeries Multi-sources (Photos & Vidéos)
-- ---------------------------------------------------------------------------
CREATE TYPE gallery_item_type AS ENUM ('image', 'video_local', 'youtube', 'vimeo', 'external');

CREATE TABLE galleries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    layout      TEXT NOT NULL DEFAULT 'grid', -- 'grid', 'masonry', 'carousel', 'justified'
    settings    JSONB NOT NULL DEFAULT '{}'::jsonb, -- lightbox, colonnes, ratio
    created_by  UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE gallery_items (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    gallery_id  UUID NOT NULL REFERENCES galleries(id) ON DELETE CASCADE,
    item_type   gallery_item_type NOT NULL DEFAULT 'image',
    media_id    UUID REFERENCES media(id) ON DELETE SET NULL,
    external_url TEXT,
    title       TEXT NOT NULL DEFAULT '',
    caption     TEXT NOT NULL DEFAULT '',
    thumbnail_url TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_gallery_items_order ON gallery_items(gallery_id, sort_order ASC);

-- ---------------------------------------------------------------------------
-- 5. Module LMS Natif (Formations & Cours façon Sensei)
-- ---------------------------------------------------------------------------
CREATE TABLE lms_courses (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title         TEXT NOT NULL,
    slug          TEXT NOT NULL UNIQUE,
    description   TEXT NOT NULL DEFAULT '',
    content       TEXT NOT NULL DEFAULT '',
    featured_media_id UUID REFERENCES media(id) ON DELETE SET NULL,
    price_cents   INTEGER NOT NULL DEFAULT 0, -- 0 = gratuit
    is_published  BOOLEAN NOT NULL DEFAULT false,
    created_by    UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lms_modules (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id   UUID NOT NULL REFERENCES lms_courses(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE lms_lessons (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id   UUID NOT NULL REFERENCES lms_courses(id) ON DELETE CASCADE,
    module_id   UUID REFERENCES lms_modules(id) ON DELETE SET NULL,
    title       TEXT NOT NULL,
    slug        TEXT NOT NULL,
    content     TEXT NOT NULL DEFAULT '',
    video_url   TEXT,
    duration_min INTEGER NOT NULL DEFAULT 10,
    is_preview  BOOLEAN NOT NULL DEFAULT false,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(course_id, slug)
);

CREATE TABLE lms_enrollments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    course_id   UUID NOT NULL REFERENCES lms_courses(id) ON DELETE CASCADE,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    progress_pct INTEGER NOT NULL DEFAULT 0,
    completed_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(course_id, user_id)
);

-- ---------------------------------------------------------------------------
-- 6. Moteur d'Automatisations (Trigger ➔ Action style Automator)
-- ---------------------------------------------------------------------------
CREATE TABLE automation_recipes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title       TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    trigger_type TEXT NOT NULL, -- ex: 'form.submitted', 'user.registered', 'course.completed'
    trigger_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    conditions  JSONB NOT NULL DEFAULT '[]'::jsonb,
    actions     JSONB NOT NULL DEFAULT '[]'::jsonb, -- liste d'actions ordonnées (email, webhook, tag...)
    is_active   BOOLEAN NOT NULL DEFAULT true,
    executions_count BIGINT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE automation_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id   UUID NOT NULL REFERENCES automation_recipes(id) ON DELETE CASCADE,
    success     BOOLEAN NOT NULL,
    details     JSONB NOT NULL DEFAULT '{}'::jsonb,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_automation_logs ON automation_logs(recipe_id, executed_at DESC);

-- ---------------------------------------------------------------------------
-- 7. Mailing & Abonnés (Newsletter & Séquences)
-- ---------------------------------------------------------------------------
CREATE TABLE email_subscribers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email       TEXT NOT NULL UNIQUE,
    first_name  TEXT NOT NULL DEFAULT '',
    last_name   TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'subscribed', -- 'subscribed', 'unsubscribed', 'bounced'
    tags        TEXT[] NOT NULL DEFAULT '{}',
    confirmed_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE email_campaigns (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject     TEXT NOT NULL,
    preview_text TEXT NOT NULL DEFAULT '',
    body_html   TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'draft', -- 'draft', 'scheduled', 'sent'
    sent_count  INTEGER NOT NULL DEFAULT 0,
    open_count  INTEGER NOT NULL DEFAULT 0,
    click_count INTEGER NOT NULL DEFAULT 0,
    scheduled_at TIMESTAMPTZ,
    sent_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- 8. Sécurité Zero-Trust & Audit Trail
-- ---------------------------------------------------------------------------
CREATE TABLE security_audit_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    action      TEXT NOT NULL, -- 'auth.login_success', 'auth.failed', 'file.upload', 'settings.changed'
    severity    TEXT NOT NULL DEFAULT 'info', -- 'info', 'warning', 'critical'
    ip_address  TEXT NOT NULL,
    user_agent  TEXT,
    metadata    JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_security_logs ON security_audit_logs(severity, created_at DESC);

-- ---------------------------------------------------------------------------
-- 9. Analytics Natifs Sans Cookies
-- ---------------------------------------------------------------------------
CREATE TABLE analytics_pageviews (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path        TEXT NOT NULL,
    referrer    TEXT,
    device_type TEXT, -- 'desktop', 'mobile', 'tablet'
    country     TEXT,
    visitor_hash TEXT NOT NULL, -- Hash journalier non réversible (zéro cookie RGPD)
    viewed_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_analytics_path_time ON analytics_pageviews(path, viewed_at DESC);
