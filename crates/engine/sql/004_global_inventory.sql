-- Additive global-service query views. Observations and fingerprints are not
-- rewritten; all view creation and the version update share one transaction.
CREATE VIEW aws_s3_bucket AS
SELECT r.*, json_extract(properties, '$.bucket_name') AS bucket_name,
       json_extract(properties, '$.bucket_region') AS bucket_region,
       json_extract(properties, '$.creation_date') AS creation_date,
       json_extract(properties, '$.versioning') AS versioning,
       json_extract(properties, '$.encryption') AS encryption,
       json_extract(properties, '$.public_access_block') AS public_access_block,
       json_extract(properties, '$.policy_status') AS policy_status,
       json_extract(properties, '$.policy_status.is_public') AS policy_is_public,
       json_extract(properties, '$.logging') AS logging,
       json_extract(properties, '$.replication') AS replication,
       json_extract(properties, '$.website') AS website,
       json_extract(properties, '$.ownership_controls') AS ownership_controls,
       json_extract(properties, '$.notifications') AS notifications,
       json_extract(properties, '$.unavailable_sections') AS unavailable_sections
FROM latest_resources r WHERE kind = 'aws_s3_bucket';

CREATE VIEW aws_iam_role AS
SELECT r.*, json_extract(properties, '$.role_name') AS role_name,
       json_extract(properties, '$.role_id') AS role_id,
       json_extract(properties, '$.path') AS path,
       json_extract(properties, '$.create_date') AS create_date,
       json_extract(properties, '$.description') AS description,
       json_extract(properties, '$.max_session_duration') AS max_session_duration,
       json_extract(properties, '$.permissions_boundary_arn') AS permissions_boundary_arn,
       json_extract(properties, '$.last_used_date') AS last_used_date,
       json_extract(properties, '$.last_used_region') AS last_used_region,
       json_extract(properties, '$.trust_statements') AS trust_statements,
       json_extract(properties, '$.details_available') AS details_available
FROM latest_resources r WHERE kind = 'aws_iam_role';

CREATE VIEW aws_iam_instance_profile AS
SELECT r.*, json_extract(properties, '$.instance_profile_name') AS instance_profile_name,
       json_extract(properties, '$.instance_profile_id') AS instance_profile_id,
       json_extract(properties, '$.path') AS path,
       json_extract(properties, '$.create_date') AS create_date,
       json_extract(properties, '$.role_arns') AS role_arns,
       json_extract(properties, '$.details_available') AS details_available
FROM latest_resources r WHERE kind = 'aws_iam_instance_profile';

CREATE VIEW aws_iam_policy AS
SELECT r.*, json_extract(properties, '$.policy_name') AS policy_name,
       json_extract(properties, '$.policy_id') AS policy_id,
       json_extract(properties, '$.path') AS path,
       json_extract(properties, '$.default_version_id') AS default_version_id,
       json_extract(properties, '$.attachment_count') AS attachment_count,
       json_extract(properties, '$.permissions_boundary_usage_count') AS permissions_boundary_usage_count,
       json_extract(properties, '$.is_attachable') AS is_attachable,
       json_extract(properties, '$.description') AS description,
       json_extract(properties, '$.policy_scope') AS policy_scope,
       json_extract(properties, '$.policy_document_collected') AS policy_document_collected,
       json_extract(properties, '$.details_available') AS details_available
FROM latest_resources r WHERE kind = 'aws_iam_policy';

CREATE VIEW aws_route53_zone AS
SELECT r.*, json_extract(properties, '$.zone_id') AS zone_id,
       json_extract(properties, '$.private_zone') AS private_zone,
       json_extract(properties, '$.resource_record_set_count') AS resource_record_set_count,
       json_extract(properties, '$.name_servers') AS name_servers,
       json_extract(properties, '$.vpcs') AS vpcs,
       json_extract(properties, '$.linked_service_principal') AS linked_service_principal,
       json_extract(properties, '$.details_available') AS details_available
