use crate::HarnessError;

mod cleanup;
pub mod command;
mod creation;
mod database;
mod names;
mod prune;
pub mod runtime;
pub(crate) mod worker;

pub use cleanup::{CleanupFailure, CleanupFailures};

pub use database::TestDatabase;
pub use prune::prune;

const URL_LIMIT: usize = 8_192;
const DATABASE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

pub const NATIVE_INSTRUCTIONS: &str = "No working Podman or Docker runtime. Use a native PostgreSQL server: see deploy/local/README.md. Set INVENTORY_POSTGRES_URL to its administrative URL; or run inventory-harness postgres up, then export INVENTORY_POSTGRES_URL=\"$(inventory-harness postgres url)\".";

pub fn admin_url() -> Result<String, HarnessError> {
    let url = std::env::var("INVENTORY_POSTGRES_URL").map_err(|_| {
        HarnessError::invalid("Set INVENTORY_POSTGRES_URL to an existing PostgreSQL server. Run inventory-harness postgres up and inventory-harness postgres url, or follow deploy/local/README.md for a native server.")
    })?;
    database::options(&url)?;
    Ok(url)
}

async fn bounded<T>(
    future: impl Future<Output = Result<T, HarnessError>>,
) -> Result<T, HarnessError> {
    tokio::time::timeout(DATABASE_TIMEOUT, future)
        .await
        .map_err(|_| HarnessError::invalid("PostgreSQL harness operation exceeded 15 seconds"))?
}

#[cfg(test)]
mod tests;
