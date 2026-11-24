use super::*;
use inventory_core::inventory::{AccountId, OperationId, Region, ScopeOutcome, ServiceId};

const LATEST: &str = "WITH RECURSIVE latest AS (SELECT DISTINCT ON(scope_key) scope_key,sweep_id,outcome,observed_at FROM inventory.scope_outcomes ORDER BY scope_key,observed_at DESC,sweep_id COLLATE \"C\" DESC) ";
// Hex preserves bytewise C ordering without asking SQL to decode untrusted UTF-8.
const ORDERED: &str = r#",
decoded AS (
    SELECT l.*,0 AS field,1 AS position,ARRAY[]::text[] COLLATE "C" AS names FROM latest l
    UNION ALL
    SELECT d.scope_key,d.sweep_id,d.outcome,d.observed_at,d.field+1,
           CASE WHEN n.size<=octet_length(d.scope_key)-d.position-3
                THEN (d.position+4+n.size)::integer END,
           array_append(d.names,
               CASE WHEN n.size<=octet_length(d.scope_key)-d.position-3
                    THEN encode(substring(d.scope_key FROM d.position+4 FOR n.size::integer),'hex') END)
    FROM decoded d CROSS JOIN LATERAL (
        SELECT CASE WHEN d.position BETWEEN 1 AND octet_length(d.scope_key)-3
                    THEN get_byte(d.scope_key,d.position-1)::bigint*16777216
                       + get_byte(d.scope_key,d.position)::bigint*65536
                       + get_byte(d.scope_key,d.position+1)::bigint*256
                       + get_byte(d.scope_key,d.position+2)::bigint END AS size
    ) n WHERE d.field<4
)
SELECT d.scope_key,d.sweep_id,d.outcome,d.observed_at,
       (SELECT max(observed_at) FROM inventory.scope_outcomes o WHERE o.scope_key=d.scope_key AND o.outcome='complete') AS last_complete_at
FROM decoded d WHERE field=4
ORDER BY observed_at,names COLLATE "C",scope_key LIMIT "#;

impl Snapshot {
    pub(in crate::database) async fn scope_freshness(
        &mut self,
        now_ms: i64,
        limit: usize,
    ) -> Result<ScopeFreshnessReport, ViewError> {
        validate_page_limit(limit)?;
        let mut builder = sqlx::QueryBuilder::<Postgres>::new(LATEST);
        builder.push("SELECT outcome,count(*) AS scopes FROM latest GROUP BY outcome");
        let grouped = builder.build().fetch_all(&mut *self.tx).await?;
        let (counts, total_scopes) = grouped.into_iter().try_fold(
            (ScopeOutcomeCounts::default(), 0_u64),
            |(mut counts, total), row| {
                let scopes = queries::unsigned(&row, "scopes")?;
                let slot = match outcome(row.try_get("outcome")?)? {
                    ScopeOutcome::Complete => &mut counts.complete,
                    ScopeOutcome::Partial => &mut counts.partial,
                    ScopeOutcome::Inaccessible => &mut counts.inaccessible,
                    ScopeOutcome::Unsupported => &mut counts.unsupported,
                    ScopeOutcome::Failed => &mut counts.failed,
                    ScopeOutcome::Unobserved => &mut counts.unobserved,
                };
                *slot = scopes;
                Ok::<_, ViewError>((
                    counts,
                    total.checked_add(scopes).ok_or(ViewError::StoredRow)?,
                ))
            },
        )?;
        let mut builder = sqlx::QueryBuilder::<Postgres>::new(LATEST);
        builder
            .push(ORDERED)
            .push_bind(i64::try_from(limit).map_err(|_| ViewError::PageLimit)?);
        let rows = builder.build().fetch_all(&mut *self.tx).await?;
        let scopes = rows
            .into_iter()
            .map(|row| decode(row, now_ms))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ScopeFreshnessReport {
            now_ms,
            scopes,
            total_scopes,
            counts,
        })
    }
}

fn decode(row: PgRow, now_ms: i64) -> Result<ScopeFreshness, ViewError> {
    let scope: Vec<u8> = row.try_get("scope_key")?;
    let (account, rest) = text(&scope)?;
    let (region, rest) = text(rest)?;
    let (service, rest) = text(rest)?;
    let (operation, rest) = text(rest)?;
    AccountId::parse(account).map_err(|_| ViewError::StoredRow)?;
    Region::parse(region).map_err(|_| ViewError::StoredRow)?;
    ServiceId::parse(service).map_err(|_| ViewError::StoredRow)?;
    OperationId::parse(operation).map_err(|_| ViewError::StoredRow)?;
    let has_parent = match rest.first() {
        Some(0) => false,
        Some(1) => true,
        _ => return Err(ViewError::StoredRow),
    };
    let observed_at_ms = row.try_get("observed_at")?;
    let last_complete_at_ms: Option<i64> = row.try_get("last_complete_at")?;
    Ok(ScopeFreshness {
        account: account.to_owned(),
        region: region.to_owned(),
        service: service.to_owned(),
        operation: operation.to_owned(),
        has_parent,
        sweep_id: row.try_get("sweep_id")?,
        outcome: outcome(row.try_get("outcome")?)?,
        observed_at_ms,
        last_complete_at_ms,
        age_ms: age(now_ms, observed_at_ms),
        complete_age_ms: last_complete_at_ms.map(|at| age(now_ms, at)),
    })
}

fn text(bytes: &[u8]) -> Result<(&str, &[u8]), ViewError> {
    let (length, rest) = bytes.split_at_checked(4).ok_or(ViewError::StoredRow)?;
    let length = u32::from_be_bytes(length.try_into().map_err(|_| ViewError::StoredRow)?);
    let length = usize::try_from(length).map_err(|_| ViewError::StoredRow)?;
    let (value, rest) = rest.split_at_checked(length).ok_or(ViewError::StoredRow)?;
    Ok((
        std::str::from_utf8(value).map_err(|_| ViewError::StoredRow)?,
        rest,
    ))
}

fn age(now: i64, observed: i64) -> u64 {
    now.saturating_sub(observed).max(0).unsigned_abs()
}

fn outcome(value: &str) -> Result<ScopeOutcome, ViewError> {
    match value {
        "complete" => Ok(ScopeOutcome::Complete),
        "partial" => Ok(ScopeOutcome::Partial),
        "inaccessible" => Ok(ScopeOutcome::Inaccessible),
        "unsupported" => Ok(ScopeOutcome::Unsupported),
        "failed" => Ok(ScopeOutcome::Failed),
        "unobserved" => Ok(ScopeOutcome::Unobserved),
        _ => Err(ViewError::StoredRow),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn postgres_given_future_or_extreme_timestamps_when_computing_age_then_values_are_bounded()
    -> Result<(), std::num::TryFromIntError> {
        assert_eq!(super::age(10, 20), 0);
        assert_eq!(super::age(20, 10), 10);
        assert_eq!(super::age(i64::MAX, i64::MIN), u64::try_from(i64::MAX)?);
        Ok(())
    }
}
