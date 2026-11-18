// Pinned operation purposes: .local/aws-models/models/redshift-serverless/service/2021-04-21/redshift-serverless-2021-04-21.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ConvertRecoveryPointToSnapshot",
        Decision::Deferred(
            "Required selectors recoveryPointId, snapshotName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members snapshot. Pinned purpose: Converts a recovery point to a snapshot. For more information about recovery points and snapshots, see Working with snapshots and recovery points .",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a custom domain association for Amazon Redshift Serverless.
    (
        "CreateCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift Serverless managed VPC endpoint.
    (
        "CreateEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a namespace in Amazon Redshift Serverless.
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift Serverless reservation, which gives you the option to commit to a specified number of Redshift Processing Units (RPUs) for a year at a discount from Serverless on-demand (OD) rates.
    ("CreateReservation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a scheduled action. A scheduled action contains a schedule and an Amazon Redshift API action. For example, you can create a schedule of when to run the CreateSnapshot API operation.
    (
        "CreateScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a snapshot of all databases in a namespace. For more information about snapshots, see Working with snapshots and recovery points .
    ("CreateSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a snapshot copy configuration that lets you copy snapshots to another Amazon Web Services Region.
    (
        "CreateSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a usage limit for a specified Amazon Redshift Serverless usage type. The usage limit is identified by the returned usage limit identifier.
    ("CreateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an workgroup in Amazon Redshift Serverless. VPC Block Public Access (BPA) enables you to block resources in VPCs and subnets that you own in a Region from reaching or being reached from the internet through internet gateways and egress-only internet gateways. If a workgroup is in an account with VPC BPA turned on, the following capabilities are blocked: Creating a public access workgroup M
    ("CreateWorkgroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a custom domain association for Amazon Redshift Serverless.
    (
        "DeleteCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Redshift Serverless managed VPC endpoint.
    (
        "DeleteEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a namespace from Amazon Redshift Serverless. Before you delete the namespace, you can create a final snapshot that has all of the data within the namespace.
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified resource policy.
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a scheduled action.
    (
        "DeleteScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a snapshot from Amazon Redshift Serverless.
    ("DeleteSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a snapshot copy configuration
    (
        "DeleteSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a usage limit from Amazon Redshift Serverless.
    ("DeleteUsageLimit", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a workgroup.
    ("DeleteWorkgroup", Decision::Denied(DenyReason::Mutating)),
    // Returns temporary database credentials, not inventory metadata.
    ("GetCredentials", Decision::Denied(DenyReason::Secret)),
    (
        "GetCustomDomainAssociation",
        Decision::Deferred(
            "Required selectors customDomainName, workgroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members customDomainName, workgroupName, customDomainCertificateArn, customDomainCertificateExpiryTime. Pinned purpose: Gets information about a specific custom domain association.",
        ),
    ),
    (
        "GetEndpointAccess",
        Decision::Deferred(
            "Required selectors endpointName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members endpoint. Pinned purpose: Returns information, such as the name, about a VPC endpoint.",
        ),
    ),
    (
        "GetIdentityCenterAuthToken",
        Decision::Deferred(
            "Required selectors workgroupNames are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members token, expirationTime. Pinned purpose: Returns an Identity Center authentication token for accessing Amazon Redshift Serverless workgroups. The token provides secure access to data within the specified workgroups using Identity Center identity propagation. The token expires after a specif",
        ),
    ),
    (
        "GetNamespace",
        Decision::Deferred(
            "Required selectors namespaceName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members namespace. Pinned purpose: Returns information about a namespace in Amazon Redshift Serverless.",
        ),
    ),
    (
        "GetRecoveryPoint",
        Decision::Deferred(
            "Required selectors recoveryPointId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members recoveryPoint. Pinned purpose: Returns information about a recovery point.",
        ),
    ),
    (
        "GetReservation",
        Decision::Deferred(
            "Required selectors reservationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members reservation. Pinned purpose: Gets an Amazon Redshift Serverless reservation. A reservation gives you the option to commit to a specified number of Redshift Processing Units (RPUs) for a year at a discount from Serverless on-demand (OD) rates.",
        ),
    ),
    (
        "GetReservationOffering",
        Decision::Deferred(
            "Required selectors offeringId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members reservationOffering. Pinned purpose: Returns the reservation offering. The offering determines the payment schedule for the reservation.",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "Required selectors resourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourcePolicy. Pinned purpose: Returns a resource policy.",
        ),
    ),
    (
        "GetScheduledAction",
        Decision::Deferred(
            "Required selectors scheduledActionName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members scheduledAction. Pinned purpose: Returns information about a scheduled action.",
        ),
    ),
    (
        "GetSnapshot",
        Decision::Deferred(
            "Requires one of snapshotName or snapshotArn; neither is marked required in the pinned structure. ListSnapshots provides the complete root; no single required-selector binding is asserted here.",
        ),
    ),
    (
        "GetTableRestoreStatus",
        Decision::Deferred(
            "Required selectors tableRestoreRequestId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members tableRestoreStatus. Pinned purpose: Returns information about a TableRestoreStatus object.",
        ),
    ),
    (
        "GetTrack",
        Decision::Deferred(
            "Required selectors trackName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members track. Pinned purpose: Get the Redshift Serverless version for a specified track.",
        ),
    ),
    (
        "GetUsageLimit",
        Decision::Deferred(
            "Required selectors usageLimitId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members usageLimit. Pinned purpose: Returns information about a usage limit.",
        ),
    ),
    (
        "GetWorkgroup",
        Decision::Deferred(
            "Required selectors workgroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members workgroup. Pinned purpose: Returns information about a specific workgroup.",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Returns the current reservation offerings in your account.
    (
        "ListReservationOfferings",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags assigned to a resource.",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: List the Amazon Redshift Serverless versions.
    ("ListTracks", Decision::Denied(DenyReason::ReferenceData)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates a resource policy. Currently, you can use policies to share snapshots across Amazon Web Services accounts.
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Restore the data from a recovery point.
    (
        "RestoreFromRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Restores a namespace from a snapshot.
    (
        "RestoreFromSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Restores a table from a recovery point to your Amazon Redshift Serverless instance. You can't use this operation to restore tables with interleaved sort keys.
    (
        "RestoreTableFromRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Restores a table from a snapshot to your Amazon Redshift Serverless instance. You can't use this operation to restore tables with interleaved sort keys .
    (
        "RestoreTableFromSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes a tag or set of tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Redshift Serverless certificate associated with a custom domain.
    (
        "UpdateCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Redshift Serverless managed endpoint.
    (
        "UpdateEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies the lakehouse configuration for a namespace. This operation allows you to manage Amazon Redshift federated permissions and Amazon Web Services IAM Identity Center trusted identity propagation.
    (
        "UpdateLakehouseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a namespace with the specified settings. Unless required, you can't update multiple parameters in one request. For example, you must specify both adminUsername and adminUserPassword to update either field, but you can't update both kmsKeyId and logExports in a single request. Similarly, an S3 Tables log-publishing update (a request where logDestinationType is s3table ) cannot be combined w
    ("UpdateNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a scheduled action.
    (
        "UpdateScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a snapshot.
    ("UpdateSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a snapshot copy configuration.
    (
        "UpdateSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Update a usage limit in Amazon Redshift Serverless. You can't update the usage type or period of a usage limit.
    ("UpdateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a workgroup with the specified configuration settings. You can't update multiple parameters in one request. For example, you can update baseCapacity or port in a single request, but you can't update both in the same request. VPC Block Public Access (BPA) enables you to block resources in VPCs and subnets that you own in a Region from reaching or being reached from the internet through inte
    ("UpdateWorkgroup", Decision::Denied(DenyReason::Mutating)),
];
