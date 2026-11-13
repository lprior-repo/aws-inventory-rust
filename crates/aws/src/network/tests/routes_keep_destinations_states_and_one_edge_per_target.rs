//! routes keep destinations states and one edge per target behavior.
use super::*;

#[test]
fn routes_keep_destinations_states_and_one_edge_per_target() -> TestResult {
    let (ctx, _receiver) = context()?;
    let table = RouteTable::builder()
        .route_table_id("rtb-one")
        .vpc_id("vpc-one")
        .owner_id("222222222222")
        .routes(
            Route::builder()
                .destination_cidr_block("10.0.0.0/16")
                .gateway_id("local")
                .state(RouteState::Active)
                .build(),
        )
        .routes(
            Route::builder()
                .destination_cidr_block("0.0.0.0/0")
                .gateway_id("igw-one")
                .state(RouteState::Active)
                .origin(RouteOrigin::CreateRoute)
                .build(),
        )
        .routes(
            Route::builder()
                .destination_ipv6_cidr_block("::/0")
                .gateway_id("igw-one")
                .state(RouteState::Blackhole)
                .origin(RouteOrigin::CreateRoute)
                .build(),
        )
        .routes(
            Route::builder()
                .destination_prefix_list_id("pl-one")
                .gateway_id("igw-one")
                .state(RouteState::Active)
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_route_table(&ctx, &mut page, &table)?;
    let body = page.finish(None, 1)?;
    let resource = body.resources.first().ok_or("missing route table")?;
    assert_eq!(resource.account_id, "222222222222");
    assert!(resource.id.contains(":222222222222:route-table/"));
    assert_eq!(
        resource.properties.pointer("/routes/0/gateway_id"),
        Some(&json!("local"))
    );
    assert_eq!(
        resource.properties.pointer("/routes/2/state"),
        Some(&json!("blackhole"))
    );
    let targets = body
        .edges
        .iter()
        .filter(|edge| edge.relation == "ROUTES_TO")
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 1);
    let target = targets.first().ok_or("missing gateway target")?;
    assert!(target.to_id.ends_with("internet-gateway/igw-one"));
    assert_eq!(
        target.to_id,
        "arn:aws:ec2:us-east-1:222222222222:internet-gateway/igw-one"
    );
    assert_eq!(
        target.properties.get("owner_resolution"),
        Some(&json!("same_vpc_owner_from_response"))
    );
    assert_eq!(
        target
            .properties
            .get("evidence")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        target.properties.pointer("/evidence/1/state"),
        Some(&json!("blackhole"))
    );
    assert_eq!(
        target
            .properties
            .pointer("/evidence/2/destination_prefix_list_id"),
        Some(&json!("pl-one"))
    );
    assert!(
        body.edges
            .iter()
            .all(|edge| !edge.to_id.ends_with("/local"))
    );
    Ok(())
}

#[test]
fn main_association_does_not_fabricate_subnets_and_explicit_refs_deduplicate() -> TestResult {
    let (ctx, _receiver) = context()?;
    let explicit = RouteTableAssociation::builder()
        .route_table_association_id("rtbassoc-explicit")
        .route_table_id("rtb-one")
        .subnet_id("subnet-one")
        .main(false)
        .build();
    let table = RouteTable::builder()
        .route_table_id("rtb-one")
        .vpc_id("vpc-one")
        .owner_id("222222222222")
        .associations(
            RouteTableAssociation::builder()
                .route_table_association_id("rtbassoc-main")
                .main(true)
                .association_state(
                    RouteTableAssociationState::builder()
                        .state(RouteTableAssociationStateCode::Associated)
                        .build(),
                )
                .build(),
        )
        .associations(explicit.clone())
        .associations(explicit)
        .associations(
            RouteTableAssociation::builder()
                .route_table_association_id("rtbassoc-gateway")
                .gateway_id("igw-one")
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_route_table(&ctx, &mut page, &table)?;
    let body = page.finish(None, 1)?;
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
            .count(),
        1
    );
    let subnet = body
        .edges
        .iter()
        .find(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
        .ok_or("missing explicit subnet association")?;
    assert_eq!(
        subnet.to_id,
        "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-one"
    );
    assert_eq!(
        subnet.properties.get("owner_resolution"),
        Some(&json!("same_vpc_owner_from_response"))
    );
    let gateway = body
        .edges
        .iter()
        .find(|edge| edge.relation == "ASSOCIATED_WITH_GATEWAY")
        .ok_or("missing gateway association")?;
    assert_eq!(
        gateway.to_id,
        "arn:aws:ec2:us-east-1:222222222222:internet-gateway/igw-one"
    );
    assert_eq!(
        gateway.properties.get("owner_resolution"),
        Some(&json!("same_vpc_owner_from_response"))
    );
    let main = body
        .edges
        .iter()
        .find(|edge| edge.relation == "MAIN_ROUTE_TABLE_FOR")
        .ok_or("missing main association")?;
    assert_eq!(main.to_id, "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-one");
    assert_eq!(
        main.properties.pointer("/evidence/0/state"),
        Some(&json!("associated"))
    );
    assert_eq!(
        body.resources
            .first()
            .and_then(|resource| resource.properties.get("is_main")),
        Some(&json!(true))
    );
    Ok(())
}
