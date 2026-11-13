-- Explicit account onboarding and per-scope coverage state for the continuous
-- CMDB control plane. These are control-plane rows, not inventory observations.
CREATE TABLE IF NOT EXISTS accounts (
    account_id TEXT PRIMARY KEY,
    partition TEXT NOT NULL,
    role_arn TEXT,
    status TEXT NOT NULL CHECK (status IN ('onboarded', 'suspended', 'removed')),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS coverage_registry (
    account_id TEXT NOT NULL,
    region TEXT NOT NULL,
    collector TEXT NOT NULL,
    state TEXT NOT NULL CHECK (
        state IN ('unknown', 'running', 'complete', 'partial', 'denied', 'unsupported', 'failed')
    ),
    last_scan_id TEXT,
    last_attempt_at INTEGER,
    last_success_at INTEGER,
    last_error_category TEXT,
    PRIMARY KEY (account_id, region, collector)
);

CREATE INDEX IF NOT EXISTS coverage_registry_state ON coverage_registry(state);

PRAGMA user_version = 8;
