use crate::consumer_fixture as native;
use crate::test_bindings as synthetic;
mod native_pages;
mod persistence;

use super::*;
use crate::{execution::routing, record::policy_dfgk_tests::fixture::decode};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde_json::{Value as Json, json};
use synthetic::TestResult;

#[test]
fn given_region_list_when_roots_are_routed_then_wire_contains_exactly_the_actual_request_region()
-> TestResult {
    let bytes = synthetic::compiled(1, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("bindings", "ListParents0")
        .ok_or("root missing")?;
    assert!(!input::requires_parent(plan));
    let home = RegionId::parse("us-east-1")?;
    for region in [
        Region::Global,
        Region::Regional(RegionId::parse("us-west-2")?),
    ] {
        let routed = routing::request_region(plan, &region, &home, None)?;
        let base = input::root_bindings(plan, None, Some(&Region::Regional(routed.clone())))?;
        let wire = crate::request::encode(plan, "https://bindings.test", &base, &[])?;
        let actual: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(actual.get("Regions"), Some(&json!([routed.as_str()])));
        assert_eq!(actual.as_object().ok_or("request not object")?.len(), 2);
    }
    assert!(matches!(
        input::root_bindings(plan, None, None),
        Err(SweepError::Operation(
            "region list binding requires the routed request region"
        ))
    ));
    assert!(matches!(
        input::root_bindings(plan, None, Some(&Region::Global)),
        Err(SweepError::Operation(
            "region list binding requires the routed request region"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_eight_alternative_producers_when_current_rows_are_projected_then_each_uses_its_same_actual_request()
-> TestResult {
    persistence::synthetic(8).await
}

#[tokio::test]
async fn given_lattice_service_or_network_when_projected_then_both_exact_arns_reach_the_native_resource_identifier()
-> TestResult {
    let compiled = native::compile(
        "vpc-lattice",
        "2022-11-30",
        &[
            "ListServices",
            "ListServiceNetworks",
            "ListAccessLogSubscriptions",
        ],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    assert!(compiled.fields.id("resourceIdentifier").is_some());
    native_pages::run(&catalog).await
}

#[tokio::test]
async fn given_workmail_users_groups_and_resources_when_projected_then_alias_group_and_mailbox_bindings_do_not_cross_parents()
-> TestResult {
    let compiled = native::compile(
        "workmail",
        "2017-10-01",
        &[
            "ListOrganizations",
            "ListUsers",
            "ListGroups",
            "ListResources",
            "ListAliases",
            "ListGroupsForEntity",
            "ListMailboxPermissions",
        ],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    native_pages::run(&catalog).await
}

#[tokio::test]
async fn given_branched_producer_when_unbranched_child_runs_then_only_ancestry_retains_the_source_stage()
-> TestResult {
    use catalog_tool::overlay;
    let source_stage = overlay::ExecutionBranch {
        discriminator: "producer-stage".into(),
        fixed: Default::default(),
        bind: Default::default(),
        emission: None,
        row_policy: None,
        scope: overlay::Scope::Region,
        request_region: None,
    };
    let bytes = synthetic::compiled_with_parents(
        1,
        |parent| parent.branches = vec![source_stage.clone()],
        |_| {},
    )?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    persistence::unbranched_source(&catalog).await
}

#[test]
fn given_a_kinesis_stream_when_building_mapped_config_request_then_only_its_arn_identifies_it()
-> TestResult {
    let compiled = native::compile(
        "kinesis-video",
        "2017-09-30",
        &["ListStreams", "DescribeMappedResourceConfiguration"],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let parent = catalog
        .lookup("kinesis-video", "ListStreams")
        .ok_or("parent")?;
    let child = catalog
        .lookup("kinesis-video", "DescribeMappedResourceConfiguration")
        .ok_or("child")?;
    let arn = "arn:aws:kinesisvideo:us-east-1:123456789012:stream/inventory/1";
    let response = serde_json::to_vec(&json!({
        "StreamInfoList": [{"StreamARN": arn, "StreamName": "inventory"}]
    }))?;
    let rows = record::item_records(&decode(parent, &response)?, parent.items())?;
    let account = AccountId::parse("123456789012")?;
    let region = RegionId::parse("us-east-1")?;
    let scope = ScopeKey::new(
        account.clone(),
        Region::Regional(region.clone()),
        ServiceId::parse(parent.service_name())?,
        OperationId::parse(parent.operation_name())?,
        None,
    )?;
    let sweep = SweepId::new("mapped-resources".into())?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let context = Context::new(parent, &sweep, &account, &region, &consumers);
    let source = RowContext {
        row: rows.first().ok_or("stream row missing")?,
        parent: None,
        scope: &scope,
        ancestry: &DiscoveryContext::empty(),
    };
    assert!(input::requires_parent(child));
    let bindings = child.bindings();
    assert_eq!(bindings.len(), 1);
    let binding = bindings.first().ok_or("stream binding missing")?;
    let (dependent, value) = resolve_binding(&context, &region, &source, &binding.source)?;
    assert!(dependent);
    let fields = [input::InputField {
        path: binding.path.clone(),
        value: value.ok_or("parent ARN missing")?,
    }];
    let request = crate::request::encode(child, "https://example.test", &empty_record()?, &fields)?;
    assert_eq!(
        serde_json::from_slice::<Json>(request.body())?,
        json!({"StreamARN": arn})
    );
    Ok(())
}
