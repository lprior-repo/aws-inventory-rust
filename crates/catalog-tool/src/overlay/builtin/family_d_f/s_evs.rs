use super::support::{arn, bound, correlated, inventory, queried, reviewed, token};
use crate::overlay::{Identity, IdentityField, MemberPath, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "evs",
        vec![
            queried(
                inventory(
                    "ListEnvironments",
                    "environmentSummaries",
                    arn("environmentArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &[
                    "environmentArn",
                    "environmentId",
                    "environmentName",
                    "environmentStatus",
                    "environmentState",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListEnvironmentHosts",
                        "environmentHosts",
                        correlated(&["environmentId"], &["hostName"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("environmentId", "ListEnvironments", "environmentId")],
                )?,
                &["hostName", "ec2InstanceId", "dedicatedHostId", "hostState"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListEnvironmentConnectors",
                        "connectors",
                        correlated(&["environmentId"], &["connectorId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("environmentId", "ListEnvironments", "environmentId")],
                )?,
                &[
                    "environmentId",
                    "connectorId",
                    "applianceFqdn",
                    "secretArn",
                    "state",
                    "status",
                ],
            )?,
            queried(
                bound(
                    inventory(
                        "ListEnvironmentVlans",
                        "environmentVlans",
                        Identity::CorrelatedKey(vec![
                            IdentityField::Request(MemberPath::new("environmentId")?),
                            IdentityField::SelectedInteger {
                                path: MemberPath::new("vlanId")?,
                                minimum: i64::from(i32::MIN),
                                maximum: i64::from(i32::MAX),
                            },
                        ]),
                        token("nextToken", "nextToken")?,
                    )?,
                    &[("environmentId", "ListEnvironments", "environmentId")],
                )?,
                &["vlanId", "cidr", "subnetId", "vlanState", "networkAclId"],
            )?,
        ],
        &[super::s_evs_decisions_0::ENTRIES],
        &[],
    )
}
