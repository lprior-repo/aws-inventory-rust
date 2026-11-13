use std::collections::BTreeMap;

use aws_sdk_ec2::{
    Client,
    types::{Instance, IpPermission, Tag},
};
use inventory_core::{Resource, Source};
use serde_json::{Value, json};

use crate::{
    AwsScanError,
    common::{Context, PageBody, paginate, require_budget, required, sdk_error},
};

#[tracing::instrument(skip_all)]
pub(crate) async fn collect(ctx: &Context) -> Result<(), AwsScanError> {
    let client = Client::new(&ctx.config);
    tokio::try_join!(
        instances(ctx, &client),
        vpcs(ctx, &client),
        subnets(ctx, &client),
        security_groups(ctx, &client),
        interfaces(ctx, &client),
        volumes(ctx, &client)
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Context, PageBody, link, resource};
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn shared_subnet_uses_reported_owner_and_vpc_owner() -> Result<(), Box<dyn std::error::Error>> {
        let (sender, _receiver) = tokio::sync::mpsc::channel(1);
        let ctx = Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages: 1,
            page_size: 100,
        };
        let subnet = resource(
            &ctx,
            "aws_vpc_subnet",
            "subnet",
            Some("subnet-bbb"),
            json!({
                "owner_id":"222222222222", "arn":"arn:aws:ec2:us-east-1:222222222222:subnet/subnet-bbb"
            }),
            &[],
        )?;
        assert_eq!(subnet.account_id, "222222222222");
        assert_eq!(
            subnet.id,
            "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-bbb"
        );
        let mut body = PageBody::default();
        link(&ctx, &mut body, &subnet, Some("vpc-aaa"), "vpc", "IN_VPC");
        let edge = body.edges.first().ok_or("missing subnet VPC edge")?;
        assert_eq!(edge.to_id, "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-aaa");
        assert_eq!(
            edge.properties
                .get("owner_resolution")
                .and_then(serde_json::Value::as_str),
            Some("subnet_owner_from_aws_response")
        );
        let instance = resource(
            &ctx,
            "aws_ec2_instance",
            "instance",
            Some("i-abc"),
            json!({}),
            &[],
        )?;
        let unknown_owner = ctx.edge(&instance, ctx.ec2_arn("subnet", "subnet-bbb"), "IN_SUBNET");
        assert_eq!(
            unknown_owner
                .properties
                .get("owner_resolution")
                .and_then(serde_json::Value::as_str),
            Some("assumed_scanning_account")
        );
        assert_eq!(
            unknown_owner
                .properties
                .get("target_resource_id")
                .and_then(serde_json::Value::as_str),
            Some("subnet-bbb")
        );
        Ok(())
    }
}

mod mapping;
use mapping::{link, map_instance, resource};
mod collection;
use collection::{instances, interfaces, security_groups, subnets, volumes, vpcs};
