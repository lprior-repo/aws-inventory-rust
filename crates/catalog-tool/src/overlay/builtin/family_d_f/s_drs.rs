use super::support::{arn, bound, inventory, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "drs",
        vec![
            queried(
                tagged(
                    inventory(
                        "DescribeJobs",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &["arn", "jobID", "type", "status"],
            )?,
            queried(
                tagged(
                    inventory(
                        "DescribeLaunchConfigurationTemplates",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "launchConfigurationTemplateID",
                    "targetInstanceTypeRightSizingMethod",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "DescribeRecoveryInstances",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "ec2InstanceID",
                    "ec2InstanceState",
                    "jobID",
                    "recoveryInstanceID",
                    "sourceServerID",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "DescribeReplicationConfigurationTemplates",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "ebsEncryptionKeyArn",
                    "replicationConfigurationTemplateID",
                    "stagingAreaSubnetId",
                    "replicationServerInstanceType",
                    "defaultLargeStagingDiskType",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "DescribeSourceNetworks",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "sourceNetworkID",
                    "sourceVpcID",
                    "replicationStatus",
                    "replicationStatusDetails",
                    "cfnStackName",
                    "sourceAccountID",
                    "launchedVpcID",
                ],
            )?,
            queried(
                tagged(
                    inventory(
                        "DescribeSourceServers",
                        "items",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "arn",
                    "sourceServerID",
                    "recoveryInstanceId",
                    "sourceNetworkID",
                ],
            )?,
            queried(
                inventory(
                    "ListRecoveryPlans",
                    "recoveryPlans",
                    arn("recoveryPlanArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["recoveryPlanArn", "name", "status"],
            )?,
            queried(
                inventory(
                    "ListRecoveryPlanExecutions",
                    "recoveryPlanExecutions",
                    arn("recoveryPlanExecutionArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["recoveryPlanExecutionArn", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListRecoveryPlanSteps",
                        "recoveryPlanSteps",
                        arn("recoveryPlanStepArn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("recoveryPlanArn", "ListRecoveryPlans", "recoveryPlanArn")],
                )?,
                &["recoveryPlanStepArn", "stepName"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListRecoveryPlanExecutionSteps",
                        "recoveryPlanExecutionSteps",
                        arn("recoveryPlanExecutionStepArn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[(
                        "recoveryPlanExecutionArn",
                        "ListRecoveryPlanExecutions",
                        "recoveryPlanExecutionArn",
                    )],
                )?,
                &["recoveryPlanExecutionStepArn", "stepName", "status"],
            )?,
        ],
        &[
            super::s_drs_decisions_0::ENTRIES,
            super::s_drs_decisions_1::ENTRIES,
        ],
        &[],
    )
}
