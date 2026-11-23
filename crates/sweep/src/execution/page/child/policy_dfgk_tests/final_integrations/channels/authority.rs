use super::*;
#[test]
fn given_partition_scoped_summary_arns_when_identity_is_derived_then_foreign_scope_and_malformed_arns_fail_closed()
-> TestResult {
    let catalog = catalog()?;
    let account = AccountId::parse("123456789012")?;
    for (service, operation, items, field, resource) in [
        (
            "ivs",
            "ListStreamKeys",
            "streamKeys",
            "arn",
            "stream-key/test",
        ),
        (
            "ivs-realtime",
            "ListPublicKeys",
            "publicKeys",
            "arn",
            "public-key/test",
        ),
        (
            "iot-wireless",
            "ListNetworkAnalyzerConfigurations",
            "NetworkAnalyzerConfigurationList",
            "Arn",
            "NetworkAnalyzerConfiguration/test",
        ),
    ] {
        let plan = fixture::plan(&catalog, service, operation)?;
        let arn_service = if service == "iot-wireless" {
            "iotwireless"
        } else {
            "ivs"
        };
        for (partition, region) in [
            ("aws", "us-east-1"),
            ("aws-cn", "cn-north-1"),
            ("aws-us-gov", "us-gov-west-1"),
        ] {
            let region = RegionId::parse(region)?;
            let valid = format!(
                "arn:{partition}:{arn_service}:{}:123456789012:{resource}",
                region.as_str()
            );
            let body = page(items, vec![json!({(field):valid})], None, "nextToken");
            let response = fixture::decode(plan, &serde_json::to_vec(&body)?)?;
            let selected = record::item_records(&response, plan.items())?;
            let row = selected.first().ok_or("summary row absent")?;
            let identity = record::resource_id(
                row,
                plan.identity(),
                Some(record::IdentityContext {
                    account: &account,
                    region: &region,
                }),
            )?;
            assert_eq!(identity.as_str(), valid);
            for hostile in [
                format!(
                    "arn:{partition}:{arn_service}:{}:999999999999:{resource}",
                    region.as_str()
                ),
                format!("arn:{partition}:{arn_service}:eu-west-1:123456789012:{resource}"),
                format!(
                    "arn:foreign:{arn_service}:{}:123456789012:{resource}",
                    region.as_str()
                ),
                "not-an-arn".to_owned(),
            ] {
                let response = fixture::decode(
                    plan,
                    &serde_json::to_vec(&page(
                        items,
                        vec![json!({(field):hostile})],
                        None,
                        "nextToken",
                    ))?,
                )?;
                let selected = record::item_records(&response, plan.items())?;
                let row = selected.first().ok_or("hostile row absent")?;
                let error = record::resource_id(
                    row,
                    plan.identity(),
                    Some(record::IdentityContext {
                        account: &account,
                        region: &region,
                    }),
                )
                .err()
                .ok_or("hostile ARN accepted")?;
                assert!(error.to_string().contains("modeled ARN"));
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_stream_key_summary_when_request_parent_is_forged_then_foreign_scope_cannot_be_repaired_by_output()
-> TestResult {
    let catalog = catalog()?;
    let scenario = Scenario::new(&catalog, "stream-key-parent-guard").await?;
    let root = fixture::plan(&catalog, "ivs", "ListChannels")?;
    let child = fixture::plan(&catalog, "ivs", "ListStreamKeys")?;
    let channel = "arn:aws:ivs:us-east-1:123456789012:channel/owner";
    scenario.seed(root).await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10000)
        .await?
        .ok_or("channel lease absent")?;
    let (request, _) = scenario.prepared_request(root, &lease)?;
    scenario
        .apply(
            &catalog,
            root,
            lease,
            &request,
            json!({"channels":[{"arn":channel}]}),
        )
        .await?;
    let lease = scenario
        .store
        .claim(&scenario.sweep, 1001, 10000)
        .await?
        .ok_or("stream-key child absent")?;
    assert_eq!(lease.scope().operation().as_str(), "ListStreamKeys");
    let (request, _) = scenario.prepared_request(child, &lease)?;
    let response = fixture::decode(
        child,
        &serde_json::to_vec(&json!({"streamKeys":[{
            "arn":"arn:aws:ivs:us-east-1:123456789012:stream-key/metadata",
            "channelArn":channel
        }]}))?,
    )?;
    let ancestry = DiscoveryContext::decode(lease.context())?;
    let context = Context::new(
        child,
        &scenario.sweep,
        &scenario.account,
        &scenario.home,
        &scenario.consumers,
    );
    let (observations, _) = project_children(
        &catalog,
        &context,
        lease.scope(),
        &request,
        &response,
        &ancestry,
    )?;
    let rows = record::item_records(&response, child.items())?;
    let rule = catalog
        .relationships()
        .iter()
        .find(|rule| {
            rule.source_kind == *child.kind()
                && rule.source == inventory_core::catalog::runtime::RelationSource::Request
        })
        .ok_or("request parent rule absent")?;
    let valid = page_rows::references(
        &catalog,
        child,
        &scenario.home,
        &rows,
        &observations,
        &request,
        None,
    )?;
    assert_eq!(valid.len(), 1);
    assert_eq!(
        valid
            .first()
            .ok_or("channel relation absent")?
            .target
            .value()
            .as_str(),
        channel
    );
    for hostile in [
        inventory_core::record::Value::Text(
            "arn:aws:ivs:us-east-1:999999999999:channel/foreign".to_owned(),
        ),
        inventory_core::record::Value::Text(
            "arn:aws-cn:ivs:us-east-1:123456789012:channel/foreign".to_owned(),
        ),
        inventory_core::record::Value::Text(
            "arn:aws:ivs:eu-west-1:123456789012:channel/foreign".to_owned(),
        ),
        inventory_core::record::Value::Text("arn:broken".to_owned()),
        inventory_core::record::Value::Int(7),
    ] {
        let forged = input::merge_input(
            &request,
            &[input::InputField {
                path: rule.source_path.clone(),
                value: hostile,
            }],
        )?;
        assert!(
            page_rows::references(
                &catalog,
                child,
                &scenario.home,
                &rows,
                &observations,
                &forged,
                None
            )
            .is_err()
        );
    }
    Ok(())
}
