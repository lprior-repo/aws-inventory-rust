use super::*;
use inventory_core::{
    catalog::runtime::RowOperandPlan,
    record::{Key, Value},
};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn field(number: u32) -> TestResult<FieldId> {
    Ok(FieldId::new(number)?)
}
fn record(fields: &[(FieldId, Value)]) -> TestResult<Record> {
    Ok(Record::encode(&Value::Map(
        fields
            .iter()
            .cloned()
            .map(|(id, value)| (Key::Field(id), value))
            .collect(),
    ))?)
}
fn evaluate_from(
    predicate: &RowPredicatePlan,
    selected: &Record,
    request: &Record,
) -> Result<bool, SweepError> {
    evaluate(predicate, &|source, path| {
        lookup(
            if source == ContextSource::SelectedItem {
                selected
            } else {
                request
            },
            path,
        )
    })
}

#[test]
fn given_absence_ownership_when_missing_null_and_empty_are_distinguished_then_only_true_absence_is_owned()
-> TestResult {
    let id = field(1)?;
    let predicate = RowPredicatePlan::Absent(RowOperandPlan {
        source: ContextSource::SelectedItem,
        path: Box::new([id]),
        value_type: RowValueType::String,
    });
    let request = record(&[])?;
    assert!(evaluate_from(&predicate, &record(&[])?, &request)?);
    assert!(!evaluate_from(
        &predicate,
        &record(&[(id, Value::Text(String::new()))])?,
        &request
    )?);
    assert!(!evaluate_from(
        &predicate,
        &record(&[(id, Value::Text("COGNITO".to_owned()))])?,
        &request
    )?);
    assert!(matches!(
        evaluate_from(&predicate, &record(&[(id, Value::Null)])?, &request),
        Err(SweepError::Operation(
            "row policy value is null or has invalid modeled type"
        ))
    ));
    assert!(matches!(
        evaluate_from(&predicate, &record(&[(id, Value::Bool(false))])?, &request),
        Err(SweepError::Operation(
            "row policy value is null or has invalid modeled type"
        ))
    ));
    Ok(())
}

#[test]
fn given_optional_parent_duplicate_when_present_then_matching_is_required_and_missing_never_hides_a_bad_request()
-> TestResult {
    let selected_id = field(1)?;
    let request_id = field(2)?;
    let predicate = RowPredicatePlan::SelectedRequestEquals {
        selected: Box::new([selected_id]),
        request: Box::new([request_id]),
        value_type: RowValueType::String,
        missing: MissingSelected::AllowAbsent,
    };
    let request = record(&[(request_id, Value::Text("parent-a".to_owned()))])?;
    assert!(evaluate_from(&predicate, &record(&[])?, &request)?);
    assert!(evaluate_from(
        &predicate,
        &record(&[(selected_id, Value::Text("parent-a".to_owned()))])?,
        &request
    )?);
    assert!(!evaluate_from(
        &predicate,
        &record(&[(selected_id, Value::Text("parent-b".to_owned()))])?,
        &request
    )?);
    assert!(matches!(
        evaluate_from(&predicate, &record(&[])?, &record(&[])?),
        Err(SweepError::Operation("row policy request parent is absent"))
    ));
    for value in [Value::Null, Value::Bool(true)] {
        assert!(matches!(
            evaluate_from(&predicate, &record(&[(selected_id, value)])?, &request),
            Err(SweepError::Operation(
                "row policy value is null or has invalid modeled type"
            ))
        ));
    }
    let required = RowPredicatePlan::SelectedRequestEquals {
        selected: Box::new([selected_id]),
        request: Box::new([request_id]),
        value_type: RowValueType::String,
        missing: MissingSelected::Reject,
    };
    assert!(matches!(
        evaluate_from(&required, &record(&[])?, &request),
        Err(SweepError::Operation(
            "row policy returned parent is absent"
        ))
    ));
    Ok(())
}

#[test]
fn given_public_zone_predicate_when_missing_null_string_or_true_then_only_literal_false_is_eligible()
-> TestResult {
    let id = field(1)?;
    let predicate = RowPredicatePlan::BooleanEquals {
        operand: RowOperandPlan {
            source: ContextSource::SelectedItem,
            path: Box::new([id]),
            value_type: RowValueType::Boolean,
        },
        value: false,
    };
    let request = record(&[])?;
    assert!(evaluate_from(
        &predicate,
        &record(&[(id, Value::Bool(false))])?,
        &request
    )?);
    assert!(!evaluate_from(
        &predicate,
        &record(&[(id, Value::Bool(true))])?,
        &request
    )?);
    assert!(matches!(
        evaluate_from(&predicate, &record(&[])?, &request),
        Err(SweepError::Operation("row policy boolean is absent"))
    ));
    for value in [Value::Null, Value::Text("false".to_owned()), Value::Int(0)] {
        assert!(matches!(
            evaluate_from(&predicate, &record(&[(id, value)])?, &request),
            Err(SweepError::Operation(
                "row policy value is null or has invalid modeled type"
            ))
        ));
    }
    Ok(())
}

