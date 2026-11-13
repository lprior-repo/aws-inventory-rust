use std::sync::Arc;

use aws_sdk_ec2::types::{
    AttachmentStatus, AvailabilityMode, IcmpTypeCode, InternetGatewayAttachment, NatGatewayAddress,
    NatGatewayState, NetworkAclAssociation, NetworkAclEntry, PortRange, RouteOrigin, RouteState,
    RouteTableAssociation, RouteTableAssociationState, RouteTableAssociationStateCode, RuleAction,
    SecurityGroupIdentifier, VpcPeeringConnectionStateReason, VpcPeeringConnectionStateReasonCode,
};
use inventory_core::CollectionEvent;
use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig};
use tokio::sync::mpsc;

use super::*;

type TestResult = Result<(), Box<dyn std::error::Error>>;

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

mod acl_keeps_order_direction_protocol_and_deny_without_access_edges;
mod network_scope_retains_nonterminal_page_then_records_denial;
mod route_target_owner_arn_and_unrecognized_gateway_are_preserved;
mod routes_keep_destinations_states_and_one_edge_per_target;
