//! read visible environment commands parameter values and firehose tokens are redacted behavior.
use super::*;

#[test]
fn read_visible_environment_commands_parameter_values_and_firehose_tokens_are_redacted()
-> TestResult {
    for (name, raw) in [
        (
            "AWS::Lambda::Function",
            json!({"Environment":{"Variables":{"TOKEN":"HIDDEN"}},"Code":{"ZipFile":"HIDDEN"},"Role":"arn:aws:iam::111111111111:role/application"}),
        ),
        (
            "AWS::ECS::TaskDefinition",
            json!({"ContainerDefinitions":[{"Name":"app","Environment":[{"Name":"TOKEN","Value":"HIDDEN"}],"Command":["HIDDEN"],"LogConfiguration":{"Options":{"secret":"HIDDEN"}}}]}),
        ),
        (
            "AWS::Batch::JobDefinition",
            json!({"Parameters":{"input":"HIDDEN"},"EksProperties":{"PodProperties":{"Containers":[{"Env":[{"Name":"TOKEN","Value":"HIDDEN"}],"Args":["HIDDEN"]}],"InitContainers":[{"Env":[{"Name":"TOKEN","Value":"HIDDEN"}]}]}},"NodeProperties":{"NodeRangeProperties":[{"EksProperties":{"PodProperties":{"Containers":[{"Env":[{"Value":"HIDDEN"}]}]}}}]}}),
        ),
        (
            "AWS::SSM::Parameter",
            json!({"Name":"/app/config","Value":"HIDDEN"}),
        ),
        (
            "AWS::KinesisFirehose::DeliveryStream",
            json!({"SplunkDestinationConfiguration":{"HECToken":"HIDDEN"},"HttpEndpointDestinationConfiguration":{"EndpointConfiguration":{"AccessKey":"HIDDEN"}}}),
        ),
        (
            "AWS::SNS::Subscription",
            json!({"Protocol":"https","Endpoint":"https://user:HIDDEN@example.test/notify"}),
        ),
        (
            "AWS::Pipes::Pipe",
            json!({"EnrichmentParameters":{"HttpParameters":{"HeaderParameters":{"Authorization":"HIDDEN"},"QueryStringParameters":{"key":"HIDDEN"}}}}),
        ),
        (
            "AWS::RDS::OptionGroup",
            json!({"OptionConfigurations":[{"OptionName":"OEM_AGENT","OptionSettings":[{"Name":"AGENT_REGISTRATION_PASSWORD","Value":"HIDDEN"}]}]}),
        ),
    ] {
        let (safe, redacted) = sanitized_properties(&descriptor(name)?, Some(&raw.to_string()))?;
        assert!(
            !safe.to_string().contains("HIDDEN"),
            "sensitive value survived for {name}"
        );
        assert!(!redacted.is_empty());
    }
    Ok(())
}

#[test]
fn relationships_deduplicate_paths_and_preserve_returned_arn_ownership() -> TestResult {
    let (ctx, _events) = context(3)?;
    let kind = descriptor("AWS::ElasticLoadBalancingV2::LoadBalancer")?;
    let arn = "arn:aws:elasticloadbalancing:us-east-1:111111111111:loadbalancer/app/demo/123";
    let body = map_description(
        &ctx,
        &kind,
        arn,
        &description(
            arn,
            json!({"LoadBalancerArn":arn,"Type":"application","Subnets":["subnet-one","subnet-one"],"SubnetMappings":[{"SubnetId":"subnet-one"}]}),
        ),
        true,
    )?;
    assert_eq!(body.edges.len(), 1);
    let edge = body.edges.first().ok_or("missing edge")?;
    assert_eq!(
        edge.to_id,
        "arn:aws:ec2:us-east-1:111111111111:subnet/subnet-one"
    );
    assert_eq!(
        edge.properties.get("owner_resolution"),
        Some(&json!("assumed_scanning_account"))
    );
    assert_eq!(
        edge.properties
            .get("property_paths")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(3)
    );
    let lambda = descriptor("AWS::Lambda::Function")?;
    let body = map_description(
        &ctx,
        &lambda,
        "demo",
        &description(
            "demo",
            json!({"FunctionName":"demo","Role":"arn:aws:iam::222222222222:role/cross-account"}),
        ),
        true,
    )?;
    assert_eq!(
        body.edges.first().ok_or("missing role edge")?.to_id,
        "arn:aws:iam::222222222222:role/cross-account"
    );
    Ok(())
}

