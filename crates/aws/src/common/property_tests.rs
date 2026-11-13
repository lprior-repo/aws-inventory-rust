//! Executable retry and pagination contracts. Fixed seeds make failures replayable.

use std::sync::atomic::{AtomicUsize, Ordering};

use aws_sdk_sts::config::http::HttpResponse;
use aws_smithy_types::{body::SdkBody, error::ErrorMetadata};
use inventory_engine::limiter::ExecutorConfig;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestCaseError};

use super::*;

fn configuration() -> Config {
    Config {
        cases: 128,
        max_shrink_iters: 4_096,
        rng_seed: RngSeed::Fixed(0x4157_535f_5245_5452),
        ..Config::default()
    }
}

fn hints() -> impl Strategy<Value = Option<RetryAfter>> {
    prop_oneof![
        Just(None),
        Just(Some(RetryAfter::Invalid)),
        Just(Some(RetryAfter::Excessive)),
        (0..=MAX_RETRY_AFTER_MS)
            .prop_map(|value| Some(RetryAfter::Delay(Duration::from_millis(value)))),
    ]
}

#[derive(Debug)]
struct ModeledError {
    metadata: ErrorMetadata,
    kind: Option<ErrorKind>,
}

impl ProvideErrorMetadata for ModeledError {
    fn meta(&self) -> &ErrorMetadata {
        &self.metadata
    }
}

impl ProvideErrorKind for ModeledError {
    fn retryable_error_kind(&self) -> Option<ErrorKind> {
        self.kind
    }

    fn code(&self) -> Option<&str> {
        self.metadata.code()
    }
}

fn modeled(
    status: u16,
    code: Option<&str>,
    kind: Option<ErrorKind>,
    message: &str,
) -> Result<SdkError<ModeledError>, TestCaseError> {
    let response = HttpResponse::new(
        status
            .try_into()
            .map_err(|error| TestCaseError::fail(format!("invalid fixture status: {error:?}")))?,
        SdkBody::empty(),
    );
    let builder = ErrorMetadata::builder().message(message);
    let metadata = match code {
        Some(code) => builder.code(code).build(),
        None => builder.build(),
    };
    Ok(SdkError::service_error(
        ModeledError { metadata, kind },
        response,
    ))
}

