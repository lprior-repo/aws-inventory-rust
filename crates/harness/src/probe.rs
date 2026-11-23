mod command;
mod error;
mod outcome;
mod report;

pub use command::command;
pub use error::ProbeError;

use aws_wire::transport::Client;
use futures_util::{StreamExt, stream};
use inventory_core::{
    catalog::runtime::{BindingSource, CallablePlan, PlanPurpose, RuntimeCatalog},
    inventory::RegionId,
};
use inventory_engine::limiter::ApiExecutor;
use inventory_sweep::{CredentialSource, call_once};
use report::{Outcome, Row};

pub async fn run(
    catalog: &RuntimeCatalog<'_>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    region: &RegionId,
    endpoint: &str,
) -> Result<String, ProbeError> {
    Probe {
        catalog,
        credentials,
        client,
        executor,
        region,
        endpoint,
    }
    .run()
    .await
}

struct Probe<'a, 'catalog, C> {
    catalog: &'a RuntimeCatalog<'catalog>,
    credentials: &'a C,
    client: &'a Client,
    executor: &'a ApiExecutor,
    region: &'a RegionId,
    endpoint: &'a str,
}

impl<C: CredentialSource> Probe<'_, '_, C> {
    async fn run(&self) -> Result<String, ProbeError> {
        let mut rows = Vec::new();
        rows.try_reserve_exact(self.catalog.plans().len())?;
        let tasks = self
            .catalog
            .plans()
            .iter()
            .filter(|plan| matches!(plan.purpose(), PlanPurpose::Enumeration))
            .map(|plan| self.row(plan));
        let mut replies = stream::iter(tasks).buffer_unordered(8);
        while let Some(row) = replies.next().await {
            rows.push(row);
        }
        report::render(rows)
    }

    async fn row<'a>(&self, plan: &'a CallablePlan<'_>) -> Row<'a> {
        let outcome = if plan
            .effective_bindings(plan.branches().first())
            .any(|binding| requires_parent(&binding.source))
        {
            Outcome::SkippedChild
        } else {
            self.outcome(plan).await
        };
        Row {
            service: plan.service_name(),
            operation: plan.operation_name(),
            protocol: plan.protocol(),
            outcome,
        }
    }

    async fn outcome(&self, plan: &CallablePlan<'_>) -> Outcome {
        match call_once(
            self.catalog,
            self.credentials,
            self.client,
            self.executor,
            plan,
            self.region,
            Some(self.endpoint),
        )
        .await
        {
            Ok(answer) if answer.items == 0 => Outcome::Empty,
            Ok(answer) => Outcome::Items(answer.items),
            Err(error) => outcome::failed(plan, &error),
        }
    }
}

fn requires_parent(source: &BindingSource) -> bool {
    match source {
        BindingSource::Account
        | BindingSource::Region
        | BindingSource::RegionList
        | BindingSource::Fixed(_) => false,
        BindingSource::Transformed { source, .. } => requires_parent(source),
        BindingSource::Alternatives(_)
        | BindingSource::ContextFromBranch { .. }
        | BindingSource::ParentIdentity { .. }
        | BindingSource::Operation { .. }
        | BindingSource::OperationArn { .. }
        | BindingSource::OperationUrl { .. }
        | BindingSource::Context { .. } => true,
    }
}
