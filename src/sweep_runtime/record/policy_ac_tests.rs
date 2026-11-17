mod archive_rules;
mod children_ab;
mod membership;
mod gateway_audit_batch;
#[path = "../../../crates/catalog-tool/src/overlay/builtin/family_a_c/consumer_fixture.rs"]
mod fixture;
use crate::sweep_runtime::execution::page::cursor;
#[path = "../execution/routing.rs"]
mod routing;

use super::*;
use catalog_tool as fixture_catalog;
use fixture::{TestResult, compile};
use inventory_core::catalog::runtime::{CallablePlan, RuntimeCatalog, RuntimeProtocol};
use serde_json::{Value as Json, json};

fn resource_key(account: &inventory_core::inventory::AccountId, region: &inventory_core::inventory::Region, kind: &str, id: &str) -> TestResult<inventory_core::inventory::ResourceKey> {
    Ok(inventory_core::inventory::ResourceKey {
        account: account.clone(), region: region.clone(),
        kind: inventory_core::inventory::Kind::parse(kind)?,
        id: inventory_core::inventory::ResourceId::parse(id)?,
    })
}

fn decode(plan: &CallablePlan<'_>, body: &Json) -> TestResult<Record> {
    let body = match plan.protocol() {
        RuntimeProtocol::RpcV2Cbor => crate::sweep_runtime::test_cbor::json_to_cbor(body)?,
        _ => serde_json::to_vec(body)?,
    };
    let protocol = match plan.protocol() {
        RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        RuntimeProtocol::RestJson1 => aws_wire::request::Protocol::RestJson1,
        RuntimeProtocol::RpcV2Cbor => aws_wire::request::Protocol::RpcV2Cbor,
        _ => return Err("A-C fixture requires a JSON or CBOR protocol".into()),
    };
    Ok(aws_wire::response::decode(aws_wire::response::ResponseContext {
        service: plan.service(), operation: plan.operation(), protocol, body: &body,
        headers: &http::HeaderMap::new(), status: http::StatusCode::OK,
        limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
    })?)
}

fn only_id(plan: &CallablePlan<'_>, body: &Json) -> TestResult<ResourceId> {
    let decoded = decode(plan, body)?;
    let rows = item_records(&decoded, plan.items())?;
    assert_eq!(rows.len(), 1);
    Ok(resource_id(rows.first().ok_or("selected row missing")?, plan.identity(), None)?)
}

#[test]
fn given_eight_real_paged_roots_when_resumed_then_wire_requests_and_terminal_pages_are_consumed() -> TestResult {
    // Given: actual embedded policies checked against pinned Smithy operations, not invented plans.
    let groups: &[(&str, &str, &[(&str, &str, &str, &str)])] = &[
        ("accessanalyzer", "2019-11-01", &[("ListAnalyzers", "analyzers", "arn", "nextToken")]),
        ("backupsearch", "2018-05-10", &[("ListSearchJobs", "SearchJobs", "SearchJobArn", "NextToken")]),
        ("backup-gateway", "2021-01-01", &[("ListGateways", "Gateways", "GatewayArn", "NextToken"),
            ("ListHypervisors", "Hypervisors", "HypervisorArn", "NextToken"),
            ("ListVirtualMachines", "VirtualMachines", "ResourceArn", "NextToken")]),
        ("cleanrooms", "2022-02-17", &[("ListCollaborations", "collaborationList", "arn", "nextToken"),
            ("ListConfiguredTables", "configuredTableSummaries", "arn", "nextToken"),
            ("ListMemberships", "membershipSummaries", "arn", "nextToken")]),
    ];
    for (service, date, cases) in groups {
        let operations = cases.iter().map(|(operation, _, _, _)| *operation).collect::<Vec<_>>();
        let compiled = compile(service, date, &operations)?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        for (operation, items, identity, token) in *cases {
            let plan = catalog.lookup(service, operation).ok_or("paged plan missing")?;
            let first_id = "arn:aws:fixture:us-east-1:123456789012:resource/first";
            let last_id = "arn:aws:fixture:us-east-1:123456789012:resource/last";
            let first = json!({(*items):[{(*identity):first_id}],(*token):"resume-token"});
            let last = json!({(*items):[{(*identity):last_id}]});
            assert_eq!(only_id(plan, &first)?.as_str(), first_id, "{service}.{operation}");
            assert_eq!(only_id(plan, &last)?.as_str(), last_id, "{service}.{operation}");
            // When: the production decoder extracts a continuation and the durable checkpoint is restored.
            let first = decode(plan, &first)?;
            let advance = cursor::Cursor::default().next(plan.pagination(), &first, &http::HeaderMap::new(), 10)?
                .ok_or("first page was incorrectly declared terminal")?;
            let restored = cursor::Cursor::decode(Some(advance.cursor.as_bytes()))?;
            assert_eq!(restored.pages, 1);
            assert!(!restored.partial);
            let fields = restored.request_fields(plan.pagination())?;
            let cursor_field = fields.first().ok_or("resumed cursor field missing")?;
            assert_eq!(cursor_field.path.as_ref(), &[compiled.fields.id(token).ok_or("modeled cursor field missing")?]);
            assert!(matches!(&cursor_field.value, inventory_core::record::Value::Text(value) if value == "resume-token"));
            assert_eq!(fields.len(), 1, "{service}.{operation}");
            // Then: actual request serialization carries the exact opaque token in query/body placement.
            let base = crate::sweep_runtime::input::root_bindings(plan, None, None)?;
            let request = crate::sweep_runtime::request::encode(plan, "https://fixture.example.test", &base, &fields)?;
            assert!(request.uri().to_string().contains("resume-token") || request.body().windows(12).any(|window| window == b"resume-token"), "{service}.{operation}");
            let last = decode(plan, &last)?;
            assert!(restored.next(plan.pagination(), &last, &http::HeaderMap::new(), 10)?.is_none(), "{service}.{operation}");
            // Wrongly typed cursors cannot be silently accepted as a terminal successful page.
            assert!(decode(plan, &json!({(*items):[],(*token):42})).is_err(), "{service}.{operation}");
        }
    }
    Ok(())
}