fn varied_case(value: &str, mask: u64) -> String {
    value
        .bytes()
        .enumerate()
        .map(|(index, byte)| {
            char::from(if mask & (1_u64 << (index % 64)) == 0 {
                byte.to_ascii_lowercase()
            } else {
                byte.to_ascii_uppercase()
            })
        })
        .collect()
}

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
        let date = DateTime::from_secs(epoch_seconds + offset_seconds)
            .fmt(Format::HttpDate).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let now = UNIX_EPOCH + Duration::from_secs(1_794_470_400) + Duration::from_millis(elapsed_millis);
        let difference = i128::from(offset_seconds) * 1_000 - i128::from(elapsed_millis);
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
        prop_assert_eq!(header_hints(std::iter::repeat_n(text.as_bytes(), MAX_RETRY_HEADER_VALUES + extra), true, UNIX_EPOCH), Some(RetryAfter::Invalid));
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

    #[test]
    fn success_status_decode_failure_is_terminal_and_does_not_disclose_the_error_body(
        status in 200_u16..300,
        suffix in "[a-z0-9]{1,64}",
    ) {
        let secret = format!("response-body-canary-{suffix}");
        let error = classify_sdk_error(modeled(status, None, None, &secret)?);
        match error {
            ApiError::Permanent(message) => {
                prop_assert!(message.contains("response decoding"));
                prop_assert!(!message.contains(&secret));
            }
            other => prop_assert!(false, "success response was misclassified: {other:?}"),
        }
    }

    #[test]
    fn service_diagnostics_are_bounded_by_characters_without_splitting_utf8(message in prop::collection::vec(any::<char>(), 1_010..1_100)) {
        let text: String = message.into_iter().collect();
        let error = classify_sdk_error(modeled(400, Some("FixtureError"), None, &text)?);
        match error {
            ApiError::Permanent(detail) => {
                prop_assert_eq!(detail.chars().count(), 1_024);
                prop_assert!(detail.starts_with("FixtureError: "));
                let expected: String = format!("FixtureError: {text}").chars().take(1_024).collect();
                prop_assert_eq!(detail, expected);
            }
            other => prop_assert!(false, "unexpected classification: {other:?}"),
        }
    }

    #[test]
    fn child_budget_rejects_both_scope_and_single_page_overflow(count in 0_usize..25_000, remaining in 0_usize..150_000) {
        let allowed = count <= remaining && count <= inventory_core::MAX_PAGE_RESOURCES;
        prop_assert_eq!(require_budget(count, remaining).is_ok(), allowed);
        if count > remaining {
            prop_assert!(matches!(require_budget(count, remaining), Err(AwsScanError::ChildLimit)));
        }
    }

    #[test]
    fn terminal_empty_page_completes_at_the_exact_page_limit(tokens in prop::collection::btree_set("[a-z0-9]{1,16}", 0..12)) {
        let mut cursor = Cursor::default();
        let cap = u32::try_from(tokens.len() + 1).map_err(|error| TestCaseError::fail(error.to_string()))?;
        for token in &tokens {
            cursor.advance(&PageBody { next_token: Some(token.clone()), ..PageBody::default() }, cap);
            prop_assert!(!cursor.done);
            prop_assert!(cursor.pending_failure.is_none());
        }
        cursor.advance(&PageBody::default(), cap);
        prop_assert!(cursor.done);
        prop_assert!(cursor.pending_failure.is_none());
        prop_assert_eq!(cursor.page_number, cap);
        prop_assert_eq!(cursor.token, None);
        prop_assert_eq!(cursor.seen.len(), tokens.len());
        prop_assert_eq!(cursor.token_bytes, tokens.iter().map(String::len).sum::<usize>());
    }

    #[test]
    fn any_previously_seen_cursor_is_a_cycle(tokens in prop::collection::btree_set("[a-z0-9]{1,16}", 1..12), repeat in any::<usize>()) {
        let mut cursor = Cursor::default();
        for token in &tokens {
            cursor.advance(&PageBody { next_token: Some(token.clone()), ..PageBody::default() }, 100);
        }
        let replay = tokens.iter().nth(repeat % tokens.len()).ok_or_else(|| TestCaseError::fail("missing replay token"))?;
        cursor.advance(&PageBody { next_token: Some(replay.clone()), ..PageBody::default() }, 100);
        prop_assert!(matches!(cursor.pending_failure, Some(("pagination_cycle", _))));
        prop_assert_eq!(cursor.seen.len(), tokens.len());
        prop_assert_eq!(cursor.token.as_deref(), Some(replay.as_str()));
    }

    #[test]
    fn page_cap_preserves_a_nonterminal_checkpoint(cap in 1_u32..32) {
        let mut cursor = Cursor::default();
        for page in 1..=cap {
            cursor.advance(&PageBody { next_token: Some(format!("page-{page}")), ..PageBody::default() }, cap);
        }
        prop_assert_eq!(cursor.page_number, cap);
        prop_assert_eq!(cursor.token, Some(format!("page-{cap}")));
        prop_assert!(!cursor.done);
        prop_assert!(matches!(cursor.pending_failure, Some(("page_limit", _))));
        prop_assert_eq!(cursor.seen.len(), usize::try_from(cap - 1).map_err(|error| TestCaseError::fail(error.to_string()))?);
    }

    #[test]
    fn continuation_limit_counts_utf8_bytes(characters in 4_097_usize..4_130) {
        let token = "λ".repeat(characters);
        prop_assert!(token.chars().count() < MAX_TOKEN_BYTES);
        let mut cursor = Cursor::default();
        cursor.advance(&PageBody { next_token: Some(token), ..PageBody::default() }, 10);
        prop_assert!(matches!(cursor.pending_failure, Some(("invalid_pagination", _))));
        prop_assert!(cursor.seen.is_empty());
        prop_assert_eq!(cursor.token_bytes, 0);
    }

    #[test]
    fn fetch_failure_preserves_all_preceding_nonterminal_pages(preceding in 0_u8..8, denied in any::<bool>()) {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(16);
        let ctx = Context {
            config: SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default()).map_err(|error| TestCaseError::fail(error.to_string()))?),
            sender,
            account: "123456789012".into(), partition: "aws".into(), region: "us-east-1".into(),
            max_pages: 20, page_size: 100,
        };
        let calls = AtomicUsize::new(0);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_time().start_paused(true).build()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        runtime.block_on(paginate(&ctx, ctx.scope("property_fixture", None, Source::Direct), |_, remaining| {
            let index = calls.fetch_add(1, Ordering::SeqCst);
            assert!(index <= usize::from(preceding), "driver fetched after terminal failure");
            assert_eq!(remaining, MAX_CHILDREN);
            async move {
                if index == usize::from(preceding) {
                    return Err(AwsScanError::Api(if denied { ApiError::Denied("fixture".into()) } else { ApiError::Timeout }));
                }
                Ok(PageBody { next_token: Some(format!("page-{index}")), ..PageBody::default() })
            }
        })).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let events: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        prop_assert_eq!(calls.load(Ordering::SeqCst), usize::from(preceding) + 1);
        prop_assert_eq!(events.len(), usize::from(preceding) + 1);
        prop_assert_eq!(events.iter().filter(|event| matches!(event, CollectionEvent::Page(page) if page.next_token.is_some())).count(), usize::from(preceding));
        let last = events.last().ok_or_else(|| TestCaseError::fail("missing failure event"))?;
        let category = if denied { "access_denied" } else { "timeout" };
        prop_assert!(matches!(last, CollectionEvent::Failure(failure) if failure.category == category));
    }
}

#[test]
fn continuation_history_rejects_growth_after_its_exact_byte_budget() {
    let mut cursor = Cursor::default();
    let count = MAX_TOKEN_TOTAL_BYTES / MAX_TOKEN_BYTES;
    for index in 0..count {
        let token = format!("{index:08x}{}", "x".repeat(MAX_TOKEN_BYTES - 8));
        cursor.advance(
            &PageBody {
                next_token: Some(token),
                ..PageBody::default()
            },
            MAX_SCAN_PAGES,
        );
        assert!(cursor.pending_failure.is_none());
    }
    assert_eq!(cursor.token_bytes, MAX_TOKEN_TOTAL_BYTES);
    assert_eq!(cursor.seen.len(), count);
    cursor.advance(
        &PageBody {
            next_token: Some("next".into()),
            ..PageBody::default()
        },
        MAX_SCAN_PAGES,
    );
    assert!(matches!(
        cursor.pending_failure,
        Some(("token_memory_limit", _))
    ));
    assert_eq!(cursor.token_bytes, MAX_TOKEN_TOTAL_BYTES);
    assert_eq!(cursor.seen.len(), count);
    assert_eq!(cursor.token.as_deref(), Some("next"));
}
