use super::*;

#[tracing::instrument(skip_all)]
pub(super) async fn route_tables(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ROUTE_TABLE.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_route_tables()
                        .max_results(ctx.page_size.clamp(5, 100))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.route_tables().len(), budget)?;
            let mut page = NetworkPage::default();
            for table in output.route_tables() {
                map_route_table(ctx, &mut page, table)?;
            }
            page.finish(output.next_token(), output.route_tables().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn internet_gateways(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(IGW.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_internet_gateways()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.internet_gateways().len(), budget)?;
            let mut page = NetworkPage::default();
            for gateway in output.internet_gateways() {
                map_internet_gateway(ctx, &mut page, gateway)?;
            }
            page.finish(output.next_token(), output.internet_gateways().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn nat_gateways(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(NAT.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_nat_gateways()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.nat_gateways().len(), budget)?;
            let mut page = NetworkPage::default();
            for gateway in output.nat_gateways() {
                map_nat_gateway(ctx, &mut page, gateway)?;
            }
            page.finish(output.next_token(), output.nat_gateways().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn endpoints(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ENDPOINT.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_vpc_endpoints()
                        .max_results(ctx.page_size.clamp(1, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.vpc_endpoints().len(), budget)?;
            let mut page = NetworkPage::default();
            for endpoint in output.vpc_endpoints() {
                map_endpoint(ctx, &mut page, endpoint)?;
            }
            page.finish(output.next_token(), output.vpc_endpoints().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn network_acls(ctx: &Context, client: &Client) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(ACL.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_network_acls()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.network_acls().len(), budget)?;
            let mut page = NetworkPage::default();
            for acl in output.network_acls() {
                map_network_acl(ctx, &mut page, acl)?;
            }
            page.finish(output.next_token(), output.network_acls().len())
        },
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(super) async fn peering_connections(
    ctx: &Context,
    client: &Client,
) -> Result<(), AwsScanError> {
    paginate(
        ctx,
        ctx.scope(PEERING.resource_kind, None, Source::Direct),
        |token, budget| async move {
            let output = ctx
                .execute("ec2", "read", || async {
                    client
                        .describe_vpc_peering_connections()
                        .max_results(ctx.page_size.clamp(5, 1_000))
                        .set_next_token(token.clone())
                        .send()
                        .await
                        .map_err(sdk_error)
                })
                .await?;
            require_budget(output.vpc_peering_connections().len(), budget)?;
            let mut page = NetworkPage::default();
            for peering in output.vpc_peering_connections() {
                map_peering(ctx, &mut page, peering)?;
            }
            page.finish(output.next_token(), output.vpc_peering_connections().len())
        },
    )
    .await
}
