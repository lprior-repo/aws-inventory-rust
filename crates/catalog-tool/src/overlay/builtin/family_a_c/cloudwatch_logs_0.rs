use super::support::*;
use crate::overlay::{
    ExecutionBranch, IdentityBranch, IdentityChoice, IdentityLeaf, IdentityPredicate,
};

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateKmsKey",
            deferred(
                "Model requires kmsKeyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified KMS key with either one log group in the account, or with all stored CloudWatch Logs query insights results in the account. When you use AssociateKmsKey , you specify either the logGroupName parameter or…",
            ),
        ),
        (
            "AssociateSourceToS3TableIntegration",
            deferred(
                "Model requires integrationArn, dataSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates a data source with an S3 Table Integration for query access in the 'logs' namespace. This enables querying log data using analytics engines that support Iceberg such as Amazon Athena, Amazon Redshift, and Apache Spark.",
            ),
        ),
        (
            "CancelExportTask",
            deferred(
                "Model requires taskId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels the specified export task. The task must be in the PENDING or RUNNING state.",
            ),
        ),
        (
            "CancelImportTask",
            deferred(
                "Model requires importId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Cancels an active import task and stops importing data from the CloudTrail Lake Event Data Store.",
            ),
        ),
        (
            "CreateDelivery",
            deferred(
                "Model requires deliverySourceName, deliveryDestinationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a delivery . A delivery is a connection between a logical delivery source and a logical delivery destination that you have already created. Only some Amazon Web Services services support being configured as a delivery…",
            ),
        ),
        (
            "CreateExportTask",
            deferred(
                "Model requires logGroupName, from, to, destination. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an export task so that you can efficiently export data from a log group to an Amazon S3 bucket. When you perform a CreateExportTask operation, you must use credentials that have permission to write to the S3 bucket that…",
            ),
        ),
        (
            "CreateImportTask",
            deferred(
                "Model requires importSourceArn, importRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an import from a data source to CloudWatch Log and creates a managed log group as the destination for the imported data. Currently, CloudTrail Event Data Store is the only supported data source. The import task must…",
            ),
        ),
        (
            "CreateLogAnomalyDetector",
            deferred(
                "Model requires logGroupArnList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an anomaly detector that regularly scans one or more log groups and look for patterns and anomalies in the logs. An anomaly detector can help surface issues by automatically discovering anomalies in your log event…",
            ),
        ),
        (
            "CreateLogGroup",
            deferred(
                "Model requires logGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a log group with the specified name. You can create up to 1,000,000 log groups per Region per account. You must use the following guidelines when naming a log group: Log group names must be unique within a Region for an…",
            ),
        ),
        (
            "CreateLogStream",
            deferred(
                "Model requires logGroupName, logStreamName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a log stream for the specified log group. A log stream is a sequence of log events that originate from a single source, such as an application instance or a resource that is being monitored. There is no limit on the…",
            ),
        ),
        (
            "CreateLookupTable",
            deferred(
                "Model requires lookupTableName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a lookup table by uploading CSV data or from CloudWatch Logs query results. You can use lookup tables to enrich log data in CloudWatch Logs queries with reference data such as user details, application names, or error…",
            ),
        ),
        (
            "CreateScheduledQuery",
            deferred(
                "Model requires name, queryLanguage, queryString, scheduleExpression, executionRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a scheduled query that runs CloudWatch Logs Insights queries at regular intervals. Scheduled queries enable proactive monitoring by automatically executing queries to detect patterns and anomalies in your log data. Query…",
            ),
        ),
        (
            "DeleteAccountPolicy",
            deferred(
                "Model requires policyName, policyType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a CloudWatch Logs account policy. This stops the account-wide policy from applying to log groups or data sources in the account. If you delete a data protection policy or subscription filter policy, any log-group level…",
            ),
        ),
        (
            "DeleteDataProtectionPolicy",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the data protection policy from the specified log group. For more information about data protection policies, see PutDataProtectionPolicy .",
            ),
        ),
        (
            "DeleteDelivery",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a delivery . A delivery is a connection between a logical delivery source and a logical delivery destination . Deleting a delivery only deletes the connection between the delivery source and delivery destination. It does…",
            ),
        ),
        (
            "DeleteDeliveryDestination",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a delivery destination . A delivery is a connection between a logical delivery source and a logical delivery destination . You can't delete a delivery destination if any current deliveries are associated with it. To find…",
            ),
        ),
        (
            "DeleteDeliveryDestinationPolicy",
            deferred(
                "Model requires deliveryDestinationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a delivery destination policy. For more information about these policies, see PutDeliveryDestinationPolicy .",
            ),
        ),
        (
            "DeleteDeliverySource",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a delivery source . A delivery is a connection between a logical delivery source and a logical delivery destination . You can't delete a delivery source if any current deliveries are associated with it. To find whether…",
            ),
        ),
        (
            "DeleteDestination",
            deferred(
                "Model requires destinationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified destination, and eventually disables all the subscription filters that publish to it. This operation does not delete the physical resource encapsulated by the destination.",
            ),
        ),
        (
            "DeleteIndexPolicy",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a log-group level field index policy that was applied to a single log group. The indexing of the log events that happened before you delete the policy will still be used for as many as 30 days to improve CloudWatch Logs…",
            ),
        ),
        (
            "DeleteIntegration",
            deferred(
                "Model requires integrationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the integration between CloudWatch Logs and OpenSearch Service. If your integration has active vended logs dashboards, you must specify true for the force parameter, otherwise the operation will fail. If you delete the…",
            ),
        ),
        (
            "DeleteLogAnomalyDetector",
            deferred(
                "Model requires anomalyDetectorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified CloudWatch Logs anomaly detector.",
            ),
        ),
        (
            "DeleteLogGroup",
            deferred(
                "Model requires logGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified log group and permanently deletes all the archived log events associated with the log group.",
            ),
        ),
        (
            "DeleteLogStream",
            deferred(
                "Model requires logGroupName, logStreamName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified log stream and permanently deletes all the archived log events associated with the log stream.",
            ),
        ),
        (
            "DeleteLookupTable",
            deferred(
                "Model requires lookupTableArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a lookup table permanently. This operation cannot be undone. Queries that reference a deleted table will return an error. Before deleting a lookup table, review any saved queries or dashboards that may reference it.",
            ),
        ),
        (
            "DeleteMetricFilter",
            deferred(
                "Model requires logGroupName, filterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified metric filter.",
            ),
        ),
        (
            "DeleteQueryDefinition",
            deferred(
                "Model requires queryDefinitionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a saved CloudWatch Logs Insights query definition. A query definition contains details about a saved CloudWatch Logs Insights query. Each DeleteQueryDefinition operation can delete one query definition. You must have the…",
            ),
        ),
        (
            "DeleteResourcePolicy",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deletes a resource policy from this account. This revokes the access of the identities in that policy to put log events to this account.",
            ),
        ),
        (
            "DeleteRetentionPolicy",
            deferred(
                "Model requires logGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified retention policy. Log events do not expire if they belong to log groups without a retention policy. If an account-level retention policy applies to this log group, the log group uses the retention period…",
            ),
        ),
        (
            "DeleteScheduledQuery",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a scheduled query and stops all future executions. This operation also removes any configured actions and associated resources.",
            ),
        ),
        (
            "DeleteSubscriptionFilter",
            deferred(
                "Model requires logGroupName, filterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified subscription filter.",
            ),
        ),
        (
            "DeleteSyslogConfiguration",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a syslog configuration for a log group. After deletion, syslog data is no longer ingested through the specified VPC endpoint.",
            ),
        ),
        (
            "DeleteTransformer",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the log transformer for the specified log group. As soon as you do this, the transformation of incoming log events according to that transformer stops. If this account has an account-level transformer that applies to this…",
            ),
        ),
        (
            "DescribeAccountPolicies",
            deferred(
                "Model requires policyType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of all CloudWatch Logs account policies in the account. To use this operation, you must be signed on with the correct permissions depending on the type of policy that you are retrieving information for. To see data…",
            ),
        ),
        (
            "DescribeConfigurationTemplates",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is configurationTemplates; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Use this operation to return the valid and default values that are used when creating delivery sources, delivery destinations, and deliveries. For more information about deliveries, see CreateDelivery .",
            ),
        ),
        (
            "DescribeDeliveries",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "deliveries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeDeliveryDestinations",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "deliveryDestinations",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeDeliverySources",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "deliverySources",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeDestinations",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "destinations",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeExportTasks",
            OperationOverlay {
                queryable: vec![path("taskId")?],
                ..read(
                    "exportTasks",
                    Identity::Key(vec![path("taskId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeFieldIndexes",
            deferred(
                "Model requires logGroupIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of field indexes discovered in log data. By default, the response includes the DEFAULT , CUSTOM , and INACTIVE index categories. To return indexes from other categories, use the indexCategories parameter. For more…",
            ),
        ),
        (
            "DescribeImportTaskBatches",
            deferred(
                "Model requires importId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets detailed information about the individual batches within an import task, including their status and any error messages. For CloudTrail Event Data Store sources, a batch refers to a subset of stored events grouped by their…",
            ),
        ),
        (
            "DescribeImportTasks",
            OperationOverlay {
                queryable: vec![path("importId")?],
                ..read(
                    "imports",
                    Identity::Key(vec![path("importId")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeIndexPolicies",
            deferred(
                "Model requires logGroupIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the field index policies of the specified log group. For more information about field index policies, see PutIndexPolicy . If a specified log group has a log-group level index policy, that policy is returned by this…",
            ),
        ),
        (
            "DescribeLogGroups",
            OperationOverlay {
                queryable: vec![path("logGroupArn")?],
                ..read(
                    "logGroups",
                    Identity::Arn(path("logGroupArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeLogStreams",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("logGroupName")?,
                    Binding::From {
                        operation: name("DescribeLogGroups")?,
                        path: path("logGroupName")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "logStreams",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeLookupTables",
            OperationOverlay {
                queryable: vec![path("lookupTableArn")?],
                ..read(
                    "lookupTables",
                    Identity::Arn(path("lookupTableArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeMetricFilters",
            OperationOverlay {
                queryable: vec![path("logGroupName")?, path("filterName")?],
                ..read(
                    "metricFilters",
                    Identity::Key(vec![path("logGroupName")?, path("filterName")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeQueries",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is queries; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of CloudWatch Logs Insights queries that are scheduled, running, or have been run recently in this account. You can request all queries or limit it to queries of a specific log group or queries with a certain…",
            ),
        ),
        (
            "DescribeQueryDefinitions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is queryDefinitions; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: This operation returns a paginated list of your saved CloudWatch Logs Insights query definitions. You can retrieve query definitions from the current account or from a source account that is linked to the current account. You can…",
            ),
        ),
        (
            "DescribeResourcePolicies",
            OperationOverlay {
                branches: ["ACCOUNT", "RESOURCE"]
                    .into_iter()
                    .map(|scope| {
                        Ok(ExecutionBranch {
                            discriminator: scope.to_owned(),
                            fixed: [(path("policyScope")?, FixedValue::String(scope.to_owned()))]
                                .into_iter()
                                .collect(),
                            scope: Scope::Region,
                            request_region: None,
                            bind: Default::default(),
                            emission: None,
                            row_policy: None,
                        })
                    })
                    .collect::<Result<Vec<_>, NameError>>()?,
                queryable: vec![
                    path("policyName")?,
                    path("policyScope")?,
                    path("resourceArn")?,
                ],
                ..read(
                    "resourcePolicies",
                    Identity::Choice(IdentityChoice::Discriminated(vec![
                        IdentityBranch::new(
                            IdentityPredicate::RequestTextEquals {
                                path: path("policyScope")?,
                                value: "ACCOUNT".to_owned(),
                            },
                            IdentityLeaf::Template("ACCOUNT:{policyName}".to_owned()),
                        )?,
                        IdentityBranch::new(
                            IdentityPredicate::RequestTextEquals {
                                path: path("policyScope")?,
                                value: "RESOURCE".to_owned(),
                            },
                            IdentityLeaf::Key(vec![path("resourceArn")?, path("policyName")?]),
                        )?,
                    ])),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "DescribeSubscriptionFilters",
            deferred(
                "Model requires logGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the subscription filters for the specified log group. You can list all the subscription filters or filter the results by prefix. The results are ASCII-sorted by filter name.",
            ),
        ),
        (
            "DisassociateKmsKey",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Disassociates the specified KMS key from the specified log group or from all CloudWatch Logs Insights query results in the account. When you use DisassociateKmsKey , you specify either the logGroupName parameter or the…",
            ),
        ),
        (
            "DisassociateSourceFromS3TableIntegration",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a data source from an S3 Table Integration, removing query access and deleting all associated data from the integration.",
            ),
        ),
        (
            "FilterLogEvents",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is events, searchedLogStreams; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists log events from the specified log group. You can list all the log events or filter the results using one or more of the following: A filter pattern A time range The log stream name, or a log stream name prefix that matches…",
            ),
        ),
        (
            "GetDataProtectionPolicy",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a log group data protection policy.",
            ),
        ),
        (
            "GetDelivery",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns complete information about one logical delivery . A delivery is a connection between a delivery source and a delivery destination . A delivery source represents an Amazon Web Services resource that sends logs to an logs…",
            ),
        ),
        (
            "GetDeliveryDestination",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves complete information about one delivery destination.",
            ),
        ),
        (
            "GetDeliveryDestinationPolicy",
            deferred(
                "Model requires deliveryDestinationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the delivery destination policy assigned to the delivery destination that you specify. For more information about delivery destinations and their policies, see PutDeliveryDestinationPolicy .",
            ),
        ),
        (
            "GetDeliverySource",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves complete information about one delivery source.",
            ),
        ),
        (
            "GetIntegration",
            deferred(
                "Model requires integrationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about one integration between CloudWatch Logs and OpenSearch Service.",
            ),
        ),
        (
            "GetLogAnomalyDetector",
            deferred(
                "Model requires anomalyDetectorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about the log anomaly detector that you specify. The KMS key ARN detected is valid.",
            ),
        ),
        ("GetLogEvents", denied(DenyReason::Content)),
        (
            "GetLogFields",
            deferred(
                "Model requires dataSourceName, dataSourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Discovers available fields for a specific data source and type. The response includes any field modifications introduced through pipelines, such as new fields or changed field types.",
            ),
        ),
        (
            "GetLogGroupFields",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is logGroupFields; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of the fields that are included in log events in the specified log group. Includes the percentage of log events that contain each field. The search is limited to a time period that you specify. This operation is…",
            ),
        ),
        (
            "GetLogObject",
            deferred(
                "Model requires logObjectPointer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a large logging object (LLO) and streams it back. This API is used to fetch the content of large portions of log events that have been ingested through the PutOpenTelemetryLogs API. When log events contain fields that…",
            ),
        ),
        (
            "GetLogRecord",
            deferred(
                "Model requires logRecordPointer. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves all of the fields and values of a single log event. All fields are retrieved, even if the original query that produced the logRecordPointer retrieved only a subset of fields. Fields are returned as field name/field…",
            ),
        ),
        (
            "GetLookupTable",
            deferred(
                "Model requires lookupTableArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the full content of a lookup table, including the CSV data.",
            ),
        ),
        (
            "GetQueryResults",
            deferred(
                "Model requires queryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the results from the specified query. Only the fields requested in the query are returned, along with a @ptr field, which is the identifier for the log record. You can use the value of @ptr in a GetLogRecord operation to…",
            ),
        ),
        (
            "GetScheduledQuery",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves details about a specific scheduled query, including its configuration, execution status, and metadata.",
            ),
        ),
        (
            "GetScheduledQueryHistory",
            deferred(
                "Model requires identifier, startTime, endTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the execution history of a scheduled query within a specified time range, including query results and destination processing status.",
            ),
        ),
        (
            "GetStorageTierPolicy",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is storageTier, lastUpdatedTime; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the storage tier policy for the account.",
            ),
        ),
        (
            "GetTransformer",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the information about the log transformer associated with this log group. This operation returns data only for transformers created at the log group level. To get information for an account-level transformer, use…",
            ),
        ),
        (
            "ListAggregateLogGroupSummaries",
            deferred(
                "Model requires groupBy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an aggregate summary of all log groups in the Region grouped by specified data source characteristics. Supports optional filtering by log group class, name patterns, and data sources. If you perform this action in a…",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("ListAnomalies", denied(DenyReason::Content)),
        (
            "ListIntegrations",
            OperationOverlay {
                queryable: vec![path("integrationName")?],
                ..read(
                    "integrationSummaries",
                    Identity::Key(vec![path("integrationName")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "ListLogAnomalyDetectors",
            OperationOverlay {
                queryable: vec![path("anomalyDetectorArn")?],
                ..read(
                    "anomalyDetectors",
                    Identity::Arn(path("anomalyDetectorArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListLogGroups",
            OperationOverlay {
                queryable: vec![path("logGroupArn")?],
                ..read(
                    "logGroups",
                    Identity::Arn(path("logGroupArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListLogGroupsForQuery",
            deferred(
                "Model requires queryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the log groups that were analyzed during a single CloudWatch Logs Insights query. This can be useful for queries that use log group name prefixes or the filterIndex command, because the log groups are…",
            ),
        ),
        (
            "ListScheduledQueries",
            OperationOverlay {
                queryable: vec![path("scheduledQueryArn")?],
                ..read(
                    "scheduledQueries",
                    Identity::Arn(path("scheduledQueryArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListSourcesForS3TableIntegration",
            deferred(
                "Model requires integrationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of data source associations for a specified S3 Table Integration, showing which data sources are currently associated for query access.",
            ),
        ),
        (
            "ListSyslogConfigurations",
            deferred(
                "Selected syslog configuration records have logGroupArn, sourceType, and optional vpcEndpointId but no configuration ARN. Without an optional-key identity encoding, omitting VPC endpoint context can collapse distinct configurations and requiring it can reject valid records.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Displays the tags associated with a CloudWatch Logs resource. Currently, log groups and destinations support tagging.",
            ),
        ),
        (
            "ListTagsLogGroup",
            deferred(
                "Model requires logGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The ListTagsLogGroup operation is on the path to deprecation. We recommend that you use ListTagsForResource instead. Lists the tags for the specified log group.",
            ),
        ),
        (
            "PutAccountPolicy",
            deferred(
                "Model requires policyName, policyDocument, policyType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an account-level data protection policy, subscription filter policy, field index policy, transformer policy, metric extraction policy, or retention policy that applies to all log groups, a subset of log groups, or a data…",
            ),
        ),
        (
            "PutBearerTokenAuthentication",
            deferred(
                "Model requires logGroupIdentifier, bearerTokenAuthenticationEnabled. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables or disables bearer token authentication for the specified log group. When enabled on a log group, bearer token authentication is enabled on operations until it is explicitly disabled. For information about the parameters…",
            ),
        ),
        (
            "PutDataProtectionPolicy",
            deferred(
                "Model requires logGroupIdentifier, policyDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a data protection policy for the specified log group. A data protection policy can help safeguard sensitive data that's ingested by the log group by auditing and masking the sensitive log data. Sensitive data is detected…",
            ),
        ),
        (
            "PutDeliveryDestination",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a logical delivery destination . A delivery destination is an Amazon Web Services resource that represents an Amazon Web Services service that logs can be sent to. CloudWatch Logs, Amazon S3, and Firehose are…",
            ),
        ),
        (
            "PutDeliveryDestinationPolicy",
            deferred(
                "Model requires deliveryDestinationName, deliveryDestinationPolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and assigns an IAM policy that grants permissions to CloudWatch Logs to deliver logs cross-account to a specified destination in this account. To configure the delivery of logs from an Amazon Web Services service in…",
            ),
        ),
        (
            "PutDeliverySource",
            deferred(
                "Model requires name, resourceArn, logType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a logical delivery source . A delivery source represents an Amazon Web Services resource that sends logs to an logs delivery destination. The destination can be CloudWatch Logs, Amazon S3, Firehose or X-Ray for…",
            ),
        ),
        (
            "PutDestination",
            deferred(
                "Model requires destinationName, targetArn, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a destination. This operation is used only to create destinations for cross-account subscriptions. A destination encapsulates a physical resource (such as an Amazon Kinesis stream). With a destination, you can…",
            ),
        ),
        (
            "PutDestinationPolicy",
            deferred(
                "Model requires destinationName, accessPolicy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates an access policy associated with an existing destination. An access policy is an IAM policy document that is used to authorize claims to register a subscription filter against a given destination.",
            ),
        ),
        (
            "PutIndexPolicy",
            deferred(
                "Model requires logGroupIdentifier, policyDocument. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a field index policy for the specified log group. Only log groups in the Standard log class support field index policies. For more information about log classes, see Log classes . You can use field index…",
            ),
        ),
        (
            "PutIntegration",
            deferred(
                "Model requires integrationName, resourceConfig, integrationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an integration between CloudWatch Logs and another service in this account. Currently, only integrations with OpenSearch Service are supported, and currently you can have only one integration in your account. Integrating…",
            ),
        ),
        (
            "PutLogEvents",
            deferred(
                "Model requires logGroupName, logStreamName, logEvents. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Uploads a batch of log events to the specified log stream. The sequence token is now ignored in PutLogEvents actions. PutLogEvents actions are always accepted and never return InvalidSequenceTokenException or…",
            ),
        ),
        (
            "PutLogGroupDeletionProtection",
            deferred(
                "Model requires logGroupIdentifier, deletionProtectionEnabled. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables or disables deletion protection for the specified log group. When enabled on a log group, deletion protection blocks all deletion operations until it is explicitly disabled. For information about the parameters that are…",
            ),
        ),
        (
            "PutMetricFilter",
            deferred(
                "Model requires logGroupName, filterName, filterPattern, metricTransformations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a metric filter and associates it with the specified log group. With metric filters, you can configure rules to extract metric data from log events ingested through PutLogEvents . The maximum number of metric…",
            ),
        ),
        (
            "PutQueryDefinition",
            deferred(
                "Model requires name, queryString. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a query definition for CloudWatch Logs Insights. For more information, see Analyzing Log Data with CloudWatch Logs Insights . To update a query definition, specify its queryDefinitionId in your request. The…",
            ),
        ),
        (
            "PutResourcePolicy",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is resourcePolicy, revisionId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates or updates a resource policy allowing other Amazon Web Services services to put log events to this account, such as Amazon Route 53. This API has the following restrictions: Supported actions - Policy only supports…",
            ),
        ),
        (
            "PutRetentionPolicy",
            deferred(
                "Model requires logGroupName, retentionInDays. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the retention of the specified log group. With a retention policy, you can configure the number of days for which to retain log events in the specified log group. Account-level retention policies created with…",
            ),
        ),
        (
            "PutStorageTierPolicy",
            deferred(
                "Model requires storageTier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the storage tier policy for the account. When you set the storage tier to INTELLIGENT_TIERING , the service automatically moves log data to the most cost-effective storage tier based on access frequency.",
            ),
        ),
        (
            "PutSubscriptionFilter",
            deferred(
                "Model requires logGroupName, filterName, filterPattern, destinationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a subscription filter and associates it with the specified log group. With subscription filters, you can subscribe to a real-time stream of log events ingested through PutLogEvents and have them delivered to a…",
            ),
        ),
        (
            "PutSyslogConfiguration",
            deferred(
                "Model requires logGroupIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a syslog configuration for a log group. This enables ingestion of syslog data through the specified VPC endpoint into the log group.",
            ),
        ),
    ])
}

pub(super) fn part_4() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutTransformer",
            deferred(
                "Model requires logGroupIdentifier, transformerConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a log transformer for a single log group. You use log transformers to transform log events into a different format, making them easier for you to process and analyze. You can also transform logs from different…",
            ),
        ),
        (
            "StartLiveTail",
            deferred(
                "Model requires logGroupIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a Live Tail streaming session for one or more log groups. A Live Tail session returns a stream of log events that have been recently ingested in the log groups. For more information, see Use Live Tail to view logs in near…",
            ),
        ),
        (
            "StartQuery",
            deferred(
                "Model requires startTime, endTime, queryString. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a query of one or more log groups or data sources using CloudWatch Logs Insights. You specify the log groups or data sources and time range to query and the query string to use. You can query up to 10 data sources in a…",
            ),
        ),
        (
            "StopQuery",
            deferred(
                "Model requires queryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a CloudWatch Logs Insights query that is in progress. If the query has already ended, the operation returns an error indicating that the specified query is not running. This operation can be used to cancel both interactive…",
            ),
        ),
        (
            "TagLogGroup",
            deferred(
                "Model requires logGroupName, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The TagLogGroup operation is on the path to deprecation. We recommend that you use TagResource instead. Adds or updates the specified tags for the specified log group. To list the tags for a log group, use ListTagsForResource .…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified CloudWatch Logs resource. Currently, the only CloudWatch Logs resources that can be tagged are log groups and destinations. Tags can help you organize and categorize…",
            ),
        ),
        (
            "TestMetricFilter",
            deferred(
                "Model requires filterPattern, logEventMessages. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tests the filter pattern of a metric filter against a sample of log event messages. You can use this operation to validate the correctness of a metric filter pattern.",
            ),
        ),
        (
            "TestTransformer",
            deferred(
                "Model requires transformerConfig, logEventMessages. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use this operation to test a log transformer. You enter the transformer configuration and a set of log events to test with. The operation responds with an array that includes the original log events and the transformed versions.",
            ),
        ),
        (
            "UntagLogGroup",
            deferred(
                "Model requires logGroupName, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The UntagLogGroup operation is on the path to deprecation. We recommend that you use UntagResource instead. Removes the specified tags from the specified log group. To list the tags for a log group, use ListTagsForResource . To…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified resource.",
            ),
        ),
        (
            "UpdateAnomaly",
            deferred(
                "Model requires anomalyDetectorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use this operation to suppress anomaly detection for a specified anomaly or pattern. If you suppress an anomaly, CloudWatch Logs won't report new occurrences of that anomaly and won't update that anomaly with new data. If you…",
            ),
        ),
        (
            "UpdateDeliveryConfiguration",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use this operation to update the configuration of a delivery to change either the S3 path pattern or the format of the delivered logs. You can't use this operation to change the source or destination of the delivery.",
            ),
        ),
        (
            "UpdateLogAnomalyDetector",
            deferred(
                "Model requires anomalyDetectorArn, enabled. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing log anomaly detector.",
            ),
        ),
        (
            "UpdateLookupTable",
            deferred(
                "Model requires lookupTableArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing lookup table by replacing all of its content with new CSV data or CloudWatch Logs query results. After the update completes, queries that use this table use the new data. This is a full replacement operation.…",
            ),
        ),
        (
            "UpdateScheduledQuery",
            deferred(
                "Model requires identifier, queryLanguage, queryString, scheduleExpression, executionRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing scheduled query with new configuration. This operation uses PUT semantics, allowing modification of query parameters, schedule, and destinations.",
            ),
        ),
    ])
}
