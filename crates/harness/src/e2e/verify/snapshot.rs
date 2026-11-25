use super::{Baseline, Counts, Keys};
use crate::HarnessError;
use sqlx::{Connection, Row, postgres::PgConnection};
use std::{io::Write, path::Path};

pub async fn capture_baseline(database: &str, artifact: &Path) -> Result<Baseline, HarnessError> {
    let mut connection = PgConnection::connect(database).await?;
    let result = read_connection(&mut connection).await;
    let closed = connection.close().await.map_err(HarnessError::from);
    let keys = result?;
    closed?;
    let mut output = std::io::BufWriter::new(std::fs::File::create(artifact)?);
    keys.iter().try_for_each(|(kind, identities)| {
        identities.iter().try_for_each(|(account, region, id)| {
            writeln!(output, "resource\t{kind}\t{account}\t{region}\t{id}")
        })
    })?;
    output.flush()?;
    Ok(Baseline(keys))
}

pub(super) async fn read_snapshot(database: &str) -> Result<(Keys, Counts), HarnessError> {
    let mut connection = PgConnection::connect(database).await?;
    let result = read_transaction(&mut connection).await;
    let closed = connection.close().await.map_err(HarnessError::from);
    result.and_then(|snapshot| closed.map(|()| snapshot))
}

async fn read_transaction(connection: &mut PgConnection) -> Result<(Keys, Counts), HarnessError> {
    let mut tx = connection
        .begin_with("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .await?;
    let snapshot = read_snapshot_queries(&mut tx).await?;
    tx.commit().await?;
    Ok(snapshot)
}

async fn read_snapshot_queries(
    connection: &mut PgConnection,
) -> Result<(Keys, Counts), HarnessError> {
    let rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT kind, count(*) FROM inventory.resources GROUP BY kind")
            .fetch_all(&mut *connection)
            .await?;
    let actual = read_connection(connection).await?;
    let counts = rows
        .into_iter()
        .map(|(kind, total)| {
            let count = usize::try_from(total).map_err(|_| {
                HarnessError::invalid(format!("invalid SQL resource count for {kind}"))
            })?;
            Ok((kind, count))
        })
        .collect::<Result<_, HarnessError>>()?;
    Ok((actual, counts))
}

pub(super) fn relevant(
    actual: &mut Keys,
    counts: &mut Counts,
    baseline: &Baseline,
    expected: &super::Expected,
    output: &mut impl Write,
) -> Result<(), HarnessError> {
    actual.iter_mut().try_for_each(|(kind, ids)| {
        let required = expected.required.get(kind);
        let deferred = expected.deferred.get(kind);
        let before = baseline.0.get(kind);
        if let Some((required, before)) = required.zip(before) {
            required
                .intersection(before)
                .filter(|id| ids.contains(*id))
                .try_for_each(|(account, region, id)| {
                    writeln!(output, "preexisting {kind} {account}/{region}/{id}")
                })?;
        }
        let total_ids = ids.len();
        ids.retain(|id| {
            if deferred.is_some_and(|keys| keys.contains(id)) {
                return false;
            }
            !before.is_some_and(|keys| keys.contains(id))
                || required.is_some_and(|keys| keys.contains(id))
        });
        let kept = counts.get_mut(kind).ok_or_else(|| {
            HarnessError::invalid(format!("missing SQL resource count for {kind}"))
        })?;
        let ignored = total_ids
            .checked_sub(ids.len())
            .ok_or_else(|| HarnessError::invalid(format!("invalid identity delta for {kind}")))?;
        *kept = kept.checked_sub(ignored).ok_or_else(|| {
            HarnessError::invalid(format!("invalid SQL resource count for {kind}"))
        })?;
        Ok(())
    })
}

async fn read_connection(connection: &mut PgConnection) -> Result<Keys, HarnessError> {
    let rows = sqlx::query("SELECT kind,account,region,id FROM inventory.resources ORDER BY kind,account,region,id LIMIT 100001")
        .fetch_all(connection).await?;
    if rows.len() > 100_000 {
        return Err(HarnessError::invalid(
            "identity verification exceeds 100000 resources",
        ));
    }
    rows.into_iter().try_fold(Keys::new(), |mut keys, row| {
        keys.entry(row.try_get("kind")?).or_default().insert((
            row.try_get("account")?,
            row.try_get("region")?,
            row.try_get("id")?,
        ));
        Ok(keys)
    })
}
