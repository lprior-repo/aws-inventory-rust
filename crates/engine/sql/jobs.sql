CREATE TABLE inventory_job_schema (
    singleton INTEGER PRIMARY KEY CHECK(singleton=1),
    version INTEGER NOT NULL CHECK(version=1)
) STRICT;
INSERT INTO inventory_job_schema VALUES(1,1);
CREATE TABLE inventory_jobs (
    job_id TEXT PRIMARY KEY CHECK(length(job_id) BETWEEN 1 AND 256),
    kind TEXT NOT NULL CHECK(length(kind) BETWEEN 1 AND 64),
    payload TEXT NOT NULL CHECK(json_valid(payload) AND length(CAST(payload AS BLOB))<=65536),
    state TEXT NOT NULL CHECK(state IN ('queued','running','succeeded','partial','failed','cancelled')),
    attempt INTEGER NOT NULL DEFAULT 0 CHECK(attempt BETWEEN 0 AND 32),
    initial_max_attempts INTEGER NOT NULL CHECK(initial_max_attempts BETWEEN 1 AND 32),
    max_attempts INTEGER NOT NULL CHECK(max_attempts BETWEEN 1 AND 32 AND attempt<=max_attempts),
    available_at INTEGER NOT NULL CHECK(available_at>=0),
    created_at INTEGER NOT NULL CHECK(created_at>=0),
    updated_at INTEGER NOT NULL CHECK(updated_at>=created_at),
    lease_owner TEXT,
    lease_token TEXT,
    lease_until INTEGER,
    cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)),
    last_error TEXT CHECK(length(CAST(last_error AS BLOB))<=8192),
    result TEXT CHECK(result IS NULL OR (json_valid(result) AND length(CAST(result AS BLOB))<=1048576)),
    CHECK((state='running' AND lease_owner IS NOT NULL AND lease_token IS NOT NULL AND lease_until IS NOT NULL)
       OR (state!='running' AND lease_owner IS NULL AND lease_token IS NULL AND lease_until IS NULL))
) STRICT;
CREATE INDEX inventory_jobs_ready ON inventory_jobs(state,available_at,created_at,job_id);
CREATE INDEX inventory_jobs_leases ON inventory_jobs(state,lease_until);
CREATE TABLE inventory_job_attempts (
    job_id TEXT NOT NULL REFERENCES inventory_jobs(job_id),
    attempt INTEGER NOT NULL CHECK(attempt BETWEEN 1 AND 32),
    worker TEXT NOT NULL,
    started_at INTEGER NOT NULL CHECK(started_at>=0),
    finished_at INTEGER,
    state TEXT NOT NULL CHECK(state IN ('running','succeeded','partial','failed','cancelled','expired')),
    error TEXT CHECK(length(CAST(error AS BLOB))<=8192),
    PRIMARY KEY(job_id,attempt)
) STRICT;
PRAGMA application_id=1095325521;
