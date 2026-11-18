// Explicit dispositions from .local/aws-models/models/database-migration-service/service/2016-01-01/database-migration-service-2016-01-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DescribeConversionConfiguration",
        "Requires MigrationProjectIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns configuration parameters for a schema conversion project.",
    ),
    (
        "DescribeEndpointSettings",
        "Requires EngineName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the possible endpoint settings available when you create an endpoint for a specific database engine.",
    ),
    (
        "DescribeEndpointTypes",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns information about the type of endpoints available.",
    ),
    (
        "DescribeEngineVersions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns information about the replication instance versions used in the project.",
    ),
    (
        "DescribeEventCategories",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists categories for all event source types, or, if specified, for a specified source type.",
    ),
    (
        "DescribeEvents",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists events for a given source identifier and source type.",
    ),
    (
        "DescribeExtensionPackAssociations",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeFleetAdvisorLsaAnalysis",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: End of support notice: On May 20, 2026, Amazon Web Services will end support for Amazon Web Services DMS Fleet Advisor;.",
    ),
    (
        "DescribeFleetAdvisorSchemaObjectSummary",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: End of support notice: On May 20, 2026, Amazon Web Services will end support for Amazon Web Services DMS Fleet Advisor;.",
    ),
    (
        "DescribeFleetAdvisorSchemas",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: End of support notice: On May 20, 2026, Amazon Web Services will end support for Amazon Web Services DMS Fleet Advisor;.",
    ),
    (
        "DescribeMetadataModel",
        "Requires SelectionRules, MigrationProjectIdentifier, Origin; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets detailed information about the specified metadata model, including its definition and corresponding converted objects in the target database if applicable.",
    ),
    (
        "DescribeMetadataModelAssessments",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeMetadataModelChildren",
        "Requires SelectionRules, MigrationProjectIdentifier, Origin; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a list of child metadata models for the specified metadata model in the database hierarchy.",
    ),
    (
        "DescribeMetadataModelConversions",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeMetadataModelCreations",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeMetadataModelExportsAsScript",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeMetadataModelExportsToTarget",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeMetadataModelImports",
        "Whole SchemaConversionRequest output includes ExportSqlDetails.ObjectURL for exported DDL archives; the model does not guarantee a secret-free URL. Selected-field projection cannot authorize the whole response.",
    ),
    (
        "DescribeOrderableReplicationInstances",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns information about the replication instance types that can be created in the specified region.",
    ),
    (
        "DescribeRecommendationLimitations",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: End of support notice: On May 20, 2026, Amazon Web Services will end support for Amazon Web Services DMS Fleet Advisor;.",
    ),
    (
        "DescribeRecommendations",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: End of support notice: On May 20, 2026, Amazon Web Services will end support for Amazon Web Services DMS Fleet Advisor;.",
    ),
    (
        "DescribeRefreshSchemasStatus",
        "Requires EndpointArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the status of the RefreshSchemas operation.",
    ),
    (
        "DescribeReplicationInstanceTaskLogs",
        "Requires ReplicationInstanceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the task logs for the specified task.",
    ),
    (
        "DescribeReplicationTableStatistics",
        "Requires ReplicationConfigArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns table and schema statistics for one or more provisioned replications that use a given DMS Serverless replication configuration.",
    ),
    (
        "DescribeReplicationTaskAssessmentResults",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns the task assessment results from the Amazon S3 bucket that DMS creates in your Amazon Web Services account.",
    ),
    (
        "DescribeSchemas",
        "Requires EndpointArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the schema for the specified endpoint.",
    ),
    (
        "DescribeTableStatistics",
        "Requires ReplicationTaskArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns table statistics on the database migration task, including table name, rows inserted, rows updated, and rows deleted.",
    ),
];
