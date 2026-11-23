use super::{Baseline, Keys};
use crate::HarnessError;
use sqlx::{Connection, Row, postgres::PgConnection};
use std::{io::Write, path::Path};

pub async fn capture_baseline(database: &str, artifact: &Path) -> Result<Baseline, HarnessError> {
    let keys = read(database, None).await?;
    let mut output = std::io::BufWriter::new(std::fs::File::create(artifact)?);
    keys.iter().try_for_each(|(kind, identities)| {
        identities.iter().try_for_each(|(account, region, id)| {
            writeln!(output, "resource\t{kind}\t{account}\t{region}\t{id}")
        })
    })?;
    output.flush()?;
    Ok(Baseline(keys))
}

pub(super) async fn read(database: &str, kinds: Option<&[&str]>) -> Result<Keys, HarnessError> {
    let mut connection = PgConnection::connect(database).await?;
    let result = read_connection(&mut connection, kinds).await;
    let closed = connection.close().await.map_err(HarnessError::from);
    result.and_then(|keys| closed.map(|()| keys))
}

async fn read_connection(
    connection: &mut PgConnection,
    kinds: Option<&[&str]>,
) -> Result<Keys, HarnessError> {
    let rows = sqlx::query("SELECT kind,account,region,id FROM inventory.resources WHERE ($1::text[] IS NULL OR kind=ANY($1)) ORDER BY kind,account,region,id LIMIT 100001")
        .bind(kinds).fetch_all(connection).await?;
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

pub(super) fn added(actual: Keys, baseline: &Baseline) -> Keys {
    actual
        .into_iter()
        .map(|(kind, mut identities)| {
            if let Some(before) = baseline.0.get(&kind) {
                identities.retain(|identity| !before.contains(identity));
            }
            (kind, identities)
        })
        .collect()
}
