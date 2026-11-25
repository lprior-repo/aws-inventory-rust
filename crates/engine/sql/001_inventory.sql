CREATE TABLE scans (
    scan_id TEXT PRIMARY KEY NOT NULL,
    started_at INTEGER NOT NULL CHECK (started_at >= 0),
    finished_at INTEGER CHECK (finished_at >= started_at),
    status TEXT NOT NULL CHECK (status IN ('running', 'complete', 'partial', 'cancelled'))
) STRICT;

CREATE TABLE scope_checkpoints (
    scan_id TEXT NOT NULL REFERENCES scans(scan_id),
    scope_key TEXT NOT NULL,
    account_id TEXT NOT NULL,
    region TEXT NOT NULL,
    collector TEXT NOT NULL,
    parent_id TEXT,
    source TEXT NOT NULL CHECK (source IN ('direct', 'aws_config', 'resource_explorer')),
    last_page INTEGER NOT NULL DEFAULT 0 CHECK (last_page >= 0),
    next_token TEXT,
    complete INTEGER NOT NULL DEFAULT 0 CHECK (complete IN (0, 1)),
    failed INTEGER NOT NULL DEFAULT 0 CHECK (failed IN (0, 1)),
    PRIMARY KEY (scan_id, scope_key)
) STRICT;

CREATE TABLE collection_pages (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    page_number INTEGER NOT NULL CHECK (page_number >= 1),
    fingerprint BLOB NOT NULL CHECK (length(fingerprint) = 32),
    next_token TEXT,
    resource_count INTEGER NOT NULL CHECK (resource_count >= 0),
    edge_count INTEGER NOT NULL CHECK (edge_count >= 0),
    PRIMARY KEY (scan_id, scope_key, page_number),
    FOREIGN KEY (scan_id, scope_key) REFERENCES scope_checkpoints(scan_id, scope_key)
) STRICT;

CREATE INDEX collection_page_tokens ON collection_pages(scan_id, scope_key, next_token);

CREATE TABLE resource_observations (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    page_number INTEGER NOT NULL,
    resource_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    region TEXT NOT NULL,
    kind TEXT NOT NULL,
    name TEXT,
    source TEXT NOT NULL CHECK (source IN ('direct', 'aws_config', 'resource_explorer')),
    source_priority INTEGER NOT NULL CHECK (source_priority BETWEEN 1 AND 3),
    observed_at INTEGER NOT NULL CHECK (observed_at >= 0),
    resource_json TEXT NOT NULL CHECK (json_valid(resource_json)),
    PRIMARY KEY (scan_id, scope_key, page_number, resource_id),
    FOREIGN KEY (scan_id, scope_key, page_number)
        REFERENCES collection_pages(scan_id, scope_key, page_number)
) STRICT;

CREATE INDEX resource_observation_identity
    ON resource_observations(resource_id, source, observed_at DESC);
CREATE INDEX resource_observation_scan ON resource_observations(scan_id, resource_id);
CREATE INDEX resource_observation_kind ON resource_observations(kind, resource_id);

CREATE TABLE edge_observations (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    page_number INTEGER NOT NULL,
    from_id TEXT NOT NULL,
    to_id TEXT NOT NULL,
    relation TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('direct', 'aws_config', 'resource_explorer')),
    source_priority INTEGER NOT NULL CHECK (source_priority BETWEEN 1 AND 3),
    observed_at INTEGER NOT NULL CHECK (observed_at >= 0),
    edge_json TEXT NOT NULL CHECK (json_valid(edge_json)),
    PRIMARY KEY (scan_id, scope_key, page_number, from_id, to_id, relation),
    FOREIGN KEY (scan_id, scope_key, page_number)
        REFERENCES collection_pages(scan_id, scope_key, page_number)
) STRICT;

CREATE INDEX edge_observation_identity
    ON edge_observations(from_id, to_id, relation, source, observed_at DESC);
CREATE INDEX edge_observation_scan ON edge_observations(scan_id, from_id, to_id, relation);

CREATE TABLE collection_failures (
    scan_id TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    PRIMARY KEY (scan_id, scope_key, category, message),
    FOREIGN KEY (scan_id, scope_key) REFERENCES scope_checkpoints(scan_id, scope_key)
) STRICT;

