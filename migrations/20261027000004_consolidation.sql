-- Additive consolidation. Existing revisions remain readable with snapshot=NULL.
ALTER TABLE revisions ADD COLUMN snapshot JSONB;

CREATE TABLE runtime_revision (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    revision BIGINT NOT NULL DEFAULT 0
);
INSERT INTO runtime_revision(singleton, revision) VALUES(TRUE, 0);

CREATE FUNCTION rustforge_bump_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    UPDATE runtime_revision SET revision = revision + 1 WHERE singleton;
    RETURN NULL;
END;
$$;

CREATE TRIGGER entries_revision AFTER INSERT OR UPDATE OR DELETE ON entries FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER settings_revision AFTER INSERT OR UPDATE OR DELETE ON settings FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER terms_revision AFTER INSERT OR UPDATE OR DELETE ON terms FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER entry_terms_revision AFTER INSERT OR UPDATE OR DELETE ON entry_terms FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER media_revision AFTER INSERT OR UPDATE OR DELETE ON media FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER plugins_revision AFTER INSERT OR UPDATE OR DELETE ON plugins FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();
CREATE TRIGGER content_types_revision AFTER INSERT OR UPDATE OR DELETE ON content_types FOR EACH STATEMENT EXECUTE FUNCTION rustforge_bump_revision();

CREATE TABLE import_origins (
    source TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    entry_id UUID NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    target_updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY(source, entity_type, source_id)
);