#[test]
fn given_nested_optional_operand_when_parent_is_absent_then_missing_is_not_confused_with_invalid_traversal()
-> TestResult {
    let parent = field(1)?;
    let leaf = field(2)?;
    let path = [parent, leaf];
    assert!(lookup(&record(&[])?, &path)?.is_none());
    let nested = Value::Map(BTreeMap::from([(
        Key::Field(leaf),
        Value::Text("owner".to_owned()),
    )]));
    let nested = record(&[(parent, nested)])?;
    assert!(matches!(
        lookup(&nested, &path)?,
        Some(ValueRef::Text("owner"))
    ));
    for invalid in [
        Value::Null,
        Value::Text("not-a-structure".to_owned()),
        Value::List(Vec::new()),
    ] {
        assert!(matches!(
            lookup(&record(&[(parent, invalid)])?, &path),
            Err(SweepError::Operation(
                "row policy path traverses a null or non-structure value"
            ))
        ));
    }
    Ok(())
}

#[test]
fn given_whole_variant_absence_when_an_empty_opposite_variant_is_present_then_it_is_not_absent()
-> TestResult {
    let variant = field(1)?;
    let predicate = RowPredicatePlan::Absent(RowOperandPlan {
        source: ContextSource::SelectedItem,
        path: Box::new([variant]),
        value_type: RowValueType::Structure,
    });
    let request = record(&[])?;
    assert!(evaluate_from(&predicate, &record(&[])?, &request)?);
    assert!(!evaluate_from(
        &predicate,
        &record(&[(variant, Value::Map(BTreeMap::new()))])?,
        &request
    )?);
    let populated = Value::Map(BTreeMap::from([(
        Key::Field(field(2)?),
        Value::Text("foreign".to_owned()),
    )]));
    assert!(!evaluate_from(
        &predicate,
        &record(&[(variant, populated)])?,
        &request
    )?);
    for invalid in [
        Value::Null,
        Value::Text("project".to_owned()),
        Value::List(Vec::new()),
        Value::Bool(false),
    ] {
        assert!(matches!(
            evaluate_from(&predicate, &record(&[(variant, invalid)])?, &request),
            Err(SweepError::Operation(
                "row policy value is null or has invalid modeled type"
            ))
        ));
    }
    Ok(())
}

#[test]
fn given_base_and_branch_policies_when_either_owns_false_then_neither_can_override_it_or_hide_an_assertion()
-> TestResult {
    let owner = field(1)?;
    let returned = field(2)?;
    let parent = field(3)?;
    let absent = |id| {
        RowPredicatePlan::Absent(RowOperandPlan {
            source: ContextSource::SelectedItem,
            path: Box::new([id]),
            value_type: RowValueType::String,
        })
    };
    let policy = |owned| RowPolicyPlan {
        owned: Some(owned),
        ..RowPolicyPlan::default()
    };
    let base = policy(absent(owner));
    let branch = policy(absent(returned));
    let request = record(&[(parent, Value::Text("parent-a".to_owned()))])?;
    for (fields, expected) in [
        (Vec::new(), true),
        (vec![(owner, Value::Text("provider".to_owned()))], false),
        (vec![(returned, Value::Text("parent-b".to_owned()))], false),
    ] {
        let selected = record(&fields)?;
        let read = |source, path: &[FieldId]| {
            lookup(
                if source == ContextSource::SelectedItem {
                    &selected
                } else {
                    &request
                },
                path,
            )
        };
        assert_eq!(select([&base, &branch].into_iter(), &read)?, expected);
    }
    let mut asserted = branch.clone();
    asserted.assertions = vec![RowPredicatePlan::SelectedRequestEquals {
        selected: Box::new([returned]),
        request: Box::new([parent]),
        value_type: RowValueType::String,
        missing: MissingSelected::Reject,
    }]
    .into_boxed_slice();
    let selected = record(&[
        (owner, Value::Text("provider".to_owned())),
        (returned, Value::Text("parent-b".to_owned())),
    ])?;
    let read = |source, path: &[FieldId]| {
        lookup(
            if source == ContextSource::SelectedItem {
                &selected
            } else {
                &request
            },
            path,
        )
    };
    for policies in [[&base, &asserted], [&asserted, &base]] {
        assert!(matches!(
            select(policies.into_iter(), &read),
            Err(SweepError::Operation(
                "returned row disagrees with its bound request parent"
            ))
        ));
    }
    Ok(())
}