#[test]
fn malformed_get_responses_do_not_become_empty_successes() -> TestResult {
    let (ctx, _events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    for output in [
        GetResourceOutput::builder().build(),
        get(ResourceDescription::builder().identifier("i-one").build()),
        get(description("i-other", json!({}))),
        GetResourceOutput::builder()
            .type_name("AWS::EC2::VPC")
            .resource_description(description("i-one", json!({})))
            .build(),
    ] {
        assert!(map_get(&ctx, &kind, "i-one", &output).is_err());
    }
    assert!(sanitized_properties(&kind, Some("[]")).is_err());
    assert!(sanitized_properties(&kind, Some("not-json")).is_err());
    assert!(
        sanitized_properties(
            &kind,
            Some(&" ".repeat(MAX_PROVIDER_BYTES.saturating_add(1)))
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn input_node_depth_and_pointer_expansion_caps_fail_explicitly() -> TestResult {
    let kind = descriptor("AWS::AppConfig::Application")?;
    let many = json!({"Items":vec![0; MAX_JSON_NODES]});
    assert!(sanitized_properties(&kind, Some(&many.to_string())).is_err());
    let mut nested = json!("leaf");
    for _ in 0..MAX_JSON_DEPTH.saturating_add(2) {
        nested = json!({"Nested":nested});
    }
    assert!(sanitized_properties(&kind, Some(&nested.to_string())).is_err());
    let many = json!({"Items":vec![json!({"Id":"one"});MAX_PATH_MATCHES.saturating_add(1)]});
    assert!(expand_pointer(&many, "/Items/*/Id").is_err());
    Ok(())
}

#[test]
fn list_identity_ledger_rejects_replays_before_enrichment_and_does_not_consume_failed_budget()
-> TestResult {
    let kind = descriptor("AWS::EC2::Instance")?;
    let ledger = Mutex::new(IdentityLedger::default());
    let one = ResourceDescription::builder().identifier("i-one").build();
    let duplicate = ListResourcesOutput::builder()
        .resource_descriptions(one.clone())
        .resource_descriptions(one.clone())
        .build();
    assert!(remember_list_page(&kind, &duplicate, &ledger, MAX_CHILDREN).is_err());
    let first = ListResourcesOutput::builder()
        .resource_descriptions(one)
        .build();
    assert!(remember_list_page(&kind, &first, &ledger, 0).is_err());
    remember_list_page(&kind, &first, &ledger, MAX_CHILDREN)?;
    assert!(remember_list_page(&kind, &first, &ledger, MAX_CHILDREN).is_err());
    let invalid = ListResourcesOutput::builder()
        .resource_descriptions(ResourceDescription::builder().build())
        .build();
    assert!(remember_list_page(&kind, &invalid, &ledger, MAX_CHILDREN).is_err());
    assert!(
        remember_list_page(
            &kind,
            &ListResourcesOutput::builder().next_token("").build(),
            &ledger,
            MAX_CHILDREN
        )
        .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn empty_terminal_list_is_complete_and_get_is_not_called() -> TestResult {
    let (ctx, mut events) = context(3)?;
    let kind = descriptor("AWS::EC2::Instance")?;
    let calls = AtomicUsize::new(0);
    collect_type_with(
        &ctx,
        &kind,
        |_| async { Ok(ListResourcesOutput::builder().build()) },
        |_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(ApiError::Permanent("unexpected read".into()))
        },
    )
    .await?;
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let collected = drain(&mut events)?;
    assert_eq!(collected.len(), 1);
    assert!(collected.iter().all(|event| matches!(event, CollectionEvent::Page(page) if page.scope.collector == "cloud_control_list" && page.next_token.is_none() && page.resources.is_empty())));
    Ok(())
}
