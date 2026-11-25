use super::*;
#[test]
fn given_portal_inline_tags_when_wire_decoded_then_the_owner_receives_complete_tags() -> TestResult
{
    use inventory_core::inventory::{ResourceKey, TagCoverage};
    let compiled = compile("apigatewayv2", "2018-11-29", &["ListPortals"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("apigatewayv2", "ListPortals")
        .ok_or("portal plan missing")?;
    let response = decode(
        plan,
        &json!({"items":[{"portalId":"portal12345","portalArn":"arn:aws:apigateway:us-east-1:123456789012:/portals/portal12345","tags":{"Owner":"inventory"}}]}),
    )?;
    let rows = item_records(&response, plan.items())?;
    let row = rows.first().ok_or("portal row missing")?;
    let identity = resource_id(row, plan.identity(), None)?;
    let owner = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: inventory_core::inventory::Region::parse("us-east-1")?,
        kind: plan.kind().clone(),
        id: identity,
    };
    let tags = tag_observation(
        row,
        &owner,
        plan.tags().ok_or("inline tag capability missing")?,
    )?;
    assert_eq!(tags.coverage(), TagCoverage::Complete);
    assert_eq!(tags.tags().len(), 1);
    let tag = tags.tags().first().ok_or("inline tag missing")?;
    assert_eq!((tag.key(), tag.value()), ("Owner", "inventory"));
    Ok(())
}

#[test]
fn given_certificate_tag_only_response_when_extracted_then_tags_keep_the_trusted_parent_identity()
-> TestResult {
    use inventory_core::{
        catalog::runtime::PlanPurpose,
        inventory::{Region, ResourceKey, TagCoverage},
    };
    let compiled = compile(
        "acm",
        "2015-12-08",
        &["ListCertificates", "ListTagsForCertificate"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let parent = catalog
        .lookup("acm", "ListCertificates")
        .ok_or("certificate list missing")?;
    let child = catalog
        .lookup("acm", "ListTagsForCertificate")
        .ok_or("certificate tags missing")?;
    let PlanPurpose::TagEndpoint(endpoint) = child.purpose() else {
        return Err("tag-only endpoint has enumeration purpose".into());
    };
    assert_eq!(child.kind(), parent.kind());
    let arn = "arn:aws:acm:us-east-1:123456789012:certificate/00000000-0000-0000-0000-000000000001";
    let decoded = decode(
        parent,
        &json!({"CertificateSummaryList":[{"CertificateArn":arn,"DomainName":"api.example.test"}]}),
    )?;
    let rows = item_records(&decoded, parent.items())?;
    let row = rows.first().ok_or("certificate parent missing")?;
    let owner = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: parent.kind().clone(),
        id: resource_id(row, parent.identity(), None)?,
    };
    let binding = endpoint
        .parents
        .first()
        .ok_or("tag parent capability missing")?
        .request
        .as_ref()
        .ok_or("tag ARN binding missing")?;
    let value = path_value(row.root(), &binding.parent)?
        .ok_or("parent ARN absent")?
        .to_value()?;
    let base = crate::input::root_bindings(child, None, None)?;
    let request = crate::request::encode(
        child,
        "https://fixture.example.test",
        &base,
        &[crate::input::InputField {
            path: binding.input.clone(),
            value,
        }],
    )?;
    assert_eq!(
        serde_json::from_slice::<Json>(request.body())?.get("CertificateArn"),
        Some(&json!(arn))
    );
    let response = decode(
        child,
        &json!({"Tags":[{"Key":"Owner","Value":"inventory"}]}),
    )?;
    let tags = tag_observation(&response, &owner, &endpoint.tags)?;
    assert_eq!(tags.resource(), &owner);
    assert_eq!(tags.coverage(), TagCoverage::Complete);
    assert_eq!(
        tags.tags()
            .iter()
            .map(|tag| (tag.key(), tag.value()))
            .collect::<Vec<_>>(),
        [("Owner", "inventory")]
    );
    let empty = tag_observation(&decode(child, &json!({"Tags":[]}))?, &owner, &endpoint.tags)?;
    assert_eq!(empty.coverage(), TagCoverage::Complete);
    assert!(empty.tags().is_empty());
    let absent = tag_observation(&decode(child, &json!({}))?, &owner, &endpoint.tags)?;
    assert_eq!(absent.coverage(), TagCoverage::Unobserved);
    let other_owner = ResourceKey {
        id: ResourceId::parse(
            "arn:aws:acm:us-east-1:123456789012:certificate/00000000-0000-0000-0000-000000000002",
        )?,
        ..owner.clone()
    };
    assert_eq!(
        tag_observation(&response, &other_owner, &endpoint.tags)?.resource(),
        &other_owner
    );
    assert_ne!(tags.resource(), &other_owner);
    Ok(())
}

#[test]
fn given_account_and_resource_log_policies_when_branched_then_scoped_keys_and_cursors_cannot_collide()
-> TestResult {
    use inventory_core::inventory::{DiscoveryContext, Region};
    let compiled = compile(
        "cloudwatch-logs",
        "2014-03-28",
        &["DescribeResourcePolicies"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog
        .lookup("cloudwatch-logs", "DescribeResourcePolicies")
        .ok_or("log policy plan missing")?;
    assert_eq!(plan.branches().len(), 2);
    let account = AccountId::parse("123456789012")?;
    let region = Region::parse("us-east-1")?;
    let context = DiscoveryContext::empty();
    let mut identities = Vec::new();
    let mut checkpoints = Vec::new();
    for scope in ["ACCOUNT", "RESOURCE"] {
        let branch = plan
            .branches()
            .iter()
            .find(|branch| branch.discriminator == scope)
            .ok_or("reviewed policy branch missing")?;
        let request = routing::branch_bindings(plan, branch, &account, &region)?;
        let request_region =
            routing::request_region(plan, &region, &RegionId::parse("us-east-1")?, Some(scope))?;
        assert_eq!(request_region.as_str(), "us-east-1");
        assert_eq!(
            routing::resolve(plan, &request_region)?
                .url
                .parse::<http::Uri>()?
                .host(),
            Some("fixture.example.test")
        );
        let wire = crate::request::encode(plan, "https://fixture.example.test", &request, &[])?;
        assert_eq!(
            serde_json::from_slice::<Json>(wire.body())?.get("policyScope"),
            Some(&json!(scope))
        );
        let arn = "arn:aws:logs:us-east-1:123456789012:log-group:one";
        let row = if scope == "ACCOUNT" {
            json!({"policyName":"shared","policyScope":scope})
        } else {
            json!({"policyName":"shared","policyScope":scope,"resourceArn":arn})
        };
        let response = decode(
            plan,
            &json!({"resourcePolicies":[row],"nextToken":format!("{scope}-cursor")}),
        )?;
        let rows = item_records(&response, plan.items())?;
        let row = rows.first().ok_or("log policy row missing")?;
        let id = resource_id_with_context(row, plan.identity(), None, &context, &request)?;
        assert!(
            resource_id(row, plan.identity(), None).is_err(),
            "request-dependent identity must require trusted request"
        );
        if scope == "ACCOUNT" {
            assert_eq!(id.as_str(), "ACCOUNT:shared");
        } else {
            assert!(id.as_str().contains(arn));
            let malformed = decode(
                plan,
                &json!({"resourcePolicies":[{"policyName":"shared","policyScope":"RESOURCE"}]}),
            )?;
            let rows = item_records(&malformed, plan.items())?;
            assert!(
                resource_id_with_context(
                    rows.first().ok_or("malformed policy row missing")?,
                    plan.identity(),
                    None,
                    &context,
                    &request
                )
                .is_err()
            );
            let other = decode(
                plan,
                &json!({"resourcePolicies":[{"policyName":"shared","resourceArn":"arn:aws:logs:us-east-1:123456789012:log-group:two"}]}),
            )?;
            let rows = item_records(&other, plan.items())?;
            assert_ne!(
                resource_id_with_context(
                    rows.first().ok_or("second policy row missing")?,
                    plan.identity(),
                    None,
                    &context,
                    &request
                )?,
                id
            );
        }
        identities.push(id);
        let next = cursor::Cursor::default()
            .next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?
            .ok_or("scoped page cursor missing")?;
        let resumed = cursor::Cursor::decode(Some(next.cursor.as_bytes()))?;
        let fields = resumed.request_fields(plan.pagination())?;
        let wire = crate::request::encode(plan, "https://fixture.example.test", &request, &fields)?;
        let body: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(body.get("policyScope"), Some(&json!(scope)));
        assert_eq!(
            body.get("nextToken"),
            Some(&json!(format!("{scope}-cursor")))
        );
        checkpoints.push(next.cursor);
    }
    assert_ne!(identities.first(), identities.last());
    assert_ne!(checkpoints.first(), checkpoints.last());
    Ok(())
}