#[test]
fn given_real_list_and_detail_responses_when_selected_then_three_views_converge_on_one_identity() -> TestResult {
    let cases = [
        ("apigatewayv2", "2018-11-29", "GetDomainNames", "GetDomainName",
            json!({"items":[{"domainName":"api.example.test","domainNameArn":"arn:aws:apigateway:us-east-1::/domainnames/api.example.test"}]}),
            json!({"domainName":"api.example.test","domainNameArn":"arn:aws:apigateway:us-east-1::/domainnames/api.example.test"}),
            "arn:aws:apigateway:us-east-1::/domainnames/api.example.test"),
        ("appconfig", "2019-10-09", "ListExtensionAssociations", "GetExtensionAssociation",
            json!({"Items":[{"Id":"abc1234","ExtensionArn":"arn:aws:appconfig:us-east-1:123456789012:extension/ex1"}]}),
            json!({"Id":"abc1234","Arn":"arn:aws:appconfig:us-east-1:123456789012:extensionassociation/abc1234"}), "abc1234"),
        ("cloudwatch-events", "2015-10-07", "ListArchives", "DescribeArchive",
            json!({"Archives":[{"ArchiveName":"nightly","EventSourceArn":"arn:aws:events:us-east-1:123456789012:event-bus/main"}]}),
            json!({"ArchiveName":"nightly","ArchiveArn":"arn:aws:events:us-east-1:123456789012:archive/nightly"}), "nightly"),
    ];
    for (service, date, list, detail, list_body, detail_body, expected) in cases {
        // Given/When: real wire field names are decoded and selected with compiled item selectors.
        let compiled = compile(service, date, &[list, detail])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let list_plan = catalog.lookup(service, list).ok_or("list plan missing")?;
        let detail_plan = catalog.lookup(service, detail).ok_or("detail plan missing")?;
        let list_id = only_id(list_plan, &list_body)?;
        let detail_id = only_id(detail_plan, &detail_body)?;
        // Then: neither alternate detail ARN nor list display name manufactures a second node.
        assert_eq!(list_id, detail_id, "{service}");
        assert_eq!(list_id.as_str(), expected);
        assert_eq!(list_plan.kind(), detail_plan.kind());
        let malformed = decode(detail_plan, &json!({}))?;
        let rows = item_records(&malformed, detail_plan.items())?;
        assert!(resource_id(rows.first().ok_or("detail root absent")?, detail_plan.identity(), None).is_err());
    }
    Ok(())
}

#[test]
fn given_child_baselines_and_application_profiles_when_requested_then_the_real_wire_filters_are_complete() -> TestResult {
    let cases = [
        ("controltower", "2018-05-10", "ListEnabledBaselines", "includeChildren", json!(true),
            json!({"enabledBaselines":[{"arn":"arn:aws:controltower:us-east-1:123456789012:enabledbaseline/child"}]}),
            "arn:aws:controltower:us-east-1:123456789012:enabledbaseline/child"),
        ("bedrock", "2023-04-20", "ListInferenceProfiles", "type", json!("APPLICATION"),
            json!({"inferenceProfileSummaries":[{"inferenceProfileArn":"arn:aws:bedrock:us-east-1:123456789012:application-inference-profile/profile1"}]}),
            "arn:aws:bedrock:us-east-1:123456789012:application-inference-profile/profile1"),
    ];
    for (service, date, operation, field, expected, response, id) in cases {
        let compiled = compile(service, date, &[operation])?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        let plan = catalog.lookup(service, operation).ok_or("filtered plan missing")?;
        let base = crate::sweep_runtime::input::root_bindings(plan, None, None)?;
        let request = crate::sweep_runtime::request::encode(plan, "https://fixture.example.test", &base, &[])?;
        let actual: Json = if request.body().is_empty() {
            let query = request.uri().query().ok_or("fixed query filter absent")?;
            let expected_pair = format!("{field}={}", expected.as_str().ok_or("query string expected")?);
            assert!(query.split('&').any(|pair| pair == expected_pair));
            expected.clone()
        } else {
            serde_json::from_slice::<Json>(request.body())?.get(field).ok_or("fixed body filter absent")?.clone()
        };
        assert_eq!(actual, expected);
        assert_eq!(only_id(plan, &response)?.as_str(), id);
    }
    Ok(())
}

