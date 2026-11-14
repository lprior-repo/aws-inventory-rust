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
        if category == "unsupported_type" {
            tracing::info!(collector = %scope.collector, category, "provider capability boundary recorded");
        } else {
            tracing::warn!(collector = %scope.collector, category, "collection scope incomplete");
        }
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

#[derive(Default)]
pub(crate) struct PageBody {
    pub resources: Vec<Resource>,
    pub edges: Vec<Edge>,
    pub next_token: Option<String>,

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

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod tests;

mod retry_hints;
pub(crate) use retry_hints::{parse_retry_after_seconds, sdk_error, sdk_error_with_hint};
mod error_classification;
use error_classification::classify_sdk_error;
pub(crate) use error_classification::error_category;
mod pagination;
pub(crate) use pagination::{collect_bounded, paginate};
mod evidence;
use evidence::network_reference_evidence;
pub(crate) use evidence::{now_millis, require_budget, required};

#[cfg(test)]
use retry_hints::{attach_hint, combine_hints, header_hints, parse_standard_retry_after};
