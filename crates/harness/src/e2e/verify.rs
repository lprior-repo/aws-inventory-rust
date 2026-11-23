use crate::HarnessError;
use sqlx::{Connection, Row, postgres::PgConnection};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};

type Keys = BTreeMap<String, BTreeSet<(String, String, String)>>;

pub fn read_text(path: &Path) -> Result<String, HarnessError> {
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(16_777_217)
        .read_to_string(&mut text)?;
    if text.len() > 16_777_216 {
        return Err(HarnessError::invalid("e2e text exceeds 16 MiB"));
    }
    Ok(text)
}

pub fn zero(summary: &str, metric: &str) -> Result<(), HarnessError> {
    let value = metric_value(summary, metric)?;
    if value != 0 {
        return Err(HarnessError::invalid(format!(
            "collector {metric}={value}; expected 0"
        )));
    }
    Ok(())
}

fn metric_value(summary: &str, metric: &str) -> Result<u64, HarnessError> {
    let line = summary
        .lines()
        .rev()
        .find(|line| line.starts_with("resource_writes_scope=invocation "))
        .ok_or_else(|| HarnessError::invalid("collector summary missing"))?;
    line.split_whitespace()
        .find_map(|field| {
            field
                .split_once('=')
                .filter(|(key, _)| *key == metric)
                .map(|(_, value)| value)
        })
        .ok_or_else(|| HarnessError::invalid(format!("collector summary lacks {metric}")))?
        .parse()
        .map_err(|_| HarnessError::invalid(format!("collector summary has invalid {metric}")))
}

pub fn outcome(summary: &str) -> String {
    [
        "resource_writes",
        "rows",
        "complete",
        "partial",
        "unsupported",
        "failed",
        "work_remaining",
    ]
    .iter()
    .filter_map(|metric| {
        metric_value(summary, metric)
            .ok()
            .map(|value| format!("{metric}={value}"))
    })
    .collect::<Vec<_>>()
    .join(" ")
}

pub async fn counts(database: &str, oracle: &Path) -> Result<(), HarnessError> {
    let expected = oracle_keys(&read_text(oracle)?)?;
    let mut connection = PgConnection::connect(database).await?;
    let result = compare_database(&mut connection, &expected).await;
    let closed = connection.close().await.map_err(HarnessError::from);
    result.and(closed)
}
async fn compare_database(
    connection: &mut PgConnection,
    expected: &Keys,
) -> Result<(), HarnessError> {
    let kinds: Vec<_> = expected.keys().map(String::as_str).collect();
    let rows = sqlx::query("SELECT kind,account,region,id FROM inventory.resources WHERE kind=ANY($1) ORDER BY kind,account,region,id LIMIT 100001")
        .bind(kinds).fetch_all(connection).await?;
    if rows.len() > 100_000 {
        return Err(HarnessError::invalid(
            "count verification exceeds 100000 resources",
        ));
    }
    let actual = rows.into_iter().try_fold(Keys::new(), |mut keys, row| {
        keys.entry(row.try_get("kind")?).or_default().insert((
            row.try_get("account")?,
            row.try_get("region")?,
            row.try_get("id")?,
        ));
        Ok::<_, HarnessError>(keys)
    })?;
    compare(expected, &actual)
}

fn oracle_keys(text: &str) -> Result<Keys, HarnessError> {
    let keys = text
        .lines()
        .filter(|line| !line.is_empty())
        .try_fold(Keys::new(), oracle_row)?;
    if keys.is_empty() {
        return Err(HarnessError::invalid(
            "seed oracle contains no resources; nothing verified",
        ));
    }
    Ok(keys)
}

fn oracle_row(mut keys: Keys, line: &str) -> Result<Keys, HarnessError> {
    let fields: Vec<_> = line.split('\t').collect();
    match fields.as_slice() {
        ["resource", kind, account, region, id]
            if [kind, account, region, id]
                .iter()
                .all(|value| !value.is_empty()) =>
        {
            let key = ((*account).into(), (*region).into(), (*id).into());
            if !keys.entry((*kind).into()).or_default().insert(key) {
                return Err(HarnessError::invalid("duplicate oracle resource"));
            }
        }
        ["edge", _, _, _, _, _] => {}
        _ => return Err(HarnessError::invalid("malformed seed oracle row")),
    }
    Ok(keys)
}

fn compare(expected: &Keys, actual: &Keys) -> Result<(), HarnessError> {
    expected.iter().try_for_each(|(kind, keys)| {
        let found = actual.get(kind);
        let count = found.map_or(0, BTreeSet::len);
        println!("count {kind} expected={} actual={count}", keys.len());
        if found != Some(keys) {
            return Err(HarnessError::invalid(format!(
                "seeded identities/counts differ for {kind}"
            )));
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests;
