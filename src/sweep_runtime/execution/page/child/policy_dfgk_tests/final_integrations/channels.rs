use super::*;

fn response(
    plan: &CallablePlan<'_>, lease: &WorkLease, wire: &http::Request<bytes::Bytes>,
    requests: &mut Vec<(String, Option<String>)>,
) -> TestResult<Json> {
    let body: Json = serde_json::from_slice(wire.body())?;
    Ok(match plan.operation_name() {
        "ListChannels" if lease.token().is_none() => {
            assert_wire_token(wire, None, "nextToken")?;
            json!({"channels":[],"nextToken":"rootpage"})
        },
        "ListChannels" => {
            assert_wire_token(wire, Some("rootpage"), "nextToken")?;
            json!({"channels":[{"arn":"arn:aws:ivs:us-east-1:123456789012:channel/one","name":"one"},{"arn":"arn:aws:ivs:us-east-1:123456789012:channel/two","name":"two"}]})
        },
        "ListStreamKeys" => {
            let channel = text(&body, "channelArn")?;
            assert_eq!(Some(channel), lease.scope().parent().map(|parent| parent.id.as_str()));
            let name = channel.rsplit('/').next().ok_or("channel name absent")?;
            requests.push((name.to_owned(), body.get("nextToken").and_then(Json::as_str).map(str::to_owned)));
            let (row, next) = match body.get("nextToken").and_then(Json::as_str) {
                None => (None, Some("keypage")),
                Some("keypage") => (Some("first"), Some("lastpage")),
                Some("lastpage") => (Some("second"), None),
                _ => return Err("unexpected stream-key cursor".into()),
            };
            let rows = row.map_or_else(Vec::new, |suffix| vec![json!({"arn":format!("arn:aws:ivs:us-east-1:123456789012:stream-key/{name}-{suffix}"),"channelArn":channel,"tags":{"env":"test"}})]);
            page("streamKeys", rows, next, "nextToken")
        },
        _ => return Err("forbidden stream detail/secret call scheduled".into()),
    })
}

#[tokio::test]
async fn given_channels_when_empty_and_second_stream_key_pages_resume_then_metadata_keeps_bound_channel_and_never_reads_secrets() -> TestResult {
    // Given two channel owners and the real embedded summary-only plans.
    let catalog = catalog()?;
    let mut scenario = Scenario::new(&catalog, "stream-key-metadata-only").await?;
    let root = fixture::plan(&catalog, "ivs", "ListChannels")?;
    let keys = fixture::plan(&catalog, "ivs", "ListStreamKeys")?;
    assert_eq!(keys.kind().as_str(), "AWS::IVS::StreamKey");
    scenario.seed(root).await?;
    let mut requests = Vec::new();
    // When durable cursors resume empty, second and last pages through actual wire requests.
    for _ in 0..8 {
        let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("expected channel/key page absent")?;
        let plan = fixture::plan(&catalog, "ivs", lease.scope().operation().as_str())?;
        let (request, wire) = scenario.prepared_request(plan, &lease)?;
        let response = response(plan, &lease, &wire, &mut requests)?;
        scenario.apply(&catalog, plan, lease, &request, response).await?;
        scenario = reopen(scenario).await?;
    }
    assert!(scenario.store.claim(&scenario.sweep, 1001, 10000).await?.is_none());
    // Then all four keys and their bound-parent edges are persisted without secret detail work.
    requests.sort();
    assert_eq!(requests, ["one", "two"].into_iter().flat_map(|name| [None, Some("keypage"), Some("lastpage")].map(|token| (name.to_owned(), token.map(str::to_owned)))).collect::<Vec<_>>());
    let saved_keys = saved(&scenario, keys.kind().as_str()).await?;
    assert_eq!(saved_keys.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(), [
        "arn:aws:ivs:us-east-1:123456789012:stream-key/one-first", "arn:aws:ivs:us-east-1:123456789012:stream-key/one-second",
        "arn:aws:ivs:us-east-1:123456789012:stream-key/two-first", "arn:aws:ivs:us-east-1:123456789012:stream-key/two-second",
    ]);
    let edges = scenario.store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 4);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "IN_CHANNEL");
        let channel = edge.to.id.as_str().rsplit('/').next().ok_or("edge channel absent")?;
        assert!(edge.from.id.as_str().contains(&format!("stream-key/{channel}-")));
    }
    Ok(())
}

