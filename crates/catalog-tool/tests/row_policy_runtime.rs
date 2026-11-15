#[path = "support/bindings.rs"]
mod fixture;
use catalog_tool::overlay::{self, ContextSource, RowOperand, RowPredicate, RowValueType};
use fixture::{compiled, TestResult};
use inventory_core::{catalog::runtime::{RuntimeCatalog, RuntimeCatalogError, RowPredicatePlan}, record::{FieldId, Key, MapRef, Record, Value, ValueRef}};
use std::collections::BTreeMap;

fn absent(source: ContextSource, path: &str) -> TestResult<RowPredicate> {
    Ok(RowPredicate::Absent(RowOperand { source,path:overlay::MemberPath::selected(path)?,value_type:RowValueType::String }))
}

fn equality() -> TestResult<RowPredicate> {
    Ok(RowPredicate::SelectedRequestEquals { selected:overlay::MemberPath::new("Id")?,
        request:overlay::MemberPath::new("OrganizationId")?,value_type:RowValueType::String,
        missing:overlay::MissingSelected::AllowAbsent })
}

#[test]
fn given_selected_request_response_policies_when_compiled_then_native_loader_retains_distinct_decisions() -> TestResult {
    // Given bounded predicates over three actual namespaces.
    let owned = absent(ContextSource::SelectedItem,"Id")?;
    let assertion = equality()?;
    let response = absent(ContextSource::Response,"ResponseId")?;
    let child = overlay::ChildEligibility { operation:overlay::OperationName::new("ListParents0")?,
        predicate:absent(ContextSource::Response,"ResponseId")? };
    let bytes = compiled(1,|plan| {
        plan.row_policy.owned = Some(owned);
        plan.row_policy.assertions = vec![assertion,response];
        plan.row_policy.child = Some(child);
    })?;
    // When native metadata is model-checked at load.
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let child = catalog.lookup("bindings","ListChildren").ok_or("child missing")?;
    let policy = child.row_policy();
    // Then filtering, assertions and child eligibility are not interchangeable.
    assert!(matches!(policy.owned,Some(RowPredicatePlan::Absent(_))));
    assert_eq!(policy.assertions.len(),2);
    assert!(matches!(policy.assertions.first(),Some(RowPredicatePlan::SelectedRequestEquals { missing:inventory_core::catalog::runtime::MissingSelected::AllowAbsent,.. })));
    assert_eq!(policy.child.as_ref().ok_or("eligibility missing")?.operation.as_ref(),"ListParents0");
    let parent = catalog.lookup("bindings","ListParents0").ok_or("parent missing")?;
    assert!(!parent.context_projection().response.is_empty(),"child eligibility response operand was not projected");
    Ok(())
}

#[test]
fn given_invalid_paths_types_or_predicate_bounds_when_compiled_then_checker_rejects_exactly_the_row_policy() -> TestResult {
    let invalid = [
        absent(ContextSource::SelectedItem,"Missing")?,
        absent(ContextSource::SelectedItem,"Count")?,
        absent(ContextSource::Request,"ResponseId")?,
        absent(ContextSource::Response,"OrganizationId")?,
        RowPredicate::BooleanEquals { operand:RowOperand { source:ContextSource::SelectedItem,
            path:overlay::MemberPath::new("Id")?,value_type:RowValueType::Boolean },value:false },
        RowPredicate::Absent(RowOperand { source:ContextSource::SelectedItem,
            path:overlay::MemberPath::new("Id")?,value_type:RowValueType::Structure }),
    ];
    for predicate in invalid {
        // Given a field not modeled in the named namespace/type. When compiled.
        let error = compiled(1,|plan| plan.row_policy.owned = Some(predicate)).err().ok_or("invalid policy accepted")?;
        // Then the pinned checker—not a serialization accident—rejects the policy.
        let error = error.downcast_ref::<overlay::CheckError>().ok_or("wrong rejection boundary")?;
        assert!(matches!(error,overlay::CheckError::Violations(violations) if violations.iter().any(|v| v.key == "row_policy" && v.rule == overlay::ViolationRule::InvalidPath)));
    }
    let predicate = equality()?;
    let error = compiled(1,|plan| plan.row_policy.assertions = vec![predicate;9]).err().ok_or("unbounded assertions accepted")?;
    assert!(matches!(error.downcast_ref::<overlay::CheckError>(),Some(overlay::CheckError::Violations(violations)) if violations.iter().any(|v| v.key == "row_policy")));
    Ok(())
}

