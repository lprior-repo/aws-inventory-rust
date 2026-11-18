use super::*;
use inventory_core::record::{Key, Value};
use std::collections::BTreeMap;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn given_items(items: Vec<Value>) -> Result<(Record, ItemPlan), Box<dyn std::error::Error>> {
    let field = FieldId::new(1)?;
    let record = Record::encode(&Value::Map(BTreeMap::from([(
        Key::Field(field),
        Value::List(items),
    )])))?;
    Ok((record, ItemPlan::Path(vec![field].into_boxed_slice())))
}

fn self_key() -> IdentityPlan {
    IdentityPlan::Key(vec![Box::<[FieldId]>::default()].into_boxed_slice())
}

#[test]
fn given_two_scalar_items_when_selected_then_distinct_records_and_identities() -> TestResult {
    let (given, selector) = given_items(vec![
        Value::Text("alpha".to_owned()),
        Value::Text("beta".to_owned()),
    ])?;
    let when = item_records(&given, &selector)?;
    let identities = when
        .iter()
        .map(|row| resource_id(row, &self_key(), None))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        identities
            .iter()
            .map(ResourceId::as_str)
            .collect::<Vec<_>>(),
        ["alpha", "beta"]
    );
    assert_eq!(when.len(), 2);
    assert_ne!(
        when.first().ok_or("first row missing")?.as_bytes(),
        when.last().ok_or("last row missing")?.as_bytes()
    );
    assert!(
        when.iter()
            .all(|row| Record::from_bytes(row.as_bytes().to_vec()).is_ok())
    );
    Ok(())
}

#[test]
fn given_long_items_when_identified_then_all_signed_bits_are_preserved() -> TestResult {
    let (given, selector) = given_items(vec![Value::Int(i64::MIN), Value::Int(i64::MAX)])?;
    let when = item_records(&given, &selector)?;
    let then = when
        .iter()
        .map(|row| resource_id(row, &self_key(), None))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        then.first().ok_or("minimum missing")?.as_str(),
        "-9223372036854775808"
    );
    assert_eq!(
        then.last().ok_or("maximum missing")?.as_str(),
        "9223372036854775807"
    );
    assert!(matches!(
        path_value(when.last().ok_or("row missing")?.root(), &[])?,
        Some(ValueRef::Int(i64::MAX))
    ));
    Ok(())
}

#[test]
fn given_boolean_items_when_identified_then_both_values_remain_distinct() -> TestResult {
    let (given, selector) = given_items(vec![Value::Bool(false), Value::Bool(true)])?;
    let when = item_records(&given, &selector)?;
    let then = when
        .iter()
        .map(|row| resource_id(row, &self_key(), None))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        then.iter().map(ResourceId::as_str).collect::<Vec<_>>(),
        ["false", "true"]
    );
    Ok(())
}

#[test]
fn given_structure_item_when_self_identified_then_identity_is_rejected() -> TestResult {
    let (given, selector) = given_items(vec![Value::Map(BTreeMap::from([(
        Key::Field(FieldId::new(2)?),
        Value::Text("native".to_owned()),
    )]))])?;
    let when = item_records(&given, &selector)?;
    let row = when.first().ok_or("structure row missing")?;
    assert!(resource_id(row, &self_key(), None).is_err());
    let keyed =
        IdentityPlan::Key(vec![vec![FieldId::new(2)?].into_boxed_slice()].into_boxed_slice());
    assert_eq!(resource_id(row, &keyed, None)?.as_str(), "native");
    Ok(())
}

#[test]
fn given_arn_scalars_when_selected_then_arn_identity_reads_the_current_item() -> TestResult {
    let (given, selector) = given_items(vec![Value::Text(
        "arn:aws:example:us-east-1:123456789012:item/alpha".to_owned(),
    )])?;
    let when = item_records(&given, &selector)?;
    let row = when.first().ok_or("ARN row missing")?;
    assert_eq!(
        resource_id(row, &IdentityPlan::Arn(Box::new([])), None)?.as_str(),
        "arn:aws:example:us-east-1:123456789012:item/alpha"
    );
    assert!(matches!(required_text(row, &[])?, Cow::Borrowed(_)));
    Ok(())
}

#[test]
fn given_1001_items_when_selected_then_no_accepted_items_are_truncated() -> TestResult {
    let (given, selector) = given_items((0_i64..1001).map(Value::Int).collect())?;
    let when = item_records(&given, &selector)?;
    assert_eq!(when.len(), 1001);
    assert_eq!(
        resource_id(when.last().ok_or("last item missing")?, &self_key(), None)?.as_str(),
        "1000"
    );
    Ok(())
}

#[test]
fn given_long_structure_member_when_keyed_then_identity_keeps_all_64_bits() -> TestResult {
    let field = FieldId::new(2)?;
    let (given, selector) = given_items(vec![Value::Map(BTreeMap::from([(
        Key::Field(field),
        Value::Int(i64::MAX),
    )]))])?;
    let when = item_records(&given, &selector)?;
    let key = IdentityPlan::Key(vec![vec![field].into_boxed_slice()].into_boxed_slice());
    assert_eq!(
        resource_id(when.first().ok_or("long row missing")?, &key, None)?.as_str(),
        "9223372036854775807"
    );
    Ok(())
}
