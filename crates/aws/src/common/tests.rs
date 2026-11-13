use super::{
    Context, Cursor, MAX_CHILDREN, MAX_RETRY_HEADER_VALUES, PageBody, attach_hint, header_hints,
    paginate, parse_retry_after_seconds, parse_standard_retry_after, require_budget, sdk_error,
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

mod driver_follows_empty_page_and_commits_terminal_empty_page;
mod unsupported_endpoint_does_not_retry_and_decode_failure_does_not_expose_body;
