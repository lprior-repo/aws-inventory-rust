use super::*;

#[test]
fn unsupported_endpoint_does_not_retry_and_decode_failure_does_not_expose_body()
-> Result<(), Box<dyn std::error::Error>> {
    let response = HttpResponse::new(501_u16.try_into()?, SdkBody::empty());
    let error: SdkError<ListFunctionsError> =
        SdkError::response_error("unsupported endpoint", response);
    assert!(matches!(sdk_error(error), ApiError::Permanent(_)));
    let response = HttpResponse::new(200_u16.try_into()?, SdkBody::empty());
    let error = SdkError::service_error(
        ListFunctionsError::unhandled(std::io::Error::other("sensitive-fixture-response")),
        response,
    );
    match sdk_error(error) {
        ApiError::Permanent(message) => {
            assert!(message.contains("response decoding"));
            assert!(!message.contains("sensitive-fixture-response"));
        }
        other => return Err(format!("unexpected decode classification: {other:?}").into()),
    }
    Ok(())
}

#[test]
fn retry_hint_seconds_and_milliseconds_have_distinct_exact_units() {
    for (text, expected) in [
        ("0", RetryAfter::Delay(Duration::ZERO)),
        (" 2\t", RetryAfter::Delay(Duration::from_secs(2))),
        ("300", RetryAfter::Delay(Duration::from_secs(300))),
        ("301", RetryAfter::Excessive),
        ("18446744073709551616", RetryAfter::Excessive),
        ("-1", RetryAfter::Invalid),
        ("+1", RetryAfter::Invalid),
        ("0.5", RetryAfter::Invalid),
        ("NaN", RetryAfter::Invalid),
        ("", RetryAfter::Invalid),
    ] {
        assert_eq!(parse_retry_after_seconds(text), expected);
    }
    assert_eq!(
        parse_retry_after_seconds(&"9".repeat(129)),
        RetryAfter::Invalid
    );
    assert_eq!(
        header_hints([b"1500".as_slice()].into_iter(), true, UNIX_EPOCH),
        Some(RetryAfter::Delay(Duration::from_millis(1_500)))
    );
    assert_eq!(
        header_hints([b"300001".as_slice()].into_iter(), true, UNIX_EPOCH),
        Some(RetryAfter::Excessive)
    );
}

#[test]
fn retry_hint_http_date_uses_an_injected_wall_clock_and_retains_subseconds()
-> Result<(), Box<dyn std::error::Error>> {
    let now = UNIX_EPOCH
        .checked_add(Duration::from_millis(1_500))
        .ok_or("test clock overflow")?;
    assert_eq!(
        parse_standard_retry_after("Thu, 01 Jan 1970 00:00:03 GMT", now),
        RetryAfter::Delay(Duration::from_millis(1_500))
    );
    assert_eq!(
        parse_standard_retry_after("Thu, 01 Jan 1970 00:00:01 GMT", now),
        RetryAfter::Delay(Duration::ZERO)
    );
    assert_eq!(
        parse_standard_retry_after("Thu, 01 Jan 1970 00:05:02 GMT", now),
        RetryAfter::Excessive
    );
    assert_eq!(
        parse_standard_retry_after("not an HTTP date", now),
        RetryAfter::Invalid
    );
    assert_eq!(
        parse_standard_retry_after("Sunday, 06-Nov-94 08:49:37 GMT", now),
        RetryAfter::Invalid
    );
    Ok(())
}

