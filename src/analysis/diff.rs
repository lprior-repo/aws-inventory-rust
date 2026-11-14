use super::{DETAIL_BYTES, encoded_len, finalized};
use anyhow::{Context, Result};
use futures_util::{Stream, StreamExt, TryStreamExt, stream};
use inventory_core::{Edge, Resource, ScanStatus};
use inventory_engine::store::{Store, StoreError};
use serde::Serialize;
use serde_json::{Value, json};
use std::cmp::Ordering;
use std::pin::Pin;

type Items<'a, T> = Pin<Box<dyn Stream<Item = Result<T, StoreError>> + Send + 'a>>;

#[tracing::instrument(skip_all)]
pub(super) async fn report(
    before: &Store,
    after: &Store,
    before_scan: &str,
    after_scan: &str,
    limit: usize,
    max_items: usize,
) -> Result<Value> {
    super::validate_diff(before_scan, after_scan, limit, max_items)?;
    let before_status = finalized(before, before_scan).await?;
    let after_status = finalized(after, after_scan).await?;
    let resources = compare(
        before.resource_stream(before_scan),
        after.resource_stream(after_scan),
        limit,
        max_items,
    )
    .await?;
    let edges = compare(
        before.edge_stream(before_scan),
        after.edge_stream(after_scan),
        limit,
        max_items,
    )
    .await?;
    let partial = before_status != ScanStatus::Complete
        || after_status != ScanStatus::Complete
        || resources.comparison_truncated
        || edges.comparison_truncated;
    Ok(json!({
        "status":if partial {"partial"} else {"complete"},
        "before_scan":{"scan_id":before_scan,"status":before_status.as_str()},
        "after_scan":{"scan_id":after_scan,"status":after_status.as_str()},
        "row_limit_per_entity_class":limit,"comparison_item_limit_per_entity_class":max_items,
        "resources":resources,"relationships":edges,
        "semantics":{
            "unit":"preferred observations in each finalized scan; not an atomic AWS snapshot",
            "newly_observed":"present only in the later selected observations; creation is not inferred",
            "no_longer_observed":"present only in the earlier selected observations; deletion is never inferred",
            "changed":"selected identity metadata, properties, tags or source differ; this is observation change, not proof of configuration drift",
            "timestamps":"top-level observed_at is reported but excluded from equality; timestamps inside provider properties remain data",
            "sources":"a source change is explicit and may explain differences in detail; schemas are not treated as equivalent",
            "collections":"JSON object key order is ignored; array order is preserved; collection scopes can differ",
            "counts":"counts cover examined union identities; exact only when comparison_truncated is false",
            "details":"row and byte bounds can truncate displayed details without truncating comparison counts"
        }
    }))
}

trait SnapshotItem: Send + Sync {
    fn compare_identity(&self, other: &Self) -> Ordering;
    fn changed_fields(&self, other: &Self) -> Vec<&'static str>;
    fn metadata(&self) -> Value;
}

impl SnapshotItem for Resource {
    fn compare_identity(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }

    fn changed_fields(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.account_id != other.account_id, "account_id"),
            (self.region != other.region, "region"),
            (self.kind != other.kind, "kind"),
            (self.name != other.name, "name"),
            (self.properties != other.properties, "properties"),
            (self.tags != other.tags, "tags"),
            (self.source != other.source, "source"),
        ]
        .into_iter()
        .filter_map(|(changed, field)| changed.then_some(field))
        .collect()
    }

    fn metadata(&self) -> Value {
        json!({"id":self.id,"kind":self.kind,"account_id":self.account_id,
            "region":self.region,"name":self.name,"source":self.source,
            "observed_at":self.observed_at})
    }
}

impl SnapshotItem for Edge {
    fn compare_identity(&self, other: &Self) -> Ordering {
        (&self.from_id, &self.to_id, &self.relation).cmp(&(
            &other.from_id,
            &other.to_id,
            &other.relation,
        ))
    }

    fn changed_fields(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.properties != other.properties, "properties"),
            (self.source != other.source, "source"),
        ]
        .into_iter()
        .filter_map(|(changed, field)| changed.then_some(field))
        .collect()
    }

    fn metadata(&self) -> Value {
        json!({"from_id":self.from_id,"to_id":self.to_id,"relation":self.relation,
            "source":self.source,"observed_at":self.observed_at})
    }
}

struct Merge<'a, T> {
    before: Items<'a, T>,
    after: Items<'a, T>,
    left: Option<T>,
    right: Option<T>,
    primed: bool,
}

struct Pair<T> {
    before: Option<T>,
    after: Option<T>,
}