#[test]
fn given_partition_scoped_summary_arns_when_identity_is_derived_then_foreign_scope_and_malformed_arns_fail_closed() -> TestResult {
    let catalog = catalog()?;
    let account = AccountId::parse("123456789012")?;
    for (service, operation, items, field, resource) in [
        ("ivs", "ListStreamKeys", "streamKeys", "arn", "stream-key/test"),
        ("ivs-realtime", "ListPublicKeys", "publicKeys", "arn", "public-key/test"),
        ("iot-wireless", "ListNetworkAnalyzerConfigurations", "NetworkAnalyzerConfigurationList", "Arn", "NetworkAnalyzerConfiguration/test"),
    ] {
        let plan = fixture::plan(&catalog, service, operation)?;
        let arn_service = if service == "iot-wireless" { "iotwireless" } else { "ivs" };
        for (partition, region) in [("aws", "us-east-1"), ("aws-cn", "cn-north-1"), ("aws-us-gov", "us-gov-west-1")] {
            let region = RegionId::parse(region)?;
            let valid = format!("arn:{partition}:{arn_service}:{}:123456789012:{resource}", region.as_str());
            let body = page(items, vec![json!({(field):valid})], None, "nextToken");
            let response = fixture::decode(plan, &serde_json::to_vec(&body)?)?;
            let selected = record::item_records(&response, plan.items())?;
            let row = selected.first().ok_or("summary row absent")?;
            let identity = record::resource_id(row, plan.identity(), Some(record::IdentityContext { account:&account, region:&region }))?;
            assert_eq!(identity.as_str(), valid);
            for hostile in [
                format!("arn:{partition}:{arn_service}:{}:999999999999:{resource}", region.as_str()),
                format!("arn:{partition}:{arn_service}:eu-west-1:123456789012:{resource}"),
                format!("arn:foreign:{arn_service}:{}:123456789012:{resource}", region.as_str()),
                "not-an-arn".to_owned(),
            ] {
                let response = fixture::decode(plan, &serde_json::to_vec(&page(items, vec![json!({(field):hostile})], None, "nextToken"))?)?;
                let selected = record::item_records(&response, plan.items())?;
                let row = selected.first().ok_or("hostile row absent")?;
                let error = record::resource_id(row, plan.identity(), Some(record::IdentityContext { account:&account, region:&region })).err().ok_or("hostile ARN accepted")?;
                assert!(error.to_string().contains("modeled ARN"));
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_stream_key_summary_when_request_parent_is_forged_then_foreign_scope_cannot_be_repaired_by_output() -> TestResult {
    let catalog = catalog()?;
    let scenario = Scenario::new(&catalog, "stream-key-parent-guard").await?;
    let root = fixture::plan(&catalog, "ivs", "ListChannels")?;
    let child = fixture::plan(&catalog, "ivs", "ListStreamKeys")?;
    let channel = "arn:aws:ivs:us-east-1:123456789012:channel/owner";
    scenario.seed(root).await?;
    let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("channel lease absent")?;
    let (request, _) = scenario.prepared_request(root, &lease)?;
    scenario.apply(&catalog, root, lease, &request, json!({"channels":[{"arn":channel}]})).await?;
    let lease = scenario.store.claim(&scenario.sweep, 1001, 10000).await?.ok_or("stream-key child absent")?;
    assert_eq!(lease.scope().operation().as_str(), "ListStreamKeys");
    let (request, _) = scenario.prepared_request(child, &lease)?;
    let response = fixture::decode(child, &serde_json::to_vec(&json!({"streamKeys":[{
        "arn":"arn:aws:ivs:us-east-1:123456789012:stream-key/metadata",
        "channelArn":channel
    }]}))?)?;
    let ancestry = DiscoveryContext::decode(lease.context())?;
    let context = Context::new(child, &scenario.sweep, &scenario.account, &scenario.home, &scenario.consumers);
    let (observations, _) = project_children(&catalog, &context, lease.scope(), &request, &response, &ancestry)?;
    let rows = record::item_records(&response, child.items())?;
    let rule = catalog.relationships().iter().find(|rule| rule.source_kind == *child.kind()
        && rule.source == inventory_core::catalog::runtime::RelationSource::Request).ok_or("request parent rule absent")?;
    let valid = page_rows::references(&catalog, child, &scenario.home, &rows, &observations, &request)?;
    assert_eq!(valid.len(), 1);
    assert_eq!(valid.first().ok_or("channel relation absent")?.target.value().as_str(), channel);
    for hostile in [
        inventory_core::record::Value::Text("arn:aws:ivs:us-east-1:999999999999:channel/foreign".to_owned()),
        inventory_core::record::Value::Text("arn:aws-cn:ivs:us-east-1:123456789012:channel/foreign".to_owned()),
        inventory_core::record::Value::Text("arn:aws:ivs:eu-west-1:123456789012:channel/foreign".to_owned()),
        inventory_core::record::Value::Text("arn:broken".to_owned()),
        inventory_core::record::Value::Int(7),
    ] {
        let forged = input::merge_input(&request, &[input::InputField { path:rule.source_path.clone(), value:hostile }])?;
        assert!(page_rows::references(&catalog, child, &scenario.home, &rows, &observations, &forged).is_err());
    }
    Ok(())
}
