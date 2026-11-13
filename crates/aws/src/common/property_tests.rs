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

mod seconds_are_exact_with_bounded_optional_whitespace;
mod success_status_decode_failure_is_terminal_and_does_not_disclose_the_error_body;
