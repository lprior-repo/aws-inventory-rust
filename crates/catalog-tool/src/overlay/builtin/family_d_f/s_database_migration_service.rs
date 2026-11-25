use super::support::{arn, inventory, key, queried, reviewed, token, without_settings};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "database-migration-service",
        vec![
            queried(
                inventory(
                    "DescribeDataProviders",
                    "DataProviders",
                    arn("DataProviderArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["DataProviderArn", "DataProviderName", "Engine"],
            )?,
            queried(
                inventory(
                    "DescribeEventSubscriptions",
                    "EventSubscriptionsList",
                    key(&["CustSubscriptionId"])?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "CustSubscriptionId",
                    "CustomerAwsId",
                    "Status",
                    "SourceType",
                ],
            )?,
            queried(
                inventory(
                    "DescribeInstanceProfiles",
                    "InstanceProfiles",
                    arn("InstanceProfileArn")?,
                    token("Marker", "Marker")?,
                )?,
                &["InstanceProfileArn", "NetworkType", "InstanceProfileName"],
            )?,
            queried(
                inventory(
                    "DescribeMigrationProjects",
                    "MigrationProjects",
                    arn("MigrationProjectArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "MigrationProjectArn",
                    "InstanceProfileArn",
                    "MigrationProjectName",
                    "InstanceProfileName",
                ],
            )?,
            queried(
                inventory(
                    "DescribeReplicationSubnetGroups",
                    "ReplicationSubnetGroups",
                    key(&["ReplicationSubnetGroupIdentifier"])?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ReplicationSubnetGroupIdentifier",
                    "VpcId",
                    "SubnetGroupStatus",
                ],
            )?,
            queried(
                inventory(
                    "DescribeReplications",
                    "Replications",
                    arn("ReplicationConfigArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ReplicationConfigArn",
                    "ReplicationConfigIdentifier",
                    "ReplicationType",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "DescribeConnections",
                    "Connections",
                    key(&["ReplicationInstanceArn", "EndpointArn"])?,
                    token("Marker", "Marker")?,
                )?,
                &["ReplicationInstanceArn", "EndpointArn", "Status"],
            )?,
            queried(
                inventory(
                    "DescribeReplicationTaskAssessmentRuns",
                    "ReplicationTaskAssessmentRuns",
                    arn("ReplicationTaskAssessmentRunArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ReplicationTaskAssessmentRunArn",
                    "ReplicationTaskArn",
                    "Status",
                ],
            )?,
            queried(
                inventory(
                    "DescribeReplicationTaskIndividualAssessments",
                    "ReplicationTaskIndividualAssessments",
                    arn("ReplicationTaskIndividualAssessmentArn")?,
                    token("Marker", "Marker")?,
                )?,
                &[
                    "ReplicationTaskIndividualAssessmentArn",
                    "ReplicationTaskAssessmentRunArn",
                    "Status",
                ],
            )?,
            queried(
                without_settings(inventory(
                    "DescribeDataMigrations",
                    "DataMigrations",
                    arn("DataMigrationArn")?,
                    token("Marker", "Marker")?,
                )?)?,
                &[
                    "DataMigrationArn",
                    "DataMigrationName",
                    "DataMigrationStatus",
                    "MigrationProjectArn",
                ],
            )?,
            queried(
                inventory(
                    "DescribePendingMaintenanceActions",
                    "PendingMaintenanceActions",
                    key(&["ResourceIdentifier"])?,
                    token("Marker", "Marker")?,
                )?,
                &["ResourceIdentifier"],
            )?,
            queried(
                inventory(
                    "DescribeFleetAdvisorCollectors",
                    "Collectors",
                    key(&["CollectorReferencedId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["CollectorReferencedId", "CollectorName"],
            )?,
            queried(
                inventory(
                    "DescribeFleetAdvisorDatabases",
                    "Databases",
                    key(&["DatabaseId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["DatabaseId", "DatabaseName"],
            )?,
        ],
        &[
            super::s_database_migration_service_decisions_0::ENTRIES,
            super::s_database_migration_service_decisions_1::ENTRIES,
            super::s_database_migration_service_decisions_2::ENTRIES,
            super::s_database_migration_service_decisions_3::ENTRIES,
        ],
        &[
            ("DescribeReplicationInstances", DenyReason::Content),
            ("DescribeCertificates", DenyReason::Secret),
            ("DescribeEndpoints", DenyReason::Secret),
            ("DescribeReplicationTasks", DenyReason::Content),
            ("DescribeReplicationConfigs", DenyReason::Content),
        ],
    )
}
