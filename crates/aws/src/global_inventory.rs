//! Account-global inventory. The coordinator invokes this once per resolved
//! account scan, outside the regional fan-out. SDK endpoint region and observed
//! resource scope are deliberately separate.

mod cloudfront;
mod iam;
mod route53;
mod s3;

use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Mutex;

use aws_types::region::Region;
use futures_util::{StreamExt, TryStreamExt, future::BoxFuture, stream};
use inventory_core::{CollectionEvent, CollectionPage, CollectionScope, Edge, Resource, Source};
use inventory_engine::limiter::ApiError;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::AwsScanError;
use crate::common::{Context, MAX_CHILDREN, PageBody, error_category, require_budget};

pub(crate) const GLOBAL_SERVICES: [&str; 4] = ["s3", "iam", "route53", "cloudfront"];
const CHILD_CONCURRENCY: usize = 4;
const MAX_COMPONENTS: usize = 4_096;
const MAX_LIST_ITEMS: usize = 1_000;

#[tracing::instrument(skip_all, fields(account = %base.account))]
pub(crate) async fn collect(
    base: &Context,
    services: &BTreeSet<String>,
) -> Result<(), AwsScanError> {
    ensure_active(base)?;
    let ctx = global_context(base)?;
    // Materialize at most four boxed futures before streaming them. This also
    // avoids the compiler's higher-ranked Send limitation for a borrowed Fn
    // adapter when this future is erased at the scan boundary (rust#100013).
    let mut work = Vec::with_capacity(GLOBAL_SERVICES.len());
    for service in GLOBAL_SERVICES {
        if services.contains(service) {
            work.push(service_future(&ctx, service));
        }
    }
    stream::iter(work)
        .buffer_unordered(4)
        .try_for_each(|()| async { Ok(()) })
        .await?;
    ensure_active(&ctx)
}

fn service_future<'a>(ctx: &'a Context, service: &str) -> BoxFuture<'a, Result<(), AwsScanError>> {
    match service {
        "s3" => Box::pin(s3::collect(ctx)),
        "iam" => Box::pin(iam::collect(ctx)),
        "route53" => Box::pin(route53::collect(ctx)),
        "cloudfront" => Box::pin(cloudfront::collect(ctx)),
        _ => Box::pin(async {
            Err(AwsScanError::InvalidOptions(
                "unknown global collector".into(),
            ))
        }),
    }
}

fn endpoint_region(partition: &str) -> Result<&'static str, AwsScanError> {
    match partition {
        "aws" => Ok("us-east-1"),
        "aws-cn" => Ok("cn-north-1"),
        "aws-us-gov" => Ok("us-gov-west-1"),
        _ => Err(AwsScanError::InvalidOptions(format!(
            "global inventory has no reviewed endpoint policy for partition {partition}"
        ))),
    }
}

fn global_context(base: &Context) -> Result<Context, AwsScanError> {
    let config = base
        .config
        .to_builder()
        .region(Region::new(endpoint_region(&base.partition)?))
        .build();
    Ok(Context {
        config,
        region: "global".to_owned(),
        ..base.clone()
    })
}

fn ensure_active(ctx: &Context) -> Result<(), AwsScanError> {
    if ctx.executor.is_cancelled() {
        Err(ApiError::Cancelled.into())
    } else {
        Ok(())
    }
}

fn components(count: usize, what: &'static str) -> Result<(), AwsScanError> {
    if count > MAX_COMPONENTS {
        return Err(AwsScanError::InvalidData(format!(
            "{what} exceeds {MAX_COMPONENTS} components"
        )));
    }
    Ok(())
}

fn checked_text(value: &str, what: &'static str) -> Result<String, AwsScanError> {
    inventory_core::validate_identifier(value, what)?;
    Ok(value.to_owned())
}

fn own_arn(ctx: &Context, arn: &str, service: &str, kind: &str) -> Result<String, AwsScanError> {
    inventory_core::validate_identifier(arn, "global resource ARN")?;
    let prefix = format!("arn:{}:{service}::{}:{kind}/", ctx.partition, ctx.account);
    if !arn
        .strip_prefix(&prefix)
        .is_some_and(|suffix| !suffix.is_empty())
    {
        return Err(AwsScanError::InvalidData(format!(
            "unexpected {service} {kind} resource ARN"
        )));
    }
    Ok(arn.to_owned())
}

fn marker(truncated: bool, token: Option<&str>) -> Result<Option<String>, AwsScanError> {
    if truncated {
        token
            .filter(|token| !token.is_empty())
            .map(|token| Some(token.to_owned()))
            .ok_or_else(|| {
                AwsScanError::InvalidData("truncated response lacks continuation marker".into())
            })
    } else {
        Ok(None)
    }
}

