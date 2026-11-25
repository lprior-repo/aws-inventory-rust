use super::{ContextSource, MemberPath, OperationName};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowValueType {
    String,
    Boolean,
    Structure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowOperand {
    pub source: ContextSource,
    pub path: MemberPath,
    pub value_type: RowValueType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingSelected {
    Reject,
    AllowAbsent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowPredicate {
    Absent(RowOperand),
    BooleanEquals {
        operand: RowOperand,
        value: bool,
    },
    SelectedRequestEquals {
        selected: MemberPath,
        request: MemberPath,
        value_type: RowValueType,
        missing: MissingSelected,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildEligibility {
    pub operation: OperationName,
    pub predicate: RowPredicate,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowPolicy {
    pub owned: Option<RowPredicate>,
    pub assertions: Vec<RowPredicate>,
    pub child: Option<ChildEligibility>,
}
