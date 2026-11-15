use super::*;
use inventory_core::record::{KeyRef, scalar_record};

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
        assert!(matches!(when.root().get(KeyRef::Field(target))?, Some(ValueRef::Text(value)) if value == given));
        assert!(when.root().get(KeyRef::Name("$"))?.is_none());
        Ok(())
    })
}

#[test]
fn given_long_parent_when_bound_then_child_input_preserves_all_64_bits() -> TestResult {
    let target = FieldId::new(7)?;
    let when = bound_child(ValueRef::Int(i64::MAX), target)?;
    assert!(matches!(when.root().get(KeyRef::Field(target))?, Some(ValueRef::Int(i64::MAX))));
    Ok(())
}

#[test]
fn given_boolean_parent_when_bound_then_child_input_preserves_boolean_type() -> TestResult {
    let target = FieldId::new(7)?;
    let when = bound_child(ValueRef::Bool(false), target)?;
    assert!(matches!(when.root().get(KeyRef::Field(target))?, Some(ValueRef::Bool(false))));
    Ok(())
}
