use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "DescribeClusterParameters",
        Decision::Deferred(
            "Required selectors ParameterGroupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Parameters, Marker. Pinned purpose: Returns a detailed list of parameters contained within the specified Amazon Redshift parameter group. For each parameter the response includes information such as parameter name, description, data type, value, whether the parameter value is modifiabl",
        ),
    ),
    (
        "DescribeClusterTracks",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeClusterVersions",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    ("DescribeClusters", Decision::Denied(DenyReason::Secret)),
    (
        "DescribeCustomDomainAssociations",
        Decision::Deferred(
            "Outer Associations group certificate ARN/expiry with CertificateAssociations[] resources. Flat item selection cannot flatten the nested domain/cluster entries while retaining outer certificate context for exact resource identity.",
        ),
    ),
    (
        "DescribeDataShares",
        Decision::Deferred(
            "Without ProducerArn or ConsumerArn selector, datashares can include foreign producers/consumer associations. DescribeDataSharesForProducer is the scoped producer-owned root; a generic copy would attribute foreign ARNs to this account.",
        ),
    ),
    (
        "DescribeDataSharesForConsumer",
        Decision::Deferred(
            "Returns datashares owned by other producer accounts. Consumer access is an association, not ownership of the DataShareArn; requires an association identity and cross-account target relation rather than a local datashare resource.",
        ),
    ),
    (
        "DescribeDefaultClusterParameters",
        Decision::Deferred(
            "Required selectors ParameterGroupFamily are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DefaultClusterParameters. Pinned purpose: Returns a list of parameter settings for the specified parameter group family. For more information about parameters and parameter groups, go to Amazon Redshift Parameter Groups in the Amazon Redshift Cluster Management Guide .",
        ),
    ),
    (
        "DescribeEventCategories",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeEvents",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Marker, Events. Pinned request/ownership contract: Returns events related to clusters, security groups, snapshots, and parameter groups for the past 14 days. Events specific to a particular cluster, security group, snapshot or parameter group can be obtained by providing the name as a parameter. By default, the past hour of events are returned.",
        ),
    ),
    (
        "DescribeLoggingStatus",
        Decision::Deferred(
            "Required selectors ClusterIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members LoggingEnabled, BucketName, S3KeyPrefix, LastSuccessfulDeliveryTime, LastFailureTime, LastFailureMessage, LogDestinationType, LogExports, S3Tables. Pinned purpose: Describes whether information, such as queries and connection attempts, is being logged for the specified Amazon Redshift cluster.",
        ),
    ),
    (
        "DescribeNodeConfigurationOptions",
        Decision::Deferred(
            "Required selectors ActionType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NodeConfigurationOptionList, Marker. Pinned purpose: Returns properties of possible node configurations such as node type, number of nodes, and disk usage for the specified action type.",
        ),
    ),
    (
        "DescribeOrderableClusterOptions",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribePartners",
        Decision::Deferred(
            "Required selectors AccountId, ClusterIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members PartnerIntegrationInfoList. Pinned purpose: Returns information about the partner integrations defined for a cluster.",
        ),
    ),
    (
        "DescribeReservedNodeOfferings",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeResize",
        Decision::Deferred(
            "Required selectors ClusterIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members TargetNodeType, TargetNumberOfNodes, TargetClusterType, Status, ImportTablesCompleted, ImportTablesInProgress, ImportTablesNotStarted, AvgResizeRateInMegaBytesPerSecond, TotalResizeDataInMegaBytes, ProgressInMegaBytes, ElapsedTimeInSeconds, EstimatedTimeToCompletionInSeconds, ResizeType, Message, TargetEncryptionType, DataTransferProgressPercent. Pinned purpose: Returns information about the last resize operation for the specified cluster. If no resize operation has ever been initiated for the specified cluster, a HTTP 404 error is returned. If a resize operation was initiated and completed, the status of th",
        ),
    ),
    (
        "DescribeStorage",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members TotalBackupSizeInMegaBytes, TotalProvisionedStorageInMegaBytes. Pinned request/ownership contract: Returns account level backups storage size and provisional storage.",
        ),
    ),
    (
        "DescribeTags",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: semantic resource selector documented by API. Pinned purpose: Returns a list of tags. You can return tags from a specific resource by specifying an ARN, or you can return all tags for a given type of resource, such as clusters, snapshots, and so on. The following are limitations for DescribeTags : You cannot specify an ARN and a resource-type value together in",
        ),
    ),
    ("DisableLogging", Decision::Denied(DenyReason::Mutating)),
    ("DisableSnapshotCopy", Decision::Denied(DenyReason::Secret)),
    (
        "DisassociateDataShareConsumer",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("EnableLogging", Decision::Denied(DenyReason::Mutating)),
    ("EnableSnapshotCopy", Decision::Denied(DenyReason::Secret)),
    (
        "FailoverPrimaryCompute",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "GetClusterCredentials",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "GetClusterCredentialsWithIAM",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "GetIdentityCenterAuthToken",
        Decision::Deferred(
            "Required selectors ClusterIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Token, ExpirationTime. Pinned purpose: Generates an encrypted authentication token that propagates the caller's Amazon Web Services IAM Identity Center identity to Amazon Redshift clusters. This API extracts the Amazon Web Services IAM Identity Center identity from enhanced credentials an",
        ),
    ),
    (
        "GetReservedNodeExchangeConfigurationOptions",
        Decision::Deferred(
            "Required selectors ActionType are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Marker, ReservedNodeConfigurationOptionList. Pinned purpose: Gets the configuration options for the reserved-node exchange. These options include information about the source reserved node and target reserved node offering. Details include the node type, the price, the node count, and the offering type.",
        ),
    ),
    (
        "GetReservedNodeExchangeOfferings",
        Decision::Deferred(
            "Required selectors ReservedNodeId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Marker, ReservedNodeOfferings. Pinned purpose: Returns an array of DC2 ReservedNodeOfferings that matches the payment type, term, and usage price of the given DC1 reserved node.",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "Required selectors ResourceArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourcePolicy. Pinned purpose: Get the resource policy for a specified resource.",
        ),
    ),
    (
        "ModifyAquaConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyAuthenticationProfile",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ModifyCluster", Decision::Denied(DenyReason::Secret)),
    (
        "ModifyClusterDbRevision",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ModifyClusterIamRoles",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ModifyClusterMaintenance",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ModifyClusterParameterGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyClusterSnapshot",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyClusterSnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyClusterSubnetGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyCustomDomainAssociation",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyEndpointAccess",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyEventSubscription",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ModifyIntegration", Decision::Denied(DenyReason::Mutating)),
    (
        "ModifyLakehouseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyQev2IdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyRedshiftIdcApplication",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifyScheduledAction",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ModifySnapshotCopyRetentionPeriod",
        Decision::Denied(DenyReason::Secret),
    ),
    (
        "ModifySnapshotSchedule",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("ModifyUsageLimit", Decision::Denied(DenyReason::Mutating)),
    ("PauseCluster", Decision::Denied(DenyReason::Secret)),
    (
        "PurchaseReservedNodeOffering",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    ("RebootCluster", Decision::Denied(DenyReason::Secret)),
];
