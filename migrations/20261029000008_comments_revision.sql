-- Migration 008 : Invalidation multi-instance du cache SSR lors de modifications de commentaires (M1-F)
CREATE TRIGGER comments_revision
AFTER INSERT OR UPDATE OR DELETE ON comments
FOR EACH STATEMENT
EXECUTE FUNCTION rustforge_bump_revision();

