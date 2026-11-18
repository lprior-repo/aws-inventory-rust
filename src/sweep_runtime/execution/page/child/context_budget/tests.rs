use super::*;
use inventory_core::{
    catalog::runtime::ContextProjectionPlan,
    inventory::{AccountId, DiscoveryFrame, OperationId, Region, ScopeKey, ServiceId},
    record::{FieldId, Key, Value},
};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn aggregate_context_budget_accepts_exact_limit_and_rejects_one_more_byte_atomically() -> TestResult
{
    // Given one page-wide budget shared by projected contexts and durable child copies.
    let mut budget = PageContextBudget::new(&DiscoveryContext::empty(), 3)?;
    budget.charge(MAX_PAGE_CONTEXT_BYTES - 1)?;
    // When cumulative admission reaches the named exact limit, then it succeeds.
    budget.charge(1)?;
    assert_eq!(budget.used, MAX_PAGE_CONTEXT_BYTES);
    // One additional byte fails without advancing the admitted total.
    assert!(matches!(
        budget.charge(1),
        Err(SweepError::Operation(
            "page discovery contexts exceed the aggregate byte bound"
        ))
    ));
    assert_eq!(budget.used, MAX_PAGE_CONTEXT_BYTES);
    Ok(())
}

#[test]
fn large_inherited_context_rejects_many_tiny_rows_before_projecting_a_page() -> TestResult {
    let selected = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(1)?),
        Value::Text("x".repeat(512 * 1024)),
    )])))?;
    let empty = Record::encode(&Value::Map(BTreeMap::new()))?;
    let projection = ContextProjectionPlan {
        selected: vec![vec![FieldId::new(1)?].into_boxed_slice()].into_boxed_slice(),
        response: Box::new([]),
        request: Box::new([]),
    };
    let ancestry = DiscoveryContext::empty().push(DiscoveryFrame::project(
        ServiceId::parse("transfer")?,
        OperationId::parse("ListServers")?,
        0,
        &projection,
        &selected,
        &empty,
        &empty,
    )?)?;
    let rows = vec![empty.clone(); MAX_PAGE_CONTEXT_BYTES / ancestry.encoded_len()? + 1];
    let catalog = crate::sweep_runtime::record::policy_dfgk_tests::fixture::catalog()?;
    let plan = catalog
        .lookup("global-accelerator", "ListAccelerators")
        .ok_or("GA fixture plan absent")?;
    let scope = ScopeKey::new(
        AccountId::parse("123456789012")?,
        Region::Global,
        ServiceId::parse(plan.service_name())?,
        OperationId::parse(plan.operation_name())?,
        None,
    )?;
    // The entire page is rejected, not a prefix of contexts that could produce partial children.
    let result = super::super::contexts(plan, &rows, &empty, &empty, &ancestry, &scope);
    assert!(matches!(
        result,
        Err(SweepError::Operation(
            "page discovery contexts exceed the aggregate byte bound"
        ))
    ));
    assert!(PageContextBudget::new(&ancestry, usize::MAX).is_err());
    Ok(())
}
