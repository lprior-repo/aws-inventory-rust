mod backend;
mod summary;

use crate::cli::SweepArgs;
use anyhow::Result;
use inventory_engine::inventory::{InventoryStore, run_projection};
use inventory_sweep::SweepSummary;

pub(super) async fn run(options: SweepArgs) -> Result<std::process::ExitCode> {
    backend::run(&options).await
}

async fn finish<S: InventoryStore>(
    store: &S,
    options: &SweepArgs,
    mut summary: SweepSummary,
    unattributed_resources: u64,
) -> Result<std::process::ExitCode> {
    if options.rebuild {
        store.request_graph_rebuild().await?;
    }
    let projection_attempted = options.graph_url.is_some();
    if let Some(endpoint) = &options.graph_url {
        project(store, endpoint, &options.graph_name).await?;
    }
    let state = store.projection_state().await?;
    summary.graph_pending = state.pending;
    report(
        options,
        summary,
        unattributed_resources,
        projection_attempted,
        &state,
    )
}

fn report(
    options: &SweepArgs,
    summary: SweepSummary,
    unattributed: u64,
    attempted: bool,
    state: &inventory_engine::inventory::ProjectionState,
) -> Result<std::process::ExitCode> {
    let code = exit_code(
        &summary.outcomes,
        summary.work_remaining,
        unattributed,
        options.drain,
        attempted,
        state,
    );
    summary::print_summary(summary, unattributed)?;
    Ok(code)
}

fn exit_code(
    outcomes: &inventory_sweep::ScopeOutcomes,
    work_remaining: u64,
    unattributed_resources: u64,
    drain: bool,
    projection_attempted: bool,
    state: &inventory_engine::inventory::ProjectionState,
) -> std::process::ExitCode {
    if work_remaining > 0 {
        return std::process::ExitCode::from(4);
    }
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
        return std::process::ExitCode::from(2);
    }
    if projection_attempted && (state.pending > 0 || state.rebuild_required) {
        return std::process::ExitCode::from(5);
    }
    std::process::ExitCode::SUCCESS
}

async fn project<S: InventoryStore>(store: &S, endpoint: &str, graph: &str) -> Result<()> {
    let backend = inventory_graph::falkor::FalkorDb::connect(endpoint, graph)?;
    for _ in 0..100_000 {
        let report = run_projection(store, &backend, 1000).await?;
        if report.offline
            || report.rebuild_incomplete
            || report.pending == 0
            || report.acknowledged == 0
        {
            return Ok(());
        }
    }
    anyhow::bail!("projection exceeds its 100000-batch budget")
}

#[cfg(test)]
mod tests {
    use super::exit_code;
    use inventory_engine::inventory::ProjectionState;
    use inventory_sweep::ScopeOutcomes;
    use std::process::ExitCode;

    #[test]
    fn given_failed_scope_without_partial_when_drained_then_exit_is_incomplete() {
        let outcomes = ScopeOutcomes {
            failed: 1,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: false,
        };
        assert_eq!(
            exit_code(&outcomes, 0, 0, true, false, &state),
            ExitCode::from(2)
        );
    }

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
        assert_eq!(
            exit_code(&outcomes, 0, 1, false, false, &state),
            ExitCode::from(2)
        );
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, false, &state),
            ExitCode::SUCCESS
        );
        let partial = ScopeOutcomes {
            partial: 1,
            ..outcomes
        };
        assert_eq!(
            exit_code(&partial, 0, 0, false, false, &state),
            ExitCode::from(2)
        );
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
        assert_eq!(
            exit_code(&outcomes, 0, 0, true, true, &state),
            ExitCode::from(3)
        );
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, false, &state),
            ExitCode::SUCCESS
        );
    }

    #[test]
    fn given_complete_scopes_when_graph_attempted_and_pending_then_exit_is_pending() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 3,
            rebuild_required: false,
        };
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, true, &state),
            ExitCode::from(5)
        );
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
        assert_eq!(
            exit_code(&outcomes, 0, 0, true, true, &state),
            ExitCode::from(3)
        );
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, false, &state),
            ExitCode::SUCCESS
        );
    }

    #[test]
    fn given_complete_scopes_when_graph_attempted_and_rebuild_then_exit_is_pending() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: true,
        };
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, true, &state),
            ExitCode::from(5)
        );
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
        assert_eq!(
            exit_code(&outcomes, 0, 0, true, false, &state),
            ExitCode::SUCCESS
        );
    }

    #[test]
    fn given_slice_ended_with_work_remaining_when_exit_code_chosen_then_four_outranks_every_other_status()
     {
        let failed = ScopeOutcomes {
            complete: 3,
            failed: 2,
            ..ScopeOutcomes::default()
        };
        let pending = ProjectionState {
            pending: 9,
            rebuild_required: true,
        };
        assert_eq!(
            exit_code(&failed, 5, 1, true, true, &pending),
            ExitCode::from(4)
        );
        assert_eq!(
            exit_code(&failed, 0, 1, true, true, &pending),
            ExitCode::from(3)
        );
    }

    #[test]
    fn given_complete_scopes_when_graph_complete_then_exit_is_success() {
        let outcomes = ScopeOutcomes {
            complete: 7,
            ..ScopeOutcomes::default()
        };
        let state = ProjectionState {
            pending: 0,
            rebuild_required: false,
        };
        assert_eq!(
            exit_code(&outcomes, 0, 0, false, true, &state),
            ExitCode::SUCCESS
        );
    }
}
