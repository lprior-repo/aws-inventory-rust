use super::support::{arn, bound, correlated, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "datazone",
        vec![
            queried(
                inventory(
                    "ListDomains",
                    "items",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "id", "name", "managedAccountId", "status"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListProjects",
                        "items",
                        key(&["domainId", "id"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("domainIdentifier", "ListDomains", "id")],
                )?,
                &["domainId", "id", "name", "projectStatus", "domainUnitId"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListEnvironments",
                        "items",
                        key(&["domainId", "id"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[
                        ("domainIdentifier", "ListProjects", "domainId"),
                        ("projectIdentifier", "ListProjects", "id"),
                    ],
                )?,
                &[
                    "domainId",
                    "id",
                    "projectId",
                    "name",
                    "environmentProfileId",
                    "awsAccountId",
                    "status",
                    "environmentConfigurationId",
                    "environmentConfigurationName",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListEnvironmentProfiles",
                        "items",
                        key(&["domainId", "id"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("domainIdentifier", "ListDomains", "id")],
                )?,
                &[
                    "domainId",
                    "id",
                    "awsAccountId",
                    "name",
                    "environmentBlueprintId",
                    "projectId",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListProjectProfiles",
                        "items",
                        correlated(&["domainIdentifier"], &["id"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("domainIdentifier", "ListDomains", "id")],
                )?,
                &["domainId", "id", "name", "status", "domainUnitId"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListDataSources",
                        "items",
                        key(&["domainId", "dataSourceId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[
                        ("domainIdentifier", "ListProjects", "domainId"),
                        ("projectIdentifier", "ListProjects", "id"),
                    ],
                )?,
                &[
                    "domainId",
                    "dataSourceId",
                    "environmentId",
                    "connectionId",
                    "name",
                    "type",
                    "status",
                    "lastRunStatus",
                ],
            )?,
        ],
        &[
            super::s_datazone_decisions_0::ENTRIES,
            super::s_datazone_decisions_1::ENTRIES,
            super::s_datazone_decisions_2::ENTRIES,
            super::s_datazone_decisions_3::ENTRIES,
            super::s_datazone_decisions_4::ENTRIES,
            super::s_datazone_decisions_5::ENTRIES,
        ],
        &[],
    )
}
