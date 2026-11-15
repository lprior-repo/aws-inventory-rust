use super::*;

#[test]
fn given_truncated_true_when_required_token_is_missing_then_page_errors() -> TestResult {
    let plan = plan()?;
    for response in [record([(14, Value::Bool(true))])?,
        record([(11, Value::Text("name".to_owned())), (14, Value::Bool(true))])?] {
        assert!(advance(&plan, &response).is_err());
    }
    Ok(())
}

#[test]
fn given_truncated_false_when_stale_tokens_are_malformed_then_pagination_stops() -> TestResult {
    let plan = plan()?;
    let response = record([(11, Value::Int(7)), (13, Value::Bool(false)), (14, Value::Bool(false))])?;
    assert!(Cursor::default().next(&plan, &response, &http::HeaderMap::new(), 10)?.is_none());
    Ok(())
}

#[test]
fn given_optional_token_wrong_native_type_when_truncated_then_page_errors() -> TestResult {
    let plan = plan()?;
    for invalid in [Value::Int(7), Value::Bool(false), Value::List(Vec::new())] {
        let response = record([(11, Value::Text("name".to_owned())),
            (12, Value::Text("A".to_owned())), (13, invalid), (14, Value::Bool(true))])?;
        assert!(advance(&plan, &response).is_err());
    }
    Ok(())
}

#[test]
fn given_invalid_truncated_flag_when_decoding_then_page_errors() -> TestResult {
    let plan = plan()?;
    for response in [empty_record()?, record([(14, Value::Text("true".to_owned()))])?] {
        assert!(advance(&plan, &response).is_err());
    }
    Ok(())
}

#[test]
fn given_partial_tuple_without_truncated_flag_when_decoding_then_page_errors() -> TestResult {
    let mut plan = plan()?;
    plan.truncated_path = None;
    assert!(advance(&plan, &record([(11, Value::Text("name".to_owned()))])?).is_err());
    Ok(())
}

#[test]
fn given_corrupt_resume_tuple_when_preparing_request_then_no_fields_are_injected() -> TestResult {
    let plan = plan()?;
    for current in [record([(1, Value::Text("name".to_owned()))])?,
        record([(1, Value::Text("name".to_owned())), (2, Value::Text("A".to_owned())),
            (99, Value::Text("injected".to_owned()))])?,
        record([(1, Value::Text("name".to_owned())), (2, Value::Text("A".to_owned())), (3, Value::Int(5))])?] {
        let cursor = Cursor { pages: 1, current: Some(current), partial: false };
        assert!(cursor.request_fields(&plan).is_err());
    }
    Ok(())
}

#[test]
fn given_scalar_parent_in_optional_path_when_decoding_then_page_errors() -> TestResult {
    let mut plan = plan()?;
    let optional = plan.tokens.last_mut().ok_or("missing optional token")?;
    optional.response_path = Box::new([FieldId::new(13)?, FieldId::new(15)?]);
    optional.response_placement = TokenPlacement::Body(optional.response_path.clone());
    let response = record([(11, Value::Text("name".to_owned())),
        (12, Value::Text("A".to_owned())), (13, Value::Text("not a map".to_owned())),
        (14, Value::Bool(true))])?;
    assert!(advance(&plan, &response).is_err());
    Ok(())
}
