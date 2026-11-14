use super::*;

#[tokio::test]
pub(super) async fn load_balancer_aliases_preserve_type_and_target_identity_fields() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("load-balancers", 1).await?;
    let resources = vec![
        fixture(
            "alb",
            "aws_elbv2_load_balancer",
            json!({"type":"application","load_balancer_name":"public-web","scheme":"internet-facing"}),
            2,
        ),
        fixture(
            "nlb",
            "aws_elbv2_load_balancer",
            json!({"type":"network","load_balancer_name":"internal-tcp","scheme":"internal"}),
            2,
        ),
        fixture(
            "gwlb",
            "aws_elbv2_load_balancer",
            json!({"type":"gateway","load_balancer_name":"inspection"}),
            2,
        ),
        fixture(
            "target-a",
            "aws_elbv2_target_health",
            json!({"target_group_arn":"tg","target_id":"i-one","target_port":443,
            "availability_zone":"us-east-1a","quic_server_id":"server-a","target_health_state":"healthy"}),
            2,
        ),
        fixture(
            "target-b",
            "aws_elbv2_target_health",
            json!({"target_group_arn":"tg","target_id":"i-one","target_port":8443,
            "availability_zone":"us-east-1a","quic_server_id":"server-b","target_health_state":"unhealthy"}),
            2,
        ),
    ];
    store
        .apply_page("load-balancers", page(resources, vec![]))
        .await?;
    store
        .finish_scan("load-balancers", ScanStatus::Complete, 3)
        .await?;
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_name,scheme FROM aws_ec2_application_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_name":"public-web","scheme":"internet-facing"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_type FROM aws_ec2_network_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_type":"network"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_type FROM aws_ec2_gateway_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_type":"gateway"}])
    );
    assert_eq!(store.query("SELECT target_port,quic_server_id,target_health_state FROM aws_elbv2_target_health ORDER BY target_port",10).await?,
        json!([{"target_port":443,"quic_server_id":"server-a","target_health_state":"healthy"},
               {"target_port":8443,"quic_server_id":"server-b","target_health_state":"unhealthy"}]));
    store.close().await;
    Ok(())
}
