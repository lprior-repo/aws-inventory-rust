use super::*;

#[tokio::test]
async fn given_typed_suppression_targets_when_pages_and_field_order_change_then_distinct_targets_keep_stable_durable_identity() -> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iot", "ListAuditSuppressions")?;
    let mut scenario = Scenario::new(&catalog, "typed-audit-suppressions").await?;
    scenario.seed(plan).await?;
    let lease = scenario.claim("ListAuditSuppressions").await?;
    let (request, wire) = scenario.request(plan, &lease)?;
    assert_eq!(wire.uri().path(), "/audit/suppressions/list");
    let first = json!({"suppressions":[
        {"checkName":"CHECK_A","resourceIdentifier":{"deviceCertificateId":"same-id"},"suppressIndefinitely":true},
        {"checkName":"CHECK_A","resourceIdentifier":{"caCertificateId":"same-id"},"suppressIndefinitely":true},
        {"checkName":"CHECK_A","resourceIdentifier":{"policyVersionIdentifier":{"policyName":"policy-a","policyVersionId":"1"}},"description":"original"}
    ],"nextToken":"resume2"});
    scenario.apply(&catalog, plan, lease, &request, &serde_json::to_vec(&first)?).await?;
    scenario = scenario.reopen().await?;
    scenario.assert_nodes("AWS::ApiOnly::IoTAuditSuppression", 3).await?;
    let lease = scenario.claim("ListAuditSuppressions").await?;
    let (request, wire) = scenario.request(plan, &lease)?;
    request_contains(&wire, "resume2")?;
    // Same nested identifier in another wire-member order is the same suppression;
    // mutable description/expiration do not participate in its identity.
    let last = br#"{"suppressions":[{"checkName":"CHECK_A","resourceIdentifier":{"policyVersionIdentifier":{"policyVersionId":"1","policyName":"policy-a"}},"description":"changed","expirationDate":1234},{"checkName":"CHECK_A","resourceIdentifier":{"issuerCertificateIdentifier":{"issuerId":"same-id","issuerCertificateSerialNumber":"serial-a"}}}]}"#;
    scenario.apply(&catalog, plan, lease, &request, last).await?;
    let nodes = scenario.assert_nodes("AWS::ApiOnly::IoTAuditSuppression", 4).await?;
    assert!(nodes.iter().all(|(_, region, _)| region == "us-east-1"));
    assert!(scenario.store.claim(&scenario.sweep, 1000, 10000).await?.is_none());
    Ok(())
}

#[test]
fn given_missing_or_null_suppression_target_when_identity_is_built_then_no_empty_or_fallback_identity_is_fabricated() -> TestResult {
    let catalog = fixture::catalog()?;
    let plan = fixture::plan(&catalog, "iot", "ListAuditSuppressions")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let request = input::root_bindings(plan, Some(&account), Some(&Region::Regional(home.clone())))?;
    for body in [json!({"suppressions":[{"checkName":"CHECK_A"}]}), json!({"suppressions":[{"checkName":"CHECK_A","resourceIdentifier":null}]})] {
        let rejected = fixture::decode(plan, &serde_json::to_vec(&body)?).err().ok_or("missing required suppression target was decoded")?;
        assert!(rejected.to_string().contains("uncorrectable required member"), "{rejected}");
    }
    let response = fixture::decode(plan, &serde_json::to_vec(&json!({"suppressions":[{"checkName":"CHECK_A","resourceIdentifier":{}}]}))?)?;
    let rows = item_records(&response, plan.items())?;
    let row = rows.first().ok_or("suppression row absent")?;
    assert!(resource_id_with_context(row, plan.identity(), Some(IdentityContext { account: &account, region: &home }),
        &inventory_core::inventory::DiscoveryContext::empty(), &request).is_err());
    Ok(())
}
