use super::{CallablePlan, ExecutionBranch, PlanPurpose};
use crate::catalog::runtime::{BindingSource, decode::producers};

impl CallablePlan<'_> {
    pub fn has_exclusive_skip_bindings(
        &self,
        parent: &CallablePlan<'_>,
        branch: Option<&ExecutionBranch>,
    ) -> bool {
        if !matches!(self.purpose(), PlanPurpose::Enumeration) {
            return false;
        }
        let parent_key = (parent.service_name(), parent.operation_name());
        let bound =
            self.effective_bindings(branch)
                .try_fold(false, |bound, binding| {
                    if matches!(binding.source, BindingSource::ParentIdentity { .. }) {
                        return None;
                    }
                    let mut sources = producers(&binding.source);
                    match sources.next() {
                        None => Some(bound),
                        Some(key) => (key == parent_key && sources.all(|key| key == parent_key))
                            .then_some(true),
                    }
                });
        bound == Some(true)
    }
}
