use super::*;

#[tokio::test]
async fn findings_use_declared_facts_and_do_not_treat_missing_reads_as_false() -> Result<()> {
    let store = Store::open(":memory:").await?;
    let resources = vec![
        resource(
            "lb",
            "aws_elbv2_load_balancer",
            json!({"scheme":"internet-facing"}),
        ),
        resource(
            "listener",
            "aws_elbv2_listener",
            json!({"load_balancer_arn":"lb","port":80,"protocol":"HTTP","default_actions":[{"type":"redirect"}]}),
        ),
        resource(
            "routes",
            "aws_vpc_route_table",
            json!({"route_table_id":"rtb-fixture","routes":[
            {"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw-fixture","state":"active"},
            {"destination_ipv6_cidr_block":"::/0","gateway_id":"igw-fixture","state":"blackhole"},
            {"destination_cidr_block":"0.0.0.0/0","nat_gateway_id":"nat-fixture","state":"active"}]}),
        ),
        resource(
            "public",
            "aws_s3_bucket",
            json!({"policy_status":{"configured":true,"is_public":true},"public_access_block":{"configured":true,"block_public_policy":true}}),
        ),
        resource(
            "unknown",
            "aws_s3_bucket",
            json!({"policy_status":null,"unavailable_sections":["policy_status"]}),
        ),
        resource(
            "absent",
            "aws_s3_bucket",
            json!({"policy_status":{"configured":false}}),
        ),
        resource(
            "inconsistent",
            "aws_s3_bucket",
            json!({"policy_status":{"is_public":true}}),
        ),
    ];
    snapshot(
        &store,
        "facts",
        resources,
        vec![edge("lb", "missing", "IN_VPC")],
        ScanStatus::Complete,
    )
    .await?;
    let result = findings::report(&store, "facts", 100).await?;
    assert_eq!(
        result["rules"]["internet_facing_http_listener"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        result["rules"]["default_route_to_internet_gateway"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        result["rules"]["blackhole_route"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        result["rules"]["s3_policy_reported_public"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        first(&result["rules"]["s3_policy_reported_public"]["rows"])["bucket_arn"],
        "public"
    );
    assert_eq!(
        result["rules"]["unresolved_architecture_reference"]["rows"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn findings_preserve_collection_gaps_and_real_row_truncation() -> Result<()> {
    let store = Store::open(":memory:").await?;
    store.begin_scan("partial", 1).await?;
    store
        .apply_page(
            "partial",
            CollectionPage {
                scope: scope(Source::Direct, "fixture"),
                page_number: 1,
                next_token: None,
                resources: vec![resource("a", "fixture", json!({}))],
                edges: vec![edge("a", "x", "LINK"), edge("a", "y", "LINK")],
            },
        )
        .await?;
    store
        .record_failure(
            "partial",
            &CollectionFailure {
                scope: scope(Source::Direct, "denied"),
                category: "denied".into(),
                message: "synthetic denial".into(),
            },
        )
        .await?;
    store
        .finish_scan("partial", ScanStatus::Partial, 20)
        .await?;
    let result = findings::report(&store, "partial", 1).await?;
    assert_eq!(result["status"], "partial");
    assert_eq!(result["truncated"], true);
    assert_eq!(
        result["rules"]["unresolved_architecture_reference"]["truncated"],
        true
    );
    assert_eq!(result["rules"]["collection_scope_gap"]["truncated"], false);
    assert_eq!(
        first(&result["rules"]["collection_scope_gap"]["rows"])["coverage_status"],
        "failed"
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn findings_never_join_a_listener_from_a_different_scan() -> Result<()> {
    let store = Store::open(":memory:").await?;
    snapshot(
        &store,
        "lb",
        vec![resource(
            "lb",
            "aws_elbv2_load_balancer",
            json!({"scheme":"internet-facing"}),
        )],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    snapshot(
        &store,
        "listener",
        vec![resource(
            "listener",
            "aws_elbv2_listener",
            json!({"load_balancer_arn":"lb","protocol":"HTTP"}),
        )],
        vec![],
        ScanStatus::Complete,
    )
    .await?;
    let result = findings::report(&store, "lb", 100).await?;
    assert_eq!(
        result["rules"]["internet_facing_http_listener"]["rows"],
        json!([])
    );
    assert!(findings::report(&store, "lb", 0).await.is_err());
    assert!(findings::report(&store, "lb", 1001).await.is_err());
    store.close().await;
    Ok(())
}
