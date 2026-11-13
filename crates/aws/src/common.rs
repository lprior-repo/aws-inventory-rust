//! Bounded page traversal, provenance, and AWS error translation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aws_sdk_sts::error::{ProvideErrorMetadata, SdkError};
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
use aws_smithy_types::{DateTime, date_time::Format};
use aws_types::SdkConfig;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{
    CollectionEvent, CollectionFailure, CollectionPage, CollectionScope, Edge, Resource, Source,
};
use inventory_engine::limiter::{ApiError, ApiExecutor, MAX_RETRY_AFTER_MS, RetryAfter, RetryKind};
use serde_json::{Value, json};
use tokio::sync::mpsc::Sender;

use crate::AwsScanError;

pub(crate) const MAX_SCAN_PAGES: u32 = 10_000;
pub(crate) const MAX_CHILDREN: usize = 100_000;
const MAX_TOKEN_BYTES: usize = 8_192;
const MAX_TOKEN_TOTAL_BYTES: usize = 8_388_608;
const MAX_RETRY_HINT_BYTES: usize = 128;
const MAX_RETRY_HEADER_VALUES: usize = 8;

#[derive(Clone)]
pub(crate) struct Context {
    pub config: SdkConfig,
    pub executor: Arc<ApiExecutor>,
    pub sender: Sender<CollectionEvent>,
    pub account: String,
    pub partition: String,
    pub region: String,
    pub max_pages: u32,
    pub page_size: i32,
}

impl Context {
    pub fn scope(&self, collector: &str, parent: Option<&str>, source: Source) -> CollectionScope {
        CollectionScope {
            account_id: self.account.clone(),
            region: self.region.clone(),
            collector: collector.to_owned(),
            parent_id: parent.map(str::to_owned),
            source,
        }
    }

    #[tracing::instrument(skip_all, fields(service, quota_group = group))]
    pub async fn execute<T, F, Fut>(
        &self,
        service: &str,
        group: &str,
        operation: F,
    ) -> Result<T, ApiError>
    where
        T: Send,
        F: FnMut() -> Fut + Send,
        Fut: Future<Output = Result<T, ApiError>> + Send,
    {
        let bucket = format!(
            "{}:{}:{}:{}:{}",
            self.partition, self.account, self.region, service, group
        );
        self.executor.execute(&bucket, operation).await
    }

    pub fn ec2_arn(&self, resource: &str, id: &str) -> String {
        format!(
            "arn:{}:ec2:{}:{}:{}/{}",
            self.partition, self.region, self.account, resource, id
        )
    }

    pub fn arn(&self, service: &str, resource: &str) -> String {
        format!(
            "arn:{}:{}:{}:{}:{}",
            self.partition, service, self.region, self.account, resource
        )
    }

    pub fn resource(
        &self,
        kind: &str,
        id: String,
        name: Option<String>,
        properties: Value,
        tags: BTreeMap<String, String>,
    ) -> Result<Resource, AwsScanError> {
        Ok(Resource {
            id,
            account_id: self.account.clone(),
            region: self.region.clone(),
            kind: kind.to_owned(),
            name,
            properties,
            tags,
            source: Source::Direct,
            observed_at: now_millis()?,
        })
    }

    pub fn edge(&self, resource: &Resource, target: String, relation: &str) -> Edge {
        let properties = network_reference_evidence(&target);
        Edge {
            from_id: resource.id.clone(),
            to_id: target,
            relation: relation.to_owned(),
            source: resource.source,
            observed_at: resource.observed_at,
            properties,
        }
    }

    #[tracing::instrument(skip_all, fields(collector = scope.collector, category))]
    pub async fn failure(
        &self,
        scope: &CollectionScope,
        category: &str,
        message: impl Into<String>,
    ) -> Result<(), AwsScanError> {
        let message: String = message.into();
        tracing::warn!(collector = %scope.collector, category, "collection scope incomplete");
        self.sender
            .send(CollectionEvent::Failure(CollectionFailure {
                scope: scope.clone(),
                category: category.to_owned(),
                message: message.chars().take(2_048).collect(),
            }))
            .await
            .map_err(|_| AwsScanError::ChannelClosed)
    }
}

