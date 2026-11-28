use super::{
    super::{
        Keys,
        coverage_status::{self, Evidence, Probe, Selection},
        read_text,
    },
    Coverage,
};
use crate::HarnessError;
use sqlx::{Connection, postgres::PgConnection};
use std::path::Path;

impl Coverage {
    pub(in crate::e2e::verify) async fn report(
        &self,
        database: &str,
        work: &Path,
        expected: &Keys,
        selection: Selection<'_>,
    ) -> Result<(), HarnessError> {
        let matrix = read_text(&work.join("seed-matrix.tsv"))?;
        let mut connection = PgConnection::connect(database).await?;
        let result = collect(&mut connection, expected).await;
        let closed = connection.close().await;
        let mut evidence = result?;
        closed?;
        self.rejected(&matrix, &mut evidence)?;
        coverage_status::publish(
            &work.join("coverage-status.report"),
            &self.callable,
            &evidence,
            selection,
        )
    }

    fn rejected(&self, matrix: &str, evidence: &mut Evidence) -> Result<(), HarnessError> {
        matrix
            .lines()
            .filter_map(|line| line.split_once("\trejected\t"))
            .try_for_each(|(step, rest)| {
                let kind = self.steps.get(step).ok_or_else(|| {
                    HarnessError::invalid("rejected seed matrix step is not in recipes")
                })?;
                let (code, _) = rest.split_once('\t').ok_or_else(|| {
                    HarnessError::invalid("rejected seed matrix row is malformed")
                })?;
                if !kind.is_empty() {
                    evidence.reject(kind, error_code(code).map_or("unknown", |code| code));
                }
                Ok(())
            })
    }
}

async fn collect(connection: &mut PgConnection, expected: &Keys) -> Result<Evidence, HarnessError> {
    let sweep: String = sqlx::query_scalar(
        "SELECT sweep_id FROM inventory.sweeps ORDER BY started_at DESC,sweep_id DESC LIMIT 1",
    )
    .fetch_one(&mut *connection)
    .await?;
    let mut evidence = probes(connection, &sweep).await?;
    if !expected.is_empty() {
        observations(connection, &sweep, expected, &mut evidence).await?;
    }
    Ok(evidence)
}

async fn probes(connection: &mut PgConnection, sweep: &str) -> Result<Evidence, HarnessError> {
    let rows: Vec<(Vec<u8>, String, Option<String>)> = sqlx::query_as(
        "SELECT scope_key,outcome,detail FROM inventory.scope_outcomes WHERE sweep_id=$1 AND outcome IN ('complete','unsupported','not_applicable','failed') LIMIT 100001",
    )
    .bind(sweep).fetch_all(connection).await?;
    check_limit(rows.len())?;
    rows.into_iter().try_fold(
        Evidence::default(),
        |mut evidence, (key, outcome, detail)| {
            let probe = match outcome.as_str() {
                "complete" => Probe::Complete,
                "unsupported" => Probe::Unsupported,
                "not_applicable" => Probe::NotApplicable,
                "failed" => Probe::Failed(failed_code(detail.as_deref())),
                _ => return Err(HarnessError::invalid("invalid coverage scope outcome")),
            };
            evidence.probe(operation(&key)?, probe);
            Ok(evidence)
        },
    )
}

async fn observations(
    connection: &mut PgConnection,
    sweep: &str,
    expected: &Keys,
    evidence: &mut Evidence,
) -> Result<(), HarnessError> {
    let kinds: Vec<_> = expected.keys().map(String::as_str).collect();
    let rows: Vec<(String, String, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT kind,account,region,id,scope_key FROM inventory.resource_scopes WHERE observed_sweep=$1 AND kind=ANY($2) LIMIT 100001",
    )
    .bind(sweep).bind(kinds).fetch_all(connection).await?;
    check_limit(rows.len())?;
    rows.into_iter()
        .try_for_each(|(kind, account, region, id, key)| {
            let identity = (account, region, id);
            if expected
                .get(&kind)
                .is_some_and(|ids| ids.contains(&identity))
            {
                evidence.observe(kind, operation(&key)?);
            }
            Ok(())
        })
}

fn operation(key: &[u8]) -> Result<String, HarnessError> {
    let scope = inventory_engine::inventory::decode_scope_key(key)
        .map_err(|error| HarnessError::invalid(format!("persisted coverage scope: {error}")))?;
    Ok(format!(
        "{}:{}",
        scope.service().as_str(),
        scope.operation().as_str()
    ))
}

fn check_limit(rows: usize) -> Result<(), HarnessError> {
    if rows > 100_000 {
        return Err(HarnessError::invalid(
            "coverage evidence exceeds 100000 rows",
        ));
    }
    Ok(())
}

fn failed_code(detail: Option<&str>) -> &str {
    let Some(detail) = detail else {
        return "unknown";
    };
    if let Some((_, failure)) = detail.split_once("AWS service failure: ") {
        let code = failure
            .split_once(" (")
            .and_then(|(_, tail)| tail.strip_suffix(')'))
            .map_or(failure, |code| code);
        return error_code(code).map_or("unknown", |code| code);
    }
    detail
        .split_once(':')
        .and_then(|(category, _)| error_code(category))
        .map_or("unknown", |code| code)
}

fn error_code(code: &str) -> Option<&str> {
    (!code.is_empty()
        && code.len() <= 128
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'#')))
    .then_some(code)
}

#[cfg(test)]
mod tests;
