use super::ContextSource;
use crate::record::FieldId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowValueType {
    String,
    Boolean,
    Structure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowOperandPlan {
    pub source: ContextSource,
    pub path: Box<[FieldId]>,
    pub value_type: RowValueType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingSelected {
    Reject,
    AllowAbsent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowPredicatePlan {
    Absent(RowOperandPlan),
    BooleanEquals {
        operand: RowOperandPlan,
        value: bool,
    },
    SelectedRequestEquals {
        selected: Box<[FieldId]>,
        request: Box<[FieldId]>,
        value_type: RowValueType,
        missing: MissingSelected,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildEligibilityPlan {
    pub operation: Box<str>,
    pub predicate: RowPredicatePlan,
}

/// Primary-selector policy; operation and actual-branch rules both apply.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowPolicyPlan {
    pub owned: Option<RowPredicatePlan>,
    pub assertions: Box<[RowPredicatePlan]>,
    pub child: Option<ChildEligibilityPlan>,
}