fn network_reference_evidence(target: &str) -> Value {
    if target.split(':').nth(2) != Some("ec2") {
        return json!({});
    }
    let Some((kind, id)) = target
        .splitn(6, ':')
        .nth(5)
        .and_then(|value| value.split_once('/'))
    else {
        return json!({});
    };
    let target_kind = match kind {
        "vpc" => "aws_vpc",
        "subnet" => "aws_vpc_subnet",
        "security-group" => "aws_vpc_security_group",
        "network-interface" => "aws_ec2_network_interface",
        _ => return json!({}),
    };
    json!({"target_resource_id":id,"target_kind":target_kind,
        "target_region":target.split(':').nth(3),
        "owner_resolution":"assumed_scanning_account"})
}

pub(crate) fn now_millis() -> Result<i64, AwsScanError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AwsScanError::InvalidData(error.to_string()))?;
    i64::try_from(elapsed.as_millis()).map_err(|error| AwsScanError::InvalidData(error.to_string()))
}

pub(crate) fn required(value: Option<&str>, field: &str) -> Result<String, AwsScanError> {
    value
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AwsScanError::InvalidData(format!("AWS response lacks {field}")))
}

pub(crate) fn require_budget(count: usize, remaining: usize) -> Result<(), AwsScanError> {
    if count > remaining {
        return Err(AwsScanError::ChildLimit);
    }
    if count > inventory_core::MAX_PAGE_RESOURCES {
        return Err(AwsScanError::InvalidData(
            "service page exceeds the maximum page resource count".into(),
        ));
    }
    Ok(())
}

pub(crate) fn sdk_error<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
) -> ApiError {
    sdk_error_with_hint(error, None)
}

/// Typed service metadata and raw headers are combined before the SDK response
/// is dropped. Malformed/oversized hints stay visible to the shared executor.
pub(crate) fn sdk_error_with_hint<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
    typed_hint: Option<RetryAfter>,
) -> ApiError {
    let now = SystemTime::now();
    let raw_hint = error.raw_response().and_then(|response| {
        let standard = header_hints(response.headers().get_all_bytes("retry-after"), false, now);
        let aws = header_hints(
            response.headers().get_all_bytes("x-amz-retry-after"),
            true,
            now,
        );
        combine_hints(standard, aws)
    });
    attach_hint(
        classify_sdk_error(error),
        combine_hints(raw_hint, typed_hint),
    )
}

fn attach_hint(error: ApiError, hint: Option<RetryAfter>) -> ApiError {
    let Some(retry_after) = hint else {
        return error;
    };
    let (kind, message) = match error {
        ApiError::Throttled(message) => (RetryKind::Throttled, message),
        ApiError::Transient(message) => (RetryKind::Transient, message),
        ApiError::Timeout => (RetryKind::Timeout, "HTTP or SDK attempt timeout".into()),
        other => return other,
    };
    ApiError::RetryAfter {
        kind,
        message,
        retry_after,
    }
}

fn combine_hints(left: Option<RetryAfter>, right: Option<RetryAfter>) -> Option<RetryAfter> {
    match (left, right) {
        (Some(RetryAfter::Excessive), _) | (_, Some(RetryAfter::Excessive)) => {
            Some(RetryAfter::Excessive)
        }
        (Some(RetryAfter::Invalid), _) | (_, Some(RetryAfter::Invalid)) => {
            Some(RetryAfter::Invalid)
        }
        (Some(RetryAfter::Delay(left)), Some(RetryAfter::Delay(right))) => {
            Some(RetryAfter::Delay(left.max(right)))
        }
        (left, None) => left,
        (None, right) => right,
    }
}

fn header_hints<'a>(
    values: impl Iterator<Item = &'a [u8]>,
    milliseconds: bool,
    now: SystemTime,
) -> Option<RetryAfter> {
    let mut result = None;
    for (index, bytes) in values
        .take(MAX_RETRY_HEADER_VALUES.saturating_add(1))
        .enumerate()
    {
        if index == MAX_RETRY_HEADER_VALUES {
            return Some(RetryAfter::Invalid);
        }
        let hint = match bounded_hint_text(bytes) {
            Some(value) if milliseconds => parse_delay_integer(value, 1),
            Some(value) => parse_standard_retry_after(value, now),
            None => RetryAfter::Invalid,
        };
        result = combine_hints(result, Some(hint));
    }
    result
}

