use super::*;
use inventory_core::record::scalar_record;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn bound_child(value: ValueRef<'_>, target: FieldId) -> Result<Record, Box<dyn std::error::Error>> {
    let given_parent = scalar_record(value)?;
    let (applicable, selected) = row_value(&given_parent, &[])?;
    assert!(applicable);
    let when_binding = input::InputField {
        path: vec![target].into_boxed_slice(),
        value: selected.ok_or("scalar parent binding absent")?,
    };
    Ok(input::merge_input(&empty_record()?, &[when_binding])?)
}

#[test]
fn given_distinct_scalar_parents_when_bound_then_child_inputs_keep_exact_values() -> TestResult {
    let target = FieldId::new(7)?;
    ["parent-one", "parent-two"].into_iter().try_for_each(|given| {
        let when = bound_child(ValueRef::Text(given), target)?;
        assert!(matches!(when.root().get_field(target)?, Some(ValueRef::Text(value)) if value == given));
        assert!(when.root().get_name("$")?.is_none());
        Ok(())
    })
}

#[test]
fn given_long_parent_when_bound_then_child_input_preserves_all_64_bits() -> TestResult {
    let target = FieldId::new(7)?;
    let when = bound_child(ValueRef::Int(i64::MAX), target)?;
    assert!(matches!(
        when.root().get_field(target)?,
        Some(ValueRef::Int(i64::MAX))
    ));
    Ok(())
}

#[test]
fn given_boolean_parent_when_bound_then_child_input_preserves_boolean_type() -> TestResult {
    let target = FieldId::new(7)?;
    let when = bound_child(ValueRef::Bool(false), target)?;
    assert!(matches!(
        when.root().get_field(target)?,
        Some(ValueRef::Bool(false))
    ));
    Ok(())
}

#[test]
fn given_discovery_intermediary_when_child_work_is_built_then_nearest_resource_parent_is_retained()
-> TestResult {
    use crate::sweep_runtime::record::policy_dfgk_tests::fixture;
    let catalog = fixture::catalog()?;
    let producer = fixture::plan(&catalog, "iottwinmaker", "ListWorkspaces")?;
    let consumer = fixture::plan(&catalog, "iottwinmaker", "ListEntities")?;
    let rows = fixture::rows(producer,br#"{"workspaceSummaries":[{"workspaceId":"workspace-a","arn":"arn:aws:iottwinmaker:us-east-1:123456789012:workspace/workspace-a","creationDateTime":0,"updateDateTime":0}]}"#)?;
    let row = rows.first().ok_or("workspace source row absent")?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("us-east-1")?;
    let ancestor = ResourceKey {
        account: account.clone(),
        region: Region::Regional(home.clone()),
        kind: producer.kind().clone(),
        id: inventory_core::inventory::ResourceId::parse(
            "arn:aws:iottwinmaker:us-east-1:123456789012:workspace/workspace-a",
        )?,
    };
    let scope = ScopeKey::new(
        account.clone(),
        Region::Regional(home.clone()),
        ServiceId::parse(producer.service_name())?,
        OperationId::parse(producer.operation_name())?,
        Some(ancestor.clone()),
    )?;
    let ancestry = DiscoveryContext::empty();
    let sweep = SweepId::new("intermediary-parent".to_owned())?;
    let consumers = super::super::consumer_index::ConsumerIndex::build(&catalog)?;
    let context = Context::new(producer, &sweep, &account, &home, &consumers);
    let source = RowContext {
        row,
        parent: None,
        scope: &scope,
        ancestry: &ancestry,
    };
    let mut budget = PageContextBudget::new(&ancestry, 1)?;
    let child = child_for_row(
        &context,
        consumer,
        &source,
        None,
        &ancestry.encode()?,
        &mut budget,
    )?
    .ok_or("reviewed dependent child was not scheduled")?;
    assert_eq!(child.scope().parent(), Some(&ancestor));
    assert_ne!(child.scope().operation(), scope.operation());
    Ok(())
}
