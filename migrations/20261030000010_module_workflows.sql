-- Extend the existing module tables; keep legacy campaigns and recipes intact.
-- No transport, task or worker is started by this migration.
ALTER TABLE email_campaigns
 ADD COLUMN name text NOT NULL DEFAULT '',
 ADD COLUMN body_text text NOT NULL DEFAULT '',
 ADD COLUMN recipient_ids uuid[] NOT NULL DEFAULT '{}',
 ADD COLUMN version bigint NOT NULL DEFAULT 1 CHECK(version>0),
 ADD COLUMN delivery_mode text NOT NULL DEFAULT 'legacy',
 ADD COLUMN created_by uuid REFERENCES users(id) ON DELETE SET NULL,
 ADD COLUMN updated_at timestamptz NOT NULL DEFAULT now();
ALTER TABLE automation_recipes
 ADD COLUMN subject text NOT NULL DEFAULT '',
 ADD COLUMN body_text text NOT NULL DEFAULT '',
 ADD COLUMN recipient_id uuid REFERENCES users(id) ON DELETE SET NULL,
 ADD COLUMN delay_seconds integer NOT NULL DEFAULT 0 CHECK(delay_seconds BETWEEN 0 AND 2592000),
 ADD COLUMN version bigint NOT NULL DEFAULT 1 CHECK(version>0),
 ADD COLUMN created_by uuid REFERENCES users(id) ON DELETE SET NULL;
ALTER TABLE automation_logs
 ADD COLUMN invocation_id uuid,
 ADD COLUMN job_id uuid REFERENCES rf_jobs(id);
CREATE UNIQUE INDEX automation_logs_invocation ON automation_logs(recipe_id,invocation_id)
 WHERE invocation_id IS NOT NULL;
