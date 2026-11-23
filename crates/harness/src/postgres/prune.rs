use futures_util::{StreamExt, TryStreamExt, stream};
use sqlx::Connection;

use super::{bounded, database, names};
use crate::HarnessError;

const PRUNE_LIMIT: usize = 256;

pub async fn prune(admin_url: &str, older_than_seconds: u64) -> Result<usize, HarnessError> {
    bounded(prune_inner(admin_url, older_than_seconds)).await
}

async fn prune_inner(admin_url: &str, older_than_seconds: u64) -> Result<usize, HarnessError> {
    let now = u64::from(names::now()?);
    let mut connection = database::connect(admin_url).await?;
    let candidates: Vec<String> = sqlx::query_scalar(
        "SELECT datname::text FROM pg_database WHERE starts_with(datname, 'inv_t_') ORDER BY datname LIMIT 257",
    ).fetch_all(&mut connection).await?;
    connection.close().await?;
    if candidates.len() > PRUNE_LIMIT {
        return Err(HarnessError::invalid("more than 256 test databases; prune manually before retrying"));
    }
    stream::iter(candidates.into_iter().filter(|name| is_old(name, now, older_than_seconds)))
        .map(Ok::<_, HarnessError>)
        .try_fold(0usize, |count, name| async move {
            database::drop_database(admin_url, &name).await?;
            Ok(count.saturating_add(1))
        }).await
}

fn is_old(name: &str, now: u64, age: u64) -> bool {
    names::created_at(name).is_some_and(|created| {
        now.checked_sub(u64::from(created)).is_some_and(|elapsed| elapsed > age)
    })
}
