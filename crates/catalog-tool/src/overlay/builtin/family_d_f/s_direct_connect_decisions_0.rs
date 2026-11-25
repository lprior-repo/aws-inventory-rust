pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AcceptDirectConnectGatewayAssociationProposal",
        "Requires directConnectGatewayId, proposalId, associatedGatewayOwnerAccount; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts a proposal request to attach a virtual private gateway or transit gateway to a Direct Connect gateway.",
    ),
    (
        "AllocateConnectionOnInterconnect",
        "Requires bandwidth, connectionName, ownerAccount, interconnectId, vlan; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deprecated.",
    ),
    (
        "AllocateHostedConnection",
        "Requires connectionId, ownerAccount, bandwidth, connectionName, vlan; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a hosted connection on the specified interconnect or a link aggregation group (LAG) of interconnects.",
    ),
    (
        "AllocatePrivateVirtualInterface",
        "Requires connectionId, ownerAccount, newPrivateVirtualInterfaceAllocation; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provisions a private virtual interface to be owned by the specified Amazon Web Services account.",
    ),
    (
        "AllocatePublicVirtualInterface",
        "Requires connectionId, ownerAccount, newPublicVirtualInterfaceAllocation; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provisions a public virtual interface to be owned by the specified Amazon Web Services account.",
    ),
    (
        "AllocateTransitVirtualInterface",
        "Requires connectionId, ownerAccount, newTransitVirtualInterfaceAllocation; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provisions a transit virtual interface to be owned by the specified Amazon Web Services account.",
    ),
    (
        "AssociateConnectionWithLag",
        "Requires connectionId, lagId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates an existing connection with a link aggregation group (LAG).",
    ),
    (
        "AssociateConnectionsToResiliencyGroup",
        "Requires connectionIdentifiers, resiliencyGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates one or more connections with the specified resiliency group.",
    ),
    (
        "AssociateHostedConnection",
        "Requires connectionId, parentConnectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a hosted connection and its virtual interfaces with a link aggregation group (LAG) or interconnect.",
    ),
    (
        "AssociateMacSecKey",
        "Requires connectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a MAC Security (MACsec) Connection Key Name (CKN)/ Connectivity Association Key (CAK) pair with a Direct Connect connection.",
    ),
    (
        "AssociateVirtualInterface",
        "Requires virtualInterfaceId, connectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a virtual interface with a specified link aggregation group (LAG) or connection.",
    ),
    (
        "ConfirmConnection",
        "Requires connectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Confirms the creation of the specified hosted connection on an interconnect.",
    ),
    (
        "ConfirmCustomerAgreement",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: The confirmation of the terms of agreement when creating the connection/link aggregation group (LAG).",
    ),
    (
        "ConfirmPrivateVirtualInterface",
        "Requires virtualInterfaceId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts ownership of a private virtual interface created by another Amazon Web Services account.",
    ),
    (
        "ConfirmPublicVirtualInterface",
        "Requires virtualInterfaceId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts ownership of a public virtual interface created by another Amazon Web Services account.",
    ),
    (
        "ConfirmTransitVirtualInterface",
        "Requires virtualInterfaceId, directConnectGatewayId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts ownership of a transit virtual interface created by another Amazon Web Services account.",
    ),
    (
        "CreateBGPPeer",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Creates a BGP peer on the specified virtual interface.",
    ),
    (
        "CreateConnection",
        "Requires location, bandwidth, connectionName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a connection between a customer network and a specific Direct Connect location.",
    ),
    (
        "CreateDirectConnectGateway",
        "Requires directConnectGatewayName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Direct Connect gateway, which is an intermediate object that enables you to connect a set of virtual interfaces and virtual private gateways.",
    ),
    (
        "CreateDirectConnectGatewayAssociation",
        "Requires directConnectGatewayId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an association between a Direct Connect gateway and a virtual private gateway.",
    ),
    (
        "CreateDirectConnectGatewayAssociationProposal",
        "Requires directConnectGatewayId, directConnectGatewayOwnerAccount, gatewayId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a proposal to associate the specified virtual private gateway or transit gateway with the specified Direct Connect gateway.",
    ),
    (
        "CreateInterconnect",
        "Requires interconnectName, bandwidth, location; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an interconnect between an Direct Connect Partner's network and a specific Direct Connect location.",
    ),
    (
        "CreateLag",
        "Requires numberOfConnections, location, connectionsBandwidth, lagName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a link aggregation group (LAG) with the specified number of bundled physical dedicated connections between the customer network and a specific Direct Connect location.",
    ),
    (
        "CreatePrivateVirtualInterface",
        "Requires connectionId, newPrivateVirtualInterface; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a private virtual interface.",
    ),
    (
        "CreatePublicVirtualInterface",
        "Requires connectionId, newPublicVirtualInterface; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a public virtual interface.",
    ),
    (
        "CreateResiliencyGroup",
        "Requires resiliencyGroupName, intendedResiliencyModel; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a resiliency group.",
    ),
    (
        "CreateTransitVirtualInterface",
        "Requires connectionId, newTransitVirtualInterface; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a transit virtual interface.",
    ),
    (
        "DeleteBGPPeer",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes the specified BGP peer on the specified virtual interface with the specified customer address and ASN.",
    ),
    (
        "DeleteConnection",
        "Requires connectionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified connection.",
    ),
    (
        "DeleteDirectConnectGateway",
        "Requires directConnectGatewayId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified Direct Connect gateway.",
    ),
    (
        "DeleteDirectConnectGatewayAssociation",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes the association between the specified Direct Connect gateway and virtual private gateway.",
    ),
    (
        "DeleteDirectConnectGatewayAssociationProposal",
        "Requires proposalId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the association proposal request between the specified Direct Connect gateway and virtual private gateway or transit gateway.",
    ),
    (
        "DeleteInterconnect",
        "Requires interconnectId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified interconnect.",
    ),
    (
        "DeleteLag",
        "Requires lagId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified link aggregation group (LAG).",
    ),
    (
        "DeleteResiliencyGroup",
        "Requires resiliencyGroupId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified resiliency group.",
    ),
];
