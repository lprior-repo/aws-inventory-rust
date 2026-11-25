CREATE VIEW IF NOT EXISTS aws_sns_topic AS
SELECT r.*,
       json_extract(properties, '$.topic_arn') AS topic_arn,
       json_extract(properties, '$.display_name') AS display_name,
       json_extract(properties, '$.owner') AS owner,
       json_extract(properties, '$.subscriptions_confirmed') AS subscriptions_confirmed,
       json_extract(properties, '$.subscriptions_pending') AS subscriptions_pending,
       json_extract(properties, '$.subscriptions_deleted') AS subscriptions_deleted,
       json_extract(properties, '$.policy_configured') AS policy_configured,
       json_extract(properties, '$.delivery_policy_configured') AS delivery_policy_configured,
       json_extract(properties, '$.kms_master_key_id') AS kms_master_key_id,
       json_extract(properties, '$.fifo_topic') AS fifo_topic,
       json_extract(properties, '$.content_based_deduplication') AS content_based_deduplication,
       json_extract(properties, '$.tags_collected') AS tags_collected
FROM latest_resources r WHERE kind = 'aws_sns_topic';

CREATE VIEW IF NOT EXISTS aws_sns_subscription AS
SELECT r.*,
       json_extract(properties, '$.subscription_arn') AS subscription_arn,
       json_extract(properties, '$.topic_arn') AS topic_arn,
       json_extract(properties, '$.protocol') AS protocol,
       json_extract(properties, '$.endpoint') AS endpoint,
       json_extract(properties, '$.owner') AS owner,
       json_extract(properties, '$.pending_confirmation') AS pending_confirmation,
       json_extract(properties, '$.identity_source') AS identity_source
FROM latest_resources r WHERE kind = 'aws_sns_subscription';

PRAGMA user_version = 10;