fn bounded_hint_text(bytes: &[u8]) -> Option<&str> {
    if bytes.len() > MAX_RETRY_HINT_BYTES {
        return None;
    }
    std::str::from_utf8(bytes)
        .ok()
        .map(|value| value.trim_matches([' ', '\t']))
        .filter(|value| !value.is_empty())
}

pub(crate) fn parse_retry_after_seconds(value: &str) -> RetryAfter {
    match bounded_hint_text(value.as_bytes()) {
        Some(value) => parse_delay_integer(value, 1_000),
        None => RetryAfter::Invalid,
    }
}

fn parse_delay_integer(value: &str, milliseconds_per_unit: u64) -> RetryAfter {
    if value.is_empty()
        || value.len() > MAX_RETRY_HINT_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return RetryAfter::Invalid;
    }
    let milliseconds = value
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_mul(milliseconds_per_unit));
    match milliseconds {
        Some(value) if value <= MAX_RETRY_AFTER_MS => {
            RetryAfter::Delay(Duration::from_millis(value))
        }
        Some(_) | None => RetryAfter::Excessive,
    }
}

fn parse_standard_retry_after(value: &str, now: SystemTime) -> RetryAfter {
    if value.is_empty() || value.len() > MAX_RETRY_HINT_BYTES {
        return RetryAfter::Invalid;
    }
    if value.bytes().all(|byte| byte.is_ascii_digit()) {
        return parse_delay_integer(value, 1_000);
    }
    let date = match DateTime::from_str(value, Format::HttpDate) {
        Ok(date) => date,
        Err(_) => return RetryAfter::Invalid,
    };
    let deadline = match SystemTime::try_from(date) {
        Ok(deadline) => deadline,
        Err(_) => return RetryAfter::Excessive,
    };
    let delay = deadline.duration_since(now).unwrap_or_default();
    if delay <= Duration::from_millis(MAX_RETRY_AFTER_MS) {
        RetryAfter::Delay(delay)
    } else {
        RetryAfter::Excessive
    }
}

fn classify_sdk_error<E: ProvideErrorMetadata + ProvideErrorKind + Debug>(
    error: SdkError<E>,
) -> ApiError {
    let status = error
        .raw_response()
        .map(|response| response.status().as_u16());
    if let Some(service) = error.as_service_error() {
        if status.is_some_and(|status| (200..300).contains(&status))
            && ProvideErrorMetadata::code(service).is_none()
        {
            return ApiError::Permanent(
                "SDK rejected an HTTP success response without a modeled error code; check response decoding and protocol compatibility".into(),
            );
        }
        let code = ProvideErrorMetadata::code(service).unwrap_or("UnknownServiceError");
        let detail = format!(
            "{}: {}",
            code,
            service.message().unwrap_or("no service message")
        );
        let detail: String = detail.chars().take(1_024).collect();
        if status == Some(501) {
            return ApiError::Permanent(detail);
        }
        let normalized = code.to_ascii_lowercase();
        if matches!(
            service.retryable_error_kind(),
            Some(ErrorKind::ThrottlingError)
        ) || status == Some(429)
            || normalized.contains("throttl")
            || normalized.contains("toomanyrequests")
            || normalized.contains("requestlimitexceeded")
            || normalized.contains("slowdown")
        {
            return ApiError::Throttled(detail);
        }
        if status == Some(408) || normalized.contains("requesttimeout") {
            return ApiError::Timeout;
        }
        if matches!(status, Some(401 | 403))
            || normalized.contains("accessdenied")
            || normalized.contains("unauthorized")
            || normalized.contains("unrecognizedclient")
            || normalized.contains("expiredtoken")
            || normalized.contains("invalidclienttoken")
            || normalized.contains("authfailure")
        {
            return ApiError::Denied(detail);
        }
        if matches!(
            service.retryable_error_kind(),
            Some(ErrorKind::ServerError | ErrorKind::TransientError)
        ) || status.is_some_and(|status| status >= 500)
            || normalized.contains("serviceunavailable")
            || normalized.contains("internal")
        {
            return ApiError::Transient(detail);
        }
        return ApiError::Permanent(detail);
    }
    if status == Some(429) {
        return ApiError::Throttled("HTTP 429 without a modeled service error".to_owned());
    }
    if status == Some(408) {
        return ApiError::Timeout;
    }
    if matches!(status, Some(401 | 403)) {
        return ApiError::Denied(
            "HTTP authentication or authorization failure without a modeled service error".into(),
        );
    }
    if status == Some(501) {
        return ApiError::Permanent("HTTP 501 Not Implemented".into());
    }
    match error {
        SdkError::TimeoutError(_) => ApiError::Timeout,
        SdkError::DispatchFailure(_) | SdkError::ResponseError(_) => {
            ApiError::Transient("SDK transport or response failure".to_owned())
        }
        _ => ApiError::Permanent("SDK request construction failure".to_owned()),
    }
}

