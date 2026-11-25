use super::{TestResult, runtime};
use inventory_core::{
    catalog::runtime::{CallablePlan, Emission, PlanPurpose, RuntimeCatalog},
    inventory::{Kind, ResourceKey},
};
use inventory_engine::inventory::decode_scope_key;
use inventory_harness::inventory_seed::{SeedScenario, SeedStep};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub(super) struct FixtureScopes {
    selectors: Vec<String>,
    operations: BTreeMap<String, BTreeSet<String>>,
}

impl FixtureScopes {
    pub(super) fn from_scenarios(
        scenarios: &[SeedScenario],
        tagged: &[ResourceKey],
    ) -> TestResult<Self> {
        let bytes = std::fs::read(runtime::catalog_path())?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let selectors = scenarios
            .iter()
            .flat_map(SeedScenario::steps)
            .filter(|step| step.identity.is_some())
            .try_fold(BTreeSet::new(), |mut selectors, step| -> TestResult<_> {
                select_sources(catalog.plans(), step, tagged, &mut selectors)?;
                Ok(selectors)
            })?;
        assert!(
            !selectors.is_empty(),
            "fixtures selected no collector operations"
        );
        let selectors: Vec<_> = selectors.into_iter().collect();
        let catalog = catalog.select_services(&selectors)?;
        let mut operations = BTreeMap::<_, BTreeSet<_>>::new();
        catalog.plans().iter().for_each(|plan| {
            operations
                .entry(plan.service_name().to_owned())
                .or_default()
                .insert(plan.operation_name().to_owned());
        });
        Ok(Self {
            selectors,
            operations,
        })
    }

    pub(super) fn selectors(&self) -> &[String] {
        &self.selectors
    }

    pub(super) async fn assert_complete(
        &self,
        database: &Path,
        stage: &str,
        denied: Option<(&str, &str)>,
    ) -> TestResult {
        let options = SqliteConnectOptions::new()
            .filename(database)
            .read_only(true);
        let mut connection = SqliteConnection::connect_with(&options).await?;
        let rows: Vec<(Vec<u8>, String, Option<String>)> = sqlx::query_as(
            "SELECT scope_key,outcome,detail FROM scope_outcomes WHERE sweep_id=(\
             SELECT sweep_id FROM sweeps ORDER BY started_at DESC,sweep_id DESC LIMIT 1)",
        )
        .fetch_all(&mut connection)
        .await?;
        connection.close().await?;
        let mut missing = BTreeMap::<_, BTreeSet<_>>::new();
        for selector in &self.selectors {
            let (service, operation) =
                selector.split_once(':').ok_or("invalid fixture selector")?;
            missing.entry(service).or_default().insert(operation);
        }
        let mut incomplete = Vec::new();
        for (encoded, outcome, detail) in rows {
            let scope = decode_scope_key(&encoded)?;
            let service = scope.service().as_str();
            let operation = scope.operation().as_str();
            if !self
                .operations
                .get(service)
                .is_some_and(|ops| ops.contains(operation))
            {
                continue;
            }
            let expected = if denied == Some((service, operation)) {
                "inaccessible"
            } else {
                "complete"
            };
            if outcome != expected {
                incomplete.push(format!(
                    "{scope:?}: {outcome} instead of {expected}, {detail:?}"
                ));
            } else if let Some(operations) = missing.get_mut(service) {
                operations.remove(operation);
            }
        }
        missing.retain(|_, operations| !operations.is_empty());
        if !missing.is_empty() || !incomplete.is_empty() {
            return Err(format!(
                "collector {stage} did not finish fixture scopes: missing selected roots {missing:#?}; unexpected outcomes {incomplete:#?}"
            )
            .into());
        }
        Ok(())
    }
}

fn select_sources(
    plans: &[CallablePlan<'_>],
    step: &SeedStep,
    tagged: &[ResourceKey],
    selectors: &mut BTreeSet<String>,
) -> TestResult {
    let kind = Kind::parse(step.kind.as_str())?;
    let same_service = plans
        .iter()
        .filter(|plan| plan.service_name() == step.service);
    let producers: Vec<_> = same_service
        .clone()
        .filter(|plan| emits(plan, &kind))
        .collect();
    if producers.is_empty() {
        return Err(format!("no collector plan for {} {}", step.service, step.kind).into());
    }
    producers.iter().for_each(|plan| {
        selectors.insert(selector(plan));
    });
    if tagged.iter().any(|target| target.kind == kind) {
        let mut endpoints = same_service
            .filter(|plan| {
                let PlanPurpose::TagEndpoint(endpoint) = plan.purpose() else {
                    return false;
                };
                endpoint.parents.iter().any(|parent| {
                    producers
                        .iter()
                        .any(|source| source.operation_name() == parent.operation.as_ref())
                })
            })
            .peekable();
        let inline = producers.iter().any(|plan| {
            plan.resource_emissions()
                .iter()
                .any(|emission| emission.emits_kind(&kind) && emission.tags().is_some())
        });
        assert!(
            inline || endpoints.peek().is_some(),
            "no collector tag source for {kind:?}"
        );
        endpoints.for_each(|plan| {
            selectors.insert(selector(plan));
        });
    }
    Ok(())
}

fn emits(plan: &CallablePlan<'_>, kind: &Kind) -> bool {
    let emits_metadata = std::iter::once(None)
        .chain(plan.branches().iter().map(Some))
        .any(|branch| plan.effective_emission(branch) == Emission::EmitMetadata);
    emits_metadata
        && plan
            .resource_emissions()
            .iter()
            .any(|emission| emission.emits_kind(kind))
}

fn selector(plan: &CallablePlan<'_>) -> String {
    format!("{}:{}", plan.service_name(), plan.operation_name())
}
