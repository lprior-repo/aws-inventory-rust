//! acl keeps order direction protocol and deny without access edges behavior.
use super::*;

#[test]
fn acl_keeps_order_direction_protocol_and_deny_without_access_edges() -> TestResult {
    let (ctx, _receiver) = context()?;
    let acl = NetworkAcl::builder()
        .network_acl_id("acl-one")
        .vpc_id("vpc-one")
        .owner_id("222222222222")
        .is_default(false)
        .entries(
            NetworkAclEntry::builder()
                .rule_number(100)
                .egress(false)
                .protocol("6")
                .rule_action(RuleAction::Allow)
                .cidr_block("10.0.0.0/16")
                .port_range(PortRange::builder().from(443).to(443).build())
                .build(),
        )
        .entries(
            NetworkAclEntry::builder()
                .rule_number(200)
                .egress(true)
                .protocol("58")
                .rule_action(RuleAction::Deny)
                .ipv6_cidr_block("::/0")
                .icmp_type_code(IcmpTypeCode::builder().r#type(1).code(-1).build())
                .build(),
        )
        .entries(
            NetworkAclEntry::builder()
                .rule_number(32767)
                .egress(false)
                .protocol("-1")
                .rule_action(RuleAction::Deny)
                .cidr_block("0.0.0.0/0")
                .build(),
        )
        .associations(
            NetworkAclAssociation::builder()
                .network_acl_association_id("aclassoc-one")
                .network_acl_id("acl-one")
                .subnet_id("subnet-one")
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_network_acl(&ctx, &mut page, &acl)?;
    let body = page.finish(None, 1)?;
    let acl = body.resources.first().ok_or("missing ACL")?;
    assert_eq!(
        acl.properties.pointer("/entries/0/port_range/from"),
        Some(&json!(443))
    );
    assert_eq!(
        acl.properties.pointer("/entries/1/egress"),
        Some(&json!(true))
    );
    assert_eq!(
        acl.properties.pointer("/entries/1/icmp_type_code/code"),
        Some(&json!(-1))
    );
    assert_eq!(
        acl.properties.pointer("/entries/2/rule_number"),
        Some(&json!(32767))
    );
    assert_eq!(
        acl.properties.pointer("/entries/2/rule_action"),
        Some(&json!("deny"))
    );
    let subnet = body
        .edges
        .iter()
        .find(|edge| edge.relation == "ASSOCIATED_WITH_SUBNET")
        .ok_or("missing ACL subnet association")?;
    assert_eq!(
        subnet.to_id,
        "arn:aws:ec2:us-east-1:222222222222:subnet/subnet-one"
    );
    assert_eq!(
        subnet.properties.get("owner_resolution"),
        Some(&json!("same_vpc_owner_from_response"))
    );
    assert!(
        body.edges
            .iter()
            .all(|edge| matches!(edge.relation.as_str(), "IN_VPC" | "ASSOCIATED_WITH_SUBNET"))
    );
    Ok(())
}

#[test]
fn peering_uses_both_reported_owners_and_regions_without_reversing_roles() -> TestResult {
    let (ctx, _receiver) = context()?;
    let peering = VpcPeeringConnection::builder()
        .vpc_peering_connection_id("pcx-one")
        .requester_vpc_info(
            VpcPeeringConnectionVpcInfo::builder()
                .vpc_id("vpc-requester")
                .owner_id("222222222222")
                .region("eu-west-1")
                .cidr_block("10.0.0.0/16")
                .build(),
        )
        .accepter_vpc_info(
            VpcPeeringConnectionVpcInfo::builder()
                .vpc_id("vpc-accepter")
                .owner_id("333333333333")
                .region("us-west-2")
                .cidr_block("10.1.0.0/16")
                .build(),
        )
        .status(
            VpcPeeringConnectionStateReason::builder()
                .code(VpcPeeringConnectionStateReasonCode::Active)
                .message("Active")
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_peering(&ctx, &mut page, &peering)?;
    let body = page.finish(None, 1)?;
    let connection = body.resources.first().ok_or("missing connection")?;
    assert_eq!(connection.account_id, "111111111111");
    assert_eq!(connection.properties.get("owner_id"), Some(&Value::Null));
    assert_eq!(
        connection.properties.get("identity_scope"),
        Some(&json!("scanning_account_and_region"))
    );
    let requester = body
        .edges
        .iter()
        .find(|edge| edge.relation == "REQUESTER_VPC")
        .ok_or("missing requester")?;
    let accepter = body
        .edges
        .iter()
        .find(|edge| edge.relation == "ACCEPTER_VPC")
        .ok_or("missing accepter")?;
    assert_eq!(
        requester.to_id,
        "arn:aws:ec2:eu-west-1:222222222222:vpc/vpc-requester"
    );
    assert_eq!(
        accepter.to_id,
        "arn:aws:ec2:us-west-2:333333333333:vpc/vpc-accepter"
    );
    assert_eq!(
        requester.properties.get("owner_resolution"),
        Some(&json!("aws_response"))
    );
    assert_eq!(
        accepter.properties.get("target_region"),
        Some(&json!("us-west-2"))
    );
    Ok(())
}

#[test]
fn repeated_resource_identity_fails_before_storage() -> TestResult {
    let (ctx, _receiver) = context()?;
    let table = RouteTable::builder().route_table_id("rtb-one").build();
    let mut page = NetworkPage::default();
    map_route_table(&ctx, &mut page, &table)?;
    map_route_table(&ctx, &mut page, &table)?;
    assert!(matches!(
        page.finish(None, 2),
        Err(AwsScanError::InvalidData(_))
    ));
    Ok(())
}

#[test]
fn nested_capacity_is_checked_before_resource_mapping() -> TestResult {
    let (ctx, _receiver) = context()?;
    let count = MAX_COMPONENTS_PER_RESOURCE
        .checked_add(1)
        .ok_or("test count overflow")?;
    let table = RouteTable::builder()
        .route_table_id("rtb-one")
        .set_routes(Some(vec![
            Route::builder().gateway_id("local").build();
            count
        ]))
        .build();
    let mut page = NetworkPage::default();
    assert!(matches!(
        map_route_table(&ctx, &mut page, &table),
        Err(AwsScanError::InvalidData(_))
    ));
    assert!(page.resources.is_empty());
    assert!(page.edges.is_empty());
    Ok(())
}

#[test]
fn absent_identifier_is_an_error_and_empty_token_is_terminal() -> TestResult {
    let (ctx, _receiver) = context()?;
    let mut page = NetworkPage::default();
    assert!(matches!(
        map_internet_gateway(&ctx, &mut page, &InternetGateway::builder().build()),
        Err(AwsScanError::InvalidData(_))
    ));
    assert!(
        NetworkPage::default()
            .finish(Some(""), 0)?
            .next_token
            .is_none()
    );
    Ok(())
}
