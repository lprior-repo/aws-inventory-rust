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
    catalog::runtime::{CallablePlan, PlanPurpose, RuntimeCatalog},
    inventory::{AccountId, RegionId},
};
use inventory_engine::limiter::ApiExecutor;
use inventory_sweep::{
    ChildProbeSamples, CredentialSource, ProbeAnswer, ProbeChildren, ProbeScope, SweepError,
    call_once, probe_children,
};
use report::{Outcome, Row};
use std::time::{Duration, Instant};

type RootResult<'a, 'catalog> = (&'a CallablePlan<'catalog>, Result<ProbeAnswer, SweepError>);
type ParentPage<'a, 'catalog> = (&'a CallablePlan<'catalog>, ProbeAnswer);

type ChildResult<'a, 'catalog> = (
    &'a CallablePlan<'catalog>,
    inventory_sweep::ChildProbeOutcome,
);

struct ProbeReport {
    text: String,
    root_wall: Duration,
    child_error: Option<SweepError>,
}

pub async fn run(
    catalog: &RuntimeCatalog<'_>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    region: &RegionId,
    endpoint: &str,
    services: &[String],
) -> Result<String, ProbeError> {
    let report = Probe {
        catalog,
        credentials,
        client,
        executor,
        region,
        endpoint,
        services,
    }
    .run(ChildProbeSamples::default())
    .await?;
    if let Some(error) = report.child_error {
        return Err(ProbeError::ChildSchedule(error));
    }
    Ok(report.text)
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

impl<'a, 'catalog, C: CredentialSource> Probe<'a, 'catalog, C> {
    async fn run(&self, child_samples: ChildProbeSamples) -> Result<ProbeReport, ProbeError> {
        validate_services(self.catalog, self.services)?;
        let started = Instant::now();
        let identity = self.bootstrap().await;
        let account = identity_account(identity.as_ref()).cloned();
        let plans = self
            .catalog
            .plans()
            .iter()
            .filter(|plan| self.selected(plan.service_name()))
            .collect::<Vec<_>>();
        let mut roots = self.roots(&plans, account.as_ref()).await;
        roots.extend(identity);
        let root_wall = started.elapsed();
        let mut rows = Vec::new();
        rows.try_reserve_exact(plans.len())?;
        let parents = self.parent_pages(roots, &mut rows)?;
        let children = probe_children(
            self.catalog,
            self.credentials,
            self.client,
            self.executor,
            self.scope(account.as_ref()),
            ProbeChildren::new(&plans, parents, child_samples),
        )
        .await;
        let child_error = append_children(&mut rows, &plans, children)?;
        Ok(ProbeReport {
            text: report::render(rows)?,
            root_wall,
            child_error,
        })
    }

    async fn roots(
        &self,
        plans: &[&'a CallablePlan<'catalog>],
        account: Option<&AccountId>,
    ) -> Vec<RootResult<'a, 'catalog>> {
        let tasks = plans
            .iter()
            .copied()
            .filter(|plan| !matches!(plan.purpose(), PlanPurpose::AccountIdentity { .. }))
            .filter(|plan| !ProbeAnswer::requires_parent(plan))
            .map(|plan| async move { (plan, self.call(plan, account).await) });
        stream::iter(tasks).buffer_unordered(8).collect().await
    }

    fn parent_pages(
        &self,
        roots: Vec<RootResult<'a, 'catalog>>,
        rows: &mut Vec<Row<'a>>,
    ) -> Result<Vec<ParentPage<'a, 'catalog>>, ProbeError> {
        let mut parents = Vec::new();
        parents.try_reserve_exact(roots.len())?;
        roots.into_iter().for_each(|(plan, answer)| {
            if self.selected(plan.service_name()) {
                rows.push(row(plan, outcome(plan, answer.as_ref())));
            }
            if let Ok(answer) = answer
                && answer.items != 0
            {
                parents.push((plan, answer));
            }
        });
        Ok(parents)
    }

    fn selected(&self, service: &str) -> bool {
        self.services.is_empty() || self.services.iter().any(|selected| selected == service)
    }

    async fn bootstrap(&self) -> Option<RootResult<'a, 'catalog>> {
        let plan = self
            .catalog
            .plans()
            .iter()
            .find(|plan| matches!(plan.purpose(), PlanPurpose::AccountIdentity { .. }))?;
        Some((plan, self.call(plan, None).await))
    }

    fn scope<'scope>(&'scope self, account: Option<&'scope AccountId>) -> ProbeScope<'scope> {
        ProbeScope {
            region: self.region,
            account,
            endpoint_override: Some(self.endpoint),
        }
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
            self.scope(account),
        )
        .await
    }
}

fn identity_account<'a>(identity: Option<&'a RootResult<'_, '_>>) -> Option<&'a AccountId> {
    identity
        .and_then(|(_, answer)| answer.as_ref().ok())
        .and_then(|answer| answer.account.as_ref())
}

fn append_children<'a, 'catalog>(
    rows: &mut Vec<Row<'a>>,
    plans: &[&'a CallablePlan<'catalog>],
    children: Result<Vec<ChildResult<'a, 'catalog>>, SweepError>,
) -> Result<Option<SweepError>, ProbeError> {
    match children {
        Ok(children) => rows.extend(
            children
                .into_iter()
                .map(|(plan, result)| row(plan, outcome::child(plan, result))),
        ),
        Err(error) => {
            let detail = std::sync::Arc::from(outcome::schedule_detail(&error)?);
            rows.extend(
                plans
                    .iter()
                    .copied()
                    .filter(|plan| ProbeAnswer::requires_parent(plan))
                    .map(|plan| row(plan, Outcome::FailedPlan(std::sync::Arc::clone(&detail)))),
            );
            return Ok(Some(error));
        }
    }
    Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_child_schedule_error_when_rendering_then_root_evidence_is_preserved()
    -> Result<(), ProbeError> {
        let catalog =
            RuntimeCatalog::from_bytes(include_bytes!("../../../catalog/aws-api.catalog"))?;
        let root = catalog
            .lookup("sns", "ListTopics")
            .ok_or(ProbeError::Invalid("missing root fixture"))?;
        let child = catalog
            .lookup("sns", "ListSubscriptionsByTopic")
            .ok_or(ProbeError::Invalid("missing child fixture"))?;
        let mut rows = vec![row(root, Outcome::Items(4))];
        let error = append_children(
            &mut rows,
            &[root, child],
            Err(SweepError::Operation("consumer index\ncould not load")),
        )?;
        assert!(matches!(
            error,
            Some(SweepError::Operation("consumer index\ncould not load"))
        ));
        assert_eq!(
            report::render(rows)?,
            concat!(
                "sns ListSubscriptionsByTopic awsQuery failed plan sweep operation failed: consumer index could not load\n",
                "sns ListTopics awsQuery items 4\n",
                "total items 1\ntotal empty 0\ntotal not-applicable 0\ntotal unsupported 0\ntotal failed 1\ntotal skipped-child 0\n",
                "protocol awsQuery items 1 empty 0 not-applicable 0 unsupported 0 failed 1 skipped-child 0\n",
            )
        );
        Ok(())
    }
}