/// Hashes only, with fixed cardinality. The mutex never crosses an await; one
/// list cursor owns each instance. It permits Fn request closures without a
/// shared account-wide cache of resource models or raw identifiers.
#[derive(Default)]
struct SeenIds(Mutex<BTreeSet<[u8; 32]>>);

impl SeenIds {
    fn record<'a>(
        &self,
        ids: impl Iterator<Item = &'a str>,
        budget: usize,
    ) -> Result<usize, AwsScanError> {
        let mut seen = self
            .0
            .lock()
            .map_err(|_| AwsScanError::InvalidData("identity ledger poisoned".into()))?;
        let mut page = BTreeSet::new();
        let mut count = 0usize;
        for id in ids.take(MAX_LIST_ITEMS.saturating_add(1)) {
            checked_text(id, "listed resource identifier")?;
            count = count.saturating_add(1);
            if count > MAX_LIST_ITEMS {
                return Err(AwsScanError::ChildLimit);
            }
            let digest: [u8; 32] = Sha256::digest(id.as_bytes()).into();
            if seen.contains(&digest) || !page.insert(digest) {
                return Err(AwsScanError::InvalidData(
                    "list response repeated a resource identifier".into(),
                ));
            }
        }
        require_budget(count, budget)?;
        if seen.len().saturating_add(count) > MAX_CHILDREN {
            return Err(AwsScanError::ChildLimit);
        }
        seen.extend(page);
        Ok(count)
    }
}

#[tracing::instrument(skip_all, fields(collector = %scope.collector))]
async fn emit(ctx: &Context, scope: CollectionScope, body: PageBody) -> Result<(), AwsScanError> {
    let page = checked_page(scope, body)?;
    ctx.sender
        .send(CollectionEvent::Page(page))
        .await
        .map_err(|_| AwsScanError::ChannelClosed)
}

fn checked_page(scope: CollectionScope, body: PageBody) -> Result<CollectionPage, AwsScanError> {
    let page = CollectionPage {
        scope,
        page_number: 1,
        next_token: body.next_token,
        resources: body.resources,
        edges: body.edges,
    };
    page.validate()?;
    Ok(page)
}

#[tracing::instrument(skip_all, fields(collector = %scope.collector))]
async fn observed(
    ctx: &Context,
    scope: CollectionScope,
    result: Result<PageBody, AwsScanError>,
    fallback: Option<PageBody>,
) -> Result<(), AwsScanError> {
    match result.and_then(|body| checked_page(scope.clone(), body)) {
        Ok(page) => {
            ensure_active(ctx)?;
            ctx.sender
                .send(CollectionEvent::Page(page))
                .await
                .map_err(|_| AwsScanError::ChannelClosed)
        }
        Err(AwsScanError::ChannelClosed) => Err(AwsScanError::ChannelClosed),
        Err(error) => {
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            if matches!(error, AwsScanError::Api(ApiError::Cancelled)) {
                return Err(error);
            }
            ensure_active(ctx)?;
            if let Some(body) = fallback {
                emit(ctx, scope, body).await?;
            }
            Ok(())
        }
    }
}

