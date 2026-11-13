-- Native SQS queue projection over the durable observation table. The fields
-- mirror the collector's bounded attribute set; no policy document is exposed.
CREATE VIEW IF NOT EXISTS aws_sqs_queue AS
SELECT r.*,
       json_extract(properties, '$.queue_name') AS queue_name,
       json_extract(properties, '$.queue_arn') AS queue_arn,
       json_extract(properties, '$.queue_url') AS queue_url,
       json_extract(properties, '$.fifo_queue') AS fifo_queue,
       json_extract(properties, '$.approximate_number_of_messages') AS approximate_number_of_messages,
       json_extract(properties, '$.approximate_number_of_messages_not_visible') AS approximate_number_of_messages_not_visible,
       json_extract(properties, '$.visibility_timeout_seconds') AS visibility_timeout_seconds,
       json_extract(properties, '$.created_timestamp') AS created_timestamp,
       json_extract(properties, '$.kms_master_key_id') AS kms_master_key_id,
       json_extract(properties, '$.sqs_managed_sse_enabled') AS sqs_managed_sse_enabled,
       json_extract(properties, '$.redrive_max_receive_count') AS redrive_max_receive_count
FROM latest_resources r WHERE kind = 'aws_sqs_queue';

PRAGMA user_version = 9;
