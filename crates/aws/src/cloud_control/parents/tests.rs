//! Literal parent models, adversarial scope outcomes, and deterministic properties.

use std::sync::atomic::AtomicUsize;

use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed};
use tokio::sync::mpsc;

use super::*;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;

fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
    let (sender, receiver) = mpsc::channel(512);
    Ok((
        Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages,
            page_size: 100,
        },
        receiver,
    ))
}

fn description(id: &str, properties: Value) -> ResourceDescription {
    ResourceDescription::builder()
        .identifier(id)
        .properties(properties.to_string())
        .build()
}

fn listed(
    type_name: &str,
    items: Vec<ResourceDescription>,
    token: Option<&str>,
) -> ListResourcesOutput {
    ListResourcesOutput::builder()
        .type_name(type_name)
        .set_resource_descriptions(Some(items))
        .set_next_token(token.map(str::to_owned))
        .build()
}

fn got(request: &GetRequest, properties: Value) -> GetResourceOutput {
    GetResourceOutput::builder()
        .type_name(&request.type_name)
        .resource_description(description(&request.identifier, properties))
        .build()
}

fn events(
    receiver: &mut mpsc::Receiver<CollectionEvent>,
) -> Result<Vec<CollectionEvent>, TestError> {
    let mut values = Vec::new();
    for _ in 0..512 {
        match receiver.try_recv() {
            Ok(value) => values.push(value),
            Err(mpsc::error::TryRecvError::Empty | mpsc::error::TryRecvError::Disconnected) => {
                return Ok(values);
            }
        }
    }
    Err("fixture event capacity exceeded".into())
}

fn selected(names: &[&str]) -> Result<Vec<ResourceType>, AwsScanError> {
    validate_selection(
        &names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
    )
}

fn one_group(types: &[ResourceType]) -> Result<RootGroup<'_>, TestError> {
    let mut groups = groups(types)?;
    if groups.len() != 1 {
        return Err("fixture must select one root group".into());
    }
    groups.pop().ok_or_else(|| "missing root group".into())
}

fn fixture_resource(properties: Value) -> Resource {
    Resource {
        id: "arn:aws:eks:us-east-1:111111111111:cluster/alpha".into(),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: "aws_eks_cluster".into(),
        name: Some("alpha".into()),
        properties: json!({"provider_properties":properties}),
        tags: BTreeMap::new(),
        source: Source::CloudControl,
        observed_at: 0,
    }
}

proptest! {
    #![proptest_config(Config { cases: 128, rng_seed: RngSeed::Fixed(0x06cc_21), ..Config::default() })]

    #[test]
    fn property_parent_models_are_escaped_exactly_and_scope_identity_is_value_sensitive(
        name in "[A-Za-z0-9_./\\\"\\\\-]{1,64}", suffix in "[a-z]{1,8}",
    ) {
        let types=selected(&["AWS::EKS::Addon"]).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let group=one_group(&types).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let child=group.children.first().ok_or_else(||TestCaseError::fail("missing child"))?;
        let context=parent_context(child,&fixture_resource(json!({"Name":name}))).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let decoded: Value=serde_json::from_str(&context.encoded_model).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(decoded,json!({"ClusterName":name}));
        let different=parent_context(child,&fixture_resource(json!({"Name":format!("{name}{suffix}")}))).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_ne!(context.scope_parent,different.scope_parent);
    }

    #[test]
    fn property_selection_is_idempotent_and_closed_under_prerequisites(indices in prop::collection::vec(0_usize..21,1..32)) {
        let children=parent_catalog().map_err(|e|TestCaseError::fail(e.to_string()))?;
        let requested:Vec<_>=indices.iter().filter_map(|index|children.get(*index).map(|row|row.type_name.clone())).collect();
        let first=validate_selection(&requested).map_err(|e|TestCaseError::fail(e.to_string()))?;
        let first_names:Vec<_>=first.iter().map(|row|row.type_name.clone()).collect();
        let second=validate_selection(&first_names).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(&first_names,&second.iter().map(|row|row.type_name.clone()).collect::<Vec<_>>());
        let planned=groups(&first).map_err(|e|TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(planned.iter().map(|group|group.children.len()).sum::<usize>(),requested.iter().collect::<BTreeSet<_>>().len());
    }
}

mod child_denial_and_child_read_fallback_remain_failed_without_parent_edges;
mod combined_catalog_adds_exactly_twenty_one_reviewed_children;
mod literal_emulator_parent_fixtures_preserve_all_sanitized_properties_and_exact_architecture_edges;
mod partial_parent_pagination_still_collects_visible_children_and_records_gap;
mod successful_empty_parent_discovery_marks_children_complete_without_child_calls;
