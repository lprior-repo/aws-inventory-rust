#[path = "support/base_catalog.rs"]
mod base_catalog;
#[path = "support/runtime_provenance.rs"]
mod runtime_provenance;
#[path = "runtime_skip_decode/scopes.rs"]
mod scopes;

use inventory_core::{
    catalog::{
        MAX_CATALOG_BYTES,
        runtime::{RuntimeCatalog, RuntimeCatalogError},
    },
    record::{FieldId, Key, MapRef, Value, ValueRef, encode_container},
};

use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const SERVICE: &str = "accessanalyzer";
const PLAN: &str = "ListAnalyzers";
const CHILD: &str = "GetAnalyzer";

fn fields(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("fixture expected a field map".into()),
    }
}

fn key(id: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(id)?))
}

fn artifact(skip_field: Option<Value>) -> TestResult<Vec<u8>> {
    artifact_for(SERVICE, PLAN, skip_field)
}

fn artifact_for(service: &str, operation: &str, skip_field: Option<Value>) -> TestResult<Vec<u8>> {
    carrier_artifact(base_catalog::base()?, service, operation, false, |plan| {
        let _previous_field = match skip_field {
            Some(value) => plan.insert(key(26)?, value),
            None => plan.remove(&key(26)?),
        };
        Ok(())
    })
}

fn carrier_artifact(
    base: &[u8],
    service: &str,
    operation: &str,
    move_last: bool,
    edit: impl FnOnce(&mut BTreeMap<Key, Value>) -> TestResult,
) -> TestResult<Vec<u8>> {
    let mut root =
        ValueRef::Map(MapRef::from_container_bytes(base, MAX_CATALOG_BYTES)?).to_value()?;
    let extension = fields(&mut root)?
        .get_mut(&key(60_000)?)
        .ok_or("missing runtime extension")?;
    let Some(Value::List(plans)) = fields(extension)?.get_mut(&key(2)?) else {
        return Err("missing plan list".into());
    };
    let (service_key, operation_key) = (key(1)?, key(2)?);
    let (service, operation) = (Value::Text(service.into()), Value::Text(operation.into()));
    let index = plans
        .iter()
        .position(|plan| {
            let Value::Map(map) = plan else {
                return false;
            };
            map.get(&service_key) == Some(&service) && map.get(&operation_key) == Some(&operation)
        })
        .ok_or("missing carrying plan")?;
    let mut plan = plans.remove(index);
    edit(fields(&mut plan)?)?;
    if move_last {
        plans.push(plan);
    } else {
        plans.insert(index, plan);
    }
    runtime_provenance::reseal(&mut root, base)?;
    Ok(encode_container(&root, MAX_CATALOG_BYTES)?)
}

fn map_value(entries: Vec<(u32, Value)>) -> TestResult<Value> {
    let entries = entries
        .into_iter()
        .map(|(id, value)| Ok((key(id)?, value)))
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    Ok(Value::Map(entries))
}

fn skips(names: impl IntoIterator<Item = &'static str>) -> Value {
    Value::List(
        names
            .into_iter()
            .map(|name| Value::Text(name.into()))
            .collect(),
    )
}

#[test]
fn decoder_rejects_duplicate_skip_entries() -> TestResult {
    let bytes = artifact(Some(skips([CHILD, CHILD])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_unknown_skip_operation() -> TestResult {
    let bytes = artifact(Some(skips(["DescribeMissingSkipTarget"])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_skip_operation_from_another_service() -> TestResult {
    let bytes = artifact(Some(skips(["DescribeTable"])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_known_same_service_operation_without_a_plan() -> TestResult {
    let bytes = artifact(Some(skips(["GetFinding"])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_bound_tag_endpoint_as_a_skip_target() -> TestResult {
    let bytes = artifact_for("sns", "ListTopics", Some(skips(["ListTagsForResource"])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_skip_operation_referring_to_its_own_plan() -> TestResult {
    let bytes = artifact(Some(skips([PLAN])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_more_than_64_skip_entries() -> TestResult {
    let bytes = artifact(Some(skips([CHILD; 65])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Bounds)));
    Ok(())
}

#[test]
fn decoder_rejects_skip_field_that_is_not_a_list() -> TestResult {
    let bytes = artifact(Some(Value::Text(CHILD.into())))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_rejects_skip_list_entry_that_is_not_text() -> TestResult {
    let bytes = artifact(Some(Value::List(vec![Value::Int(1)])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_accepts_single_same_service_skip_operation() -> TestResult {
    let bytes = artifact(Some(skips([CHILD])))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(SERVICE, PLAN)
        .ok_or("missing decoded plan")?;
    assert_eq!(plan.skips(), &[Box::<str>::from("GetAnalyzer")]);
    Ok(())
}

#[test]
fn decoder_accepts_absent_skip_field_as_no_skips() -> TestResult {
    let bytes = artifact(None)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(SERVICE, PLAN)
        .ok_or("missing decoded plan")?;
    assert!(plan.skips().is_empty());
    Ok(())
}

#[test]
fn decoder_rejects_case_variant_of_a_valid_skip_operation() -> TestResult {
    let bytes = artifact(Some(skips(["getanalyzer"])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}

#[test]
fn decoder_accepts_skip_child_declared_before_its_carrier() -> TestResult {
    let bytes = carrier_artifact(base_catalog::base()?, SERVICE, PLAN, true, |plan| {
        plan.insert(key(26)?, skips([CHILD]));
        Ok(())
    })?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup(SERVICE, PLAN)
        .ok_or("missing decoded plan")?;
    assert_eq!(plan.skips(), &[Box::<str>::from("GetAnalyzer")]);
    Ok(())
}

#[test]
fn decoder_rejects_skip_carried_by_a_non_enumeration_plan() -> TestResult {
    let endpoint_carrier = |skip_field: Option<Value>| {
        carrier_artifact(base_catalog::base()?, SERVICE, PLAN, false, |plan| {
            plan.insert(key(12)?, map_value(vec![(1, Value::Int(4))])?);
            plan.insert(
                key(6)?,
                map_value(vec![(1, Value::Int(2)), (2, Value::List(Vec::new()))])?,
            );
            let _previous_field = match skip_field {
                Some(value) => plan.insert(key(26)?, value),
                None => plan.remove(&key(26)?),
            };
            Ok(())
        })
    };
    RuntimeCatalog::from_bytes(&endpoint_carrier(None)?)?;
    let bytes = endpoint_carrier(Some(skips([CHILD])))?;
    let result = RuntimeCatalog::from_bytes(&bytes);
    assert!(matches!(result, Err(RuntimeCatalogError::Capability)));
    Ok(())
}
