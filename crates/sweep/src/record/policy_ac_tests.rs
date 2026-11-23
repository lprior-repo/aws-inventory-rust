mod archive_rules;
mod children_ab;
use crate::consumer_fixture as fixture;
mod gateway_audit_batch;
mod membership;
mod views;
mod tags_and_policies;
use crate::execution::page::cursor;
use crate::execution::routing;

use super::*;
use fixture::{TestResult, compile};
use inventory_core::catalog::runtime::{CallablePlan, RuntimeCatalog, RuntimeProtocol};
use serde_json::{Value as Json, json};

fn resource_key(
    account: &inventory_core::inventory::AccountId,
    region: &inventory_core::inventory::Region,
    kind: &str,
    id: &str,
) -> TestResult<inventory_core::inventory::ResourceKey> {
    Ok(inventory_core::inventory::ResourceKey {
        account: account.clone(),
        region: region.clone(),
        kind: inventory_core::inventory::Kind::parse(kind)?,
        id: inventory_core::inventory::ResourceId::parse(id)?,
    })
}

fn decode(plan: &CallablePlan<'_>, body: &Json) -> TestResult<Record> {
    let body = match plan.protocol() {
        RuntimeProtocol::RpcV2Cbor => crate::test_cbor::json_to_cbor(body)?,
        _ => serde_json::to_vec(body)?,
    };
    let protocol = match plan.protocol() {
        RuntimeProtocol::AwsJson10 => aws_wire::request::Protocol::AwsJson10,
        RuntimeProtocol::AwsJson11 => aws_wire::request::Protocol::AwsJson11,
        RuntimeProtocol::RestJson1 => aws_wire::request::Protocol::RestJson1,
        RuntimeProtocol::RpcV2Cbor => aws_wire::request::Protocol::RpcV2Cbor,
        _ => return Err("A-C fixture requires a JSON or CBOR protocol".into()),
    };
    Ok(aws_wire::response::decode(
        aws_wire::response::ResponseContext {
            service: plan.service(),
            operation: plan.operation(),
            protocol,
            body: &body,
            headers: &http::HeaderMap::new(),
            status: http::StatusCode::OK,
            limits: aws_wire::response::DecodeLimits::default().preserve_field_presence(),
        },
    )?)
}

fn only_id(plan: &CallablePlan<'_>, body: &Json) -> TestResult<ResourceId> {
    let decoded = decode(plan, body)?;
    let rows = item_records(&decoded, plan.items())?;
    assert_eq!(rows.len(), 1);
    Ok(resource_id(
        rows.first().ok_or("selected row missing")?,
        plan.identity(),
        None,
    )?)
}

type RootCase = (&'static str, &'static str, &'static str, &'static str);

#[test]
fn given_eight_real_paged_roots_when_resumed_then_wire_requests_and_terminal_pages_are_consumed()
-> TestResult {
    // Given: actual embedded policies checked against pinned Smithy operations, not invented plans.
    let groups: &[(&str, &str, &[RootCase])] = &[
        (
            "accessanalyzer",
            "2019-11-01",
            &[("ListAnalyzers", "analyzers", "arn", "nextToken")],
        ),
        (
            "backupsearch",
            "2018-05-10",
            &[("ListSearchJobs", "SearchJobs", "SearchJobArn", "NextToken")],
        ),
        (
            "backup-gateway",
            "2021-01-01",
            &[
                ("ListGateways", "Gateways", "GatewayArn", "NextToken"),
                (
                    "ListHypervisors",
                    "Hypervisors",
                    "HypervisorArn",
                    "NextToken",
                ),
                (
                    "ListVirtualMachines",
                    "VirtualMachines",
                    "ResourceArn",
                    "NextToken",
                ),
            ],
        ),
        (
            "cleanrooms",
            "2022-02-17",
            &[
                (
                    "ListCollaborations",
                    "collaborationList",
                    "arn",
                    "nextToken",
                ),
                (
                    "ListConfiguredTables",
                    "configuredTableSummaries",
                    "arn",
                    "nextToken",
                ),
                ("ListMemberships", "membershipSummaries", "arn", "nextToken"),
            ],
        ),
    ];
    for (service, date, cases) in groups {
        let operations = cases
            .iter()
            .map(|(operation, _, _, _)| *operation)
            .collect::<Vec<_>>();
        let compiled = compile(service, date, &operations)?;
        let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
        for (operation, items, identity, token) in *cases {
            let plan = catalog
                .lookup(service, operation)
                .ok_or("paged plan missing")?;
            let first_id = "arn:aws:fixture:us-east-1:123456789012:resource/first";
            let last_id = "arn:aws:fixture:us-east-1:123456789012:resource/last";
            let first = json!({(*items):[{(*identity):first_id}],(*token):"resume-token"});
            let last = json!({(*items):[{(*identity):last_id}]});
            assert_eq!(
                only_id(plan, &first)?.as_str(),
                first_id,
                "{service}.{operation}"
            );
            assert_eq!(
                only_id(plan, &last)?.as_str(),
                last_id,
                "{service}.{operation}"
            );
            // When: the production decoder extracts a continuation and the durable checkpoint is restored.
            let first = decode(plan, &first)?;
            let advance = cursor::Cursor::default()
                .next(plan.pagination(), &first, &http::HeaderMap::new(), 10)?
                .ok_or("first page was incorrectly declared terminal")?;
            let restored = cursor::Cursor::decode(Some(advance.cursor.as_bytes()))?;
            assert_eq!(restored.pages, 1);
            assert!(!restored.partial);
            let fields = restored.request_fields(plan.pagination())?;
            let cursor_field = fields.first().ok_or("resumed cursor field missing")?;
            assert_eq!(
                cursor_field.path.as_ref(),
                &[compiled
                    .fields
                    .id(token)
                    .ok_or("modeled cursor field missing")?]
            );
            assert!(
                matches!(&cursor_field.value, inventory_core::record::Value::Text(value) if value == "resume-token")
            );
            assert_eq!(fields.len(), 1, "{service}.{operation}");
            // Then: actual request serialization carries the exact opaque token in query/body placement.
            let base = crate::input::root_bindings(plan, None, None)?;
            let request =
                crate::request::encode(plan, "https://fixture.example.test", &base, &fields)?;
            assert!(
                request.uri().to_string().contains("resume-token")
                    || request
                        .body()
                        .windows(12)
                        .any(|window| window == b"resume-token"),
                "{service}.{operation}"
            );
            let last = decode(plan, &last)?;
            assert!(
                restored
                    .next(plan.pagination(), &last, &http::HeaderMap::new(), 10)?
                    .is_none(),
                "{service}.{operation}"
            );
            // Wrongly typed cursors cannot be silently accepted as a terminal successful page.
            assert!(
                decode(plan, &json!({(*items):[],(*token):42})).is_err(),
                "{service}.{operation}"
            );
        }
    }
    Ok(())
}


