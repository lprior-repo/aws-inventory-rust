use super::*;
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
