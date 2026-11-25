use super::support::{arn, bound, inventory, key, queried, reviewed, scoped, tagged, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "direct-connect",
        vec![
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
        &[(
            "DescribeVirtualInterfaces",
            crate::classes::DenyReason::Secret,
        )],
    )
}
