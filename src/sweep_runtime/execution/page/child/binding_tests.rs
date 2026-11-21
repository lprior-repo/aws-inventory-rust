use crate::sweep_runtime::consumer_fixture as native;
mod native_pages;
mod persistence;
#[path = "../../../../../crates/catalog-tool/tests/support/bindings.rs"]
mod synthetic;

use super::*;
use crate::sweep_runtime::{execution::routing, record::policy_dfgk_tests::fixture::decode};
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
        let wire =
            crate::sweep_runtime::request::encode(plan, "https://bindings.test", &base, &[])?;
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
