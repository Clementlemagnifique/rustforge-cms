-- Recoverable inbox deletion. No automatic purge or data removal.
ALTER TABLE form_submissions ADD COLUMN deleted_at timestamptz;
CREATE INDEX idx_submissions_inbox ON form_submissions(form_id, created_at DESC, id DESC)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_submissions_trash ON form_submissions(form_id, deleted_at)
    WHERE deleted_at IS NOT NULL;
