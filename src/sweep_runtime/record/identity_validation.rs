use super::SweepError;
use inventory_core::{catalog::runtime::{ContextSource, IdentityTextOperand, IdentityTextPlan}, record::FieldId};

/// Pure recipe unit tests can deliberately omit models; that route is absent in release code.
pub(in crate::sweep_runtime) enum IdentityTextValidation<'p, 'catalog> {
    Modeled(&'p IdentityTextPlan<'catalog>),
    #[cfg(test)]
    RecipeOnly,
}

impl<'p, 'catalog> IdentityTextValidation<'p, 'catalog> {
    pub(super) fn selected_operand(&self, path: &[FieldId]) -> Result<IdentityTextOperandValidation<'_, 'catalog>, SweepError> {
        match self {
            Self::Modeled(plan) => Ok(IdentityTextOperandValidation::Modeled(plan.operand(ContextSource::SelectedItem, path)?)),
            #[cfg(test)]
            Self::RecipeOnly => Ok(IdentityTextOperandValidation::RecipeOnly),
        }
    }
}

/// One leaf's validator, resolved once even when that leaf is a string list.
pub(super) enum IdentityTextOperandValidation<'p, 'catalog> {
    Modeled(&'p IdentityTextOperand<'catalog>),
    #[cfg(test)]
    RecipeOnly,
}

impl IdentityTextOperandValidation<'_, '_> {
    pub(super) fn accepts(&self, text: &str) -> Result<(), SweepError> {
        match self {
            Self::Modeled(operand) => {
                if operand.accepts(text)? { Ok(()) }
                else { Err(SweepError::Operation("modeled identity text operand violates source constraints")) }
            },
            #[cfg(test)]
            Self::RecipeOnly => Ok(()),
        }
    }
}
