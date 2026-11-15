#[path = "support/identity.rs"]
mod fixture;
use catalog_tool::overlay::{Identity, IdentityField, MemberPath};
use fixture::{compiled, TestResult};
use inventory_core::{catalog::runtime::{IdentityFieldPlan, IdentityPlan, RuntimeCatalog, RuntimeCatalogError}, record::{FieldId, Key, MapRef, Record, Value, ValueRef}};
use std::collections::BTreeMap;

fn integer() -> Result<Identity, catalog_tool::overlay::NameError> {
    Ok(Identity::CorrelatedKey(vec![IdentityField::SelectedInteger {
        path: MemberPath::new("Count")?, minimum: 1, maximum: i64::MAX,
    }]))
}

#[test]
fn given_typed_integral_identity_when_compiled_then_signed_bounds_and_numeric_path_round_trip() -> TestResult {
    let bytes = compiled(integer()?, "active-wire", false)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog.lookup("things", "ListThings").ok_or("plan absent")?;
    let IdentityPlan::CorrelatedKey(fields) = plan.identity() else { return Err("identity not correlated".into()) };
    assert!(matches!(fields.as_ref(), [IdentityFieldPlan::SelectedInteger { minimum:1, maximum:i64::MAX, .. }]));
    Ok(())
}

#[test]
fn given_wrong_modeled_identity_operand_when_compiled_then_no_text_to_integer_or_scalar_to_list_coercion_occurs() -> TestResult {
    for identity in [
        Identity::CorrelatedKey(vec![IdentityField::SelectedInteger { path: MemberPath::new("Name")?, minimum: 1, maximum: i64::MAX }]),
        Identity::CorrelatedKey(vec![IdentityField::RequestInteger { path: MemberPath::new("Mode")?, minimum: 1, maximum: i64::MAX }]),
        Identity::CorrelatedKey(vec![IdentityField::SelectedStringList { path: MemberPath::new("Name")?, max_items: 64, max_item_bytes: 255 }]),
        Identity::CorrelatedKey(vec![IdentityField::SelectedInteger { path: MemberPath::new("Count")?, minimum: 2, maximum: 1 }]),
    ] {
        assert!(compiled(identity, "active-wire", false).is_err());
    }
    Ok(())
}

fn map(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    let Value::Map(map) = value else { return Err("metadata not map".into()) };
    Ok(map)
}

fn field(number: u32) -> TestResult<Key> { Ok(Key::Field(FieldId::new(number)?)) }

fn corrupt(bytes: &[u8], change: impl FnOnce(&mut BTreeMap<Key, Value>) -> TestResult) -> TestResult<Record> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = map(map(&mut root)?.get_mut(&field(60_000)?).ok_or("extension absent")?)?;
    let Value::List(plans) = extension.get_mut(&field(2)?).ok_or("plans absent")? else { return Err("plans not list".into()) };
    let plan = map(plans.first_mut().ok_or("plan absent")?)?;
    let identity = map(plan.get_mut(&field(6)?).ok_or("identity absent")?)?;
    let Value::List(fields) = identity.get_mut(&field(2)?).ok_or("identity operands absent")? else { return Err("operands not list".into()) };
    change(map(fields.first_mut().ok_or("operand absent")?)?)?;
    let encoded = Record::encode(&Value::Map(extension.clone()))?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    map(extension.get_mut(&field(7)?).ok_or("receipt absent")?)?.insert(field(7)?, Value::Bytes(digest.to_vec()));
    Ok(Record::encode(&root)?)
}

#[test]
fn given_rehashed_corrupt_operand_metadata_when_loaded_then_unknown_tags_bounds_and_types_fail_closed() -> TestResult {
    let bytes = compiled(integer()?, "active-wire", false)?;
    for (tag, lower, upper) in [(5, 2, 1), (7, 0, 255), (7, 65, 255), (7, 64, 0), (7, 64, 4097)] {
        let record = corrupt(&bytes, |operand| {
            operand.insert(field(1)?, Value::Int(tag));
            operand.insert(field(3)?, Value::Int(lower));
            operand.insert(field(4)?, Value::Int(upper));
            Ok(())
        })?;
        assert!(matches!(RuntimeCatalog::from_bytes(record.as_bytes()), Err(RuntimeCatalogError::Bounds)));
    }
    for tag in [7, 999] {
        let record = corrupt(&bytes, |operand| {
            operand.insert(field(1)?, Value::Int(tag));
            operand.insert(field(3)?, Value::Int(64));
            operand.insert(field(4)?, Value::Int(255));
            Ok(())
        })?;
        assert!(matches!(RuntimeCatalog::from_bytes(record.as_bytes()), Err(RuntimeCatalogError::Capability)));
    }
    let record = corrupt(&bytes, |operand| { operand.insert(field(3)?, Value::Text("1".into())); Ok(()) })?;
    assert!(matches!(RuntimeCatalog::from_bytes(record.as_bytes()), Err(RuntimeCatalogError::Capability)));
    let record = corrupt(&bytes, |operand| { operand.insert(field(9)?, Value::Int(0)); Ok(()) })?;
    assert!(matches!(RuntimeCatalog::from_bytes(record.as_bytes()), Err(RuntimeCatalogError::Capability)));
    Ok(())
}
