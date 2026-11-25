use super::support::{bound, correlated, inherited, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "deadline",
        vec![
            queried(
                inventory(
                    "ListFarms",
                    "farms",
                    key(&["farmId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["farmId", "kmsKeyArn", "displayName"],
            )?,
            queried(
                inventory(
                    "ListLicenseEndpoints",
                    "licenseEndpoints",
                    key(&["licenseEndpointId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["licenseEndpointId", "status", "statusMessage", "vpcId"],
            )?,
            queried(
                inventory(
                    "ListMonitors",
                    "monitors",
                    key(&["monitorId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["monitorId", "roleArn", "displayName"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListFleets",
                        "fleets",
                        correlated(&["farmId"], &["fleetId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &[
                    "farmId",
                    "fleetId",
                    "displayName",
                    "status",
                    "statusMessage",
                    "autoScalingStatus",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListQueues",
                        "queues",
                        correlated(&["farmId"], &["queueId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["farmId", "queueId", "displayName", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListLimits",
                        "limits",
                        correlated(&["farmId"], &["limitId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["farmId", "limitId", "displayName", "amountRequirementName"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListBudgets",
                        "budgets",
                        correlated(&["farmId"], &["budgetId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["budgetId", "displayName", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListStorageProfiles",
                        "storageProfiles",
                        correlated(&["farmId"], &["storageProfileId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["storageProfileId", "displayName", "osFamily"],
            )?,
            queried(
                inherited(
                    bound(
                        inventory(
                            "ListQueueEnvironments",
                            "environments",
                            correlated(&["farmId", "queueId"], &["queueEnvironmentId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[("queueId", "ListQueues", "queueId")],
                    )?,
                    "farmId",
                    "ListQueues",
                    "farmId",
                )?,
                &["queueEnvironmentId", "name", "priority"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListQueueFleetAssociations",
                        "queueFleetAssociations",
                        correlated(&["farmId"], &["queueId", "fleetId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["queueId", "fleetId", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListQueueLimitAssociations",
                        "queueLimitAssociations",
                        correlated(&["farmId"], &["queueId", "limitId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("farmId", "ListFarms", "farmId")],
                )?,
                &["queueId", "limitId", "status"],
            )?,
            queried(
                inherited(
                    bound(
                        inventory(
                            "ListVolumes",
                            "volumes",
                            correlated(&["farmId", "fleetId"], &["volumeId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[("fleetId", "ListFleets", "fleetId")],
                    )?,
                    "farmId",
                    "ListFleets",
                    "farmId",
                )?,
                &[
                    "farmId",
                    "fleetId",
                    "volumeId",
                    "state",
                    "availabilityZoneId",
                    "attachedWorkerId",
                ],
            )?,
        ],
        &[
            super::s_deadline_decisions_0::ENTRIES,
            super::s_deadline_decisions_1::ENTRIES,
            super::s_deadline_decisions_2::ENTRIES,
            super::s_deadline_decisions_3::ENTRIES,
        ],
        &[],
    )
}
