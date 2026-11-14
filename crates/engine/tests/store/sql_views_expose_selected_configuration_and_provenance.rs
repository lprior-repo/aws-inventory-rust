use super::*;

#[tokio::test]
pub(super) async fn sql_views_expose_selected_configuration_and_provenance() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("one", 1).await?;
    let mut lambda = resource(
        "arn:aws:lambda:us-east-1:111111111111:function:worker",
        Source::Direct,
        10,
    );
    lambda.kind = "aws_lambda_function".to_owned();
    lambda.properties =
        json!({"function_name":"worker", "runtime":"provided.al2023", "memory_size":256});
    store
        .apply_page("one", page("lambda", Source::Direct, vec![lambda]))
        .await?;
    assert_eq!(
        store.query(
            "SELECT function_name,runtime,memory_size,source,observed_at FROM aws_lambda_function",
            10
        ).await?,
        json!([{"function_name":"worker", "runtime":"provided.al2023", "memory_size":256,
            "source":"direct", "observed_at":10}])
    );
    for view in [
        "aws_vpc",
        "aws_vpc_subnet",
        "aws_vpc_security_group",
        "aws_dynamodb_table",
        "aws_ecs_cluster",
        "aws_ecs_service",
        "aws_ecs_task",
        "aws_eks_cluster",
        "aws_eks_node_group",
    ] {
        assert_eq!(
            store.query(&format!("SELECT * FROM {view}"), 10).await?,
            json!([])
        );
    }
    Ok(())
}
