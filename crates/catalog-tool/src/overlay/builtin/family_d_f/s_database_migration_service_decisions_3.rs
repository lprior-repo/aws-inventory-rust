// Explicit dispositions from .local/aws-models/models/database-migration-service/service/2016-01-01/database-migration-service-2016-01-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("StopDataMigration", "Requires DataMigrationIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the specified data migration."),
    ("StopReplication", "Requires ReplicationConfigArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For a given DMS Serverless replication configuration, DMS stops any and all ongoing DMS Serverless replications."),
    ("StopReplicationTask", "Requires ReplicationTaskArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the replication task."),
    ("TestConnection", "Requires ReplicationInstanceArn, EndpointArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Tests the connection between the replication instance and the endpoint."),
    ("UpdateSubscriptionsToEventBridge", "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Migrates 10 active and enabled Amazon SNS subscriptions at a time and converts them to corresponding Amazon EventBridge rules."),
];