#[test]
fn given_child_eligibility_for_an_unbound_producer_when_compiled_then_it_cannot_gate_an_independent_row_set() -> TestResult {
    let child = overlay::ChildEligibility { operation:overlay::OperationName::new("ListChildren")?,
        predicate:absent(ContextSource::SelectedItem,"Id")? };
    let error = compiled(1,|plan| plan.row_policy.child = Some(child)).err().ok_or("unbound predicate source accepted")?;
    assert!(matches!(error.downcast_ref::<overlay::CheckError>(),Some(overlay::CheckError::Violations(violations)) if violations.iter().any(|v| v.key == "row_policy")));
    Ok(())
}

fn key(number: u32) -> TestResult<Key> { Ok(Key::Field(FieldId::new(number)?)) }
fn map(value: &mut Value) -> TestResult<&mut BTreeMap<Key,Value>> {
    match value { Value::Map(fields) => Ok(fields), _ => Err("metadata is not a map".into()) }
}
fn get(value: &mut Value, number: u32) -> TestResult<&mut Value> {
    map(value)?.get_mut(&key(number)?).ok_or("metadata field missing".into())
}

fn corrupt(bytes: &[u8], mutation: usize) -> TestResult<Record> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = get(&mut root,60_000)?;
    let Value::List(plans) = get(extension,2)? else { return Err("plans missing".into()); };
    let child = plans.iter_mut().find(|plan| match plan {
        Value::Map(fields) => fields.values().any(|value| matches!(value,Value::Text(text) if text == "ListChildren")), _ => false,
    }).ok_or("child missing")?;
    let policy = get(child,22)?;
    let owned = get(policy,1)?;
    match mutation {
        0 => { *get(get(owned,2)?,3)? = Value::Int(1); }
        1 => { *get(get(owned,2)?,1)? = Value::Int(99); }
        2 => { *get(get(owned,2)?,2)? = Value::List(vec![Value::Int(999_999)]); }
        3 => { *get(owned,1)? = Value::Int(99); }
        4 => { map(owned)?.insert(key(77)?,Value::Bool(true)); }
        _ => return Err("unknown corruption".into()),
    }
    // Reseal the compiled-input receipt: validation must not rely on the digest alone.
    let encoded = Record::encode(extension)?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    *get(get(extension,7)?,7)? = Value::Bytes(digest.to_vec());
    Ok(Record::encode(&root)?)
}

#[test]
fn given_resealed_corrupt_row_predicates_when_native_catalog_loads_then_model_and_bounded_codec_reject_each() -> TestResult {
    // Given a valid producer-checked owned filter.
    let predicate = absent(ContextSource::SelectedItem,"Id")?;
    let bytes = compiled(1,|plan| plan.row_policy.owned = Some(predicate))?;
    for mutation in 0..5 {
        // When corruption is resealed with a correct compiled-input receipt.
        let corrupt = corrupt(&bytes,mutation)?;
        // Then malformed tags/unknown fields/model-incompatible types never reach scheduling.
        assert!(matches!(RuntimeCatalog::from_bytes(corrupt.as_bytes()),Err(RuntimeCatalogError::Capability)),"accepted mutation {mutation}");
    }
    Ok(())
}

