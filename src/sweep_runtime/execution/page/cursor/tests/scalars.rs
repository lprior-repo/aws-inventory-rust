use super::*;

#[test]
fn given_integer_and_boolean_tuple_when_resuming_then_native_types_are_retained() -> TestResult {
    let mut plan = plan()?;
    let mut integer = token(1, 11)?;
    integer.value_type = TokenType::Integer;
    let mut boolean = token(2, 12)?;
    boolean.value_type = TokenType::Boolean;
    plan.tokens = vec![integer, boolean].into_boxed_slice();
    let response = record([
        (11, Value::Int(17)),
        (12, Value::Bool(false)),
        (14, Value::Bool(true)),
    ])?;
    let next = advance(&plan, &response)?;
    let fields = restored(&next)?.request_fields(&plan)?;
    let input = input::merge_input(&empty_record()?, &fields)?;
    let expected = record([(1, Value::Int(17)), (2, Value::Bool(false))])?;
    assert_eq!(input.as_bytes(), expected.as_bytes());
    Ok(())
}

#[test]
fn given_empty_optional_component_when_decoding_then_required_tuple_continues() -> TestResult {
    let plan = plan()?;
    for optional in [Value::Null, Value::Text(String::new())] {
        let response = record([
            (11, Value::Text("name".to_owned())),
            (12, Value::Text("A".to_owned())),
            (13, optional),
            (14, Value::Bool(true)),
        ])?;
        let next = advance(&plan, &response)?;
        assert_eq!(restored(&next)?.request_fields(&plan)?.len(), 2);
    }
    Ok(())
}

#[test]
fn given_empty_required_component_when_truncated_then_page_is_rejected() -> TestResult {
    let plan = plan()?;
    for missing in [Value::Null, Value::Text(String::new())] {
        let response = record([
            (11, missing),
            (12, Value::Text("A".to_owned())),
            (14, Value::Bool(true)),
        ])?;
        assert!(advance(&plan, &response).is_err());
    }
    Ok(())
}

#[test]
fn given_typed_header_tokens_when_checkpointing_then_integer_and_boolean_types_resume() -> TestResult
{
    let mut plan = plan()?;
    let mut integer = token(1, 11)?;
    integer.value_type = TokenType::Integer;
    integer.response_placement = TokenPlacement::Header("x-next-count".to_owned());
    let mut boolean = token(2, 12)?;
    boolean.value_type = TokenType::Boolean;
    boolean.response_placement = TokenPlacement::Header("x-next-flag".to_owned());
    plan.tokens = vec![integer, boolean].into_boxed_slice();
    let response = record([(14, Value::Bool(true))])?;
    let mut headers = http::HeaderMap::new();
    headers.insert("x-next-count", http::HeaderValue::from_static("42"));
    headers.insert("x-next-flag", http::HeaderValue::from_static("false"));
    let next = Cursor::default()
        .next(&plan, &response, &headers, 10)?
        .ok_or("continuation")?;
    let fields = restored(&next)?.request_fields(&plan)?;
    let resumed = input::merge_input(&empty_record()?, &fields)?;
    assert_eq!(
        resumed.as_bytes(),
        record([(1, Value::Int(42)), (2, Value::Bool(false))])?.as_bytes()
    );
    for (name, invalid) in [("x-next-count", "1.5"), ("x-next-flag", "yes")] {
        let mut invalid_headers = headers.clone();
        invalid_headers.insert(name, http::HeaderValue::from_str(invalid)?);
        assert!(
            Cursor::default()
                .next(&plan, &response, &invalid_headers, 10)
                .is_err()
        );
    }
    Ok(())
}
