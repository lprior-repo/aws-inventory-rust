use crate::{CredentialSource, SweepError, call, execution::routing, input, metrics, record};
use aws_wire::transport::Client;
use inventory_core::{
    catalog::runtime::{
        CallablePlan, ExecutionBranch, PlanPurpose, PlanScope, ResolvedEndpoint, RuntimeCatalog,
    },
    inventory::{AccountId, DiscoveryContext, OperationId, Region, RegionId, ServiceId},
    record::Record,
};
use inventory_engine::limiter::ApiExecutor;

mod children;
pub use children::{ChildProbeOutcome, ChildProbeSamples, ProbeChildren, probe_children};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProbeAnswer {
    pub items: u64,
    pub http_version: http::Version,
    pub account: Option<AccountId>,
    page: ProbePage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbePage {
    service: ServiceId,
    operation: OperationId,
    request: Record,
    response: Record,
    region: Region,
    branch: Option<usize>,
    ancestry: DiscoveryContext,
}

impl ProbeAnswer {
    #[must_use]
    pub fn requires_parent(plan: &CallablePlan<'_>) -> bool {
        matches!(plan.purpose(), PlanPurpose::TagEndpoint(_))
            || input::requires_parent_branch(plan, plan.branches().first())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProbeScope<'a> {
    pub region: &'a RegionId,
    pub account: Option<&'a AccountId>,
    pub endpoint_override: Option<&'a str>,
}

pub async fn call_once(
    catalog: &RuntimeCatalog<'_>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    plan: &CallablePlan<'_>,
    scope: ProbeScope<'_>,
) -> Result<ProbeAnswer, SweepError> {
    Probe {
        catalog,
        credentials,
        client,
        executor,
        plan,
        scope,
    }
    .run()
    .await
}

struct Probe<'a, 'catalog, C> {
    catalog: &'a RuntimeCatalog<'catalog>,
    credentials: &'a C,
    client: &'a Client,
    executor: &'a ApiExecutor,
    plan: &'a CallablePlan<'catalog>,
    scope: ProbeScope<'a>,
}

impl<C: CredentialSource> Probe<'_, '_, C> {
    async fn run(&self) -> Result<ProbeAnswer, SweepError> {
        self.validate()?;
        let scope = self.scope();
        let branch = self.plan.branches().first();
        let region = routing::request_region(
            self.plan,
            &scope,
            self.scope.region,
            branch.map(|branch| branch.discriminator.as_str()),
        )?;
        let endpoint = routing::resolve(self.plan, &region)?;
        let base =
            input::root_bindings_branch(self.plan, branch, self.scope.account, Some(&scope))?;
        self.first_page(
            &endpoint,
            base,
            scope,
            branch.map(|_| 0),
            DiscoveryContext::empty(),
        )
        .await
    }

    async fn first_page(
        &self,
        endpoint: &ResolvedEndpoint,
        base: Record,
        scope: Region,
        branch: Option<usize>,
        ancestry: DiscoveryContext,
    ) -> Result<ProbeAnswer, SweepError> {
        let fields = first_page_fields(self.plan);
        let response = self.request(endpoint, &base, &fields, &scope).await?;
        let (count, _) = record::item_selection(&response.record, self.plan.items())?;
        let account = self.account(&response.record)?;
        let request = if fields.is_empty() {
            base
        } else {
            input::merge_input(&base, &fields)?
        };
        Ok(ProbeAnswer {
            items: u64::try_from(count)
                .map_err(|_| SweepError::Operation("probe item count overflow"))?,
            http_version: response.http_version,
            account,
            page: ProbePage {
                service: ServiceId::parse(self.plan.service_name())?,
                operation: OperationId::parse(self.plan.operation_name())?,
                request,
                response: response.record,
                region: scope,
                branch,
                ancestry,
            },
        })
    }

    fn validate(&self) -> Result<(), SweepError> {
        let found = self
            .catalog
            .lookup(self.plan.service_name(), self.plan.operation_name());
        if !found.is_some_and(|plan| std::ptr::eq(plan, self.plan))
            || ProbeAnswer::requires_parent(self.plan)
        {
            return Err(SweepError::InvalidConfig(
                "probe requires a root callable from this catalog",
            ));
        }
        Ok(())
    }

    fn scope(&self) -> Region {
        match self.plan.scope() {
            PlanScope::Account => Region::Global,
            PlanScope::Region | PlanScope::HomeRegion => {
                Region::Regional(self.scope.region.clone())
            }
        }
    }

    fn account(&self, response: &Record) -> Result<Option<AccountId>, SweepError> {
        match self.plan.purpose() {
            PlanPurpose::AccountIdentity { account_path } => {
                crate::execution::bootstrap::account_id(response, account_path).map(Some)
            }
            _ => Ok(None),
        }
    }

    async fn request(
        &self,
        endpoint: &ResolvedEndpoint,
        base: &Record,
        fields: &[input::InputField],
        scope: &Region,
    ) -> Result<call::DecodedResponse, SweepError> {
        let metrics = metrics::Accumulators::default();
        call::Call {
            client: self.client,
            executor: self.executor,
            credentials: self.credentials,
            plan: self.plan,
            endpoint,
            url: self
                .scope
                .endpoint_override
                .map_or(endpoint.url.as_str(), |url| url),
            base,
            fields,
            metrics: &metrics,
            account: self
                .scope
                .account
                .map_or(crate::bucket::BOOTSTRAP_ACCOUNT, AccountId::as_str),
            scope,
        }
        .run()
        .await
    }
}

fn branch_at<'a>(
    plan: &'a CallablePlan<'_>,
    ordinal: Option<usize>,
) -> Option<&'a ExecutionBranch> {
    ordinal.and_then(|ordinal| plan.branches().get(ordinal))
}

fn first_page_fields(plan: &CallablePlan<'_>) -> Vec<input::InputField> {
    plan.pagination()
        .page_size
        .iter()
        .map(|size| input::InputField {
            path: size.path.clone(),
            value: size.value(),
        })
        .collect()
}
