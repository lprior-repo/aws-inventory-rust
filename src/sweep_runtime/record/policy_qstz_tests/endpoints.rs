use super::super::super::{execution::routing, input};
use super::*;
use inventory_core::inventory::Region;

#[test]
fn given_regional_and_cloudfront_waf_branches_when_requested_then_scope_region_cursors_and_detail_pairs_stay_distinct()
-> TestResult {
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    let list = catalog
        .lookup("wafv2", "ListWebACLs")
        .ok_or("WAF list absent")?;
    let detail = catalog
        .lookup("wafv2", "GetWebACL")
        .ok_or("WAF detail absent")?;
    let tags = catalog
        .lookup("wafv2", "ListTagsForResource")
        .ok_or("WAF tags absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("eu-west-1")?;
    let mut ids = Vec::new();
    for (branch_name, scope, expected_region, resource_scope) in [
        (
            "REGIONAL",
            Region::Regional(RegionId::parse("eu-west-1")?),
            "eu-west-1",
            "regional",
        ),
        (
            "REGIONAL",
            Region::Regional(RegionId::parse("us-west-2")?),
            "us-west-2",
            "regional",
        ),
        ("CLOUDFRONT", Region::Global, "us-east-1", "global"),
    ] {
        let branch = list
            .branches()
            .iter()
            .find(|branch| branch.discriminator == branch_name)
            .ok_or("WAF branch absent")?;
        let region = routing::request_region(list, &scope, &home, Some(branch_name))?;
        assert_eq!(region.as_str(), expected_region);
        let base = routing::branch_bindings(list, branch, &account, &scope)?;
        let endpoint = routing::resolve(list, &region)?;
        let first = super::super::super::request::encode(list, &endpoint.url, &base, &[])?;
        let body: serde_json::Value = serde_json::from_slice(first.body())?;
        assert_eq!(
            body.get("Scope").and_then(serde_json::Value::as_str),
            Some(branch_name)
        );
        assert_eq!(
            first.uri().host(),
            Some(format!("wafv2.{expected_region}.amazonaws.com").as_str())
        );
        let arn = format!(
            "arn:aws:wafv2:{expected_region}:123456789012:{resource_scope}/webacl/shared/same-id"
        );
        let response = decode(
            list,
            &serde_json::to_vec(
                &serde_json::json!({ "WebACLs": [{ "Name": "shared", "Id": "same-id", "ARN": arn }], "NextMarker": "page-two" }),
            )?,
        )?;
        let token = list
            .pagination()
            .tokens
            .first()
            .ok_or("WAF continuation absent")?;
        let value = path_value(response.root(), &token.response_path)?
            .ok_or("WAF next marker absent")?
            .to_value()?;
        let second = super::super::super::request::encode(
            list,
            &endpoint.url,
            &base,
            &[input::InputField {
                path: token.request_path.clone(),
                value,
            }],
        )?;
        let body: serde_json::Value = serde_json::from_slice(second.body())?;
        assert_eq!(
            body.get("Scope").and_then(serde_json::Value::as_str),
            Some(branch_name)
        );
        assert_eq!(
            body.get("NextMarker").and_then(serde_json::Value::as_str),
            Some("page-two")
        );
        let row = item_records(&response, list.items())?
            .into_iter()
            .next()
            .ok_or("WAF summary absent")?;
        ids.push(
            resource_id(&row, list.identity(), None)?
                .as_str()
                .to_owned(),
        );
        let detail_branch = detail
            .branches()
            .iter()
            .find(|branch| branch.discriminator == branch_name)
            .ok_or("WAF detail branch absent")?;
        assert_eq!(detail_branch.discriminator, branch.discriminator);
        let encoded = super::super::super::request::encode(
            detail,
            &endpoint.url,
            &base,
            &child_fields(&catalog, list, detail, &row)?,
        )?;
        let body: serde_json::Value = serde_json::from_slice(encoded.body())?;
        assert_eq!(
            body.get("Scope").and_then(serde_json::Value::as_str),
            Some(branch_name)
        );
        assert_eq!(
            body.get("Name").and_then(serde_json::Value::as_str),
            Some("shared")
        );
        assert_eq!(
            body.get("Id").and_then(serde_json::Value::as_str),
            Some("same-id")
        );
        let tag_region = routing::request_region(tags, &scope, &home, Some(branch_name))?;
        assert_eq!(tag_region, region);
        assert!(
            tags.branches()
                .iter()
                .find(|branch| branch.discriminator == branch_name)
                .ok_or("WAF tag branch absent")?
                .bindings
                .is_empty()
        );
    }
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 3);
    assert_eq!(list.kind().as_str(), "AWS::WAFv2::WebACL");
    assert_eq!(list.kind(), detail.kind());
    assert!(routing::request_region(list, &Region::Global, &home, None).is_err());
    Ok(())
}

#[tokio::test]
async fn given_real_timestream_discovery_wire_when_lease_is_applied_then_query_and_write_hosts_remain_separate()
-> TestResult {
    use inventory_core::catalog::runtime::RequestRouting;
    let catalog = RuntimeCatalog::from_bytes(bytes()?)?;
    assert!(
        catalog
            .lookup("timestream-query", "DescribeScheduledQuery")
            .is_none(),
        "endpoint discovery must not authorize the customer SQL content response"
    );
    for (service, operation, host, response) in [
        ("timestream-write", "ListDatabases", "ingest-cell1.timestream.us-east-1.amazonaws.com", br#"{"Databases":[{"Arn":"arn:aws:timestream:us-east-1:123456789012:database/inventory","DatabaseName":"inventory"}]}"#.as_slice()),
        ("timestream-query", "ListScheduledQueries", "query-cell2.timestream.us-east-1.amazonaws.com", br#"{"ScheduledQueries":[{"Arn":"arn:aws:timestream:us-east-1:123456789012:scheduled-query/inventory","Name":"inventory"}]}"#.as_slice()),
    ] {
        let plan = catalog.lookup(service, operation).ok_or("Timestream inventory absent")?;
        let discovery = catalog.lookup(service, "DescribeEndpoints").ok_or("Timestream discovery absent")?;
        let RequestRouting::Discovered { endpoints_path, address_path, cache_minutes_path, trusted_dns_suffix, .. } = plan.routing() else { return Err("Timestream did not require discovered routing".into()); };
        let wire = serde_json::to_vec(&serde_json::json!({ "Endpoints": [{ "Address": host, "CachePeriodInMinutes": 1 }] }))?;
        let response_lease = decode(discovery, &wire)?;
        let (url, ttl) = super::super::super::execution::discovery::lease(&response_lease, endpoints_path, address_path, cache_minutes_path, &trusted_dns_suffix.replace("{region}", "us-east-1"))?;
        assert_eq!(url, format!("https://{host}"));
        assert_eq!(ttl, std::time::Duration::from_secs(60));
        let account = AccountId::parse("123456789012")?;
        let scope = Region::Regional(RegionId::parse("us-east-1")?);
        let base = input::root_bindings(plan, Some(&account), Some(&scope))?;
        let encoded = super::super::super::request::encode(plan, &url, &base, &[])?;
        assert_eq!(encoded.uri().host(), Some(host));
        let decoded = decode(plan, response)?;
        let observations = observations_with_request(plan, item_records(&decoded, plan.items())?, &base)?;
        let (persisted, _) = persist(plan, vec![(plan, None, base, observations)]).await?;
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted.first().ok_or("Timestream resource absent")?.0, plan.kind().as_str());
    }
    Ok(())
}
