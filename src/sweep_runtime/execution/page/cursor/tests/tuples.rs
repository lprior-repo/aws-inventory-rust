use super::*;

#[test]
fn given_absent_optional_identifier_when_resuming_then_required_tuple_survives() -> TestResult {
    let plan = plan()?;
    let next = advance(&plan, &response()?)?;
    let cursor = restored(&next)?;
    let fields = cursor.request_fields(&plan)?;
    assert_eq!(fields.len(), 2);
    let input = input::merge_input(&empty_record()?, &fields)?;
    let expected = record([(1, Value::Text("name.example.".to_owned())), (2, Value::Text("A".to_owned()))])?;
    assert_eq!(input.as_bytes(), expected.as_bytes());
    assert_eq!(cursor.pages, 1);
    assert!(!cursor.partial);
    assert_eq!(next.continuation_digest, Some(token_digest(&expected)?));
    Ok(())
}

#[test]
fn given_changed_optional_identifier_when_digesting_then_entire_tuple_matters() -> TestResult {
    let plan = plan()?;
    let absent = advance(&plan, &response()?)?;
    let present = record([(11, Value::Text("name.example.".to_owned())),
        (12, Value::Text("A".to_owned())), (13, Value::Text("weighted".to_owned())),
        (14, Value::Bool(true))])?;
    let first = advance(&plan, &present)?;
    let duplicate = advance(&plan, &present)?;
    assert_ne!(first.continuation_digest, absent.continuation_digest);
    assert_eq!(first.continuation_digest, duplicate.continuation_digest);
    let fields = restored(&first)?.request_fields(&plan)?;
    assert_eq!(fields.len(), 3);
    let repeated = restored(&first)?.next(&plan, &present, &http::HeaderMap::new(), 10)?
        .ok_or("expected duplicate continuation for SQLite rejection")?;
    assert_eq!(first.continuation_digest, repeated.continuation_digest);
    Ok(())
}

#[test]
fn given_two_required_components_when_resuming_then_neither_is_lost() -> TestResult {
    let mut plan = plan()?;
    plan.tokens = vec![token(1, 11)?, token(2, 12)?].into_boxed_slice();
    let next = advance(&plan, &response()?)?;
    let fields = restored(&next)?.request_fields(&plan)?;
    assert_eq!(fields.len(), 2);
    assert_eq!(fields.first().map(|field| &field.value), Some(&Value::Text("name.example.".to_owned())));
    assert_eq!(fields.last().map(|field| &field.value), Some(&Value::Text("A".to_owned())));
    Ok(())
}

#[test]
fn given_no_truncated_flag_when_required_tuple_exists_then_page_advances() -> TestResult {
    let mut plan = plan()?;
    plan.truncated_path = None;
    let next = advance(&plan, &response()?)?;
    assert_eq!(restored(&next)?.request_fields(&plan)?.len(), 2);
    assert!(Cursor::default().next(&plan, &empty_record()?, &http::HeaderMap::new(), 10)?.is_none());
    Ok(())
}

#[test]
fn given_optional_identifier_disappears_when_advancing_then_previous_value_is_removed() -> TestResult {
    let plan = plan()?;
    let present = record([(11, Value::Text("name.example.".to_owned())),
        (12, Value::Text("A".to_owned())), (13, Value::Text("weighted".to_owned())),
        (14, Value::Bool(true))])?;
    let first = advance(&plan, &present)?;
    let next = restored(&first)?.next(&plan, &response()?, &http::HeaderMap::new(), 10)?
        .ok_or("expected next tuple")?;
    let fields = restored(&next)?.request_fields(&plan)?;
    assert_eq!(fields.len(), 2);
    assert!(!fields.iter().any(|field| field.path.first().is_some_and(|id| id.value() == 3)));
    Ok(())
}

#[test]
fn given_one_through_eight_components_when_checkpointing_then_every_native_value_resumes() -> TestResult {
    for count in 1..=8 {
        let mut plan = plan()?;
        plan.tokens = (1..=count).map(|id| token(id, id + 20))
            .collect::<Result<Vec<_>, _>>()?.into_boxed_slice();
        plan.truncated_path = None;
        let response = record((1..=count).map(|id| (id + 20, Value::Text(format!("token-{id}")))))?;
        let next = advance(&plan, &response)?;
        let fields = restored(&next)?.request_fields(&plan)?;
        let resumed = input::merge_input(&empty_record()?, &fields)?;
        let expected = record((1..=count).map(|id| (id, Value::Text(format!("token-{id}")))))?;
        assert_eq!(resumed.as_bytes(), expected.as_bytes());
        assert_eq!(next.continuation_digest, Some(token_digest(&expected)?));
    }
    Ok(())
}