CREATE TABLE graph_export_checkpoints (
    scan_id TEXT NOT NULL REFERENCES scans(scan_id),
    target TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    resources_exported INTEGER NOT NULL CHECK (resources_exported >= 0),
    edges_exported INTEGER NOT NULL CHECK (edges_exported >= 0),
    complete INTEGER NOT NULL CHECK (complete IN (0, 1)),
    PRIMARY KEY (scan_id, target, snapshot_id)
) STRICT;

CREATE VIEW scan_resources AS
SELECT scan_id, resource_id AS id, resource_id AS arn, account_id, region, kind,
       name, source, source_priority, observed_at, resource_json,
       json_extract(resource_json, '$.properties') AS properties,
       json_extract(resource_json, '$.tags') AS tags
FROM (
    SELECT r.*, row_number() OVER (
        PARTITION BY scan_id, resource_id
        ORDER BY source_priority DESC, observed_at DESC, scope_key, page_number DESC
    ) AS selection_rank
    FROM resource_observations r
) WHERE selection_rank = 1;

CREATE VIEW latest_resource_sources AS
SELECT scan_id, resource_id AS id, resource_id AS arn, account_id, region, kind,
       name, source, source_priority, observed_at, resource_json,
       json_extract(resource_json, '$.properties') AS properties,
       json_extract(resource_json, '$.tags') AS tags
FROM (
    SELECT r.*, row_number() OVER (
        PARTITION BY resource_id, source
        ORDER BY observed_at DESC, scan_id DESC, scope_key, page_number DESC
    ) AS selection_rank
    FROM resource_observations r
) WHERE selection_rank = 1;

CREATE VIEW latest_resources AS
SELECT scan_id, id, arn, account_id, region, kind, name, source, source_priority,
       observed_at, resource_json, properties, tags
FROM (
    SELECT r.*, row_number() OVER (
        PARTITION BY id ORDER BY source_priority DESC, observed_at DESC, scan_id DESC
    ) AS selection_rank
    FROM latest_resource_sources r
) WHERE selection_rank = 1;

CREATE VIEW scan_edges AS
SELECT scan_id, from_id, to_id, relation, source, source_priority, observed_at, edge_json
FROM (
    SELECT e.*, row_number() OVER (
        PARTITION BY scan_id, from_id, to_id, relation
        ORDER BY source_priority DESC, observed_at DESC, scope_key, page_number DESC
    ) AS selection_rank
    FROM edge_observations e
) WHERE selection_rank = 1;

CREATE VIEW latest_edges AS
SELECT scan_id, from_id, to_id, relation, source, source_priority, observed_at, edge_json
FROM (
    SELECT e.*, row_number() OVER (
        PARTITION BY from_id, to_id, relation
        ORDER BY source_priority DESC, observed_at DESC, scan_id DESC, scope_key, page_number DESC
    ) AS selection_rank
    FROM edge_observations e
) WHERE selection_rank = 1;

CREATE VIEW scan_coverage AS
SELECT c.*, s.status AS scan_status, s.started_at, s.finished_at,
       CASE WHEN c.failed = 1 THEN 'failed'
            WHEN c.complete = 1 THEN 'complete' ELSE 'incomplete' END AS coverage_status
FROM scope_checkpoints c JOIN scans s ON c.scan_id = s.scan_id;

CREATE VIEW aws_resource AS SELECT * FROM latest_resources;

CREATE VIEW aws_ec2_instance AS
SELECT r.*, json_extract(properties, '$.instance_id') AS instance_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_id') AS subnet_id,
       json_extract(properties, '$.instance_type') AS instance_type,
       json_extract(properties, '$.state') AS instance_state,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.private_ip_address') AS private_ip_address,
       json_extract(properties, '$.public_ip_address') AS public_ip_address,
       json_extract(properties, '$.image_id') AS image_id,
       json_extract(properties, '$.architecture') AS architecture
FROM latest_resources r WHERE kind = 'aws_ec2_instance';

CREATE VIEW aws_vpc AS
SELECT r.*, json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.cidr_block') AS cidr_block,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.is_default') AS is_default
FROM latest_resources r WHERE kind = 'aws_vpc';

CREATE VIEW aws_vpc_subnet AS
SELECT r.*, json_extract(properties, '$.subnet_id') AS subnet_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.cidr_block') AS cidr_block,
       json_extract(properties, '$.availability_zone') AS availability_zone,
       json_extract(properties, '$.state') AS state
FROM latest_resources r WHERE kind = 'aws_vpc_subnet';

