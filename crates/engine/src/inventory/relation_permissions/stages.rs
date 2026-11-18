use super::{Authority, StoreError, bindings, unreviewed};
use inventory_core::{
    catalog::runtime::{CallablePlan, Emission, ExecutionBranch, PlanPurpose, RuntimeCatalog},
    inventory::ScopeKey,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Output {
    Metadata,
    Tags,
    None,
}

#[derive(Clone, Debug)]
struct Stage {
    output: Output,
    bindings: Vec<bindings::FixedBinding>,
}

#[derive(Clone, Debug)]
enum Selection {
    Unbranched(Stage),
    Branched(BTreeMap<String, Stage>),
}

#[derive(Clone, Debug)]
pub(super) struct Stages {
    pub(super) authority: Authority,
    selection: Selection,
}

impl Stages {
    pub(super) fn compile(
        catalog: &RuntimeCatalog<'_>,
        plan: &CallablePlan<'_>,
    ) -> Result<Self, StoreError> {
        let selection = if plan.branches().is_empty() {
            Selection::Unbranched(stage(plan, None)?)
        } else {
            Selection::Branched(
                plan.branches()
                    .iter()
                    .map(|branch| Ok((branch.discriminator.clone(), stage(plan, Some(branch))?)))
                    .collect::<Result<_, StoreError>>()?,
            )
        };
        let metadata = match &selection {
            Selection::Unbranched(stage) => stage.output == Output::Metadata,
            Selection::Branched(stages) => stages
                .values()
                .any(|stage| stage.output == Output::Metadata),
        };
        let mut authority = Authority {
            relations: Vec::new(),
            trust: Vec::new(),
            emissions: plan.resource_emissions().iter().cloned().collect(),
        };
        if metadata {
            for relation in catalog.relationships() {
                if plan.supports_relationship(relation)? {
                    authority.relations.push(relation.clone());
                }
            }
            authority.trust = super::trust::rules(catalog, plan)?;
        }
        Ok(Self {
            authority,
            selection,
        })
    }

    pub(super) fn checked(
        &self,
        scope: &ScopeKey,
        request: &[u8],
    ) -> Result<(&Authority, Output), StoreError> {
        let stage = match (&self.selection, scope.branch()) {
            (Selection::Unbranched(stage), None) => stage,
            (Selection::Branched(stages), Some(branch)) => {
                stages.get(branch).ok_or_else(unreviewed)?
            }
            _ => return Err(unreviewed()),
        };
        bindings::validate(&stage.bindings, request)?;
        Ok((&self.authority, stage.output))
    }
}

fn stage(plan: &CallablePlan<'_>, branch: Option<&ExecutionBranch>) -> Result<Stage, StoreError> {
    let output = if plan.effective_emission(branch) == Emission::DiscoveryOnly {
        Output::None
    } else {
        match plan.purpose() {
            PlanPurpose::TagEndpoint(_) => Output::Tags,
            PlanPurpose::EndpointDiscovery => Output::None,
            _ => Output::Metadata,
        }
    };
    Ok(Stage {
        output,
        bindings: bindings::compile(plan, branch)?,
    })
}
