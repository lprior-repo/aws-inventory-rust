//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kafka/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{Identity, IdentityField, KindClassification, MemberPath, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut clusters = policy::query(policy::tags(policy::enumerate(
        "ClusterInfoList", policy::arn("ClusterArn")?, Some(("NextToken", "NextToken")),
    )?, "Tags")?, &["ClusterArn", "ClusterType"])?;
    clusters.classification = Some(KindClassification {
        field: crate::overlay::MemberPath::new("ClusterType")?,
        variants: [
            ("PROVISIONED".to_owned(), "AWS::MSK::Cluster".to_owned()),
            ("SERVERLESS".to_owned(), "AWS::MSK::ServerlessCluster".to_owned()),
        ].into_iter().collect(),
    });
    let approved = vec![
        ("ListClustersV2", clusters),
        ("ListConfigurations", policy::query(policy::enumerate("Configurations", policy::arn("Arn")?, Some(("NextToken", "NextToken")))?, &["Arn"])?),
        ("ListReplicators", policy::query(policy::enumerate("Replicators", policy::arn("ReplicatorArn")?, Some(("NextToken", "NextToken")))?, &["ReplicatorArn"])?),
        ("ListVpcConnections", policy::query(policy::enumerate("VpcConnections", policy::arn("VpcConnectionArn")?, Some(("NextToken", "NextToken")))?, &["VpcConnectionArn", "TargetClusterArn"])?),
        ("ListTopics", policy::query(policy::bind(policy::enumerate("Topics", policy::arn("TopicArn")?, Some(("NextToken", "NextToken")))?, "ClusterArn", "ListClustersV2", "ClusterArn")?, &["TopicArn", "TopicName"])?),
        ("ListChannels", policy::query(policy::bind(policy::enumerate("Channels", policy::arn("ChannelArn")?, Some(("NextToken", "NextToken")))?, "ClusterArn", "ListClustersV2", "ClusterArn")?, &["ChannelArn", "ChannelName"])?),
        ("ListConfigurationRevisions", policy::query(policy::bind(
            policy::enumerate("Revisions", Identity::CorrelatedKey(vec![
                IdentityField::Request(MemberPath::new("Arn")?),
                IdentityField::Selected(MemberPath::new("Revision")?),
            ]), Some(("NextToken", "NextToken")))?,
            "Arn", "ListConfigurations", "Arn",
        )?, &["Revision"])?),
    ];
    policy::finish(
        "kafka", approved, DEFERRED,
        "Cluster, configuration, replicator, VPC connection, topic and channel summaries are metadata. Configuration revisions contain only revision/date/description and retain configuration-qualified identity. ClusterType dispatches provisioned/serverless kinds; unknown types fail closed. Explicit channel tokens are modeled without a paginator trait. Configuration bodies remain content-denied; client VPC ownership is not inferred.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeChannel", "DescribeCluster", "DescribeClusterOperation", "DescribeClusterOperationV2",
    "DescribeClusterV2", "DescribeConfiguration", "DescribeReplicator", "DescribeTopic",
    "DescribeTopicPartitions", "DescribeVpcConnection", "GetBootstrapBrokers", "GetClusterPolicy",
    "GetCompatibleKafkaVersions", "ListClientVpcConnections", "ListClusterOperations",
    "ListClusterOperationsV2", "ListClusters", "ListKafkaVersions",
    "ListNodes", "ListScramSecrets", "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "BatchAssociateScramSecret", "BatchDisassociateScramSecret", "CreateChannel", "CreateCluster",
    "CreateClusterV2", "CreateConfiguration", "CreateReplicator", "CreateTopic", "CreateVpcConnection",
    "DeleteChannel", "DeleteCluster", "DeleteClusterPolicy", "DeleteConfiguration", "DeleteReplicator",
    "DeleteTopic", "DeleteVpcConnection", "PutClusterPolicy", "RebootBroker", "RejectClientVpcConnection",
    "TagResource", "UntagResource", "UpdateBrokerCount", "UpdateBrokerStorage", "UpdateBrokerType",
    "UpdateChannel", "UpdateClusterConfiguration", "UpdateClusterKafkaVersion", "UpdateConfiguration",
    "UpdateConnectivity", "UpdateMonitoring", "UpdateRebalancing", "UpdateReplicationInfo", "UpdateSecurity",
    "UpdateStorage", "UpdateTopic",
];

const CONTENT: &[&str] = &[
    "DescribeConfigurationRevision",
];