FROM latest_resources r WHERE kind = 'aws_route53_zone';

CREATE VIEW aws_route53_record AS
SELECT r.*, json_extract(properties, '$.zone_id') AS zone_id,
       json_extract(properties, '$.type') AS type,
       json_extract(properties, '$.set_identifier') AS set_identifier,
       json_extract(properties, '$.ttl') AS ttl,
       json_extract(properties, '$.records') AS records,
       json_extract(properties, '$.rdata_omitted') AS rdata_omitted,
       json_extract(properties, '$.rdata_count') AS rdata_count,
       json_extract(properties, '$.alias_target') AS alias_target,
       json_extract(properties, '$.weight') AS weight,
       json_extract(properties, '$.region') AS routing_region,
       json_extract(properties, '$.failover') AS failover,
       json_extract(properties, '$.health_check_id') AS health_check_id,
       json_extract(properties, '$.multi_value_answer') AS multi_value_answer
FROM latest_resources r WHERE kind = 'aws_route53_record';

CREATE VIEW aws_cloudfront_distribution AS
SELECT r.*, json_extract(properties, '$.distribution_id') AS distribution_id,
       json_extract(properties, '$.domain_name') AS domain_name,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.enabled') AS enabled,
       json_extract(properties, '$.staging') AS staging,
       json_extract(properties, '$.http_version') AS http_version,
       json_extract(properties, '$.is_ipv6_enabled') AS is_ipv6_enabled,
       json_extract(properties, '$.aliases') AS aliases,
       json_extract(properties, '$.origins') AS origins,
       json_extract(properties, '$.origin_groups') AS origin_groups,
       json_extract(properties, '$.default_cache_behavior') AS default_cache_behavior,
       json_extract(properties, '$.cache_behaviors') AS cache_behaviors,
       json_extract(properties, '$.viewer_certificate') AS viewer_certificate,
       json_extract(properties, '$.web_acl_id') AS web_acl_id
FROM latest_resources r WHERE kind = 'aws_cloudfront_distribution';

CREATE VIEW aws_cloudfront_origin AS
SELECT r.id AS distribution_arn, r.account_id, r.region, r.source, r.observed_at,
       json_extract(r.properties, '$.distribution_id') AS distribution_id,
       json_extract(origin.value, '$.id') AS origin_id,
       json_extract(origin.value, '$.domain_name') AS domain_name,
       json_extract(origin.value, '$.s3_origin') AS s3_origin,
       json_extract(origin.value, '$.vpc_origin_id') AS vpc_origin_id,
       json_extract(origin.value, '$.origin_access_control_id') AS origin_access_control_id,
       json_extract(origin.value, '$.custom_header_count') AS custom_header_count,
       origin.value AS properties
FROM latest_resources r, json_each(r.properties, '$.origins') origin
WHERE r.kind = 'aws_cloudfront_distribution';

CREATE VIEW aws_iam_instance_profile_role AS
SELECT r.id AS instance_profile_arn, r.account_id, r.region, r.source, r.observed_at,
       role.value AS role_arn
FROM latest_resources r, json_each(r.properties, '$.role_arns') role
WHERE r.kind = 'aws_iam_instance_profile' AND role.type = 'text';

CREATE VIEW aws_route53_zone_vpc AS
SELECT r.id AS zone_arn, r.account_id, r.source, r.observed_at,
       json_extract(r.properties, '$.zone_id') AS zone_id,
       json_extract(vpc.value, '$.vpc_id') AS vpc_id,
       json_extract(vpc.value, '$.vpc_region') AS vpc_region,
       json_extract(vpc.value, '$.owner_resolution') AS owner_resolution
FROM latest_resources r, json_each(r.properties, '$.vpcs') vpc
WHERE r.kind = 'aws_route53_zone';

PRAGMA user_version = 4;
