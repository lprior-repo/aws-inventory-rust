// Pinned operation purposes: .local/aws-models/models/s3tables/service/2018-05-10/s3tables-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CompleteTableMigration",
        Decision::Deferred(
            "Required selectors tableARN are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members . Pinned purpose: Completes the migration of a table. The table's most recent migration must be in the succeeded , failed or cancelled state. After you complete the migration, you can't sync the table again, and you can write to and maintain the table as you do any ot",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a namespace. A namespace is a logical grouping of tables within your table bucket, which you can use to organize tables. For more information, see Create a namespace in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:CreateNamespace permission to use this operation.
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new table associated with the given namespace in a table bucket. For more information, see Creating an Amazon S3 table in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:CreateTable permission to use this operation. If you use this operation with the optional metadata request parameter you must have the s3tables:PutTableData permission. If you use th
    ("CreateTable", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a table bucket. For more information, see Creating a table bucket in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:CreateTableBucket permission to use this operation. If you use this operation with the optional encryptionConfiguration parameter you must have the s3tables:PutTableBucketEncryption permission. If you use this operation with the storageC
    ("CreateTableBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a namespace. For more information, see Delete a namespace in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:DeleteNamespace permission to use this operation.
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a table. For more information, see Deleting an Amazon S3 table in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:DeleteTable permission to use this operation.
    ("DeleteTable", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a table bucket. For more information, see Deleting a table bucket in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:DeleteTableBucket permission to use this operation.
    ("DeleteTableBucket", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the encryption configuration for a table bucket. Permissions You must have the s3tables:DeleteTableBucketEncryption permission to use this operation.
    (
        "DeleteTableBucketEncryption",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the metrics configuration for a table bucket. Permissions You must have the s3tables:DeleteTableBucketMetricsConfiguration permission to use this operation.
    (
        "DeleteTableBucketMetricsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a table bucket policy. For more information, see Deleting a table bucket policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:DeleteTableBucketPolicy permission to use this operation.
    (
        "DeleteTableBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the replication configuration for a table bucket. After deletion, new table updates will no longer be replicated to destination buckets, though existing replicated tables will remain in destination buckets. Permissions You must have the s3tables:DeleteTableBucketReplication permission to use this operation.
    (
        "DeleteTableBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a table policy. For more information, see Deleting a table policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:DeleteTablePolicy permission to use this operation.
    ("DeleteTablePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the replication configuration for a specific table. After deletion, new updates to this table will no longer be replicated to destination tables, though existing replicated copies will remain in destination buckets. Permissions You must have the s3tables:DeleteTableReplication permission to use this operation.
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
    // Changes service state rather than reading inventory. Pinned purpose: Sets the encryption configuration for a table bucket. Permissions You must have the s3tables:PutTableBucketEncryption permission to use this operation. If you choose SSE-KMS encryption you must grant the S3 Tables maintenance principal access to your KMS key. For more information, see Permissions requirements for S3 Tables SSE-KMS encryption in the Amazon Simple Storage Service User Guide .
    (
        "PutTableBucketEncryption",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sets, updates, or removes the server access logging configuration for a table bucket. When logging is enabled, Amazon S3 delivers a record of the requests made to the table bucket and its tables to the target general purpose bucket that you specify. To disable logging, call this operation with an empty tableBucketLoggingStatus . For more information, see Server access logging for table buckets in
    (
        "PutTableBucketLogging",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new maintenance configuration or replaces an existing maintenance configuration for a table bucket. For more information, see Amazon S3 table bucket maintenance in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:PutTableBucketMaintenanceConfiguration permission to use this operation.
    (
        "PutTableBucketMaintenanceConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sets the metrics configuration for a table bucket. Permissions You must have the s3tables:PutTableBucketMetricsConfiguration permission to use this operation.
    (
        "PutTableBucketMetricsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new table bucket policy or replaces an existing table bucket policy for a table bucket. For more information, see Adding a table bucket policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:PutTableBucketPolicy permission to use this operation.
    (
        "PutTableBucketPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates the replication configuration for a table bucket. This operation defines how tables in the source bucket are replicated to destination buckets. Replication helps ensure data availability and disaster recovery across regions or accounts. Permissions You must have the s3tables:PutTableBucketReplication permission to use this operation. The IAM role specified in the configuration m
    (
        "PutTableBucketReplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Sets or updates the storage class configuration for a table bucket. This configuration serves as the default storage class for all new tables created in the bucket, allowing you to optimize storage costs at the bucket level. Permissions You must have the s3tables:PutTableBucketStorageClass permission to use this operation.
    (
        "PutTableBucketStorageClass",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new maintenance configuration or replaces an existing maintenance configuration for a table. For more information, see S3 Tables maintenance in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:PutTableMaintenanceConfiguration permission to use this operation.
    (
        "PutTableMaintenanceConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new table policy or replaces an existing table policy for a table. For more information, see Adding a table policy in the Amazon Simple Storage Service User Guide . Permissions You must have the s3tables:PutTablePolicy permission to use this operation.
    ("PutTablePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates the expiration configuration settings for records in a table, including the status of the configuration. If you enable record expiration for a table, records expire and are automatically removed from the table after the number of days that you specify. Permissions You must have the s3tables:PutTableRecordExpirationConfiguration permission to use this operation.
    (
        "PutTableRecordExpirationConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates the replication configuration for a specific table. This operation allows you to define table-level replication independently of bucket-level replication, providing granular control over which tables are replicated and where. Permissions You must have the s3tables:PutTableReplication permission to use this operation. The IAM role specified in the configuration must have permissi
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
    // Changes service state rather than reading inventory. Pinned purpose: Starts migrating an Amazon Web Services Glue Data Catalog table into a new table in a table bucket. The migration runs asynchronously: this operation returns the migration ID and its initial state, and you track progress with GetTableMigration . The namespace must already exist, and a table with the specified name must not. The migration creates the table during planning and commits data to it as
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
    // Changes service state rather than reading inventory. Pinned purpose: Applies one or more user-defined tags to an Amazon S3 Tables resource or updates existing tags. Each tag is a label consisting of a key and value pair. Tags can help you organize, track costs for, and control access to your resources. You can add up to 50 tags for each S3 resource. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . Permissions For tables a
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified user-defined tags from an Amazon S3 Tables resource. You can pass one or more tag keys. For a list of S3 resources that support tagging, see Managing tags for Amazon S3 resources . Permissions For tables and table buckets, you must have the s3tables:UntagResource permission to use this operation.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the metadata location for a table. The metadata location of a table must be an S3 URI that begins with the table's warehouse location. The metadata location for an Apache Iceberg table must end with .metadata.json , or if the metadata file is Gzip-compressed, .metadata.json.gz . Permissions You must have the s3tables:UpdateTableMetadataLocation permission to use this operation.
    (
        "UpdateTableMetadataLocation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Requests a state change for a migration. Identify the migration by its migrationId or by the tableARN of its destination table. Set requestedState to queued to continue a migration in the pendingReview state; the migration moves to queued or migrating . Set it to cancelled to cancel a migration in the new , planning , pendingReview , queued or migrating state; the migration moves to cancelling , f
    (
        "UpdateTableMigration",
        Decision::Denied(DenyReason::Mutating),
    ),
];
