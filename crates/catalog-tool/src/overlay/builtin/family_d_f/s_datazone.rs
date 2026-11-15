// Pinned model: .local/aws-models/models/datazone/service/2018-05-10/datazone-2018-05-10.json
use super::support::{arn, bound, correlated, inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "datazone",
        vec![
        // Lists Amazon DataZone domains.
        queried(inventory("ListDomains", "items", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "id", "name", "managedAccountId", "status"])?,
        // Lists Amazon DataZone projects.
        queried(bound(inventory("ListProjects", "items", key(&["domainId", "id"])?, token("nextToken", "nextToken")?)?, &[("domainIdentifier", "ListDomains", "id")])?, &["domainId", "id", "name", "projectStatus", "domainUnitId"])?,
        // Lists Amazon DataZone environments.
        queried(bound(inventory("ListEnvironments", "items", key(&["domainId", "id"])?, token("nextToken", "nextToken")?)?, &[("domainIdentifier", "ListProjects", "domainId"), ("projectIdentifier", "ListProjects", "id")])?, &["domainId", "id", "projectId", "name", "environmentProfileId", "awsAccountId", "status", "environmentConfigurationId", "environmentConfigurationName"])?,
        // Lists Amazon DataZone environment profiles.
        queried(bound(inventory("ListEnvironmentProfiles", "items", key(&["domainId", "id"])?, token("nextToken", "nextToken")?)?, &[("domainIdentifier", "ListDomains", "id")])?, &["domainId", "id", "awsAccountId", "name", "environmentBlueprintId", "projectId"])?,
        // Project-profile summaries exclude environment deployment configuration and scripts.
        queried(bound(inventory("ListProjectProfiles", "items", correlated(&["domainIdentifier"], &["id"])?, token("nextToken", "nextToken")?)?,
            &[("domainIdentifier", "ListDomains", "id")])?, &["domainId", "id", "name", "status", "domainUnitId"])?,
        // Lists data sources in Amazon DataZone.
        queried(bound(inventory("ListDataSources", "items", key(&["domainId", "dataSourceId"])?, token("nextToken", "nextToken")?)?, &[("domainIdentifier", "ListProjects", "domainId"), ("projectIdentifier", "ListProjects", "id")])?, &["domainId", "dataSourceId", "environmentId", "connectionId", "name", "type", "status", "lastRunStatus"])?,
        ],
        &[super::s_datazone_decisions_0::ENTRIES, super::s_datazone_decisions_1::ENTRIES, super::s_datazone_decisions_2::ENTRIES, super::s_datazone_decisions_3::ENTRIES, super::s_datazone_decisions_4::ENTRIES, super::s_datazone_decisions_5::ENTRIES],
        &[

        ],
    )
}
