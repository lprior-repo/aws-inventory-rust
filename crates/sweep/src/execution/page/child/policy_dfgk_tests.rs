mod accelerators;
pub(super) mod complete_df;
mod ecr_images;
mod final_integrations;
mod iot_names;
mod scenario;

use super::super::{consumer_index::ConsumerIndex, cursor, rows as page_rows};
use super::*;
use crate::{execution::routing, record::policy_dfgk_tests::fixture, resource_projection};
use inventory_core::{
    catalog::runtime::{Emission, PlanScope, RuntimeCatalog},
    inventory::UnresolvedReference,
};
use inventory_engine::inventory::{BinaryStore, PageCommit, SchemaRegistry, WorkLease};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

// Shared by policy scenarios: real projected observations and actual child planning.
pub(crate) fn project_children(
    catalog: &RuntimeCatalog<'_>,
    context: &Context<'_, '_>,
    scope: &ScopeKey,
    request: &Record,
    response: &Record,
    ancestry: &DiscoveryContext,
) -> TestResult<(Vec<Observation>, Vec<WorkSpec>)> {
    let plan = context.parent_plan;
    let items = selected_items(plan, scope.branch(), request, response)?;
    let mut budget = PageContextBudget::new(ancestry, items.len())?;
    let contexts = contexts_from(
        plan,
        Rows {
            items: &items,
            offset: 0,
        },
        request,
        response,
        ancestry,
        scope,
        &mut budget,
    )?;
    let region = routing::request_region(plan, scope.region(), context.home, scope.branch())?;
    let branch = routing::execution_branch(plan, scope.branch())?;
    let observations = if plan.effective_emission(branch) == Emission::DiscoveryOnly {
        Vec::new()
    } else {
        items
            .iter()
            .zip(&contexts)
            .map(|(row, ancestry)| {
                page_rows::observation(
                    catalog,
                    record::IdentityContext {
                        account: context.account,
                        region: &region,
                    },
                    plan,
                    scope.region(),
                    row.clone(),
                    request,
                    ancestry,
                )
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let children = work(
        context,
        &items,
        &observations,
        request,
        &contexts,
        scope,
        &mut budget,
    )?;
    Ok((observations, children))
}

pub(crate) fn projected_references(
    catalog: &RuntimeCatalog<'_>,
    context: &Context<'_, '_>,
    scope: &ScopeKey,
    request: &Record,
    response: &Record,
    observations: &[Observation],
) -> TestResult<Vec<UnresolvedReference>> {
    let items = selected_items(context.parent_plan, scope.branch(), request, response)?;
    let region = routing::request_region(
        context.parent_plan,
        scope.region(),
        context.home,
        scope.branch(),
    )?;
    Ok(page_rows::references(
        catalog,
        context.parent_plan,
        &region,
        &items,
        observations,
        request,
        scope.parent(),
    )?)
}

fn selected_items(
    plan: &CallablePlan<'_>,
    scope_branch: Option<&str>,
    request: &Record,
    response: &Record,
) -> TestResult<Vec<Record>> {
    let branch = routing::execution_branch(plan, scope_branch)?;
    let mut items = Vec::new();
    for row in record::item_records(response, plan.items())? {
        if crate::row_policy::selected(plan, branch, &row, request, response)? {
            items.push(resource_projection::project(plan, &row)?);
        }
    }
    Ok(items)
}

fn projected_items(plan: &CallablePlan<'_>, response: &Record) -> TestResult<Vec<Record>> {
    record::item_records(response, plan.items())?
        .iter()
        .map(|row| resource_projection::project(plan, row).map_err(Into::into))
        .collect()
}

struct Scenario<'a, 'catalog> {
    directory: tempfile::TempDir,
    store: BinaryStore,
    sweep: SweepId,
    account: AccountId,
    home: RegionId,
    consumers: ConsumerIndex<'a, 'catalog>,
}

fn text<'a>(value: &'a Json, name: &str) -> TestResult<&'a str> {
    value
        .get(name)
        .and_then(Json::as_str)
        .ok_or_else(|| format!("encoded request omitted {name}").into())
}