pub(crate) fn error_category(error: &AwsScanError) -> &'static str {
    match error {
        AwsScanError::Api(ApiError::Denied(_)) => "access_denied",
        AwsScanError::Api(ApiError::Cancelled) => "cancelled",
        AwsScanError::Api(ApiError::Throttled(_)) => "throttled",
        AwsScanError::Api(ApiError::RetryAfter {
            kind: RetryKind::Throttled,
            ..
        }) => "throttled",
        AwsScanError::Api(ApiError::RetryAfter {
            kind: RetryKind::Timeout,
            ..
        }) => "timeout",
        AwsScanError::Api(ApiError::RetryAfterRejected(_)) => "retry_after_rejected",
        AwsScanError::Api(ApiError::Timeout) => "timeout",
        AwsScanError::Api(ApiError::Exhausted(_)) => "retry_budget_exhausted",
        AwsScanError::Api(_) => "api_error",
        AwsScanError::InvalidData(_) | AwsScanError::Validation(_) => "invalid_response",
        AwsScanError::InvalidOptions(_) => "invalid_options",
        AwsScanError::ChannelClosed => "consumer_closed",
        AwsScanError::ChildLimit => "child_limit",
    }
}

#[derive(Default)]
pub(crate) struct PageBody {
    pub resources: Vec<Resource>,
    pub edges: Vec<Edge>,
    pub next_token: Option<String>,
    /// Items enumerated before enrichment, including items whose describe failed.
    pub listed_count: usize,
}

