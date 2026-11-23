use crate::inventory::work::transitions::encode_scope;
use crate::inventory::{MAX_TOKEN_BYTES, PageCommit, SchemaRegistry, StoreError};
use sqlx::{Postgres, Transaction};
use std::collections::BTreeSet;

const LEASE: &str = "WITH keys AS (SELECT DISTINCT hashtextextended(ROW(a,r,k)::text,14) AS key FROM UNNEST($6::text[],$7::text[],$8::text[]) u(a,r,k)), locks AS MATERIALIZED (SELECT pg_advisory_xact_lock(key) FROM keys ORDER BY key) SELECT state IN(1,2) AND generation=$3 AND authority_generation=generation AND lease_until>$4 AND lease_until=$5 FROM work_queue CROSS JOIN (SELECT count(*) FROM locks) held WHERE sweep_id=$1 AND scope_key=$2 FOR UPDATE OF work_queue";

pub(super) fn page(registry: &SchemaRegistry, page: &PageCommit) -> Result<(), StoreError> {
    bounds(page)?;
    continuation(page)?;
    identity(page)?;
    for child in &page.children {
        child.validate_context()?;
        if child.sweep() != page.lease.sweep() || !registry.allows(child.scope()) {
            return Err(StoreError::Invalid(
                "child work is not in this sweep or bound catalog",
            ));
        }
    }
    registry.permissions.validate_page(page)
}

fn bounds(page: &PageCommit) -> Result<(), StoreError> {
    let tag_count = page.tags.iter().try_fold(0_usize, |count, tags| {
        count
            .checked_add(tags.tags().len())
            .ok_or(StoreError::Invalid("tag count overflow"))
    })?;
    if page.observed_at_ms < 0
        || page.observations.len() > 100_000
        || page.edges.len() > 10_000
        || page.unresolved.len() > 10_000
        || page.tags.len() > 50_000
        || page.children.len() > 10_000
        || tag_count > 50_000
    {
        return Err(StoreError::Invalid("page exceeds storage bounds"));
    }
    unique_tags(page)
}

fn unique_tags(page: &PageCommit) -> Result<(), StoreError> {
    let unique = page
        .tags
        .iter()
        .map(|tags| tags.resource())
        .collect::<BTreeSet<_>>();
    if unique.len() != page.tags.len() {
        return Err(StoreError::Invalid(
            "page contains duplicate resource tag observations",
        ));
    }
    Ok(())
}

fn continuation(page: &PageCommit) -> Result<(), StoreError> {
    if page.done != page.continuation_digest.is_none()
        || page.done != page.next_token.is_none()
        || page
            .next_token
            .as_ref()
            .is_some_and(|token| token.len() > MAX_TOKEN_BYTES)
    {
        return Err(StoreError::Invalid(
            "page continuation token and digest are inconsistent or out of bounds",
        ));
    }
    Ok(())
}

fn identity(page: &PageCommit) -> Result<(), StoreError> {
    let scope = page.lease.scope();
    let keys = page
        .observations
        .iter()
        .map(|item| item.key())
        .chain(page.edges.iter().map(|edge| &edge.from))
        .chain(page.unresolved.iter().map(|reference| &reference.from))
        .chain(page.tags.iter().map(|tags| tags.resource()));
    if keys
        .into_iter()
        .any(|key| &key.account != scope.account() || &key.region != scope.region())
    {
        return Err(StoreError::Invalid(
            "page contains data outside its leased account and region",
        ));
    }
    Ok(())
}

pub(super) async fn lease(
    tx: &mut Transaction<'_, Postgres>,
    page: &PageCommit,
) -> Result<(), StoreError> {
    let (accounts, regions, kinds) = groups(page);
    let valid: Option<bool> = sqlx::query_scalar(LEASE)
        .bind(page.lease.sweep().as_str())
        .bind(encode_scope(page.lease.scope())?)
        .bind(page.lease.generation())
        .bind(page.observed_at_ms)
        .bind(page.lease.lease_until_ms)
        .bind(accounts)
        .bind(regions)
        .bind(kinds)
        .fetch_optional(&mut **tx)
        .await?;
    if valid != Some(true) {
        return Err(StoreError::StaleLease);
    }
    Ok(())
}

type Groups<'a> = (Vec<&'a str>, Vec<&'a str>, Vec<&'a str>);

fn groups(page: &PageCommit) -> Groups<'_> {
    let groups = page
        .observations
        .iter()
        .map(|observation| observation.key())
        .chain(page.tags.iter().map(|tags| tags.resource()))
        .map(|key| (key.account.as_str(), key.region.as_str(), key.kind.as_str()))
        .collect::<BTreeSet<_>>();
    let accounts = groups
        .iter()
        .map(|(account, _, _)| *account)
        .collect::<Vec<_>>();
    let regions = groups
        .iter()
        .map(|(_, region, _)| *region)
        .collect::<Vec<_>>();
    let kinds = groups.iter().map(|(_, _, kind)| *kind).collect::<Vec<_>>();
    (accounts, regions, kinds)
}
