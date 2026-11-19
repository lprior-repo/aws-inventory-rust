use crate::{
    cli::SweepArgs,
    sweep_runtime::{self, SweepConfig, SweepSummary},
};
use anyhow::{Context, Result};
use inventory_aws::model_credentials::ProfileCredentials;
use inventory_core::catalog::runtime::RuntimeCatalog;
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, run_projection};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub(super) async fn run(options: SweepArgs) -> Result<std::process::ExitCode> {
    let (store, mut summary) = collect(&options).await?;
    let unattributed_resources = store.unattributed_authority_count().await?;
    if let Some(endpoint) = &options.graph_url {
        project(&store, endpoint, &options.graph_name).await?;
    }
    let state = store.projection_state().await?;
    summary.graph_pending = state.pending;
    let code = exit_code(
        &summary.outcomes,
        unattributed_resources,
        options.drain,
        &state,
    );
    print_summary(summary, unattributed_resources)?;
    Ok(code)
}

async fn collect(options: &SweepArgs) -> Result<(BinaryStore, SweepSummary)> {
    let bytes = read_catalog(&options.catalog)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let store = open_store(options, &catalog).await?;
    let credentials =
        ProfileCredentials::load(options.profile.as_deref(), options.region.as_deref()).await?;
    let summary = sweep_runtime::run(&catalog, &store, config(options), &credentials).await?;
    Ok((store, summary))
}

fn exit_code(
    outcomes: &sweep_runtime::ScopeOutcomes,
    unattributed_resources: u64,
    drain: bool,
    state: &inventory_engine::inventory::ProjectionState,
) -> std::process::ExitCode {
    if drain && (state.pending > 0 || state.rebuild_required) {
        return std::process::ExitCode::from(3);
    }
    if unattributed_resources > 0
        || outcomes.partial > 0
        || outcomes.inaccessible > 0
        || outcomes.unsupported > 0
        || outcomes.failed > 0
        || outcomes.unobserved > 0
    {
        std::process::ExitCode::from(2)
    } else {
        std::process::ExitCode::SUCCESS
    }
}

fn read_catalog(path: &Path) -> Result<Vec<u8>> {
    let limit = inventory_core::catalog::MAX_CATALOG_BYTES;
    let file = File::open(path).with_context(|| format!("opening catalog {}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(u64::try_from(limit)?.saturating_add(1))
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(
        bytes.len() <= limit,
        "runtime catalog exceeds {limit} bytes"
    );
    Ok(bytes)
}

async fn open_store(options: &SweepArgs, catalog: &RuntimeCatalog<'_>) -> Result<BinaryStore> {
    if std::fs::metadata(&options.database).is_ok_and(|metadata| metadata.len() > 0) {
        return Ok(BinaryStore::open_with_catalog(&options.database, catalog.raw_bytes()).await?);
    }
    anyhow::ensure!(!options.resume, "cannot resume a missing database");
    let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
    Ok(BinaryStore::create(&options.database, registry).await?)
}

fn config(options: &SweepArgs) -> SweepConfig {
    SweepConfig {
        region: options.region.clone(),
        max_pages: options.max_pages,
        concurrency: options.concurrency,
        resume: options.resume,
        endpoint: options.endpoint.clone(),
        ca_bundle: options.ca_bundle.clone(),
    }
}

async fn project(store: &BinaryStore, endpoint: &str, graph: &str) -> Result<()> {
    loop {
        let report = run_projection(store, endpoint, graph, 1000).await?;
        if report.offline
            || report.rebuild_incomplete
            || report.pending == 0
            || report.acknowledged == 0
        {
            return Ok(());
        }
    }
}

fn print_summary(summary: SweepSummary, unattributed_resources: u64) -> Result<()> {
    let outcomes = summary.outcomes;
    writeln!(
        std::io::stdout().lock(),
        "resource_writes_scope=invocation resource_writes={} rows={} scopes={} complete={} partial={} inaccessible={} unsupported={} failed={} unobserved={} graph_pending={} unattributed_resources={}",
        summary.resource_writes,
        summary.rows,
        summary.scopes,
        outcomes.complete,
        outcomes.partial,
        outcomes.inaccessible,
        outcomes.unsupported,
        outcomes.failed,
        outcomes.unobserved,
        summary.graph_pending,
        unattributed_resources
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{exit_code, sweep_runtime::ScopeOutcomes};
    use inventory_engine::inventory::ProjectionState;
    use std::process::ExitCode;

    #[test]
    fn given_complete_fresh_scopes_when_legacy_authority_is_unresolved_then_exit_is_incomplete() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: false,
        };
        assert_eq!(exit_code(&outcomes, 1, false, &state), ExitCode::from(2));
        assert_eq!(exit_code(&outcomes, 0, false, &state), ExitCode::SUCCESS);
        let partial = ScopeOutcomes {
            partial: 1,
            ..outcomes
        };
        assert_eq!(exit_code(&partial, 0, false, &state), ExitCode::from(2));
    }

    #[test]
    fn given_complete_scopes_when_drain_and_pending_then_exit_is_graph_pending() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 3,
            rebuild_required: false,
        };
        assert_eq!(exit_code(&outcomes, 0, true, &state), ExitCode::from(3));
        assert_eq!(exit_code(&outcomes, 0, false, &state), ExitCode::SUCCESS);
    }

    #[test]
    fn given_complete_scopes_when_drain_and_rebuild_required_then_exit_is_graph_pending() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: true,
        };
        assert_eq!(exit_code(&outcomes, 0, true, &state), ExitCode::from(3));
        assert_eq!(exit_code(&outcomes, 0, false, &state), ExitCode::SUCCESS);
    }

    #[test]
    fn given_complete_scopes_when_drain_and_drained_then_exit_is_success() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: false,
        };
        assert_eq!(exit_code(&outcomes, 0, true, &state), ExitCode::SUCCESS);
    }
}
