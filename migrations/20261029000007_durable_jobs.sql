-- File commune : aucun worker n'est démarré implicitement par cette migration.
CREATE TABLE rf_jobs (
    id uuid PRIMARY KEY,
    kind varchar(96) NOT NULL CHECK (kind ~ '^[A-Za-z0-9._:-]+$'),
    payload_version integer NOT NULL CHECK (payload_version BETWEEN 1 AND 65535),
    payload jsonb NOT NULL CHECK (jsonb_typeof(payload)='object' AND octet_length(payload::text)<=65536),
    output jsonb CHECK (output IS NULL OR (jsonb_typeof(output)='object' AND octet_length(output::text)<=65536)),
    dedup_key varchar(200) NOT NULL UNIQUE CHECK (dedup_key ~ '^[A-Za-z0-9._:-]+$'),
    state varchar(24) NOT NULL DEFAULT 'queued' CHECK (state IN ('queued','running','retry','succeeded','failed','cancelled','paused_restore')),
    priority integer NOT NULL DEFAULT 0 CHECK (priority BETWEEN -100 AND 100),
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0 AND attempts <= max_attempts),
    max_attempts integer NOT NULL DEFAULT 5 CHECK (max_attempts BETWEEN 1 AND 20),
    available_at timestamptz NOT NULL DEFAULT now(),
    lease_token uuid,
    lease_until timestamptz,
    error_code varchar(96) CHECK (error_code IS NULL OR error_code ~ '^[A-Za-z0-9._:-]+$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz,
    CHECK ((state='running') = (lease_token IS NOT NULL AND lease_until IS NOT NULL)),
    CHECK (state='running' OR (lease_token IS NULL AND lease_until IS NULL))
);
CREATE INDEX rf_jobs_ready ON rf_jobs (kind, priority DESC, available_at, id)
    WHERE state IN ('queued','retry');
CREATE INDEX rf_jobs_expired ON rf_jobs (lease_until, id) WHERE state='running';
CREATE INDEX rf_jobs_retention ON rf_jobs (finished_at, id)
    WHERE state IN ('succeeded','failed','cancelled');
