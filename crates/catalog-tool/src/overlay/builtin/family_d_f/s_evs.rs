// Pinned model: .local/aws-models/models/evs/service/2023-07-27/evs-2023-07-27.json
use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::overlay::{Identity, IdentityField, MemberPath, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "evs",
        vec![
        // Lists the Amazon EVS environments in your Amazon Web Services account in the specified Amazon Web Services Region.
        queried(inventory("ListEnvironments", "environmentSummaries", arn("environmentArn")?, token("nextToken", "nextToken")?)?, &["environmentArn", "environmentId", "environmentName", "environmentStatus", "environmentState"])?,
        // Key names and secret ARN references are metadata; no key material or secret retrieval.
        queried(bound(inventory("ListEnvironmentHosts", "environmentHosts", correlated(&["environmentId"], &["hostName"])?, token("nextToken", "nextToken")?)?,
            &[("environmentId", "ListEnvironments", "environmentId")])?, &["hostName", "ec2InstanceId", "dedicatedHostId", "hostState"])?,
        queried(bound(inventory("ListEnvironmentConnectors", "connectors", correlated(&["environmentId"], &["connectorId"])?, token("nextToken", "nextToken")?)?,
            &[("environmentId", "ListEnvironments", "environmentId")])?, &["environmentId", "connectorId", "applianceFqdn", "secretArn", "state", "status"])?,
        // VlanId is an unconstrained Smithy integer: retain its signed 32-bit modeled domain.
        queried(bound(inventory("ListEnvironmentVlans", "environmentVlans", Identity::CorrelatedKey(vec![
            IdentityField::Request(MemberPath::new("environmentId")?),
            IdentityField::SelectedInteger { path: MemberPath::new("vlanId")?, minimum: i64::from(i32::MIN), maximum: i64::from(i32::MAX) },
        ]), token("nextToken", "nextToken")?)?,
            &[("environmentId", "ListEnvironments", "environmentId")])?, &["vlanId", "cidr", "subnetId", "vlanState", "networkAclId"])?,
        ],
        &[super::s_evs_decisions_0::ENTRIES],
        &[

        ],
    )
}
