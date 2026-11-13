-- Native KMS projections over the durable observation table. Key policies and
-- key material are never stored; rotation state is exposed only when collected.
CREATE VIEW IF NOT EXISTS aws_kms_key AS
SELECT r.*,
       json_extract(properties, '$.key_arn') AS key_arn,
       json_extract(properties, '$.key_id') AS key_id,
       json_extract(properties, '$.aws_account_id') AS aws_account_id,
       json_extract(properties, '$.description') AS description,
       json_extract(properties, '$.key_manager') AS key_manager,
       json_extract(properties, '$.key_usage') AS key_usage,
       json_extract(properties, '$.key_state') AS key_state,
       json_extract(properties, '$.key_spec') AS key_spec,
       json_extract(properties, '$.origin') AS origin,
       json_extract(properties, '$.enabled') AS enabled,
       json_extract(properties, '$.multi_region') AS multi_region,
       json_extract(properties, '$.creation_date') AS creation_date,
       json_extract(properties, '$.rotation_enabled') AS rotation_enabled,
       json_extract(properties, '$.rotation_period_in_days') AS rotation_period_in_days,
       json_extract(properties, '$.rotation_collected') AS rotation_collected,
       json_extract(properties, '$.tags_collected') AS tags_collected
FROM latest_resources r WHERE kind = 'aws_kms_key';

CREATE VIEW IF NOT EXISTS aws_kms_alias AS
SELECT r.*,
       json_extract(properties, '$.alias_arn') AS alias_arn,
       json_extract(properties, '$.alias_name') AS alias_name,
       json_extract(properties, '$.target_key_id') AS target_key_id,
       json_extract(properties, '$.creation_date') AS creation_date,
       json_extract(properties, '$.last_updated_date') AS last_updated_date,
       json_extract(properties, '$.tags_collected') AS tags_collected
FROM latest_resources r WHERE kind = 'aws_kms_alias';

PRAGMA user_version = 11;
