use super::*;
use aws_sdk_elasticloadbalancingv2::types::{
    ActionTypeEnum, AuthenticateOidcActionConfig, ForwardActionConfig, HostHeaderConditionConfig,
    LoadBalancerTypeEnum, ProtocolEnum, QueryStringConditionConfig, QueryStringKeyValuePair,
    RewriteConfig, TargetGroupTuple, TargetHealth, TargetHealthReasonEnum, TargetHealthStateEnum,
    TargetTypeEnum, TransformTypeEnum, UrlRewriteConfig,
};
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use std::sync::Arc;
use tokio::sync::mpsc;

type TestError = Box<dyn std::error::Error>;
const LB: &str = "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/front/abc";
const LISTENER: &str =
    "arn:aws:elasticloadbalancing:us-east-1:111111111111:listener/app/front/abc/def";
const GROUP: &str = "arn:aws:elasticloadbalancing:us-east-1:111111111111:targetgroup/api/abc";
const SECOND_GROUP: &str =
    "arn:aws:elasticloadbalancing:us-east-1:111111111111:targetgroup/canary/def";

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
            max_pages: 10,
            page_size: 100,
        },
        receiver,
    ))
}

fn first_resource(body: &PageBody) -> Result<&Resource, TestError> {
    body.resources
        .first()
        .ok_or_else(|| "missing mapped resource".into())
}

fn target_edge(body: &PageBody) -> Result<&Edge, TestError> {
    body.edges
        .iter()
        .find(|edge| edge.relation == "TARGETS")
        .ok_or_else(|| "missing target relation".into())
}

fn registered(id: &str, port: Option<i32>, zone: Option<&str>) -> TargetHealthDescription {
    TargetHealthDescription::builder()
        .target(
            TargetDescription::builder()
                .id(id)
                .set_port(port)
                .set_availability_zone(zone.map(str::to_owned))
                .build(),
        )
        .health_check_port("8080")
        .target_health(
            TargetHealth::builder()
                .state(TargetHealthStateEnum::Unhealthy)
                .reason(TargetHealthReasonEnum::FailedHealthChecks)
                .description("Health checks failed")
                .build(),
        )
        .build()
}

fn weighted_forward() -> Action {
    Action::builder()
        .r#type(ActionTypeEnum::Forward)
        .order(2)
        .target_group_arn(GROUP)
        .forward_config(
            ForwardActionConfig::builder()
                .target_groups(
                    TargetGroupTuple::builder()
                        .target_group_arn(GROUP)
                        .weight(90)
                        .build(),
                )
                .target_groups(
                    TargetGroupTuple::builder()
                        .target_group_arn(SECOND_GROUP)
                        .weight(10)
                        .build(),
                )
                .target_groups(
                    TargetGroupTuple::builder()
                        .target_group_arn(GROUP)
                        .weight(0)
                        .build(),
                )
                .build(),
        )
        .build()
}

mod all_three_load_balancer_types_keep_network_topology_without_duplicate_edges;
mod authentication_action_exposes_presence_without_copying_client_secret;
mod malformed_identity_and_nested_overflow_are_explicit_errors;
