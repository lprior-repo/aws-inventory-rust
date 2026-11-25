mod binding;
#[cfg(test)]
mod binding_tests;
mod context_budget;
mod contexts;
mod eligibility;
mod enumeration;
mod work;
pub(crate) use binding::consumes_current;
#[cfg(test)]
pub(crate) use binding::resolve_binding;
#[cfg(test)]
use binding::row_value;
pub(crate) use contexts::project_frame;
pub(super) use contexts::*;
#[cfg(test)]
use enumeration::child_for_row;
pub(crate) use enumeration::{FieldResolution, child_region, eligible, empty_record, fields};
pub(super) use work::{WorkBatch, work, work_with_row_children};
#[cfg(test)]
pub(crate) mod policy_dfgk_tests;
#[cfg(test)]
mod scalar_tests;
pub(super) use context_budget::PageContextBudget;
#[cfg(test)]
pub(super) use context_budget::Rows;

use super::consumer_index::ConsumerIndex;
use crate::{SweepError, input, record};
use inventory_core::{
    catalog::runtime::{BindingSource, CallablePlan, ExecutionBranch, PlanPurpose, PlanScope},
    inventory::{
        AccountId, DiscoveryContext, DiscoveryFrame, OperationId, Region, RegionId, ResourceKey,
        ScopeKey, ServiceId,
    },
    record::{FieldId, Record, Value, ValueRef},
};
use inventory_engine::inventory::{SweepId, WorkSpec};

#[cfg(test)]
use inventory_core::inventory::Observation;

const MAX_PAGE_CHILDREN: usize = 10_000;

pub(crate) struct Context<'a, 'catalog> {
    consumers: &'a ConsumerIndex<'a, 'catalog>,
    parent_plan: &'a CallablePlan<'catalog>,
    sweep: &'a SweepId,
    account: &'a AccountId,
    home: &'a RegionId,
}

impl<'a, 'catalog> Context<'a, 'catalog> {
    pub(crate) fn new(
        parent_plan: &'a CallablePlan<'catalog>,
        sweep: &'a SweepId,
        account: &'a AccountId,
        home: &'a RegionId,
        consumers: &'a ConsumerIndex<'a, 'catalog>,
    ) -> Self {
        Self {
            consumers,
            parent_plan,
            sweep,
            account,
            home,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BindingContext<'a, 'catalog> {
    pub parent_plan: &'a CallablePlan<'catalog>,
    pub account: Option<&'a AccountId>,
}

#[derive(Clone, Copy)]
pub(crate) struct BindingRow<'a> {
    pub row: &'a Record,
    pub region: &'a Region,
    pub branch: Option<&'a str>,
    pub ancestry: &'a DiscoveryContext,
}

pub(crate) struct RowContext<'a> {
    pub row: &'a Record,
    pub parent: Option<&'a ResourceKey>,
    pub scope: &'a ScopeKey,
    pub ancestry: &'a DiscoveryContext,
}

impl RowContext<'_> {
    fn bindings(&self) -> BindingRow<'_> {
        BindingRow {
            row: self.row,
            region: self.scope.region(),
            branch: self.scope.branch(),
            ancestry: self.ancestry,
        }
    }
}

impl<'catalog> Context<'_, 'catalog> {
    fn bindings(&self) -> BindingContext<'_, 'catalog> {
        BindingContext {
            parent_plan: self.parent_plan,
            account: Some(self.account),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn child_binding_reads_actual_parent_identity() -> Result<(), Box<dyn std::error::Error>> {
        let field = FieldId::new(7)?;
        let map = std::collections::BTreeMap::from([(
            inventory_core::record::Key::Field(field),
            Value::Text("vpc-actual".to_owned()),
        )]);
        let row = Record::encode(&Value::Map(map))?;
        assert_eq!(
            row_value(&row, &[field])?,
            (true, Some(Value::Text("vpc-actual".to_owned())))
        );
        Ok(())
    }

    #[test]
    fn child_scope_obeys_compiled_region_class() -> Result<(), Box<dyn std::error::Error>> {
        let home = RegionId::parse("us-east-1")?;
        assert_eq!(
            child_region(PlanScope::Account, &Region::Regional(home.clone()), &home),
            Region::Global
        );
        assert_eq!(
            child_region(PlanScope::HomeRegion, &Region::Global, &home),
            Region::Regional(home)
        );
        Ok(())
    }
}

#[cfg(test)]
mod policy_gk_children_tests;
#[cfg(test)]
mod policy_lp_completion_tests;
#[cfg(test)]
mod policy_qs_tests;
#[cfg(test)]
mod policy_tz_completion_tests;
