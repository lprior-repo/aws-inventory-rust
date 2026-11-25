use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CompleteTableMigration",
        Decision::Deferred(
            "Required selectors tableARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Completes the migration of a table. The table's most recent migration must be in the succeeded , failed or cancelled state. After you complete the migration, you can't sync the table again, and you can write to and maintain the table as you do any ot",
        ),
    ),
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    ("CreateTable", Decision::Denied(DenyReason::Mutating)),
    ("CreateTableBucket", Decision::Denied(DenyReason::Mutating)),
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTable", Decision::Denied(DenyReason::Mutating)),
    ("DeleteTableBucket", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTableBucketEncryption",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteTableBucketMetricsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteTableBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteTableBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTablePolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteTableReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetNamespace",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members namespace, createdAt, createdBy, ownerAccountId, namespaceId, tableBucketId. Pinned purpose: Gets details about a namespace. For more information, see Table namespaces in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetNamespace permission to use this operation.",
        ),
    ),
    (
        "GetTable",
        Decision::Deferred(
            "Requires one of tableArn or tableBucketARN+namespace+name; all are optional in the Smithy structure. ListTables is the parent-bound table root; choosing one incomplete selector silently skips valid tables.",
        ),
    ),
    (
        "GetTableBucket",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members arn, name, ownerAccountId, createdAt, tableBucketId, type. Pinned purpose: Gets details on a table bucket. For more information, see Viewing details about an Amazon S3 table bucket in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableBucket permission to use this operation.",
        ),
    ),
    (
        "GetTableBucketEncryption",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members encryptionConfiguration. Pinned purpose: Gets the encryption configuration for a table bucket. Permissions You must have the s3tables:GetTableBucketEncryption permission to use this operation.",
        ),
    ),
    (
        "GetTableBucketLogging",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableBucketLoggingStatus. Pinned purpose: Gets the server access logging configuration for a table bucket. For more information, see Server access logging for table buckets in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableBucketLogging permissi",
        ),
    ),
    (
        "GetTableBucketMaintenanceConfiguration",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableBucketARN, configuration. Pinned purpose: Gets details about a maintenance configuration for a given table bucket. For more information, see Amazon S3 table bucket maintenance in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableBucketMaintenanceCo",
        ),
    ),
    (
        "GetTableBucketMetricsConfiguration",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableBucketARN, id. Pinned purpose: Gets the metrics configuration for a table bucket. Permissions You must have the s3tables:GetTableBucketMetricsConfiguration permission to use this operation.",
        ),
    ),
    (
        "GetTableBucketPolicy",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourcePolicy. Pinned purpose: Gets details about a table bucket policy. For more information, see Viewing a table bucket policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableBucketPolicy permission to use this operation.",
        ),
    ),
    (
        "GetTableBucketReplication",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members versionToken, configuration. Pinned purpose: Retrieves the replication configuration for a table bucket.This operation returns the IAM role, versionToken , and replication rules that define how tables in this bucket are replicated to other buckets. Permissions You must have the s3tables:GetTabl",
        ),
    ),
    (
        "GetTableBucketStorageClass",
        Decision::Deferred(
            "Required selectors tableBucketARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members storageClassConfiguration. Pinned purpose: Retrieves the storage class configuration for a specific table. This allows you to view the storage class settings that apply to an individual table, which may differ from the table bucket's default configuration. Permissions You must have the s3tabl",
        ),
    ),
    (
        "GetTableEncryption",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members encryptionConfiguration. Pinned purpose: Gets the encryption configuration for a table. Permissions You must have the s3tables:GetTableEncryption permission to use this operation.",
        ),
    ),
    (
        "GetTableMaintenanceConfiguration",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableARN, configuration. Pinned purpose: Gets details about the maintenance configuration of a table. For more information, see S3 Tables maintenance in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableMaintenanceConfiguration permission to use t",
        ),
    ),
    (
        "GetTableMaintenanceJobStatus",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableARN, status. Pinned purpose: Gets the status of a maintenance job for a table. For more information, see S3 Tables maintenance in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTableMaintenanceJobStatus permission to use this operation.",
        ),
    ),
    (
        "GetTableMetadataLocation",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members versionToken, metadataLocation, warehouseLocation. Pinned purpose: Gets the location of the table metadata. Permissions You must have the s3tables:GetTableMetadataLocation permission to use this operation.",
        ),
    ),
    (
        "GetTableMigration",
        Decision::Deferred(
            "Requires one of migrationId or table ARN/bucket/name selector combinations, none modeled required. ListTableMigrations enumerates complete migration metadata.",
        ),
    ),
    (
        "GetTablePolicy",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourcePolicy. Pinned purpose: Gets details about a table policy. For more information, see Viewing a table policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:GetTablePolicy permission to use this operation.",
        ),
    ),
    (
        "GetTableRecordExpirationConfiguration",
        Decision::Deferred(
            "Required selectors tableArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members configuration. Pinned purpose: Retrieves the expiration configuration settings for records in a table, and the status of the configuration. If the status of the configuration is enabled , records expire and are automatically removed from the table after the specified number of day",
        ),
    ),
    (
        "GetTableRecordExpirationJobStatus",
        Decision::Deferred(
            "Required selectors tableArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status, lastRunTimestamp, failureMessage, metrics. Pinned purpose: Retrieves the status, metrics, and details of the latest record expiration job for a table. This includes when the job ran, and whether it succeeded or failed. If the job ran successfully, this also includes statistics about the records that were rem",
        ),
    ),
    (
        "GetTableReplication",
        Decision::Deferred(
            "Required selectors tableArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members versionToken, configuration. Pinned purpose: Retrieves the replication configuration for a specific table. Permissions You must have the s3tables:GetTableReplication permission to use this operation.",
        ),
    ),
    (
        "GetTableReplicationStatus",
        Decision::Deferred(
            "Required selectors tableArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members sourceTableArn, destinations. Pinned purpose: Retrieves the replication status for a table, including the status of replication to each destination. This operation provides visibility into replication health and progress. Permissions You must have the s3tables:GetTableReplicationStatus permissio",
        ),
    ),
    (
        "GetTableStorageClass",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members storageClassConfiguration. Pinned purpose: Retrieves the storage class configuration for a specific table. This allows you to view the storage class settings that apply to an individual table, which may differ from the table bucket's default configuration. Permissions You must have the s3tabl",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists all of the tags applied to a specified Amazon S3 Tables resource. Each tag is a label consisting of a key and value pair. Tags can help you organize, track costs for, and control access to resources. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . P",
        ),
    ),
    (
        "PutTableBucketEncryption",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketLogging",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketMaintenanceConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketMetricsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableBucketStorageClass",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableMaintenanceConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutTablePolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "PutTableRecordExpirationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "PutTableReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RenameTable",
        Decision::Deferred(
            "Required selectors tableBucketARN, namespace, name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Renames a table or a namespace. For more information, see S3 Tables in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:RenameTable permission to use this operation.",
        ),
    ),
    (
        "StartTableMigration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "SyncTableMigration",
        Decision::Deferred(
            "Required selectors roleARN, tableARN, migrationOutput are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members migrationId, state. Pinned purpose: Migrates the changes in the source table since the previous start or sync, such as added and deleted source objects, into the destination table. The sync runs asynchronously: this operation returns the migration ID of the sync and its initial state, ",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateTableMetadataLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateTableMigration",
        Decision::Denied(DenyReason::Mutating),
    ),
];
