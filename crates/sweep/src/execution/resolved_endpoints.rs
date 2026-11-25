use std::{collections::BTreeMap, sync::Arc};

use inventory_core::{
    catalog::runtime::{CallablePlan, ResolvedEndpoint, RuntimeCatalog},
    inventory::RegionId,
};
use tokio::sync::Mutex;

use crate::SweepError;

const MAX_ENDPOINTS: usize = 65_536;

#[derive(Default)]
pub(crate) struct ResolvedEndpoints(Mutex<Entries>);

#[derive(Default)]
struct Entries {
    by_plan: BTreeMap<usize, BTreeMap<RegionId, Arc<ResolvedEndpoint>>>,
    len: usize,
}

impl Entries {
    fn len(&self) -> usize {
        self.len
    }
}

impl ResolvedEndpoints {
    #[cfg(test)]
    pub(super) async fn keys(&self) -> Vec<(usize, RegionId)> {
        self.0
            .lock()
            .await
            .by_plan
            .iter()
            .flat_map(|(index, regions)| regions.keys().map(move |region| (*index, region.clone())))
            .collect()
    }

    pub(super) async fn resolve(
        &self,
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
        region: &RegionId,
    ) -> Result<Arc<ResolvedEndpoint>, SweepError> {
        let index = catalog
            .plan_index(plan.service_name(), plan.operation_name())
            .ok_or(SweepError::Operation(
                "endpoint plan is absent from run catalog",
            ))?;
        let mut entries = self.0.lock().await;
        if let Some(endpoint) = entries
            .by_plan
            .get(&index)
            .and_then(|regions| regions.get(region))
        {
            return Ok(Arc::clone(endpoint));
        }
        let endpoint = match plan.purpose() {
            inventory_core::catalog::runtime::PlanPurpose::AccountIdentity { .. }
            | inventory_core::catalog::runtime::PlanPurpose::RegionDiscovery { .. }
            | inventory_core::catalog::runtime::PlanPurpose::EndpointDiscovery => plan.endpoint(
                region,
                &inventory_core::inventory::Region::Regional(region.clone()),
            )?,
            _ => super::routing::resolve(plan, region)?,
        };
        let endpoint = Arc::new(endpoint);
        if entries.len() < MAX_ENDPOINTS {
            entries
                .by_plan
                .entry(index)
                .or_default()
                .insert(region.clone(), Arc::clone(&endpoint));
            entries.len = entries.len.saturating_add(1);
        }
        Ok(endpoint)
    }
}

#[cfg(test)]
mod tests;
