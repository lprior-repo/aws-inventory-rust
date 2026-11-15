// Pinned operation purposes: .local/aws-models/models/redshift/service/2012-12-01/redshift-2012-12-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Exchanges a DC1 Reserved Node for a DC2 Reserved Node with no changes to the configuration (term, payment type, or number of nodes) and no additional costs.
    ("AcceptReservedNodeExchange", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a partner integration to a cluster. This operation authorizes a partner to push status updates for the specified database. To complete the integration, you also set up the integration on the partner website.
    ("AddPartner", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: From a datashare consumer account, associates a datashare with the account (AssociateEntireAccount) or the specified namespace (ConsumerArn). If you make this association, the consumer can consume the datashare.
    ("AssociateDataShareConsumer", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds an inbound (ingress) rule to an Amazon Redshift security group. Depending on whether the application accessing your cluster is running on the Internet or an Amazon EC2 instance, you can authorize inbound access to either a Classless Interdomain Routing (CIDR)/Internet Protocol (IP) range or to an Amazon EC2 security group. You can add as many as 20 ingress rules to an Amazon Redshift security
    ("AuthorizeClusterSecurityGroupIngress", Decision::Denied(DenyReason::Mutating)),
    ("AuthorizeDataShare", Decision::Deferred("Required selectors DataShareArn, ConsumerIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataShareArn, ProducerArn, AllowPubliclyAccessibleConsumers, DataShareAssociations, ManagedBy, DataShareType. Pinned purpose: From a data producer account, authorizes the sharing of a datashare with one or more consumer accounts or managing entities. To authorize a datashare for a data consumer, the producer account must have the correct access permissions.")),
    ("AuthorizeEndpointAccess", Decision::Deferred("Required selectors Account are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Grantor, Grantee, ClusterIdentifier, AuthorizeTime, ClusterStatus, Status, AllowedAllVPCs, AllowedVPCs, EndpointCount. Pinned purpose: Grants access to a cluster.")),
    ("AuthorizeSnapshotAccess", Decision::Deferred("Required selectors AccountWithRestoreAccess are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Snapshot. Pinned purpose: Authorizes the specified Amazon Web Services account to restore the specified snapshot. For more information about working with snapshots, go to Amazon Redshift Snapshots in the Amazon Redshift Cluster Management Guide .")),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a set of cluster snapshots.
    ("BatchDeleteClusterSnapshots", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Modifies the settings for a set of cluster snapshots.
    ("BatchModifyClusterSnapshots", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels a resize operation for a cluster.
    ("CancelResize", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Copies the specified automated cluster snapshot to a new manual cluster snapshot. The source must be an automated snapshot and it must be in the available state. When you delete a cluster, Amazon Redshift deletes any automated snapshots of the cluster. Also, when the retention period of the snapshot expires, Amazon Redshift automatically deletes it. If you want to keep an automated snapshot for a 
    ("CopyClusterSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an authentication profile with the specified parameters.
    ("CreateAuthenticationProfile", Decision::Denied(DenyReason::Mutating)),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Creates a new cluster with the specified parameters. To create a cluster in Virtual Private Cloud (VPC), you must provide a cluster subnet group name. The cluster subnet group identifies the subnets of your VPC that Amazon Redshift uses when creating the cluster. For more information about managing . Projection cannot authorize this call.
    ("CreateCluster", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift parameter group. Creating parameter groups is independent of creating clusters. You can associate a cluster with a parameter group when you create the cluster. You can also associate an existing cluster with a parameter group after the cluster is created by using ModifyCluster . Parameters in the parameter group define specific behavior that applies to the databases you 
    ("CreateClusterParameterGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon Redshift security group. You use security groups to control access to non-VPC clusters. For information about managing security groups, go to Amazon Redshift Cluster Security Groups in the Amazon Redshift Cluster Management Guide .
    ("CreateClusterSecurityGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a manual snapshot of the specified cluster. The cluster must be in the available state. For more information about working with snapshots, go to Amazon Redshift Snapshots in the Amazon Redshift Cluster Management Guide .
    ("CreateClusterSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Amazon Redshift subnet group. You must provide a list of one or more subnets in your existing Amazon Virtual Private Cloud (Amazon VPC) when creating Amazon Redshift subnet group. For information about subnet groups, go to Amazon Redshift Cluster Subnet Groups in the Amazon Redshift Cluster Management Guide .
    ("CreateClusterSubnetGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Used to create a custom domain name for a cluster. Properties include the custom domain name, the cluster the custom domain is associated with, and the certificate Amazon Resource Name (ARN).
    ("CreateCustomDomainAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Redshift-managed VPC endpoint.
    ("CreateEndpointAccess", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift event notification subscription. This action requires an ARN (Amazon Resource Name) of an Amazon SNS topic created by either the Amazon Redshift console, the Amazon SNS console, or the Amazon SNS API. To obtain an ARN with Amazon SNS, you must create a topic in Amazon SNS and subscribe to the topic. The ARN is displayed in the SNS console. You can specify the source type
    ("CreateEventSubscription", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an HSM client certificate that an Amazon Redshift cluster will use to connect to the client's HSM in order to store and retrieve the keys used to encrypt the cluster databases. The command returns a public key, which you must store in the HSM. In addition to creating the HSM certificate, you must create an Amazon Redshift HSM configuration that provides a cluster the information needed to 
    ("CreateHsmClientCertificate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an HSM configuration that contains the information required by an Amazon Redshift cluster to store and use database encryption keys in a Hardware Security Module (HSM). After creating the HSM configuration, you can specify it as a parameter when creating a cluster. The cluster will then store its encryption keys in the HSM. In addition to creating an HSM configuration, you must also create
    ("CreateHsmConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a zero-ETL integration or S3 event integration with Amazon Redshift.
    ("CreateIntegration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift Query Editor (QEV2) IAM Identity Center application.
    ("CreateQev2IdcApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Amazon Redshift application for use with IAM Identity Center.
    ("CreateRedshiftIdcApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a scheduled action. A scheduled action contains a schedule and an Amazon Redshift API action. For example, you can create a schedule of when to run the ResizeCluster API operation.
    ("CreateScheduledAction", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a snapshot copy grant that permits Amazon Redshift to use an encrypted symmetric key from Key Management Service (KMS) to encrypt copied snapshots in a destination region. For more information about managing snapshot copy grants, go to Amazon Redshift Database Encryption in the Amazon Redshift Cluster Management Guide .
    ("CreateSnapshotCopyGrant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Create a snapshot schedule that can be associated to a cluster and which overrides the default system backup schedule.
    ("CreateSnapshotSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds tags to a cluster. A resource can have up to 50 tags. If you try to create more than 50 tags for a resource, you will receive an error and the attempt will fail. If you specify a key that already exists for the resource, the value for that key will be updated with the new value.
    ("CreateTags", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a usage limit for a specified Amazon Redshift feature on a cluster. The usage limit is identified by the returned usage limit identifier.
    ("CreateUsageLimit", Decision::Denied(DenyReason::Mutating)),
    ("DeauthorizeDataShare", Decision::Deferred("Required selectors DataShareArn, ConsumerIdentifier are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members DataShareArn, ProducerArn, AllowPubliclyAccessibleConsumers, DataShareAssociations, ManagedBy, DataShareType. Pinned purpose: From a datashare producer account, removes authorization from the specified datashare.")),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an authentication profile.
    ("DeleteAuthenticationProfile", Decision::Denied(DenyReason::Mutating)),
    // Full modeled response contains authentication/secret material Cluster.PendingModifiedValues.MasterUserPassword. Pinned purpose: Deletes a previously provisioned cluster without its final snapshot being created. A successful response from the web service indicates that the request was received correctly. Use DescribeClusters to monitor the status of the deletion. The delete operation cannot be canceled or reverted once submit. Projection cannot authorize this call.
    ("DeleteCluster", Decision::Denied(DenyReason::Secret)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a specified Amazon Redshift parameter group. You cannot delete a parameter group if it is associated with a cluster.
    ("DeleteClusterParameterGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Redshift security group. You cannot delete a security group that is associated with any clusters. You cannot delete the default security group. For information about managing security groups, go to Amazon Redshift Cluster Security Groups in the Amazon Redshift Cluster Management Guide .
    ("DeleteClusterSecurityGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified manual snapshot. The snapshot must be in the available state, with no other users authorized to access the snapshot. Unlike automated snapshots, manual snapshots are retained even after you delete your cluster. Amazon Redshift does not delete your manual snapshots. You must delete manual snapshot explicitly to avoid getting charged. If other accounts are authorized to access 
    ("DeleteClusterSnapshot", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified cluster subnet group.
    ("DeleteClusterSubnetGroup", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Contains information about deleting a custom domain association for a cluster.
    ("DeleteCustomDomainAssociation", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a Redshift-managed VPC endpoint.
    ("DeleteEndpointAccess", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Redshift event notification subscription.
    ("DeleteEventSubscription", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified HSM client certificate.
    ("DeleteHsmClientCertificate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified Amazon Redshift HSM configuration.
    ("DeleteHsmConfiguration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a zero-ETL integration or S3 event integration with Amazon Redshift.
    ("DeleteIntegration", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a partner integration from a cluster. Data can still flow to the cluster until the integration is deleted at the partner's website.
    ("DeletePartner", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Redshift Query Editor (QEV2) IAM Identity Center application.
    ("DeleteQev2IdcApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Amazon Redshift IAM Identity Center application.
    ("DeleteRedshiftIdcApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the resource policy for a specified resource.
    ("DeleteResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a scheduled action.
    ("DeleteScheduledAction", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified snapshot copy grant.
    ("DeleteSnapshotCopyGrant", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a snapshot schedule.
    ("DeleteSnapshotSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes tags from a resource. You must provide the ARN of the resource from which you want to delete the tag or tags.
    ("DeleteTags", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a usage limit from a cluster.
    ("DeleteUsageLimit", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deregisters a cluster or serverless namespace from the Amazon Web Services Glue Data Catalog.
    ("DeregisterNamespace", Decision::Denied(DenyReason::Mutating)),
    ("DescribeAccountAttributes", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members AccountAttributes. Pinned request/ownership contract: Returns a list of attributes attached to an account")),
    // AuthenticationProfileContent is a free-form JSON authentication configuration. It is not safe to fetch arbitrary authentication payloads merely to project profile names.
    ("DescribeAuthenticationProfiles", Decision::Denied(DenyReason::Secret)),
];
