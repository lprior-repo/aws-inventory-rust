// Pinned operation purposes: .local/aws-models/models/redshift/service/2012-12-01/redshift-2012-12-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Registers a cluster or serverless namespace to the Amazon Web Services Glue Data Catalog.
    ("RegisterNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: From a datashare consumer account, rejects the specified datashare.
    ("RejectDataShare", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Sets one or more parameters of the specified parameter group to their default values and sets the source values of the parameters to "engine-default". To reset the entire parameter group specify the ResetAllParameters parameter. For parameter changes to take effect you must reboot any associated clusters.
    (
        "ResetClusterParameterGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Changes the size of the cluster. You can change the cluster's type, or change the number or type of nodes. The default behavior is to use the elastic resize method. With an elastic resize, your cluster is available for read and write operations more quickly than with the classic resize method. Elast. Projection cannot authorize this call.
    ("ResizeCluster", Decision::Denied(DenyReason::Secret)),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Creates a new cluster from a snapshot. By default, Amazon Redshift creates the resulting cluster with the same configuration as the original cluster from which the snapshot was created, except that the new cluster is created with the default cluster security and parameter groups. After Amazon Redshi. Projection cannot authorize this call.
    (
        "RestoreFromClusterSnapshot",
        Decision::Denied(DenyReason::Secret),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new table from a table in an Amazon Redshift cluster snapshot. You must create the new table within the Amazon Redshift cluster that the snapshot was taken from. You cannot use RestoreTableFromClusterSnapshot to restore a table with the same name as an existing table in an Amazon Redshift cluster. That is, you cannot overwrite an existing table in a cluster with a restored table. If you
    (
        "RestoreTableFromClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Resumes a paused cluster.. Projection cannot authorize this call.
    ("ResumeCluster", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Revokes an ingress rule in an Amazon Redshift security group for a previously authorized IP range or Amazon EC2 security group. To add an ingress rule, see AuthorizeClusterSecurityGroupIngress . For information about managing security groups, go to Amazon Redshift Cluster Security Groups in the Amazon Redshift Cluster Management Guide .
    (
        "RevokeClusterSecurityGroupIngress",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Revokes access to a cluster.
    (
        "RevokeEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the ability of the specified Amazon Web Services account to restore the specified snapshot. If the account is currently restoring the snapshot, the restore will run to completion. For more information about working with snapshots, go to Amazon Redshift Snapshots in the Amazon Redshift Cluster Management Guide .
    (
        "RevokeSnapshotAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Rotates the encryption keys for a cluster.. Projection cannot authorize this call.
    ("RotateEncryptionKey", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the status of a partner integration.
    (
        "UpdatePartnerStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
];
