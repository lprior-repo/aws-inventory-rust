use super::*;

#[test]
fn given_reached_page_bound_when_tuple_continues_then_partial_checkpoint_is_retained() -> TestResult {
    let plan = plan()?;
    let first = advance(&plan, &response()?)?;
    let next = restored(&first)?.next(&plan, &response()?, &http::HeaderMap::new(), 2)?
        .ok_or("expected partial checkpoint")?;
    let cursor = restored(&next)?;
    assert_eq!(cursor.pages, 2);
    assert!(cursor.partial);
    assert_eq!(next.continuation_digest, first.continuation_digest);
    Ok(())
}

#[test]
fn given_oversized_tuple_when_checkpointing_then_partial_is_bounded() -> TestResult {
    let plan = plan()?;
    let response = record([(11, Value::Text("n".repeat(MAX_TOKEN_RECORD))),
        (12, Value::Text("A".to_owned())), (14, Value::Bool(true))])?;
    let next = advance(&plan, &response)?;
    assert!(next.cursor.as_bytes().len() <= MAX_CURSOR);
    assert!(restored(&next)?.partial);
    assert!(next.continuation_digest.is_some());
    Ok(())
}

#[test]
fn given_invalid_page_counter_when_resuming_then_cursor_is_rejected() -> TestResult {
    for pages in [0, -1, i64::MAX] {
        let cursor = record([(PAGES, Value::Int(pages)),
            (CURRENT, Value::Bytes(empty_record()?.into_bytes())), (PARTIAL, Value::Bool(false))])?;
        assert!(Cursor::decode(Some(cursor.as_bytes())).is_err());
    }
    Ok(())
}

#[test]
fn given_optional_header_when_absent_then_required_tuple_still_advances() -> TestResult {
    let mut plan = plan()?;
    let optional = plan.tokens.last_mut().ok_or("missing optional token")?;
    optional.response_placement = TokenPlacement::Header("x-next-id".to_owned());
    let next = advance(&plan, &response()?)?;
    assert_eq!(restored(&next)?.request_fields(&plan)?.len(), 2);
    let mut headers = http::HeaderMap::new();
    headers.append("x-next-id", http::HeaderValue::from_static("a"));
    headers.append("x-next-id", http::HeaderValue::from_static("b"));
    assert!(Cursor::default().next(&plan, &response()?, &headers, 10).is_err());
    Ok(())
}
