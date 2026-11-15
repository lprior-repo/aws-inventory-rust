use super::*;
use crate::sweep_runtime::{execution::{page::child::{Context, policy_dfgk_tests::project_children}, routing}, input, record::policy_dfgk_tests::fixture};
use crate::sweep_runtime::execution::page::consumer_index::ConsumerIndex;
use inventory_core::{inventory::{AccountId, OperationId, ScopeKey, ServiceId}, record::Value};
use inventory_engine::inventory::SweepId;
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn ga_request_parent_relations_reject_hostile_scope_and_type_without_selected_parent_fields() -> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("eu-west-1")?;
    let sweep = SweepId::new("request-origin-hostile-scope")?;
    for (root_operation, child_operation) in [
        ("ListAccelerators", "ListListeners"),
        ("ListCustomRoutingAccelerators", "ListCustomRoutingListeners"),
    ] {
        // Given real compiled discovery that generates a durable typed child request.
        let parent = fixture::plan(&catalog, "global-accelerator", root_operation)?;
        let child = fixture::plan(&catalog, "global-accelerator", child_operation)?;
        let parent_arn = "arn:aws:globalaccelerator::123456789012:accelerator/parent";
        let scope = ScopeKey::new(account.clone(), Region::Global,
            ServiceId::parse(parent.service_name())?, OperationId::parse(root_operation)?, None)?;
        let root_request = input::root_bindings(parent, Some(&account), Some(&Region::Global))?;
        let response = fixture::decode(parent, &serde_json::to_vec(&json!({"Accelerators":[{"AcceleratorArn":parent_arn}]}))?)?;
        let context = Context::new(parent, &sweep, &account, &home, &consumers);
        let (_, children) = project_children(&context, &scope, &root_request, &response, &DiscoveryContext::empty())?;
        let work = children.iter().find(|work| work.scope().operation().as_str() == child_operation)
            .ok_or("real child discovery did not schedule listener enumeration")?;
        let request = Record::from_bytes(work.bindings().to_vec())?;
        let ancestry = DiscoveryContext::decode(work.context())?;
        let response = fixture::decode(child, &serde_json::to_vec(&json!({"Listeners":[{
            "ListenerArn":"arn:aws:globalaccelerator::123456789012:accelerator/parent/listener/child"
        }]}))?)?;
        let context = Context::new(child, &sweep, &account, &home, &consumers);
        let (observations, _) = project_children(&context, work.scope(), &request, &response, &ancestry)?;
        let rows = record::item_records(&response, child.items())?;
        let rule = catalog.relationships().iter().find(|rule| rule.source_kind == *child.kind() && rule.source == RelationSource::Request)
            .ok_or("reviewed request-origin listener relationship absent")?;
        assert!(child.supports_relationship(rule)?);
        let request_region = routing::request_region(child, work.scope().region(), &home, work.scope().branch())?;
        let valid = references(&catalog, child, &request_region, &rows, &observations, &request)?;
        assert_eq!(valid.len(), 1);
        assert_eq!(valid.first().ok_or("valid request parent reference absent")?.target.value().as_str(), parent_arn);
        for observation in &observations {
            assert!(record::path_value(observation.record().root(), &rule.source_path)?.is_none());
            assert!(matches!(record::path_value(request.root(), &rule.source_path)?,
                Some(ValueRef::Text(value)) if value == parent_arn));
        }
        // When only the actual bound parent request is hostile, no selected output may rescue it.
        for hostile in [
            Value::Text("arn:aws:globalaccelerator::999999999999:accelerator/foreign".to_owned()),
            Value::Text("arn:aws-cn:globalaccelerator::123456789012:accelerator/foreign".to_owned()),
            Value::Text("arn:aws:globalaccelerator:us-east-1:123456789012:accelerator/foreign".to_owned()),
            Value::Text("arn:broken".to_owned()), Value::Int(7),
        ] {
            let hostile = input::merge_input(&request, &[input::InputField { path: rule.source_path.clone(), value: hostile }])?;
            assert!(references(&catalog, child, &request_region, &rows, &observations, &hostile).is_err());
        }
    }
    Ok(())
}