impl<T: SnapshotItem> Merge<'_, T> {
    #[tracing::instrument(skip_all)]
    async fn next_pair(&mut self) -> Result<Option<Pair<T>>> {
        if !self.primed {
            self.left = self.before.try_next().await?;
            self.right = self.after.try_next().await?;
            self.primed = true;
        }
        let ordering = match (&self.left, &self.right) {
            (None, None) => return Ok(None),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(left), Some(right)) => left.compare_identity(right),
        };
        let before = if ordering != Ordering::Greater {
            self.left.take()
        } else {
            None
        };
        let after = if ordering != Ordering::Less {
            self.right.take()
        } else {
            None
        };
        if ordering != Ordering::Greater {
            self.left = self.before.try_next().await?;
        }
        if ordering != Ordering::Less {
            self.right = self.after.try_next().await?;
        }
        Ok(Some(Pair { before, after }))
    }
}

#[derive(Default, Serialize)]
struct Counts {
    examined: usize,
    newly_observed: usize,
    no_longer_observed: usize,
    changed: usize,
    unchanged: usize,
    source_changed: usize,
}

#[derive(Serialize)]
struct Comparison {
    counts: Counts,
    comparison_truncated: bool,
    details_truncated: bool,
    detail_byte_limit: usize,
    rows: Vec<Value>,
    #[serde(skip)]
    row_limit: usize,
    #[serde(skip)]
    detail_bytes: usize,
}

impl Comparison {
    fn new(limit: usize) -> Result<Self> {
        let mut rows = Vec::new();
        rows.try_reserve(limit)
            .context("reserve diff output rows")?;
        Ok(Self {
            counts: Counts::default(),
            comparison_truncated: false,
            details_truncated: false,
            detail_byte_limit: DETAIL_BYTES,
            rows,
            row_limit: limit,
            detail_bytes: 0,
        })
    }

    fn observe<T: SnapshotItem>(&mut self, pair: Pair<T>) -> Result<()> {
        self.counts.examined = self.counts.examined.saturating_add(1);
        let fields = match (&pair.before, &pair.after) {
            (Some(before), Some(after)) => before.changed_fields(after),
            _ => Vec::new(),
        };
        let change = self.classify(&pair, &fields)?;
        if change == "unchanged" {
            return Ok(());
        }
        if self.rows.len() == self.row_limit {
            self.details_truncated = true;
            return Ok(());
        }
        let row = json!({
            "change":change,"changed_fields":fields,"deletion_inferred":false,
            "before":pair.before.as_ref().map(SnapshotItem::metadata),
            "after":pair.after.as_ref().map(SnapshotItem::metadata)
        });
        let next = self
            .detail_bytes
            .checked_add(encoded_len(&row)?)
            .context("diff byte count")?;
        if next > DETAIL_BYTES {
            self.details_truncated = true;
            return Ok(());
        }
        self.detail_bytes = next;
        self.rows.push(row);
        Ok(())
    }

    fn classify<T>(&mut self, pair: &Pair<T>, fields: &[&str]) -> Result<&'static str> {
        let (counter, name) = match (&pair.before, &pair.after) {
            (None, Some(_)) => (&mut self.counts.newly_observed, "newly_observed"),
            (Some(_), None) => (&mut self.counts.no_longer_observed, "no_longer_observed"),
            (Some(_), Some(_)) if fields.is_empty() => (&mut self.counts.unchanged, "unchanged"),
            (Some(_), Some(_)) => (&mut self.counts.changed, "changed"),
            (None, None) => anyhow::bail!("diff merge produced an empty pair"),
        };
        *counter = counter.saturating_add(1);
        if fields.contains(&"source") {
            self.counts.source_changed = self.counts.source_changed.saturating_add(1);
        }
        Ok(name)
    }
}

#[tracing::instrument(skip_all)]
async fn compare<T: SnapshotItem>(
    before: Items<'_, T>,
    after: Items<'_, T>,
    limit: usize,
    max_items: usize,
) -> Result<Comparison> {
    let state = Merge {
        before,
        after,
        left: None,
        right: None,
        primed: false,
    };
    let probe = max_items
        .checked_add(1)
        .context("diff comparison probe overflow")?;
    stream::try_unfold(state, |mut state| async move {
        let pair = state.next_pair().await?;
        Ok::<_, anyhow::Error>(pair.map(|pair| (pair, state)))
    })
    .take(probe)
    .enumerate()
    .map(|(index, item)| item.map(|item| (index, item)))
    .try_fold(
        Comparison::new(limit)?,
        |mut result, (index, pair)| async move {
            tokio::task::coop::consume_budget().await;
            if index >= max_items {
                result.comparison_truncated = true;
            } else {
                result.observe(pair)?;
            }
            Ok(result)
        },
    )
    .await
}