#[test]
fn retry_header_values_are_bounded_and_invalid_bytes_cannot_disappear() {
    let values = [b"1".as_slice(), b"3".as_slice(), b"2".as_slice()];
    assert_eq!(
        header_hints(values.into_iter(), false, UNIX_EPOCH),
        Some(RetryAfter::Delay(Duration::from_secs(3)))
    );
    assert_eq!(
        header_hints([b"2".as_slice(), &[0xff]].into_iter(), false, UNIX_EPOCH),
        Some(RetryAfter::Invalid)
    );
    assert_eq!(
        header_hints(
            std::iter::repeat_n(b"1".as_slice(), MAX_RETRY_HEADER_VALUES.saturating_add(1)),
            false,
            UNIX_EPOCH
        ),
        Some(RetryAfter::Invalid)
    );
    assert_eq!(header_hints(std::iter::empty(), false, UNIX_EPOCH), None);
}

#[test]
fn retry_headers_do_not_make_denied_or_permanent_errors_retryable() {
    for error in [
        ApiError::Denied("fixture denial".into()),
        ApiError::Permanent("fixture invalid request".into()),
    ] {
        assert_eq!(attach_hint(error.clone(), Some(RetryAfter::Invalid)), error);
    }
}

#[test]
fn generic_sdk_classifier_keeps_the_longest_header_and_original_retry_class()
-> Result<(), Box<dyn std::error::Error>> {
    for (status, expected_kind) in [
        (429_u16, RetryKind::Throttled),
        (503_u16, RetryKind::Transient),
    ] {
        let mut response = HttpResponse::new(status.try_into()?, SdkBody::empty());
        response.headers_mut().try_insert("Retry-After", "2")?;
        response
            .headers_mut()
            .try_insert("x-amz-retry-after", "2500")?;
        let error: SdkError<ListFunctionsError> =
            SdkError::response_error("fixture decode failure", response);
        assert!(
            matches!(sdk_error(error), ApiError::RetryAfter { kind, retry_after: RetryAfter::Delay(delay), .. }
            if kind == expected_kind && delay == Duration::from_millis(2_500))
        );
    }
    let mut response = HttpResponse::new(429_u16.try_into()?, SdkBody::empty());
    response.headers_mut().try_insert("retry-after", "bad")?;
    let error: SdkError<ListFunctionsError> =
        SdkError::response_error("fixture decode failure", response);
    assert!(matches!(
        sdk_error(error),
        ApiError::RetryAfter {
            kind: RetryKind::Throttled,
            retry_after: RetryAfter::Invalid,
            ..
        }
    ));
    Ok(())
}

#[test]
fn unmodeled_http_denial_with_retry_header_remains_terminal()
-> Result<(), Box<dyn std::error::Error>> {
    for status in [401_u16, 403_u16] {
        let mut response = HttpResponse::new(status.try_into()?, SdkBody::empty());
        response.headers_mut().try_insert("retry-after", "3")?;
        let error: SdkError<ListFunctionsError> =
            SdkError::response_error("fixture decode failure", response);
        assert!(matches!(sdk_error(error), ApiError::Denied(_)));
    }
    Ok(())
}

#[test]
fn empty_page_with_token_remains_open() {
    let mut cursor = Cursor::default();
    cursor.advance(
        &PageBody {
            next_token: Some("next".into()),
            ..PageBody::default()
        },
        10,
    );
    assert!(!cursor.done);
    assert!(cursor.pending_failure.is_none());
    assert_eq!(cursor.page_number, 1);
}

#[test]
fn detects_non_adjacent_cursor_cycle() {
    let mut cursor = Cursor::default();
    for token in ["a", "b", "a"] {
        cursor.advance(
            &PageBody {
                next_token: Some(token.into()),
                ..PageBody::default()
            },
            10,
        );
    }
    assert!(matches!(
        cursor.pending_failure,
        Some(("pagination_cycle", _))
    ));
}

#[test]
fn capped_page_keeps_nonterminal_token() {
    let mut cursor = Cursor::default();
    cursor.advance(
        &PageBody {
            next_token: Some("next".into()),
            ..PageBody::default()
        },
        1,
    );
    assert_eq!(cursor.token.as_deref(), Some("next"));
    assert!(matches!(cursor.pending_failure, Some(("page_limit", _))));
    assert!(!cursor.done);
}