CREATE VIEW aws_vpc_security_group AS
SELECT r.*, json_extract(properties, '$.group_id') AS group_id,
       json_extract(properties, '$.group_name') AS group_name,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.description') AS description,
       json_extract(properties, '$.ip_permissions') AS ip_permissions
FROM latest_resources r WHERE kind = 'aws_vpc_security_group';

CREATE VIEW aws_ec2_network_interface AS
SELECT r.*, json_extract(properties, '$.network_interface_id') AS network_interface_id,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_id') AS subnet_id,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.private_ip_address') AS private_ip_address,
       json_extract(properties, '$.interface_type') AS interface_type,
       json_extract(properties, '$.attachment') AS attachment
FROM latest_resources r WHERE kind = 'aws_ec2_network_interface';

CREATE VIEW aws_ebs_volume AS
SELECT r.*, json_extract(properties, '$.volume_id') AS volume_id,
       json_extract(properties, '$.size') AS size,
       json_extract(properties, '$.state') AS state,
       json_extract(properties, '$.volume_type') AS volume_type,
       json_extract(properties, '$.encrypted') AS encrypted,
       json_extract(properties, '$.availability_zone') AS availability_zone,
       json_extract(properties, '$.attachments') AS attachments
FROM latest_resources r WHERE kind = 'aws_ebs_volume';

CREATE VIEW aws_dynamodb_table AS
SELECT r.*, json_extract(properties, '$.table_name') AS table_name,
       json_extract(properties, '$.table_status') AS table_status,
       json_extract(properties, '$.item_count') AS item_count,
       json_extract(properties, '$.table_size_bytes') AS table_size_bytes,
       json_extract(properties, '$.key_schema') AS key_schema,
       json_extract(properties, '$.billing_mode') AS billing_mode
FROM latest_resources r WHERE kind = 'aws_dynamodb_table';

CREATE VIEW aws_lambda_function AS
SELECT r.*, json_extract(properties, '$.function_name') AS function_name,
       json_extract(properties, '$.runtime') AS runtime,
       json_extract(properties, '$.handler') AS handler,
       json_extract(properties, '$.memory_size') AS memory_size,
       json_extract(properties, '$.timeout') AS timeout,
       json_extract(properties, '$.role') AS role,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_ids') AS subnet_ids
FROM latest_resources r WHERE kind = 'aws_lambda_function';

CREATE VIEW aws_ecs_cluster AS
SELECT r.*, json_extract(properties, '$.cluster_arn') AS cluster_arn,
       json_extract(properties, '$.cluster_name') AS cluster_name,
       json_extract(properties, '$.status') AS status
FROM latest_resources r WHERE kind = 'aws_ecs_cluster';

CREATE VIEW aws_ecs_service AS
SELECT r.*, json_extract(properties, '$.service_arn') AS service_arn,
       json_extract(properties, '$.service_name') AS service_name,
       json_extract(properties, '$.cluster_arn') AS cluster_arn,
       json_extract(properties, '$.task_definition') AS task_definition,
       json_extract(properties, '$.desired_count') AS desired_count,
       json_extract(properties, '$.running_count') AS running_count
FROM latest_resources r WHERE kind = 'aws_ecs_service';

CREATE VIEW aws_ecs_task AS
SELECT r.*, json_extract(properties, '$.task_arn') AS task_arn,
       json_extract(properties, '$.cluster_arn') AS cluster_arn,
       json_extract(properties, '$.task_definition_arn') AS task_definition_arn,
       json_extract(properties, '$.last_status') AS last_status,
       json_extract(properties, '$.desired_status') AS desired_status
FROM latest_resources r WHERE kind = 'aws_ecs_task';

CREATE VIEW aws_eks_cluster AS
SELECT r.*, json_extract(properties, '$.name') AS cluster_name,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.version') AS version,
       json_extract(properties, '$.endpoint') AS endpoint,
       json_extract(properties, '$.vpc_id') AS vpc_id,
       json_extract(properties, '$.subnet_ids') AS subnet_ids
FROM latest_resources r WHERE kind = 'aws_eks_cluster';

CREATE VIEW aws_eks_node_group AS
SELECT r.*, json_extract(properties, '$.nodegroup_name') AS nodegroup_name,
       json_extract(properties, '$.cluster_name') AS cluster_name,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.instance_types') AS instance_types,
       json_extract(properties, '$.node_role') AS node_role,
       json_extract(properties, '$.subnets') AS subnets
FROM latest_resources r WHERE kind = 'aws_eks_node_group';

PRAGMA user_version = 1;
