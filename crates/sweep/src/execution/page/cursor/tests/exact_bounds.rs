use super::*;

fn single_token_plan() -> Result<PagePlan, SweepError> {
    Ok(PagePlan {
        items: ItemPlan::Root,
        tokens: vec![token(1, 11)?].into_boxed_slice(),
        page_size: None,
        max_page_size: None,
        truncated_path: None,
    })
}

fn request(payload_bytes: usize) -> Result<Record, SweepError> {
    record([(1, Value::Text("t".repeat(payload_bytes)))])
}

fn persisted(current: &Record, partial: bool) -> Result<Record, SweepError> {
    record([
        (65_530, Value::Int(1)),
        (65_532, Value::Bytes(current.as_bytes().to_vec())),
        (65_533, Value::Bool(partial)),
    ])
}

#[test]
fn cursor_preserves_and_resumes_a_request_of_exactly_65000_encoded_bytes() -> TestResult {
    let expected = request(64_995)?;
    assert_eq!(expected.as_bytes().len(), 65_000);
    let expected_checkpoint = persisted(&expected, false)?;
    assert_eq!(expected_checkpoint.as_bytes().len(), 65_015);
    let plan = single_token_plan()?;
    let response = record([(11, Value::Text("t".repeat(64_995)))])?;
    let next = advance(&plan, &response)?;
    let resumed = restored(&next)?;
    let fields = resumed.request_fields(&plan)?;
    assert_eq!(next.cursor.as_bytes(), expected_checkpoint.as_bytes());
    assert_eq!(
        next.continuation_digest,
        Some(*expected.content_hash(&Exclusions::none())?.as_bytes())
    );
    assert_eq!(resumed.pages, 1);
    assert!(!resumed.partial);
    assert_eq!(
        resumed
            .current
            .as_ref()
            .ok_or("missing resumed request")?
            .as_bytes(),
        expected.as_bytes()
    );
    assert_eq!(fields.len(), 1);
    let field = fields.first().ok_or("missing resumed token")?;
    assert_eq!(field.path.as_ref(), &[FieldId::new(1)?]);
    assert_eq!(field.value, Value::Text("t".repeat(64_995)));
    Ok(())
}

#[test]
fn cursor_marks_a_65001_byte_encoded_request_partial_without_persisting_its_token() -> TestResult {
    let expected = request(64_996)?;
    assert_eq!(expected.as_bytes().len(), 65_001);
    let empty = Record::from_bytes(vec![0xa0])?;
    let expected_checkpoint = persisted(&empty, true)?;
    assert_eq!(expected_checkpoint.as_bytes().len(), 14);
    let plan = single_token_plan()?;
    let response = record([(11, Value::Text("t".repeat(64_996)))])?;
    let next = advance(&plan, &response)?;
    let resumed = restored(&next)?;
    assert_eq!(next.cursor.as_bytes(), expected_checkpoint.as_bytes());
    assert_eq!(
        next.continuation_digest,
        Some(*expected.content_hash(&Exclusions::none())?.as_bytes())
    );
    assert_eq!(resumed.pages, 1);
    assert!(resumed.partial);
    assert_eq!(
        resumed
            .current
            .as_ref()
            .ok_or("missing partial marker request")?
            .as_bytes(),
        &[0xa0]
    );
    Ok(())
}

fn rejects_with_operation(bytes: &[u8], expected: &'static str) -> TestResult {
    match Cursor::decode(Some(bytes)) {
        Err(SweepError::Operation(message)) => assert_eq!(message, expected),
        Err(error) => return Err(format!("expected Operation({expected:?}), got {error:?}").into()),
        Ok(_) => return Err("invalid persisted checkpoint unexpectedly resumed".into()),
    }
    Ok(())
}

#[test]
fn cursor_at_exactly_65536_bytes_reaches_the_nested_token_bound_not_the_outer_bound() -> TestResult
{
    let current = request(65_516)?;
    assert_eq!(current.as_bytes().len(), 65_521);
    let checkpoint = persisted(&current, false)?;
    assert_eq!(checkpoint.as_bytes().len(), 65_536);
    rejects_with_operation(
        checkpoint.as_bytes(),
        "persisted pagination cursor has invalid token",
    )
}

#[test]
fn cursor_rejects_65537_encoded_bytes_with_the_outer_bound_error_before_nested_parsing()
-> TestResult {
    let current = request(65_517)?;
    assert_eq!(current.as_bytes().len(), 65_522);
    let checkpoint = persisted(&current, false)?;
    assert_eq!(checkpoint.as_bytes().len(), 65_537);
    rejects_with_operation(
        checkpoint.as_bytes(),
        "persisted pagination cursor exceeds its bound",
    )
}
