-- Current selection is freshness-first across sources. Source authority only
-- breaks equal observation timestamps. Snapshot views (scan_resources,
-- scan_edges) and per-source latest_resource_sources keep their existing
-- priority-first semantics for provenance and reproducible exports.
DROP VIEW latest_resources;
DROP VIEW latest_edges;

CREATE VIEW latest_resources AS
SELECT scan_id, resource_id AS id, resource_id AS arn, account_id, region, kind,
       name, source, source_priority, observed_at, resource_json,
       json_extract(resource_json, '$.properties') AS properties,
       json_extract(resource_json, '$.tags') AS tags
FROM (
    SELECT r.*, row_number() OVER (
        PARTITION BY resource_id
        ORDER BY observed_at DESC, source_priority DESC, scan_id DESC,
                 scope_key, page_number DESC
    ) AS selection_rank
    FROM resource_observations r
) WHERE selection_rank = 1;

CREATE VIEW latest_edges AS
SELECT scan_id, from_id, to_id, relation, source, source_priority, observed_at, edge_json
FROM (
    SELECT e.*, row_number() OVER (
        PARTITION BY from_id, to_id, relation
        ORDER BY observed_at DESC, source_priority DESC, scan_id DESC,
                 scope_key, page_number DESC
    ) AS selection_rank
    FROM edge_observations e
) WHERE selection_rank = 1;

PRAGMA user_version = 7;