#[derive(Default)]
struct Cursor {
    token: Option<String>,
    seen: BTreeSet<String>,
    token_bytes: usize,
    page_number: u32,
    children: usize,
    done: bool,
    pending_failure: Option<(&'static str, String)>,
}

impl Cursor {
    fn advance(&mut self, body: &PageBody, max_pages: u32) {
        self.page_number = self.page_number.saturating_add(1);
        self.children = self.children.saturating_add(body.listed_count);
        self.token = body.next_token.clone();
        let Some(token) = &body.next_token else {
            self.done = true;
            return;
        };
        if token.is_empty() || token.len() > MAX_TOKEN_BYTES {
            self.pending_failure = Some((
                "invalid_pagination",
                "invalid or oversized continuation token".into(),
            ));
        } else if self.seen.contains(token) {
            self.pending_failure = Some((
                "pagination_cycle",
                "AWS repeated a continuation token".into(),
            ));
        } else if self.page_number >= max_pages {
            self.pending_failure = Some((
                "page_limit",
                format!("page cap {max_pages} reached with a continuation token"),
            ));
        } else if self.children >= MAX_CHILDREN {
            self.pending_failure = Some((
                "child_limit",
                format!("scope child cap {MAX_CHILDREN} reached"),
            ));
        } else if self.token_bytes.saturating_add(token.len()) > MAX_TOKEN_TOTAL_BYTES {
            self.pending_failure = Some((
                "token_memory_limit",
                "continuation token history capacity reached".into(),
            ));
        } else {
            self.token_bytes = self.token_bytes.saturating_add(token.len());
            self.seen.insert(token.clone());
        }
    }
}

/// One SDK page at a time. The only retained history is a bounded set of cursors.
/// Empty pages with continuation tokens are deliberately not terminal.
#[tracing::instrument(skip_all, fields(collector = scope.collector, region = scope.region))]
pub(crate) async fn paginate<F, Fut>(
    ctx: &Context,
    scope: CollectionScope,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn(Option<String>, usize) -> Fut + Send + Sync,
    Fut: Future<Output = Result<PageBody, AwsScanError>> + Send,
{
    // One erased driver per scope bounds trait/layout expansion through nested
    // parent collectors. Requests themselves retain their typed SDK futures.
    let work: BoxFuture<'_, Result<(), AwsScanError>> = Box::pin(paginate_inner(ctx, scope, fetch));
    work.await
}

async fn paginate_inner<F, Fut>(
    ctx: &Context,
    scope: CollectionScope,
    fetch: F,
) -> Result<(), AwsScanError>
where
    F: Fn(Option<String>, usize) -> Fut + Send + Sync,
    Fut: Future<Output = Result<PageBody, AwsScanError>> + Send,
{
    let pages = stream::unfold(Cursor::default(), |mut cursor| {
        let fetch = &fetch;
        let scope = &scope;
        async move {
            if cursor.done {
                return None;
            }
            if let Some((category, message)) = cursor.pending_failure.take() {
                cursor.done = true;
                let event = CollectionEvent::Failure(CollectionFailure {
                    scope: scope.clone(),
                    category: category.to_owned(),
                    message,
                });
                return Some((Ok(event), cursor));
            }
            match fetch(
                cursor.token.clone(),
                MAX_CHILDREN.saturating_sub(cursor.children),
            )
            .await
            {
                Ok(body) => {
                    if cursor.children.saturating_add(body.listed_count) > MAX_CHILDREN {
                        cursor.done = true;
                        return Some((
                            Ok(CollectionEvent::Failure(CollectionFailure {
                                scope: scope.clone(),
                                category: "child_limit".into(),
                                message: format!(
                                    "scope child cap {MAX_CHILDREN} would be exceeded"
                                ),
                            })),
                            cursor,
                        ));
                    }
                    cursor.advance(&body, ctx.max_pages);
                    let page = CollectionPage {
                        scope: scope.clone(),
                        page_number: cursor.page_number,
                        next_token: body.next_token,
                        resources: body.resources,
                        edges: body.edges,
                    };
                    match page.validate() {
                        Ok(()) => Some((Ok(CollectionEvent::Page(page)), cursor)),
                        Err(error) => {
                            cursor.done = true;
                            Some((
                                Ok(CollectionEvent::Failure(CollectionFailure {
                                    scope: scope.clone(),
                                    category: "invalid_response".into(),
                                    message: error.to_string(),
                                })),
                                cursor,
                            ))
                        }
                    }
                }
                Err(AwsScanError::ChannelClosed) => {
                    cursor.done = true;
                    Some((Err(AwsScanError::ChannelClosed), cursor))
                }
                Err(error) => {
                    cursor.done = true;
                    let event = CollectionEvent::Failure(CollectionFailure {
                        scope: scope.clone(),
                        category: error_category(&error).to_owned(),
                        message: error.to_string(),
                    });
                    Some((Ok(event), cursor))
                }
            }
        }
    });
    pages
        .try_for_each(|event| async move {
            match &event {
                CollectionEvent::Page(page) => {
                    tracing::debug!(collector = %page.scope.collector, page = page.page_number,
                        resources = page.resources.len(), edges = page.edges.len(), "inventory page collected");
                    if page.next_token.is_none() {
                        tracing::info!(collector = %page.scope.collector, pages = page.page_number, "collection scope exhausted its available results");
                    }
                }
                CollectionEvent::Failure(failure) => {
                    tracing::warn!(collector = %failure.scope.collector, category = %failure.category, "collection scope incomplete");
                }
            }
            ctx.sender
                .send(event)
                .await
                .map_err(|_| AwsScanError::ChannelClosed)
        })
        .await
}

pub(crate) async fn collect_bounded<I, F, Fut, T>(
    items: I,
    concurrency: usize,
    fetch: F,
) -> Result<Vec<T>, AwsScanError>
where
    I: IntoIterator + Send,
    I::IntoIter: Send,
    I::Item: Send,
    T: Send,
    F: Fn(I::Item) -> Fut + Send,
    Fut: Future<Output = Result<T, AwsScanError>> + Send,
{
    let work: BoxFuture<'_, Result<Vec<T>, AwsScanError>> = Box::pin(async move {
        stream::iter(items)
            .map(fetch)
            .buffer_unordered(concurrency)
            .try_collect()
            .await
    });
    work.await
}

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests {
    use super::{
        Context, Cursor, MAX_CHILDREN, MAX_RETRY_HEADER_VALUES, PageBody, attach_hint,
        header_hints, paginate, parse_retry_after_seconds, parse_standard_retry_after,
        require_budget, sdk_error,
    };
    use aws_sdk_lambda::operation::list_functions::ListFunctionsError;
    use aws_sdk_sts::{config::http::HttpResponse, error::SdkError};
    use aws_smithy_types::body::SdkBody;
    use inventory_core::{CollectionEvent, Source};
    use inventory_engine::limiter::{ApiError, ApiExecutor, ExecutorConfig, RetryAfter, RetryKind};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, UNIX_EPOCH};
    use tokio::sync::mpsc;

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

    fn context(max_pages: u32) -> Result<(Context, mpsc::Receiver<CollectionEvent>), ApiError> {
        let (sender, receiver) = mpsc::channel(8);
        let context = Context {
            config: aws_types::SdkConfig::builder().build(),
            executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
            sender,
            account: "111111111111".into(),
            partition: "aws".into(),
            region: "us-east-1".into(),
            max_pages,
            page_size: 100,
        };
        Ok((context, receiver))
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

    #[tokio::test]
    async fn driver_follows_empty_page_and_commits_terminal_empty_page()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut events) = context(10)?;
        paginate(
            &ctx,
            ctx.scope("test", None, Source::Direct),
            |token, _budget| async move {
                Ok(PageBody {
                    next_token: token.is_none().then(|| "next".to_owned()),
                    ..PageBody::default()
                })
            },
        )
        .await?;
        let first = events.try_recv()?;
        let second = events.try_recv()?;
        assert!(
            matches!(first, CollectionEvent::Page(page) if page.page_number == 1 && page.next_token.as_deref() == Some("next"))
        );
        assert!(
            matches!(second, CollectionEvent::Page(page) if page.page_number == 2 && page.next_token.is_none())
        );
        assert!(events.try_recv().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn driver_records_cap_without_terminal_page() -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut events) = context(1)?;
        paginate(
            &ctx,
            ctx.scope("test", None, Source::Direct),
            |_, _budget| async {
                Ok(PageBody {
                    next_token: Some("next".into()),
                    ..PageBody::default()
                })
            },
        )
        .await?;
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some())
        );
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "page_limit")
        );
        assert!(events.try_recv().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn driver_preserves_partial_page_before_permission_failure()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut events) = context(10)?;
        paginate(
            &ctx,
            ctx.scope("test", None, Source::Direct),
            |token, _budget| async move {
                if token.is_some() {
                    return Err(ApiError::Denied("fixture denial".into()).into());
                }
                Ok(PageBody {
                    next_token: Some("next".into()),
                    ..PageBody::default()
                })
            },
        )
        .await?;
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some())
        );
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "access_denied")
        );
        assert!(events.try_recv().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn child_budget_is_checked_before_enrichment() -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut events) = context(10)?;
        let enriched = Arc::new(AtomicBool::new(false));
        paginate(
            &ctx,
            ctx.scope("test", None, Source::Direct),
            |token, remaining| {
                let enriched = Arc::clone(&enriched);
                async move {
                    if token.is_none() {
                        return Ok(PageBody {
                            next_token: Some("last".into()),
                            listed_count: MAX_CHILDREN.saturating_sub(1),
                            ..PageBody::default()
                        });
                    }
                    require_budget(2, remaining)?;
                    enriched.store(true, Ordering::Relaxed);
                    Ok(PageBody::default())
                }
            },
        )
        .await?;
        assert!(!enriched.load(Ordering::Relaxed));
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Page(page) if page.next_token.is_some())
        );
        assert!(
            matches!(events.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "child_limit")
        );
        assert!(events.try_recv().is_err());
        Ok(())
    }
}