#[test]
fn given_child_response_eligibility_when_its_producer_projection_is_removed_then_native_loading_rejects_the_resealed_plan() -> TestResult {
    let eligibility = overlay::ChildEligibility { operation:overlay::OperationName::new("ListParents0")?,
        predicate:absent(ContextSource::Response,"ResponseId")? };
    let bytes = compiled(1,|plan| plan.row_policy.child = Some(eligibility))?;
    RuntimeCatalog::from_bytes(&bytes)?;
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(&bytes)?).to_value()?;
    let extension = get(&mut root,60_000)?;
    let Value::List(plans) = get(extension,2)? else { return Err("plans missing".into()); };
    let parent = plans.iter_mut().find(|plan| match plan {
        Value::Map(fields) => fields.values().any(|value| matches!(value,Value::Text(text) if text == "ListParents0")), _ => false,
    }).ok_or("parent missing")?;
    *get(get(parent,20)?,2)? = Value::List(Vec::new());
    let encoded = Record::encode(extension)?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    *get(get(extension,7)?,7)? = Value::Bytes(digest.to_vec());
    let corrupt = Record::encode(&root)?;
    assert!(matches!(RuntimeCatalog::from_bytes(corrupt.as_bytes()),Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn given_modeled_structure_absence_when_compiled_then_the_native_operand_retains_structure_type() -> TestResult {
    let predicate = RowPredicate::Absent(RowOperand { source:ContextSource::SelectedItem,
        path:overlay::MemberPath::selected("$")?,value_type:RowValueType::Structure });
    let bytes = compiled(1,|plan| plan.row_policy.owned = Some(predicate))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let policy = catalog.lookup("bindings","ListChildren").ok_or("child missing")?.row_policy();
    assert!(matches!(&policy.owned,Some(RowPredicatePlan::Absent(operand))
        if operand.value_type == inventory_core::catalog::runtime::RowValueType::Structure && operand.path.is_empty()));
    Ok(())
}

#[test]
fn given_additive_stage_policy_when_compiled_then_native_loader_preserves_base_and_branch_rules_and_rejects_branch_codec_tampering() -> TestResult {
    let base = absent(ContextSource::SelectedItem,"Id")?;
    let owned = absent(ContextSource::Response,"ResponseId")?;
    let assertion = equality()?;
    let bytes = compiled(1,|plan| {
        plan.row_policy.owned = Some(base);
        plan.branches = vec![overlay::ExecutionBranch { discriminator:"PORTAL".to_owned(),
            fixed:BTreeMap::new(),bind:BTreeMap::new(),emission:None,scope:overlay::Scope::Region,
            request_region:None,row_policy:Some(overlay::RowPolicy {
                owned:Some(owned),assertions:vec![assertion],child:None,
            }),
        }];
    })?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("bindings","ListChildren").ok_or("child missing")?;
    assert!(plan.row_policy().owned.is_some());
    let branch = plan.branches().first().ok_or("branch missing")?;
    let policy = branch.row_policy.as_ref().ok_or("branch policy missing")?;
    assert!(matches!(policy.owned,Some(RowPredicatePlan::Absent(_))));
    assert_eq!(policy.assertions.len(),1);
    for mutation in 0..2 {
        let mut root = ValueRef::Map(MapRef::from_bytes_bounded(&bytes)?).to_value()?;
        let extension = get(&mut root,60_000)?;
        let Value::List(plans) = get(extension,2)? else { return Err("plans missing".into()); };
        let child = plans.iter_mut().find(|plan| match plan {
            Value::Map(fields) => fields.values().any(|value| matches!(value,Value::Text(text) if text == "ListChildren")), _ => false,
        }).ok_or("child missing")?;
        let Value::List(branches) = get(child,16)? else { return Err("branches missing".into()); };
        let policy = get(branches.first_mut().ok_or("branch missing")?,6)?;
        if mutation == 0 {
            map(policy)?.insert(key(77)?,Value::Bool(true));
        } else {
            let Value::List(assertions) = get(policy,2)? else { return Err("assertions missing".into()); };
            *get(assertions.first_mut().ok_or("assertion missing")?,4)? = Value::Int(2);
        }
        let encoded = Record::encode(extension)?;
        let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
        *get(get(extension,7)?,7)? = Value::Bytes(digest.to_vec());
        let corrupt = Record::encode(&root)?;
        assert!(matches!(RuntimeCatalog::from_bytes(corrupt.as_bytes()),Err(RuntimeCatalogError::Capability)),
            "accepted branch policy mutation {mutation}");
    }
    Ok(())
}
