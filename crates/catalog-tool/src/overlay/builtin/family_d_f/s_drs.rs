// Pinned model: .local/aws-models/models/drs/service/2020-02-26/drs-2020-02-26.json
use super::support::{arn, bound, inventory, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "drs",
        vec![
        // Returns a list of Jobs.
        queried(tagged(inventory("DescribeJobs", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "jobID", "type", "status"])?,
        // Lists all Launch Configuration Templates, filtered by Launch Configuration Template IDs
        queried(tagged(inventory("DescribeLaunchConfigurationTemplates", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "launchConfigurationTemplateID", "targetInstanceTypeRightSizingMethod"])?,
        // Lists all Recovery Instances or multiple Recovery Instances by ID.
        queried(tagged(inventory("DescribeRecoveryInstances", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "ec2InstanceID", "ec2InstanceState", "jobID", "recoveryInstanceID", "sourceServerID"])?,
        // Lists all ReplicationConfigurationTemplates, filtered by Source Server IDs.
        queried(tagged(inventory("DescribeReplicationConfigurationTemplates", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "ebsEncryptionKeyArn", "replicationConfigurationTemplateID", "stagingAreaSubnetId", "replicationServerInstanceType", "defaultLargeStagingDiskType"])?,
        // Lists all Source Networks or multiple Source Networks filtered by ID.
        queried(tagged(inventory("DescribeSourceNetworks", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "sourceNetworkID", "sourceVpcID", "replicationStatus", "replicationStatusDetails", "cfnStackName", "sourceAccountID", "launchedVpcID"])?,
        // Lists all Source Servers or multiple Source Servers filtered by ID.
        queried(tagged(inventory("DescribeSourceServers", "items", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "sourceServerID", "recoveryInstanceId", "sourceNetworkID"])?,
        // Lists all Recovery Plans in the account.
        queried(inventory("ListRecoveryPlans", "recoveryPlans", arn("recoveryPlanArn")?, token("nextToken", "nextToken")?)?, &["recoveryPlanArn", "name", "status"])?,
        // Lists executions of Recovery Plans, optionally filtered by plan or status.
        queried(inventory("ListRecoveryPlanExecutions", "recoveryPlanExecutions", arn("recoveryPlanExecutionArn")?, token("nextToken", "nextToken")?)?, &["recoveryPlanExecutionArn", "status"])?,
        // Step configurations contain server ARN references or wait durations, not executable scripts.
        queried(bound(inventory("ListRecoveryPlanSteps", "recoveryPlanSteps", arn("recoveryPlanStepArn")?, token("nextToken", "nextToken")?)?,
            &[("recoveryPlanArn", "ListRecoveryPlans", "recoveryPlanArn")])?, &["recoveryPlanStepArn", "stepName"])?,
        queried(bound(inventory("ListRecoveryPlanExecutionSteps", "recoveryPlanExecutionSteps", arn("recoveryPlanExecutionStepArn")?, token("nextToken", "nextToken")?)?,
            &[("recoveryPlanExecutionArn", "ListRecoveryPlanExecutions", "recoveryPlanExecutionArn")])?, &["recoveryPlanExecutionStepArn", "stepName", "status"])?,
        ],
        &[super::s_drs_decisions_0::ENTRIES, super::s_drs_decisions_1::ENTRIES],
        &[

        ],
    )
}
