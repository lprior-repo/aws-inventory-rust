use super::*;

#[test]
fn route_target_owner_arn_and_unrecognized_gateway_are_preserved() -> TestResult {
    let (ctx, _receiver) = context()?;
    let core_arn = "arn:aws:networkmanager::333333333333:core-network/core-network-one";
    let table = RouteTable::builder()
        .route_table_id("rtb-one")
        .routes(
            Route::builder()
                .destination_cidr_block("10.2.0.0/16")
                .instance_id("i-one")
                .instance_owner_id("222222222222")
                .build(),
        )
        .routes(
            Route::builder()
                .destination_cidr_block("10.3.0.0/16")
                .core_network_arn(core_arn)
                .build(),
        )
        .routes(
            Route::builder()
                .destination_cidr_block("10.4.0.0/16")
                .gateway_id("future-target-one")
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_route_table(&ctx, &mut page, &table)?;
    let body = page.finish(None, 1)?;
    assert_eq!(body.edges.len(), 2);
    let instance = body
        .edges
        .iter()
        .find(|edge| edge.to_id.ends_with("instance/i-one"))
        .ok_or("missing instance")?;
    assert_eq!(
        instance.to_id,
        "arn:aws:ec2:us-east-1:222222222222:instance/i-one"
    );
    let core = body
        .edges
        .iter()
        .find(|edge| edge.to_id == core_arn)
        .ok_or("missing core network")?;
    assert_eq!(
        core.properties.get("owner_resolution"),
        Some(&json!("aws_returned_arn"))
    );
    assert_eq!(
        body.resources
            .first()
            .and_then(|resource| resource.properties.pointer("/routes/2/gateway_id")),
        Some(&json!("future-target-one"))
    );
    Ok(())
}

#[test]
fn internet_gateway_uses_reported_owner_and_attachment_state() -> TestResult {
    let (ctx, _receiver) = context()?;
    let gateway = InternetGateway::builder()
        .internet_gateway_id("igw-one")
        .owner_id("222222222222")
        .attachments(
            InternetGatewayAttachment::builder()
                .vpc_id("vpc-one")
                .state(AttachmentStatus::from("available"))
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_internet_gateway(&ctx, &mut page, &gateway)?;
    let body = page.finish(None, 1)?;
    assert_eq!(
        body.resources
            .first()
            .map(|resource| resource.account_id.as_str()),
        Some("222222222222")
    );
    let attachment = body.edges.first().ok_or("missing attachment")?;
    assert_eq!(attachment.relation, "ATTACHED_TO_VPC");
    assert_eq!(
        attachment.to_id,
        "arn:aws:ec2:us-east-1:222222222222:vpc/vpc-one"
    );
    assert_eq!(
        attachment.properties.pointer("/evidence/0/state"),
        Some(&json!("available"))
    );
    Ok(())
}

#[test]
fn regional_nat_allows_absent_subnet_and_maps_address_targets() -> TestResult {
    let (ctx, _receiver) = context()?;
    let gateway = NatGateway::builder()
        .nat_gateway_id("nat-one")
        .vpc_id("vpc-one")
        .availability_mode(AvailabilityMode::from("regional"))
        .state(NatGatewayState::Available)
        .route_table_id("nat-one")
        .nat_gateway_addresses(
            NatGatewayAddress::builder()
                .allocation_id("eipalloc-one")
                .network_interface_id("eni-one")
                .private_ip("10.0.1.4")
                .public_ip("203.0.113.10")
                .is_primary(true)
                .build(),
        )
        .build();
    let mut page = NetworkPage::default();
    map_nat_gateway(&ctx, &mut page, &gateway)?;
    let body = page.finish(None, 1)?;
    let nat = body.resources.first().ok_or("missing NAT")?;
    assert!(nat.id.ends_with("natgateway/nat-one"));
    assert_eq!(nat.properties.get("subnet_id"), Some(&Value::Null));
    assert_eq!(
        nat.properties.get("availability_mode"),
        Some(&json!("regional"))
    );
    assert_eq!(
        nat.properties.pointer("/addresses/0/is_primary"),
        Some(&json!(true))
    );
    assert!(body.edges.iter().all(|edge| edge.relation != "IN_SUBNET"));
    assert!(
        body.edges
            .iter()
            .all(|edge| edge.relation != "USES_ROUTE_TABLE")
    );
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "USES_ELASTIC_IP"
                && edge.to_id.ends_with("elastic-ip/eipalloc-one"))
    );
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.relation == "USES_NETWORK_INTERFACE"
                && edge.to_id.ends_with("network-interface/eni-one"))
    );
    Ok(())
}

#[test]
fn endpoint_shared_network_refs_stay_uncertain_and_duplicate_refs_share_edges() -> TestResult {
    let (ctx, _receiver) = context()?;
    let service_arn = "arn:aws:vpc-lattice:us-west-2:333333333333:servicenetwork/sn-one";
    let endpoint = VpcEndpoint::builder()
        .vpc_endpoint_id("vpce-one")
        .owner_id("222222222222")
        .vpc_id("vpc-shared")
        .subnet_ids("subnet-one")
        .subnet_ids("subnet-one")
        .route_table_ids("rtb-one")
        .network_interface_ids("eni-one")
        .groups(
            SecurityGroupIdentifier::builder()
                .group_id("sg-one")
                .group_name("endpoint")
                .build(),
        )
        .groups(
            SecurityGroupIdentifier::builder()
                .group_id("sg-one")
                .group_name("endpoint")
                .build(),
        )
        .service_network_arn(service_arn)
        .service_name("com.amazonaws.us-east-1.s3")
        .build();
    let mut page = NetworkPage::default();
    map_endpoint(&ctx, &mut page, &endpoint)?;
    let body = page.finish(None, 1)?;
    let endpoint = body.resources.first().ok_or("missing endpoint")?;
    assert_eq!(endpoint.account_id, "222222222222");
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "USES_SUBNET")
            .count(),
        1
    );
    assert_eq!(
        body.edges
            .iter()
            .filter(|edge| edge.relation == "USES_SECURITY_GROUP")
            .count(),
        1
    );
    let vpc = body
        .edges
        .iter()
        .find(|edge| edge.relation == "IN_VPC")
        .ok_or("missing VPC ref")?;
    assert_eq!(
        vpc.properties.get("owner_resolution"),
        Some(&json!("assumed_scanning_account"))
    );
    assert!(
        body.edges
            .iter()
            .any(|edge| edge.to_id == service_arn && edge.relation == "TARGETS_SERVICE_NETWORK")
    );
    assert!(
        body.edges
            .iter()
            .all(|edge| !edge.to_id.contains("com.amazonaws"))
    );
    Ok(())
}