#[tracing::instrument(skip_all, fields(section))]
async fn section<F, Fut>(
    ctx: &Context,
    parent: &str,
    section: &'static str,
    fetch: F,
) -> Result<(&'static str, Option<Value>), AwsScanError>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = Result<Value, AwsScanError>> + Send,
{
    ensure_active(ctx)?;
    let scope = ctx.scope(section, Some(parent), Source::Direct);
    match fetch().await {
        Ok(value) => {
            ensure_active(ctx)?;
            emit(ctx, scope, PageBody::default()).await?;
            Ok((section, Some(value)))
        }
        Err(error) => {
            ctx.failure(&scope, error_category(&error), error.to_string())
                .await?;
            if matches!(
                error,
                AwsScanError::Api(ApiError::Cancelled) | AwsScanError::ChannelClosed
            ) {
                return Err(error);
            }
            Ok((section, None))
        }
    }
}

fn single(resource: Resource, edges: Vec<Edge>) -> PageBody {
    PageBody {
        resources: vec![resource],
        edges,
        listed_count: 1,
        ..PageBody::default()
    }
}

fn reference(
    ctx: &Context,
    source: &Resource,
    target: &str,
    relation: &str,
    evidence: Value,
) -> Result<Edge, AwsScanError> {
    inventory_core::validate_identifier(target, "global reference target")?;
    let mut edge = ctx.edge(source, target.to_owned(), relation);
    let properties = edge.properties.as_object_mut().ok_or_else(|| {
        AwsScanError::InvalidData("reference properties are not an object".into())
    })?;
    let evidence = evidence
        .as_object()
        .ok_or_else(|| AwsScanError::InvalidData("reference evidence is not an object".into()))?;
    properties.extend(evidence.clone());
    if !properties.contains_key("owner_resolution")
        && let Some(owner) = properties.get("target_owner_resolution").cloned()
    {
        properties.insert("owner_resolution".to_owned(), owner);
    }
    edge.validate()?;
    Ok(edge)
}

fn edge_set(edges: impl IntoIterator<Item = Edge>) -> Result<Vec<Edge>, AwsScanError> {
    let mut merged = BTreeMap::new();
    for (index, edge) in edges
        .into_iter()
        .take(MAX_COMPONENTS.saturating_add(1))
        .enumerate()
    {
        if index == MAX_COMPONENTS {
            return Err(AwsScanError::ChildLimit);
        }
        let evidence = edge.properties.clone();
        let retained: serde_json::Map<String, Value> = [
            "owner_resolution",
            "target_owner_resolution",
            "target_resource_id",
            "target_kind",
            "target_region",
        ]
        .into_iter()
        .filter_map(|key| {
            evidence
                .get(key)
                .map(|value| (key.to_owned(), value.clone()))
        })
        .collect();
        crate::ecs::merge_reference_edge(
            &mut merged,
            Edge {
                properties: Value::Object(retained),
                ..edge
            },
            evidence,
        )?;
    }
    Ok(merged.into_values().collect())
}

fn digest_identity(namespace: &str, fields: &[&str]) -> Result<String, AwsScanError> {
    if fields.is_empty() || fields.len() > 16 {
        return Err(AwsScanError::InvalidData(
            "compound identity requires 1 to 16 components".into(),
        ));
    }
    let mut hash = Sha256::new();
    for field in fields {
        inventory_core::validate_identifier(field, "identity component")?;
        hash.update(
            u64::try_from(field.len())
                .map_err(|_| AwsScanError::InvalidData("identity length overflow".into()))?
                .to_be_bytes(),
        );
        hash.update(field.as_bytes());
    }
    Ok(format!("aws-inventory:{namespace}:{:x}", hash.finalize()))
}

fn dns_reference(name: &str) -> Result<String, AwsScanError> {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if name.is_empty()
        || name.len() > 1_024
        || name.contains(['/', '@', ':'])
        || name
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(AwsScanError::InvalidData(
            "invalid DNS reference name".into(),
        ));
    }
    Ok(format!("dns:{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_engine::limiter::{ApiExecutor, ExecutorConfig};
    use serde_json::json;
    use std::sync::Arc;

    pub(super) fn context()
    -> Result<(Context, tokio::sync::mpsc::Receiver<CollectionEvent>), Box<dyn std::error::Error>>
    {
        let (sender, receiver) = tokio::sync::mpsc::channel(128);
        let config = aws_types::SdkConfig::builder()
            .region(Region::new("eu-west-1"))
            .build();
        Ok((
            Context {
                config,
                executor: Arc::new(ApiExecutor::new(ExecutorConfig::default())?),
                sender,
                account: "123456789012".into(),
                partition: "aws".into(),
                region: "global".into(),
                max_pages: 10,
                page_size: 100,
            },
            receiver,
        ))
    }

    #[test]
    fn global_scope_does_not_become_sdk_endpoint_region() -> Result<(), Box<dyn std::error::Error>>
    {
        let (base, _rx) = context()?;
        let ctx = global_context(&base)?;
        assert_eq!(ctx.region, "global");
        assert_eq!(ctx.config.region().map(Region::as_ref), Some("us-east-1"));
        assert_eq!(endpoint_region("aws-cn")?, "cn-north-1");
        assert_eq!(endpoint_region("aws-us-gov")?, "us-gov-west-1");
        assert!(endpoint_region("aws-iso").is_err());
        Ok(())
    }

    #[test]
    fn marker_requires_evidence_of_more_pages() -> Result<(), AwsScanError> {
        assert_eq!(marker(false, Some("stale"))?, None);
        assert_eq!(marker(true, Some("next"))?, Some("next".into()));
        assert!(marker(true, None).is_err());
        assert!(marker(true, Some("")).is_err());
        Ok(())
    }

    #[test]
    fn identity_ledger_rejects_duplicates_and_budget_before_commit() -> Result<(), AwsScanError> {
        let ledger = SeenIds::default();
        assert!(ledger.record(["a", "b"].into_iter(), 1).is_err());
        assert_eq!(ledger.record(["a", "b"].into_iter(), 2)?, 2);
        assert!(ledger.record(["b"].into_iter(), 2).is_err());
        assert!(ledger.record(["c", "c"].into_iter(), 2).is_err());
        assert_eq!(ledger.record(["c"].into_iter(), 1)?, 1);
        Ok(())
    }

    #[test]
    fn compound_identity_is_unambiguous() -> Result<(), AwsScanError> {
        assert_ne!(
            digest_identity("record", &["ab", "c"])?,
            digest_identity("record", &["a", "bc"])?
        );
        assert_eq!(dns_reference("EXAMPLE.com.")?, "dns:example.com");
        assert!(dns_reference("https://credential@example.com").is_err());
        assert!(dns_reference("credential example.com").is_err());
        assert!(digest_identity("record", &["value"; 17]).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn detail_failure_precedes_fallback_and_keeps_scope_incomplete()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        let scope = ctx.scope("aws_iam_role", Some("role-id"), Source::Direct);
        let resource = ctx.resource(
            "aws_iam_role",
            "role-id".into(),
            None,
            json!({"details_available":false}),
            BTreeMap::new(),
        )?;
        observed(
            &ctx,
            scope.clone(),
            Err(ApiError::Denied("denied".into()).into()),
            Some(single(resource, Vec::new())),
        )
        .await?;
        assert!(matches!(rx.try_recv()?, CollectionEvent::Failure(failure)
            if failure.scope == scope && failure.category == "access_denied"));
        assert!(matches!(rx.try_recv()?, CollectionEvent::Page(page)
            if page.scope == scope && page.resources.len() == 1));
        assert!(rx.try_recv().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn invalid_child_metadata_does_not_drop_valid_siblings()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        crate::common::collect_bounded(["bad", "good"], 2, |id| {
            let ctx = &ctx;
            async move {
                let resource = ctx.resource(
                    "synthetic_global",
                    if id == "bad" {
                        String::new()
                    } else {
                        id.into()
                    },
                    None,
                    json!({}),
                    BTreeMap::new(),
                )?;
                observed(
                    ctx,
                    ctx.scope("synthetic_global", Some(id), Source::Direct),
                    Ok(single(resource, Vec::new())),
                    None,
                )
                .await
            }
        })
        .await?;
        let mut failures = 0usize;
        let mut valid = 0usize;
        while let Ok(event) = rx.try_recv() {
            match event {
                CollectionEvent::Failure(failure) => {
                    assert_eq!(failure.scope.parent_id.as_deref(), Some("bad"));
                    failures = failures.saturating_add(1);
                }
                CollectionEvent::Page(page) => {
                    assert_eq!(
                        page.resources.first().map(|resource| resource.id.as_str()),
                        Some("good")
                    );
                    valid = valid.saturating_add(1);
                }
            }
        }
        assert_eq!((failures, valid), (1, 1));
        Ok(())
    }

    #[tokio::test]
    async fn cancellation_and_closed_consumers_never_become_fallback_success()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        ctx.executor.cancel();
        let scope = ctx.scope("synthetic_global", Some("cancelled"), Source::Direct);
        let result = observed(
            &ctx,
            scope,
            Err(ApiError::Cancelled.into()),
            Some(PageBody::default()),
        )
        .await;
        assert!(matches!(
            result,
            Err(AwsScanError::Api(ApiError::Cancelled))
        ));
        assert!(
            matches!(rx.try_recv()?, CollectionEvent::Failure(failure) if failure.category == "cancelled")
        );
        assert!(rx.try_recv().is_err());

        let (active, receiver) = context()?;
        drop(receiver);
        assert!(matches!(
            observed(
                &active,
                active.scope("synthetic_global", None, Source::Direct),
                Ok(PageBody::default()),
                None
            )
            .await,
            Err(AwsScanError::ChannelClosed)
        ));
        Ok(())
    }

    #[tokio::test]
    async fn unavailable_section_is_distinct_from_successful_absence()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        let (_, denied) = section(&ctx, "bucket", "policy_status", || async {
            Err(ApiError::Denied("denied".into()).into())
        })
        .await?;
        let (_, absent) = section(&ctx, "bucket", "policy_status", || async {
            Ok(json!({"configured":false}))
        })
        .await?;
        assert_eq!(denied, None);
        assert_eq!(absent, Some(json!({"configured":false})));
        assert!(matches!(rx.try_recv()?, CollectionEvent::Failure(_)));
        assert!(matches!(rx.try_recv()?, CollectionEvent::Page(_)));
        Ok(())
    }

    #[tokio::test]
    async fn pre_cancelled_global_scan_never_constructs_service_work()
    -> Result<(), Box<dyn std::error::Error>> {
        let (ctx, mut rx) = context()?;
        ctx.executor.cancel();
        let services = GLOBAL_SERVICES.into_iter().map(str::to_owned).collect();
        assert!(matches!(
            collect(&ctx, &services).await,
            Err(AwsScanError::Api(ApiError::Cancelled))
        ));
        assert_eq!(ctx.executor.stats().attempts, 0);
        assert!(rx.try_recv().is_err());
        Ok(())
    }
}