#[test]
fn given_portal_inline_tags_when_wire_decoded_then_the_owner_receives_complete_tags() -> TestResult {
    use inventory_core::inventory::{ResourceKey, TagCoverage};
    let compiled = compile("apigatewayv2", "2018-11-29", &["ListPortals"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("apigatewayv2", "ListPortals").ok_or("portal plan missing")?;
    let response = decode(plan, &json!({"items":[{"portalId":"portal12345","portalArn":"arn:aws:apigateway:us-east-1:123456789012:/portals/portal12345","tags":{"Owner":"inventory"}}]}))?;
    let rows = item_records(&response, plan.items())?;
    let row = rows.first().ok_or("portal row missing")?;
    let identity = resource_id(row, plan.identity(), None)?;
    let owner = ResourceKey { account: AccountId::parse("123456789012")?, region: inventory_core::inventory::Region::parse("us-east-1")?, kind: plan.kind().clone(), id: identity };
    let tags = tag_observation(row, &owner, plan.tags().ok_or("inline tag capability missing")?)?;
    assert_eq!(tags.coverage(), TagCoverage::Complete);
    assert_eq!(tags.tags().len(), 1);
    let tag = tags.tags().first().ok_or("inline tag missing")?;
    assert_eq!((tag.key(), tag.value()), ("Owner", "inventory"));
    Ok(())
}

#[test]
fn given_certificate_tag_only_response_when_extracted_then_tags_keep_the_trusted_parent_identity() -> TestResult {
    use inventory_core::{catalog::runtime::PlanPurpose, inventory::{Region, ResourceKey, TagCoverage}};
    let compiled = compile("acm", "2015-12-08", &["ListCertificates", "ListTagsForCertificate"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let parent = catalog.lookup("acm", "ListCertificates").ok_or("certificate list missing")?;
    let child = catalog.lookup("acm", "ListTagsForCertificate").ok_or("certificate tags missing")?;
    let PlanPurpose::TagEndpoint(endpoint) = child.purpose() else { return Err("tag-only endpoint has enumeration purpose".into()); };
    assert_eq!(child.kind(), parent.kind());
    let arn = "arn:aws:acm:us-east-1:123456789012:certificate/00000000-0000-0000-0000-000000000001";
    let decoded = decode(parent, &json!({"CertificateSummaryList":[{"CertificateArn":arn,"DomainName":"api.example.test"}]}))?;
    let rows = item_records(&decoded, parent.items())?;
    let row = rows.first().ok_or("certificate parent missing")?;
    let owner = ResourceKey { account: AccountId::parse("123456789012")?, region: Region::parse("us-east-1")?,
        kind: parent.kind().clone(), id: resource_id(row, parent.identity(), None)? };
    let binding = endpoint.parents.first().ok_or("tag parent capability missing")?.request.as_ref().ok_or("tag ARN binding missing")?;
    let value = path_value(row.root(), &binding.parent)?.ok_or("parent ARN absent")?.to_value()?;
    let base = crate::sweep_runtime::input::root_bindings(child, None, None)?;
    let request = crate::sweep_runtime::request::encode(child, "https://fixture.example.test", &base, &[
        crate::sweep_runtime::input::InputField { path: binding.input.clone(), value },
    ])?;
    assert_eq!(serde_json::from_slice::<Json>(request.body())?.get("CertificateArn"), Some(&json!(arn)));
    // No ARN appears in the response. Production tag extraction must attach to durable parent ownership.
    let response = decode(child, &json!({"Tags":[{"Key":"Owner","Value":"inventory"}]}))?;
    let tags = tag_observation(&response, &owner, &endpoint.tags)?;
    assert_eq!(tags.resource(), &owner);
    assert_eq!(tags.coverage(), TagCoverage::Complete);
    assert_eq!(tags.tags().iter().map(|tag| (tag.key(), tag.value())).collect::<Vec<_>>(), [("Owner", "inventory")]);
    let empty = tag_observation(&decode(child, &json!({"Tags":[]}))?, &owner, &endpoint.tags)?;
    assert_eq!(empty.coverage(), TagCoverage::Complete);
    assert!(empty.tags().is_empty());
    let absent = tag_observation(&decode(child, &json!({}))?, &owner, &endpoint.tags)?;
    assert_eq!(absent.coverage(), TagCoverage::Unobserved);
    // A second parent's response cannot inherit the first parent's key from the payload.
    let other_owner = ResourceKey { id: ResourceId::parse("arn:aws:acm:us-east-1:123456789012:certificate/00000000-0000-0000-0000-000000000002")?, ..owner.clone() };
    assert_eq!(tag_observation(&response, &other_owner, &endpoint.tags)?.resource(), &other_owner);
    assert_ne!(tags.resource(), &other_owner);
    Ok(())
}

#[test]
fn given_account_and_resource_log_policies_when_branched_then_scoped_keys_and_cursors_cannot_collide() -> TestResult {
    use inventory_core::inventory::{DiscoveryContext, Region};
    let compiled = compile("cloudwatch-logs", "2014-03-28", &["DescribeResourcePolicies"])?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let plan = catalog.lookup("cloudwatch-logs", "DescribeResourcePolicies").ok_or("log policy plan missing")?;
    assert_eq!(plan.branches().len(), 2);
    let account = AccountId::parse("123456789012")?;
    let region = Region::parse("us-east-1")?;
    let context = DiscoveryContext::empty();
    let mut identities = Vec::new();
    let mut checkpoints = Vec::new();
    for scope in ["ACCOUNT", "RESOURCE"] {
        let branch = plan.branches().iter().find(|branch| branch.discriminator == scope).ok_or("reviewed policy branch missing")?;
        let request = routing::branch_bindings(plan, branch, &account, &region)?;
        let request_region = routing::request_region(plan, &region, &RegionId::parse("us-east-1")?, Some(scope))?;
        assert_eq!(request_region.as_str(), "us-east-1");
        assert_eq!(routing::resolve(plan, &request_region)?.url.parse::<http::Uri>()?.host(), Some("fixture.example.test"));
        let wire = crate::sweep_runtime::request::encode(plan, "https://fixture.example.test", &request, &[])?;
        assert_eq!(serde_json::from_slice::<Json>(wire.body())?.get("policyScope"), Some(&json!(scope)));
        let arn = "arn:aws:logs:us-east-1:123456789012:log-group:one";
        let row = if scope == "ACCOUNT" { json!({"policyName":"shared","policyScope":scope}) }
            else { json!({"policyName":"shared","policyScope":scope,"resourceArn":arn}) };
        let response = decode(plan, &json!({"resourcePolicies":[row],"nextToken":format!("{scope}-cursor")}))?;
        let rows = item_records(&response, plan.items())?;
        let row = rows.first().ok_or("log policy row missing")?;
        let id = resource_id_with_context(row, plan.identity(), None, &context, &request)?;
        assert!(resource_id(row, plan.identity(), None).is_err(), "request-dependent identity must require trusted request");
        if scope == "ACCOUNT" { assert_eq!(id.as_str(), "ACCOUNT:shared"); }
        else {
            assert!(id.as_str().contains(arn));
            let malformed = decode(plan, &json!({"resourcePolicies":[{"policyName":"shared","policyScope":"RESOURCE"}]}))?;
            let rows = item_records(&malformed, plan.items())?;
            assert!(resource_id_with_context(rows.first().ok_or("malformed policy row missing")?, plan.identity(), None, &context, &request).is_err());
            let other = decode(plan, &json!({"resourcePolicies":[{"policyName":"shared","resourceArn":"arn:aws:logs:us-east-1:123456789012:log-group:two"}]}))?;
            let rows = item_records(&other, plan.items())?;
            assert_ne!(resource_id_with_context(rows.first().ok_or("second policy row missing")?, plan.identity(), None, &context, &request)?, id);
        }
        identities.push(id);
        let next = cursor::Cursor::default().next(plan.pagination(), &response, &http::HeaderMap::new(), 10)?.ok_or("scoped page cursor missing")?;
        let resumed = cursor::Cursor::decode(Some(next.cursor.as_bytes()))?;
        let fields = resumed.request_fields(plan.pagination())?;
        let wire = crate::sweep_runtime::request::encode(plan, "https://fixture.example.test", &request, &fields)?;
        let body: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(body.get("policyScope"), Some(&json!(scope)));
        assert_eq!(body.get("nextToken"), Some(&json!(format!("{scope}-cursor"))));
        checkpoints.push(next.cursor);
    }
    assert_ne!(identities.first(), identities.last());
    assert_ne!(checkpoints.first(), checkpoints.last());
    Ok(())
}
