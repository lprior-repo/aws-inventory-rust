mod command;
mod compare;
mod error;
mod outcome;
mod report;

pub use command::command;
pub use error::ProbeError;

use aws_wire::transport::Client;
use futures_util::{StreamExt, stream};
use inventory_core::{
    catalog::runtime::{BindingSource, CallablePlan, PlanPurpose, RuntimeCatalog},
    inventory::{AccountId, RegionId},
};
use inventory_engine::limiter::ApiExecutor;
use inventory_sweep::{CredentialSource, ProbeAnswer, ProbeScope, SweepError, call_once};
use report::{Outcome, Row};

pub async fn run(
    catalog: &RuntimeCatalog<'_>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    region: &RegionId,
    endpoint: &str,
    services: &[String],
) -> Result<String, ProbeError> {
    validate_services(catalog, services)?;
    Probe {
        catalog,
        credentials,
        client,
        executor,
        region,
        endpoint,
        services,
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
    services: &'a [String],
}

impl<C: CredentialSource> Probe<'_, '_, C> {
    async fn run(&self) -> Result<String, ProbeError> {
        let (identity, account) = self.bootstrap().await;
        let mut rows = Vec::new();
        rows.try_reserve_exact(self.selected_count())?;
        rows.extend(identity.filter(|row| self.selected(row.service)));
        let tasks = self
            .catalog
            .plans()
            .iter()
            .filter(|plan| !matches!(plan.purpose(), PlanPurpose::AccountIdentity { .. }))
            .filter(|plan| self.selected(plan.service_name()))
            .map(|plan| self.row(plan, account.as_ref()));
        let rows = stream::iter(tasks)
            .buffer_unordered(8)
            .fold(rows, |mut rows, row| async move {
                rows.push(row);
                rows
            })
            .await;
        report::render(rows)
    }

    fn selected(&self, service: &str) -> bool {
        self.services.is_empty() || self.services.iter().any(|selected| selected == service)
    }

    fn selected_count(&self) -> usize {
        self.catalog
            .plans()
            .iter()
            .filter(|plan| self.selected(plan.service_name()))
            .count()
    }

    async fn bootstrap(&self) -> (Option<Row<'_>>, Option<AccountId>) {
        let plan = self
            .catalog
            .plans()
            .iter()
            .find(|plan| matches!(plan.purpose(), PlanPurpose::AccountIdentity { .. }));
        let Some(plan) = plan else {
            return (None, None);
        };
        let answer = self.call(plan, None).await;
        let row = row(plan, outcome(plan, answer.as_ref()));
        let account = answer.ok().and_then(|answer| answer.account);
        (Some(row), account)
    }

    async fn row<'a>(&self, plan: &'a CallablePlan<'_>, account: Option<&AccountId>) -> Row<'a> {
        let outcome = if matches!(plan.purpose(), PlanPurpose::TagEndpoint(_))
            || plan
                .effective_bindings(plan.branches().first())
                .any(|binding| requires_parent(&binding.source))
        {
            Outcome::SkippedChild
        } else {
            outcome(plan, self.call(plan, account).await.as_ref())
        };
        row(plan, outcome)
    }

    async fn call(
        &self,
        plan: &CallablePlan<'_>,
        account: Option<&AccountId>,
    ) -> Result<ProbeAnswer, SweepError> {
        call_once(
            self.catalog,
            self.credentials,
            self.client,
            self.executor,
            plan,
            ProbeScope {
                region: self.region,
                account,
                endpoint_override: Some(self.endpoint),
            },
        )
        .await
    }
}

fn validate_services(catalog: &RuntimeCatalog<'_>, services: &[String]) -> Result<(), ProbeError> {
    if services.iter().any(|service| {
        !catalog
            .plans()
            .iter()
            .any(|plan| plan.service_name() == service)
    }) {
        return Err(ProbeError::Invalid("unknown probe service"));
    }
    Ok(())
}

fn row<'a>(plan: &'a CallablePlan<'_>, outcome: Outcome) -> Row<'a> {
    Row {
        service: plan.service_name(),
        operation: plan.operation_name(),
        protocol: plan.protocol(),
        outcome,
    }
}

fn outcome(plan: &CallablePlan<'_>, answer: Result<&ProbeAnswer, &SweepError>) -> Outcome {
    match answer {
        Ok(answer) if answer.items == 0 => Outcome::Empty,
        Ok(answer) => Outcome::Items(answer.items),
        Err(error) => outcome::failed(plan, error),
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
