#[path = "state/file.rs"]
mod file;
#[path = "state/graph.rs"]
mod graph;
#[path = "state/oracle.rs"]
mod oracle;
#[path = "state/postgres.rs"]
mod postgres;

use anyhow::{Context, Result, ensure};
use inventory_engine::inventory::ProjectionEdgeHeader;
use inventory_harness::{inventory_seed::TypedSeedOracle, postgres::TestDatabase};
use std::path::{Path, PathBuf};

pub(super) use graph::assert_graph;
type MissCounters = Vec<(String, String, String, String, Vec<u8>, i64)>;

pub(super) enum Database {
    File(PathBuf),
    Postgres(TestDatabase),
}

impl Database {
    pub(super) async fn create(
        environment: &super::guard::Environment,
        path: PathBuf,
    ) -> Result<Self> {
        match &environment.database_url {
            Some(url) => Ok(Self::Postgres(TestDatabase::create(url).await?)),
            None => Ok(Self::File(path)),
        }
    }

    pub(super) fn argument(&self) -> Result<String> {
        match self {
            Self::File(path) => path
                .to_str()
                .map(str::to_owned)
                .context("E2E paths must be UTF-8"),
            Self::Postgres(database) => Ok(database.url().to_owned()),
        }
    }

    async fn edges(&self) -> Result<Vec<ProjectionEdgeHeader>> {
        match self {
            Self::File(path) => file::edges(path).await,
            Self::Postgres(database) => postgres::edges(database.url()).await,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct InventorySnapshot {
    pub(super) resources: Vec<(inventory_core::inventory::ResourceKey, [u8; 32])>,
    pub(super) edges: Vec<ProjectionEdgeHeader>,
    pub(super) miss_counters: MissCounters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ActiveCheckpoint {
    pub(super) sweep_id: String,
    pub(super) resources_observed: u64,
    pub(super) work_pending: u64,
}

pub(super) async fn database_snapshot(
    database: &Database,
    oracle: &TypedSeedOracle,
) -> Result<InventorySnapshot> {
    let snapshot = match database {
        Database::File(path) => file::snapshot(path).await?,
        Database::Postgres(database) => postgres::snapshot(database.url()).await?,
    };
    oracle::assert_seeded_rows(oracle, &snapshot)?;
    Ok(snapshot)
}

pub(super) async fn active_checkpoint(database: &Database) -> Result<ActiveCheckpoint> {
    match database {
        Database::File(path) => file::active(path).await,
        Database::Postgres(database) => postgres::active(database.url()).await,
    }
}

pub(super) async fn assert_sweep_finished(database: &Database, sweep: &str) -> Result<()> {
    match database {
        Database::File(path) => file::finished(path, sweep).await,
        Database::Postgres(database) => postgres::finished(database.url(), sweep).await,
    }
}
