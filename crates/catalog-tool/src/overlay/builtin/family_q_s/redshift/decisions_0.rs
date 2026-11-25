use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptReservedNodeExchange",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("AddPartner", Decision::Denied(DenyReason::Mutating)),
    (
        "AssociateDataShareConsumer",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AuthorizeClusterSecurityGroupIngress",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AuthorizeDataShare",
        Decision::Deferred(
            "Required selectors DataShareArn, ConsumerIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataShareArn, ProducerArn, AllowPubliclyAccessibleConsumers, DataShareAssociations, ManagedBy, DataShareType. Pinned purpose: From a data producer account, authorizes the sharing of a datashare with one or more consumer accounts or managing entities. To authorize a datashare for a data consumer, the producer account must have the correct access permissions.",
        ),
    ),
    (
        "AuthorizeEndpointAccess",
        Decision::Deferred(
            "Required selectors Account are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Grantor, Grantee, ClusterIdentifier, AuthorizeTime, ClusterStatus, Status, AllowedAllVPCs, AllowedVPCs, EndpointCount. Pinned purpose: Grants access to a cluster.",
        ),
    ),
    (
        "AuthorizeSnapshotAccess",
        Decision::Deferred(
            "Required selectors AccountWithRestoreAccess are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Snapshot. Pinned purpose: Authorizes the specified Amazon Web Services account to restore the specified snapshot. For more information about working with snapshots, go to Amazon Redshift Snapshots in the Amazon Redshift Cluster Management Guide .",
        ),
    ),
    (
        "BatchDeleteClusterSnapshots",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchModifyClusterSnapshots",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CancelResize", Decision::Denied(DenyReason::Mutating)),
    (
        "CopyClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAuthenticationProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateCluster", Decision::Denied(DenyReason::Secret)),
    (
        "CreateClusterParameterGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateClusterSecurityGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateClusterSubnetGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateEventSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateHsmClientCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateHsmConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateIntegration", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateQev2IdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateRedshiftIdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateSnapshotCopyGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateTags", Decision::Denied(DenyReason::Mutating)),
    ("CreateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    (
        "DeauthorizeDataShare",
        Decision::Deferred(
            "Required selectors DataShareArn, ConsumerIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataShareArn, ProducerArn, AllowPubliclyAccessibleConsumers, DataShareAssociations, ManagedBy, DataShareType. Pinned purpose: From a datashare producer account, removes authorization from the specified datashare.",
        ),
    ),
    (
        "DeleteAuthenticationProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteCluster", Decision::Denied(DenyReason::Secret)),
    (
        "DeleteClusterParameterGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteClusterSecurityGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteClusterSubnetGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteEventSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteHsmClientCertificate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteHsmConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteIntegration", Decision::Denied(DenyReason::Mutating)),
    ("DeletePartner", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteQev2IdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteRedshiftIdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteSnapshotCopyGrant",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteTags", Decision::Denied(DenyReason::Mutating)),
    ("DeleteUsageLimit", Decision::Denied(DenyReason::Mutating)),
    (
        "DeregisterNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeAccountAttributes",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AccountAttributes. Pinned request/ownership contract: Returns a list of attributes attached to an account",
        ),
    ),
    (
        "DescribeAuthenticationProfiles",
        Decision::Denied(DenyReason::Secret),
    ),
];
