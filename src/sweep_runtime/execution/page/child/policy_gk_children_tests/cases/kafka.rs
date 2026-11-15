use super::*;

fn clusters() -> [Parent; 2] {
    [("cluster-a", "PROVISIONED", "AWS::MSK::Cluster"), ("cluster-b", "SERVERLESS", "AWS::MSK::ServerlessCluster")]
        .map(|(name, cluster_type, kind)| {
            let arn = format!("arn:aws:kafka:us-east-1:123456789012:cluster/{name}/00000000-0000-0000-0000-000000000001-2");
            Parent { binding: arn.clone(), id: arn.clone(), kind,
                row: json!({"clusterArn":arn,"clusterType":cluster_type,"clusterName":name}) }
        })
}

fn cluster_child(operation: &'static str, items: &'static str, kind: &'static str) -> Case {
    Case { service: "kafka", date: "2018-11-14", root: "ListClustersV2", root_items: "clusterInfoList",
        operation, items, input: "ClusterArn", token: "NextToken", kind, relation: "IN_CLUSTER",
        forbidden: &["DescribeTopic", "DescribeTopicPartitions", "DescribeChannel", "DescribeConfigurationRevision", "ListScramSecrets", "GetBootstrapBrokers"] }
}

#[tokio::test]
async fn given_both_cluster_kinds_when_topic_wire_pages_resume_then_same_names_have_distinct_arns_and_exact_cluster_edges() -> TestResult {
    exercise(cluster_child("ListTopics", "topics", "AWS::MSK::Topic"), clusters()).await
}

#[tokio::test]
async fn given_both_cluster_kinds_when_channel_explicit_tokens_resume_then_lowercase_wire_rows_and_cluster_edges_are_persisted() -> TestResult {
    exercise(cluster_child("ListChannels", "channels", "AWS::MSK::Channel"), clusters()).await
}

#[tokio::test]
async fn given_two_configurations_when_revision_one_repeats_then_configuration_qualified_revisions_survive_without_body_reads() -> TestResult {
    let parents = ["configuration-a", "configuration-b"].map(|name| {
        let arn = format!("arn:aws:kafka:us-east-1:123456789012:configuration/{name}/00000000-0000-0000-0000-000000000001-2");
        Parent { binding: arn.clone(), id: arn.clone(), kind: "AWS::MSK::Configuration",
            row: json!({"arn":arn,"name":name,"creationTime":"2026-01-01T00:00:00Z", "kafkaVersions":["3.6.0"],
                "latestRevision":{"revision":2,"creationTime":"2026-01-01T00:00:00Z"},"state":"ACTIVE"}) }
    });
    exercise(Case { service: "kafka", date: "2018-11-14", root: "ListConfigurations", root_items: "configurations",
        operation: "ListConfigurationRevisions", items: "revisions", input: "Arn", token: "NextToken",
        kind: "AWS::ApiOnly::MSKConfigurationRevision", relation: "VERSION_OF",
        forbidden: &["DescribeConfigurationRevision", "DescribeConfiguration"] }, parents).await
}
