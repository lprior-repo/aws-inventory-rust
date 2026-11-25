use super::*;
use inventory_core::{
    inventory::{Observation, ResourceId, ResourceKey},
    record::{Exclusions, Key},
};
use inventory_engine::inventory::WorkSpec;

#[test]
fn given_required_child_sources_when_missing_or_empty_then_only_those_requests_are_counted_skipped()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let parent = catalog
        .lookup("timestream-influxdb", "ListDbParameterGroups")
        .ok_or("parent missing")?;
    let plan = catalog
        .lookup("timestream-influxdb", "ListTagsForResource")
        .ok_or("tag child missing")?;
    let field = compiled.fields.id("arn").ok_or("ARN field missing")?;
    for value in [None, Some(""), Some(" "), Some("invalid-arn")] {
        let fields = value.map(|value| (Key::Field(field), Value::Text(value.into())));
        let row = Record::encode(&Value::Map(fields.into_iter().collect()))?;
        let (work, skipped) = source_work(parent, plan, &row)?;
        let present = value.is_some_and(|value| !value.is_empty());
        assert_eq!(work.is_some(), present);
        assert_eq!(skipped, u64::from(!present));
    }
    Ok(())
}

fn source_work(
    parent_plan: &CallablePlan<'_>,
    plan: &CallablePlan<'_>,
    row: &Record,
) -> TestResult<(Option<WorkSpec>, u64)> {
    let owner = owner(parent_plan)?;
    let observation = Observation::new(owner.clone(), row.clone(), &Exclusions::none(), None)?;
    let request = input::root_bindings(parent_plan, Some(&owner.account), Some(&owner.region))?;
    let sweep = SweepId::new("empty-required-source".to_owned())?;
    let context = ChildContext {
        parent_plan,
        plan,
        request: &request,
        sweep: &sweep,
        discovery: &[],
        branch: None,
    };
    let mut budget =
        super::super::super::child::PageContextBudget::new(&DiscoveryContext::empty(), 0)?;
    let work = child_work(&context, row, &observation, &mut budget)?;
    Ok((work, budget.skipped_requests()))
}

fn owner(plan: &CallablePlan<'_>) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: plan.kind().clone(),
        id: ResourceId::parse("abc123")?,
    })
}

fn compiled() -> TestResult<fixture::Compiled> {
    fixture::compile(
        "timestream-influxdb",
        "2023-01-27",
        &[
            "ListDbBackups",
            "ListDbClusters",
            "ListDbInstances",
            "ListDbParameterGroups",
            "ListTagsForResource",
        ],
    )
}
