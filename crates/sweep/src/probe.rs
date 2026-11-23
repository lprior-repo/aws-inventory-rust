use crate::{CredentialSource, SweepError, call, execution::routing, input, metrics, record};
use aws_wire::transport::Client;
use inventory_core::{
    catalog::runtime::{CallablePlan, PlanPurpose, PlanScope, ResolvedEndpoint, RuntimeCatalog},
    inventory::{Region, RegionId},
    record::Record,
};
use inventory_engine::limiter::ApiExecutor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProbeAnswer {
    pub items: u64,
    pub http_version: http::Version,
}

pub async fn call_once(
    catalog: &RuntimeCatalog<'_>,
    credentials: &impl CredentialSource,
    client: &Client,
    executor: &ApiExecutor,
    plan: &CallablePlan<'_>,
    region: &RegionId,
    endpoint_override: Option<&str>,
) -> Result<ProbeAnswer, SweepError> {
    Probe {
        catalog,
        credentials,
        client,
        executor,
        plan,
        region,
        endpoint_override,
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
    region: &'a RegionId,
    endpoint_override: Option<&'a str>,
}

impl<C: CredentialSource> Probe<'_, '_, C> {
    async fn run(&self) -> Result<ProbeAnswer, SweepError> {
        self.validate()?;
        let scope = self.scope();
        let branch = self.plan.branches().first();
        let region = routing::request_region(
            self.plan,
            &scope,
            self.region,
            branch.map(|branch| branch.discriminator.as_str()),
        )?;
        let endpoint = routing::resolve(self.plan, &region)?;
        let base = input::root_bindings_branch(self.plan, branch, None, Some(&scope))?;
        let response = self.request(&endpoint, &base).await?;
        let (count, _) = record::item_selection(&response.record, self.plan.items())?;
        Ok(ProbeAnswer {
            items: u64::try_from(count)
                .map_err(|_| SweepError::Operation("probe item count overflow"))?,
            http_version: response.http_version,
        })
    }

    fn validate(&self) -> Result<(), SweepError> {
        let found = self
            .catalog
            .lookup(self.plan.service_name(), self.plan.operation_name());
        if !found.is_some_and(|plan| std::ptr::eq(plan, self.plan))
            || !matches!(self.plan.purpose(), PlanPurpose::Enumeration)
            || input::requires_parent_branch(self.plan, self.plan.branches().first())
        {
            return Err(SweepError::InvalidConfig(
                "probe requires a root enumeration from this catalog",
            ));
        }
        Ok(())
    }

    fn scope(&self) -> Region {
        match self.plan.scope() {
            PlanScope::Account => Region::Global,
            PlanScope::Region | PlanScope::HomeRegion => Region::Regional(self.region.clone()),
        }
    }

    async fn request(
        &self,
        endpoint: &ResolvedEndpoint,
        base: &Record,
    ) -> Result<call::DecodedResponse, SweepError> {
        let fields = first_page_fields(self.plan);
        let metrics = metrics::Accumulators::default();
        call::Call {
            client: self.client,
            executor: self.executor,
            credentials: self.credentials,
            plan: self.plan,
            endpoint,
            url: self.endpoint_override.unwrap_or(&endpoint.url),
            base,
            fields: &fields,
            metrics: &metrics,
        }
        .run()
        .await
    }
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
