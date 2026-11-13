-- The caller holds BEGIN IMMEDIATE with foreign_keys disabled. It retains and
-- restores all views/triggers and indexes on these tables in the transaction.
CREATE TABLE new_scope_checkpoints (
    scan_id TEXT NOT NULL REFERENCES scans(scan_id),
    scope_key TEXT NOT NULL,
    account_id TEXT NOT NULL,
    region TEXT NOT NULL,
    collector TEXT NOT NULL,
    parent_id TEXT,
    source TEXT NOT NULL CHECK (source IN ('direct', 'cloud_control', 'aws_config', 'resource_explorer')),
    last_page INTEGER NOT NULL DEFAULT 0 CHECK (last_page >= 0),
    next_token TEXT,
    complete INTEGER NOT NULL DEFAULT 0 CHECK (complete IN (0, 1)),
    failed INTEGER NOT NULL DEFAULT 0 CHECK (failed IN (0, 1)),
    PRIMARY KEY (scan_id, scope_key)
) STRICT;
INSERT INTO new_scope_checkpoints SELECT * FROM scope_checkpoints;
DROP TABLE scope_checkpoints;
ALTER TABLE new_scope_checkpoints RENAME TO scope_checkpoints;

CREATE TABLE new_resource_observations (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    page_number INTEGER NOT NULL,
    resource_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    region TEXT NOT NULL,
    kind TEXT NOT NULL,
    name TEXT,
    source TEXT NOT NULL CHECK (source IN ('direct', 'cloud_control', 'aws_config', 'resource_explorer')),
    source_priority INTEGER NOT NULL CHECK (source_priority BETWEEN 1 AND 4),
    observed_at INTEGER NOT NULL CHECK (observed_at >= 0),
    resource_json TEXT NOT NULL CHECK (json_valid(resource_json)),
    PRIMARY KEY (scan_id, scope_key, page_number, resource_id),
    FOREIGN KEY (scan_id, scope_key, page_number)
        REFERENCES collection_pages(scan_id, scope_key, page_number)
) STRICT;
INSERT INTO new_resource_observations
SELECT scan_id, scope_key, page_number, resource_id, account_id, region, kind,
       name, source, CASE WHEN source='direct' THEN 4 ELSE source_priority END,
       observed_at, resource_json FROM resource_observations;
DROP TABLE resource_observations;
ALTER TABLE new_resource_observations RENAME TO resource_observations;

CREATE TABLE new_edge_observations (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    page_number INTEGER NOT NULL,
    from_id TEXT NOT NULL,
    to_id TEXT NOT NULL,
    relation TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('direct', 'cloud_control', 'aws_config', 'resource_explorer')),
    source_priority INTEGER NOT NULL CHECK (source_priority BETWEEN 1 AND 4),
    observed_at INTEGER NOT NULL CHECK (observed_at >= 0),
    edge_json TEXT NOT NULL CHECK (json_valid(edge_json)),
    PRIMARY KEY (scan_id, scope_key, page_number, from_id, to_id, relation),
    FOREIGN KEY (scan_id, scope_key, page_number)
        REFERENCES collection_pages(scan_id, scope_key, page_number)
) STRICT;
INSERT INTO new_edge_observations
SELECT scan_id, scope_key, page_number, from_id, to_id, relation, source,
       CASE WHEN source='direct' THEN 4 ELSE source_priority END,
       observed_at, edge_json FROM edge_observations;
DROP TABLE edge_observations;
ALTER TABLE new_edge_observations RENAME TO edge_observations;

CREATE VIEW aws_ecs_service AS
SELECT r.*, json_extract(properties, '$.service_arn') AS service_arn,
       json_extract(properties, '$.service_name') AS service_name,
       json_extract(properties, '$.cluster_arn') AS cluster_arn,
       json_extract(properties, '$.task_definition') AS task_definition,
       json_extract(properties, '$.desired_count') AS desired_count,
       json_extract(properties, '$.running_count') AS running_count,
       json_extract(properties, '$.load_balancers') AS load_balancers,
       json_extract(properties, '$.target_group_arns') AS target_group_arns,
       json_extract(properties, '$.classic_load_balancer_names') AS classic_load_balancer_names
FROM latest_resources r WHERE kind = 'aws_ecs_service';

CREATE VIEW aws_ecs_task_definition AS
SELECT r.*, json_extract(properties, '$.task_definition_arn') AS task_definition_arn,
       json_extract(properties, '$.family') AS family,
       json_extract(properties, '$.revision') AS revision,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.discovery_status_filter') AS discovery_status_filter,
       json_extract(properties, '$.task_role_arn') AS task_role_arn,
       json_extract(properties, '$.execution_role_arn') AS execution_role_arn,
       json_extract(properties, '$.network_mode') AS network_mode,
       json_extract(properties, '$.cpu') AS cpu,
       json_extract(properties, '$.memory') AS memory,
       json_extract(properties, '$.requires_compatibilities') AS requires_compatibilities,
       json_extract(properties, '$.runtime_platform') AS runtime_platform,
       json_extract(properties, '$.container_definitions') AS container_definitions,
       json_extract(properties, '$.volumes') AS volumes
FROM latest_resources r WHERE kind = 'aws_ecs_task_definition';

CREATE VIEW aws_cloudcontrol_resource AS
SELECT r.*, json_extract(properties, '$.cloudformation_type') AS cloudformation_type,
       json_extract(properties, '$.resource_identifier') AS resource_identifier,
       json_extract(properties, '$.provider_properties') AS provider_properties
FROM latest_resource_sources r WHERE source = 'cloud_control';

PRAGMA user_version = 3;
