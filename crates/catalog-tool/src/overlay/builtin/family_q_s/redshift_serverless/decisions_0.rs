use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ConvertRecoveryPointToSnapshot",
        Decision::Deferred(
            "Required selectors recoveryPointId, snapshotName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members snapshot. Pinned purpose: Converts a recovery point to a snapshot. For more information about recovery points and snapshots, see Working with snapshots and recovery points .",
        ),
    ),
    (
        "CreateCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateNamespace", Decision::Denied(DenyReason::Mutating)),
    ("CreateReservation", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateSnapshot", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    ("CreateWorkgroup", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSnapshot", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteUsageLimit", Decision::Denied(DenyReason::Mutating)),
    ("DeleteWorkgroup", Decision::Denied(DenyReason::Mutating)),
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
    ("ListTracks", Decision::Denied(DenyReason::ReferenceData)),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "RestoreFromRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RestoreFromSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RestoreTableFromRecoveryPoint",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RestoreTableFromSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateLakehouseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateNamespace", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateSnapshot", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateSnapshotCopyConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    ("UpdateWorkgroup", Decision::Denied(DenyReason::Mutating)),
];
