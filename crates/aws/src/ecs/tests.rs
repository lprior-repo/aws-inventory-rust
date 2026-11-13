use std::sync::Arc;

use aws_sdk_ecs::types::{
    AdvancedConfiguration, AwsVpcConfiguration, LoadBalancer, NetworkConfiguration,
};
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;
const SERVICE: &str = "arn:aws:ecs:us-east-1:111111111111:service/demo/web";
const GROUP: &str =
    "arn:aws:elasticloadbalancing:us-west-2:222222222222:targetgroup/web/0123456789abcdef";
const DEFINITION: &str = "arn:aws:ecs:us-east-1:111111111111:task-definition/web:7";

fn context() -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(8);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages: 3,
            page_size: 100,
        },
        receiver,
    ))
}

#[test]
fn service_groups_aggregate_primary_alternate_and_container_evidence() -> TestResult {
    let (ctx, _receiver) = context()?;
    let service = Service::builder()
        .service_arn(SERVICE)
        .service_name("web")
        .load_balancers(
            LoadBalancer::builder()
                .target_group_arn(GROUP)
                .container_name("web")
                .container_port(8080)
                .advanced_configuration(
                    AdvancedConfiguration::builder()
                        .alternate_target_group_arn(GROUP)
                        .production_listener_rule("arn:aws:elasticloadbalancing:us-west-2:222222222222:listener-rule/app/demo/lb/listener/rule")
                        .role_arn("arn:aws:iam::222222222222:role/deploy")
                        .build(),
                )
                .build(),
        )
        .load_balancers(
            LoadBalancer::builder()
                .target_group_arn(GROUP)
                .container_name("admin")
                .container_port(9090)
                .build(),
        )
        .build();
    let mut body = PageBody::default();
    map_service(&ctx, &mut body, &service)?;
    assert_eq!(body.edges.len(), 1);
    let edge = body.edges.first().ok_or("missing target group edge")?;
    assert_eq!(edge.to_id, GROUP);
    assert_eq!(edge.relation, "USES_TARGET_GROUP");
    assert_eq!(
        edge.properties.get("target_account_id"),
        Some(&json!("222222222222"))
    );
    assert_eq!(
        edge.properties.get("target_region"),
        Some(&json!("us-west-2"))
    );
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("aws_returned_arn"))
    );
    assert_eq!(
        edge.properties.pointer("/evidence/1/configuration_role"),
        Some(&json!("alternate"))
    );
    assert_eq!(
        edge.properties.pointer("/evidence/2/container_port"),
        Some(&json!(9090))
    );
    let resource = body.resources.first().ok_or("missing ECS service")?;
    assert_eq!(
        resource.properties.get("target_group_arns"),
        Some(&json!([GROUP]))
    );
    assert_eq!(
        resource
            .properties
            .pointer("/load_balancers/0/advanced_configuration/role_arn"),
        Some(&json!("arn:aws:iam::222222222222:role/deploy"))
    );
    Ok(())
}

#[test]
fn classic_name_is_a_distinct_unobserved_reference_with_owner_assumption() -> TestResult {
    let (ctx, _receiver) = context()?;
    let balancer = LoadBalancer::builder()
        .load_balancer_name("legacy-web")
        .container_name("web")
        .container_port(80)
        .build();
    let service = Service::builder()
        .service_arn(SERVICE)
        .load_balancers(balancer.clone())
        .load_balancers(balancer)
        .build();
    let mut body = PageBody::default();
    map_service(&ctx, &mut body, &service)?;
    assert_eq!(body.resources.len(), 1);
    assert_eq!(body.edges.len(), 1);
    let edge = body.edges.first().ok_or("missing Classic ELB edge")?;
    assert_eq!(edge.relation, "USES_CLASSIC_LOAD_BALANCER");
    assert_eq!(
        edge.to_id,
        "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/legacy-web"
    );
    assert_eq!(
        edge.properties.get("target_kind"),
        Some(&json!("aws_ec2_classic_load_balancer"))
    );
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("assumed_scanning_account"))
    );
    assert_eq!(
        edge.properties
            .get("evidence")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
    Ok(())
}

#[test]
fn service_network_duplicates_do_not_duplicate_edges_and_revision_is_retained() -> TestResult {
    let (ctx, _receiver) = context()?;
    let service = Service::builder()
        .service_arn(SERVICE)
        .cluster_arn("arn:aws:ecs:us-east-1:111111111111:cluster/demo")
        .task_definition(DEFINITION)
        .network_configuration(
            NetworkConfiguration::builder()
                .awsvpc_configuration(
                    AwsVpcConfiguration::builder()
                        .subnets("subnet-one")
                        .subnets("subnet-one")
                        .security_groups("sg-one")
                        .security_groups("sg-one")
                        .build()?,
                )
                .build(),
        )
        .build();
    let mut body = PageBody::default();
    map_service(&ctx, &mut body, &service)?;
    assert_eq!(body.edges.len(), 4);
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "USES_TASK_DEFINITION" && edge.to_id == DEFINITION)
    );
    let subnet = body
        .edges
        .iter()
        .find(|edge| edge.relation == "USES_SUBNET")
        .ok_or("missing subnet")?;
    assert_eq!(
        subnet.properties.get("owner_resolution"),
        Some(&json!("assumed_scanning_account"))
    );
    assert_eq!(
        subnet
            .properties
            .get("evidence")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(2)
    );
    Ok(())
}

#[test]
fn malformed_service_load_balancer_references_do_not_emit_resources() -> TestResult {
    let (ctx, _receiver) = context()?;
    for balancer in [
        LoadBalancer::builder().target_group_arn("web").build(),
        LoadBalancer::builder()
            .target_group_arn(
                "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/web/id",
            )
            .build(),
        LoadBalancer::builder().load_balancer_name("").build(),
    ] {
        let service = Service::builder()
            .service_arn(SERVICE)
            .load_balancers(balancer)
            .build();
        let mut body = PageBody::default();
        assert!(map_service(&ctx, &mut body, &service).is_err());
        assert!(body.resources.is_empty());
    }
    Ok(())
}

#[test]
fn service_reference_bound_precedes_mapping() -> TestResult {
    let (ctx, _receiver) = context()?;
    let service = Service::builder()
        .service_arn(SERVICE)
        .set_load_balancers(Some(vec![
            LoadBalancer::builder().build();
            MAX_SERVICE_REFERENCES.saturating_add(1)
        ]))
        .build();
    let mut body = PageBody::default();
    assert!(matches!(
        map_service(&ctx, &mut body, &service),
        Err(AwsScanError::InvalidData(_))
    ));
    assert!(body.resources.is_empty() && body.edges.is_empty());
    Ok(())
}
