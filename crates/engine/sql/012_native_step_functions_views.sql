CREATE VIEW IF NOT EXISTS aws_step_functions_state_machine AS
SELECT r.*,
       json_extract(properties, '$.state_machine_arn') AS state_machine_arn,
       json_extract(properties, '$.name') AS name,
       json_extract(properties, '$.state_machine_type') AS state_machine_type,
       json_extract(properties, '$.type') AS type,
       json_extract(properties, '$.status') AS status,
       json_extract(properties, '$.creation_date') AS creation_date,
       json_extract(properties, '$.role_arn') AS role_arn,
       json_extract(properties, '$.revision_id') AS revision_id,
       json_extract(properties, '$.description') AS description,
       json_extract(properties, '$.logging_configuration') AS logging_configuration,
       json_extract(properties, '$.tracing_configuration') AS tracing_configuration,
       json_extract(properties, '$.encryption_configuration') AS encryption_configuration,
       json_extract(properties, '$.tags_collected') AS tags_collected
FROM latest_resources r WHERE kind = 'aws_step_functions_state_machine';

CREATE VIEW IF NOT EXISTS aws_step_functions_activity AS
SELECT r.*,
       json_extract(properties, '$.activity_arn') AS activity_arn,
       json_extract(properties, '$.name') AS name,
       json_extract(properties, '$.creation_date') AS creation_date,
       json_extract(properties, '$.encryption_configuration') AS encryption_configuration,
       json_extract(properties, '$.tags_collected') AS tags_collected
FROM latest_resources r WHERE kind = 'aws_step_functions_activity';

PRAGMA user_version = 12;
