// Pinned model: .local/aws-models/models/database-migration-service/service/2016-01-01/database-migration-service-2016-01-01.json
use super::support::{arn, inventory, key, queried, reviewed, token, without_settings};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "database-migration-service",
        vec![
        // Provider settings contain Secrets Manager references, never inline database passwords.
        queried(inventory("DescribeDataProviders", "DataProviders", arn("DataProviderArn")?, token("Marker", "Marker")?)?, &["DataProviderArn", "DataProviderName", "Engine"])?,
        // Lists all the event subscriptions for a customer account.
        queried(inventory("DescribeEventSubscriptions", "EventSubscriptionsList", key(&["CustSubscriptionId"])?, token("Marker", "Marker")?)?, &["CustSubscriptionId", "CustomerAwsId", "Status", "SourceType"])?,
        // Returns a paginated list of instance profiles for your account in the current region.
        queried(inventory("DescribeInstanceProfiles", "InstanceProfiles", arn("InstanceProfileArn")?, token("Marker", "Marker")?)?, &["InstanceProfileArn", "NetworkType", "InstanceProfileName"])?,
        // Returns a paginated list of migration projects for your account in the current region.
        queried(inventory("DescribeMigrationProjects", "MigrationProjects", arn("MigrationProjectArn")?, token("Marker", "Marker")?)?, &["MigrationProjectArn", "InstanceProfileArn", "MigrationProjectName", "InstanceProfileName"])?,
        // Returns information about the replication subnet groups.
        queried(inventory("DescribeReplicationSubnetGroups", "ReplicationSubnetGroups", key(&["ReplicationSubnetGroupIdentifier"])?, token("Marker", "Marker")?)?, &["ReplicationSubnetGroupIdentifier", "VpcId", "SubnetGroupStatus"])?,
        // Provisioned serverless replication status omits customer TableMappings/ReplicationSettings bodies.
        queried(inventory("DescribeReplications", "Replications", arn("ReplicationConfigArn")?, token("Marker", "Marker")?)?, &["ReplicationConfigArn", "ReplicationConfigIdentifier", "ReplicationType", "Status"])?,
        // Existing connection-test status, without initiating tests or fetching endpoint passwords.
        queried(inventory("DescribeConnections", "Connections", key(&["ReplicationInstanceArn", "EndpointArn"])?, token("Marker", "Marker")?)?, &["ReplicationInstanceArn", "EndpointArn", "Status"])?,
        // Assessment-run control metadata and result locations, not assessment result contents.
        queried(inventory("DescribeReplicationTaskAssessmentRuns", "ReplicationTaskAssessmentRuns", arn("ReplicationTaskAssessmentRunArn")?, token("Marker", "Marker")?)?, &["ReplicationTaskAssessmentRunArn", "ReplicationTaskArn", "Status"])?,
        queried(inventory("DescribeReplicationTaskIndividualAssessments", "ReplicationTaskIndividualAssessments", arn("ReplicationTaskIndividualAssessmentArn")?, token("Marker", "Marker")?)?, &["ReplicationTaskIndividualAssessmentArn", "ReplicationTaskAssessmentRunArn", "Status"])?,
        // WithoutSettings omits the SecretString SelectionRules held inside DataMigrationSettings.
        queried(without_settings(inventory("DescribeDataMigrations", "DataMigrations", arn("DataMigrationArn")?, token("Marker", "Marker")?)?)?, &["DataMigrationArn", "DataMigrationName", "DataMigrationStatus", "MigrationProjectArn"])?,
        queried(inventory("DescribePendingMaintenanceActions", "PendingMaintenanceActions", key(&["ResourceIdentifier"])?, token("Marker", "Marker")?)?, &["ResourceIdentifier"])?,
        queried(inventory("DescribeFleetAdvisorCollectors", "Collectors", key(&["CollectorReferencedId"])?, token("NextToken", "NextToken")?)?, &["CollectorReferencedId", "CollectorName"])?,
        queried(inventory("DescribeFleetAdvisorDatabases", "Databases", key(&["DatabaseId"])?, token("NextToken", "NextToken")?)?, &["DatabaseId", "DatabaseName"])?,
        ],
        &[super::s_database_migration_service_decisions_0::ENTRIES, super::s_database_migration_service_decisions_1::ENTRIES, super::s_database_migration_service_decisions_2::ENTRIES, super::s_database_migration_service_decisions_3::ENTRIES],
        &[
            // ReplicationInstances[].KerberosAuthenticationSettings.Krb5FileContents returns a raw configuration file; there is no omission input.
            ("DescribeReplicationInstances", DenyReason::Content),
            // CertificateWallet can return an imported Oracle Wallet certificate; no wallet/credential content retrieval is authorized.
            ("DescribeCertificates", DenyReason::Secret),
            // Returns information about the endpoints for your account in the current region.
            ("DescribeEndpoints", DenyReason::Secret),
            // WithoutSettings does not document omission of the customer TableMappings transformation program.
            ("DescribeReplicationTasks", DenyReason::Content),
            // Includes TableMappings and ReplicationSettings bodies, with no omission control.
            ("DescribeReplicationConfigs", DenyReason::Content),
        ],
    )
}
