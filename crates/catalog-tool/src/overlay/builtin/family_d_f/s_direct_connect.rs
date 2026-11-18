// Pinned model: .local/aws-models/models/direct-connect/service/2012-10-25/direct-connect-2012-10-25.json
use super::support::{arn, bound, inventory, key, queried, reviewed, scoped, tagged, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "direct-connect",
        vec![
            // Displays the specified connection or all connections in this Region.
            queried(
                tagged(
                    inventory(
                        "DescribeConnections",
                        "connections",
                        key(&["connectionId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "connectionId",
                    "connectionName",
                    "connectionState",
                    "partnerName",
                    "lagId",
                    "awsLogicalDeviceId",
                    "providerName",
                    "portEncryptionStatus",
                ],
            )?,
            // Lists all your Direct Connect gateways or only the specified Direct Connect gateway.
            queried(
                scoped(
                    tagged(
                        inventory(
                            "DescribeDirectConnectGateways",
                            "directConnectGateways",
                            key(&["directConnectGatewayId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        "tags",
                    )?,
                    Scope::Account,
                ),
                &[
                    "directConnectGatewayId",
                    "directConnectGatewayName",
                    "directConnectGatewayState",
                    "stateChangeError",
                ],
            )?,
            // Lists the interconnects owned by the Amazon Web Services account or only the specified interconnect.
            queried(
                tagged(
                    inventory(
                        "DescribeInterconnects",
                        "interconnects",
                        key(&["interconnectId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "interconnectId",
                    "interconnectName",
                    "interconnectState",
                    "lagId",
                    "awsLogicalDeviceId",
                    "providerName",
                    "portEncryptionStatus",
                ],
            )?,
            // Describes all your link aggregation groups (LAG) or the specified LAG.
            queried(
                tagged(
                    inventory(
                        "DescribeLags",
                        "lags",
                        key(&["lagId"])?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &[
                    "lagId",
                    "lagName",
                    "lagState",
                    "awsLogicalDeviceId",
                    "providerName",
                ],
            )?,
            // Lists the resiliency groups owned by your Amazon Web Services account in the current Amazon Web Services Region.
            queried(
                inventory(
                    "ListResiliencyGroups",
                    "items",
                    arn("resiliencyGroupArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &[
                    "resiliencyGroupArn",
                    "resiliencyGroupId",
                    "resiliencyGroupName",
                    "resiliencyGroupType",
                    "state",
                ],
            )?,
            // Bind the documented one-of gateway selector even though the Smithy members are individually optional.
            queried(
                scoped(
                    bound(
                        inventory(
                            "DescribeDirectConnectGatewayAssociationProposals",
                            "directConnectGatewayAssociationProposals",
                            key(&["directConnectGatewayId", "proposalId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[(
                            "directConnectGatewayId",
                            "DescribeDirectConnectGateways",
                            "directConnectGatewayId",
                        )],
                    )?,
                    Scope::Account,
                ),
                &["directConnectGatewayId", "proposalId"],
            )?,
            queried(
                scoped(
                    bound(
                        inventory(
                            "DescribeDirectConnectGatewayAssociations",
                            "directConnectGatewayAssociations",
                            key(&["directConnectGatewayId", "associationId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[(
                            "directConnectGatewayId",
                            "DescribeDirectConnectGateways",
                            "directConnectGatewayId",
                        )],
                    )?,
                    Scope::Account,
                ),
                &["directConnectGatewayId", "associationId"],
            )?,
            queried(
                scoped(
                    bound(
                        inventory(
                            "DescribeDirectConnectGatewayAttachments",
                            "directConnectGatewayAttachments",
                            key(&["directConnectGatewayId", "virtualInterfaceId"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[(
                            "directConnectGatewayId",
                            "DescribeDirectConnectGateways",
                            "directConnectGatewayId",
                        )],
                    )?,
                    Scope::Account,
                ),
                &["directConnectGatewayId", "virtualInterfaceId"],
            )?,
        ],
        &[
            super::s_direct_connect_decisions_0::ENTRIES,
            super::s_direct_connect_decisions_1::ENTRIES,
        ],
        &[
            // Both virtualInterfaces[].authKey and bgpPeers[].authKey return BGP authentication keys.
            (
                "DescribeVirtualInterfaces",
                crate::classes::DenyReason::Secret,
            ),
        ],
    )
}
