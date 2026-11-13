//! seconds are exact with bounded optional whitespace properties.
use super::*;

proptest! {
    #![proptest_config(configuration())]

    #[test]
    fn seconds_are_exact_with_bounded_optional_whitespace(
        seconds in 0_u64..=MAX_RETRY_AFTER_MS / 1_000,
        leading in 0_usize..8,
        trailing in 0_usize..8,
        zeros in 0_usize..12,
    ) {
        let text = format!("{}{}{seconds}{}", " ".repeat(leading), "0".repeat(zeros), "\t".repeat(trailing));
        prop_assert_eq!(parse_retry_after_seconds(&text), RetryAfter::Delay(Duration::from_secs(seconds)));
    }

    #[test]
    fn integer_seconds_cannot_overflow_or_cross_the_delay_ceiling(value in any::<u64>()) {
        let expected = match value.checked_mul(1_000) {
            Some(milliseconds) if milliseconds <= MAX_RETRY_AFTER_MS => RetryAfter::Delay(Duration::from_millis(milliseconds)),
            _ => RetryAfter::Excessive,
        };
        prop_assert_eq!(parse_retry_after_seconds(&value.to_string()), expected);
    }

    #[test]
    fn aws_millisecond_header_is_not_interpreted_as_seconds(milliseconds in 301_u64..=MAX_RETRY_AFTER_MS) {
        let text = milliseconds.to_string();
        prop_assert_eq!(header_hints([text.as_bytes()].into_iter(), true, UNIX_EPOCH), Some(RetryAfter::Delay(Duration::from_millis(milliseconds))));
        prop_assert_eq!(parse_retry_after_seconds(&text), RetryAfter::Excessive);
    }

    #[test]
    fn oversized_decimal_hints_remain_invalid_even_when_the_value_is_small(length in 129_usize..512) {
        prop_assert_eq!(parse_retry_after_seconds(&"0".repeat(length)), RetryAfter::Invalid);
    }

    #[test]
    fn non_decimal_suffixes_are_never_partial_numbers(
        value in any::<u64>(),
        suffix in prop::sample::select(vec!["x", ".0", "+1", "-1", "\n", "\r", "\0", "é", "１"]),
    ) {
        prop_assert_eq!(parse_retry_after_seconds(&format!("{value}{suffix}")), RetryAfter::Invalid);
    }

    #[test]
    fn http_dates_preserve_fractional_waits_and_clamp_past_deadlines(
        offset_seconds in -600_i64..=600,
        elapsed_millis in 0_u64..1_000,
    ) {
        let epoch_seconds = 1_794_470_400_i64;
        let date = DateTime::from_secs(
            epoch_seconds
                .checked_add(offset_seconds)
                .ok_or_else(|| TestCaseError::fail("epoch seconds overflow"))?,
        )
        .fmt(Format::HttpDate).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let now = UNIX_EPOCH
            .checked_add(Duration::from_secs(1_794_470_400))
            .and_then(|time| time.checked_add(Duration::from_millis(elapsed_millis)))
            .ok_or_else(|| TestCaseError::fail("system clock overflow"))?;
        let difference = i128::from(offset_seconds)
            .checked_mul(1_000)
            .and_then(|value| value.checked_sub(i128::from(elapsed_millis)))
            .ok_or_else(|| TestCaseError::fail("wait difference overflow"))?;
        let expected = if difference <= 0 {
            RetryAfter::Delay(Duration::ZERO)
        } else if difference > i128::from(MAX_RETRY_AFTER_MS) {
            RetryAfter::Excessive
        } else {
            RetryAfter::Delay(Duration::from_millis(u64::try_from(difference).map_err(|error| TestCaseError::fail(error.to_string()))?))
        };
        prop_assert_eq!(parse_standard_retry_after(&date, now), expected);
    }

    #[test]
    fn combining_hints_is_commutative_associative_and_idempotent(left in hints(), middle in hints(), right in hints()) {
        prop_assert_eq!(combine_hints(left, right), combine_hints(right, left));
        prop_assert_eq!(combine_hints(combine_hints(left, middle), right), combine_hints(left, combine_hints(middle, right)));
        prop_assert_eq!(combine_hints(left, left), left);
        prop_assert_eq!(combine_hints(left, None), left);
        prop_assert_eq!(combine_hints(None, left), left);
    }

    #[test]
    fn invalid_and_excessive_directives_cannot_be_hidden_by_a_shorter_valid_hint(delay in 0_u64..=MAX_RETRY_AFTER_MS) {
        let valid = Some(RetryAfter::Delay(Duration::from_millis(delay)));
        prop_assert_eq!(combine_hints(valid, Some(RetryAfter::Invalid)), Some(RetryAfter::Invalid));
        prop_assert_eq!(combine_hints(valid, Some(RetryAfter::Excessive)), Some(RetryAfter::Excessive));
        prop_assert_eq!(combine_hints(Some(RetryAfter::Invalid), Some(RetryAfter::Excessive)), Some(RetryAfter::Excessive));
    }

    #[test]
    fn repeated_valid_headers_select_the_longest_delay(values in prop::collection::vec(0_u64..=MAX_RETRY_AFTER_MS, 0..=MAX_RETRY_HEADER_VALUES)) {
        let text: Vec<String> = values.iter().map(ToString::to_string).collect();
        let actual = header_hints(text.iter().map(|value| value.as_bytes()), true, UNIX_EPOCH);
        let expected = values.iter().copied().max().map(|value| RetryAfter::Delay(Duration::from_millis(value)));
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn header_count_and_encoding_limits_are_fail_closed(extra in 1_usize..16, valid in 0_u64..=MAX_RETRY_AFTER_MS) {
        let text = valid.to_string();
        prop_assert_eq!(header_hints(std::iter::repeat_n(text.as_bytes(), MAX_RETRY_HEADER_VALUES.saturating_add(extra)), true, UNIX_EPOCH), Some(RetryAfter::Invalid));
        prop_assert_eq!(header_hints([text.as_bytes(), &[0xff], text.as_bytes()].into_iter(), true, UNIX_EPOCH), Some(RetryAfter::Invalid));
    }

    #[test]
    fn hints_cannot_make_terminal_failures_retryable(hint in hints(), message in "[a-z0-9]{0,32}") {
        for error in [ApiError::Denied(message.clone()), ApiError::Permanent(message.clone()), ApiError::Cancelled, ApiError::Exhausted(message.clone())] {
            prop_assert_eq!(attach_hint(error.clone(), hint), error);
        }
    }

    #[test]
    fn retryable_classes_keep_their_kind_when_a_directive_is_attached(hint in hints(), message in "[a-z0-9]{0,32}") {
        for (error, kind) in [(ApiError::Throttled(message.clone()), RetryKind::Throttled), (ApiError::Transient(message.clone()), RetryKind::Transient), (ApiError::Timeout, RetryKind::Timeout)] {
            let actual = attach_hint(error.clone(), hint);
            match hint {
                Some(expected) => prop_assert!(matches!(actual, ApiError::RetryAfter { kind: actual_kind, retry_after, .. } if actual_kind == kind && retry_after == expected), "directive changed its retry kind or delay"),
                None => prop_assert_eq!(actual, error),
            }
        }
    }

    #[test]
    fn throttling_error_codes_are_ascii_case_insensitive(
        code in prop::sample::select(vec!["ThrottlingException", "TooManyRequestsException", "RequestLimitExceeded", "SlowDown"]),
        mask in any::<u64>(),
    ) {
        let code = varied_case(code, mask);
        prop_assert!(matches!(classify_sdk_error(modeled(400, Some(&code), None, "fixture")?), ApiError::Throttled(_)));
    }

    #[test]
    fn authentication_error_codes_are_ascii_case_insensitive(
        code in prop::sample::select(vec!["AccessDeniedException", "UnauthorizedOperation", "UnrecognizedClientException", "ExpiredTokenException", "InvalidClientTokenId", "AuthFailure"]),
        mask in any::<u64>(),
    ) {
        let code = varied_case(code, mask);
        prop_assert!(matches!(classify_sdk_error(modeled(400, Some(&code), None, "fixture")?), ApiError::Denied(_)));
    }

    #[test]
    fn timeout_and_server_codes_retain_distinct_retry_classes(mask in any::<u64>()) {
        let timeout = varied_case("RequestTimeoutException", mask);
        let unavailable = varied_case("ServiceUnavailableException", mask);
        let internal = varied_case("InternalFailure", mask);
        prop_assert_eq!(classify_sdk_error(modeled(400, Some(&timeout), None, "fixture")?), ApiError::Timeout);
        prop_assert!(matches!(classify_sdk_error(modeled(400, Some(&unavailable), None, "fixture")?), ApiError::Transient(_)));
        prop_assert!(matches!(classify_sdk_error(modeled(400, Some(&internal), None, "fixture")?), ApiError::Transient(_)));
    }

    #[test]
    fn modeled_retry_traits_apply_without_matching_error_names(kind in prop::sample::select(vec![ErrorKind::ThrottlingError, ErrorKind::ServerError, ErrorKind::TransientError])) {
        let actual = classify_sdk_error(modeled(400, Some("FixtureError"), Some(kind), "fixture")?);
        match kind {
            ErrorKind::ThrottlingError => prop_assert!(matches!(actual, ApiError::Throttled(_))),
            _ => prop_assert!(matches!(actual, ApiError::Transient(_))),
        }
    }

    #[test]
    fn unsupported_operations_stay_terminal_despite_retry_metadata(
        kind in prop::option::of(prop::sample::select(vec![ErrorKind::ThrottlingError, ErrorKind::ServerError, ErrorKind::TransientError])),
        code in prop::sample::select(vec!["NotImplemented", "ThrottlingException", "ServiceUnavailable", "AccessDenied"]),
    ) {
        prop_assert!(matches!(classify_sdk_error(modeled(501, Some(code), kind, "fixture")?), ApiError::Permanent(_)));
    }

    #[test]
    fn ordinary_server_statuses_retry_but_unknown_client_errors_do_not(
        server_status in (500_u16..=599).prop_filter("501 is unsupported, not transient", |status| *status != 501),
        client_status in prop::sample::select(vec![400_u16, 404, 409, 422]),
        suffix in "[a-f0-9]{0,32}",
    ) {
        let code = format!("Fixture{suffix}");
        prop_assert!(matches!(classify_sdk_error(modeled(server_status, Some(&code), None, "fixture")?), ApiError::Transient(_)));
        prop_assert!(matches!(classify_sdk_error(modeled(client_status, Some(&code), None, "fixture")?), ApiError::Permanent(_)));
    }

}
