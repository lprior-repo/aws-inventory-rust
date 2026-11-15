use super::{ContextSource, MemberPath, OperationName};

/// Only the modeled value types needed by reviewed row policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowValueType { String, Boolean, Structure }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowOperand {
    pub source: ContextSource,
    pub path: MemberPath,
    pub value_type: RowValueType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingSelected { Reject, AllowAbsent }

/// Deliberately bounded: no expressions, coercion, scripts or operation exceptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowPredicate {
    Absent(RowOperand),
    BooleanEquals { operand: RowOperand, value: bool },
    SelectedRequestEquals { selected: MemberPath, request: MemberPath, value_type: RowValueType, missing: MissingSelected },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChildEligibility {
    pub operation: OperationName,
    pub predicate: RowPredicate,
}

/// Predicates over the primary discovery selector, never unrelated extra emitters.
/// Operation and actual execution-branch policies are additive, not overrides.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowPolicy {
    /// False excludes the owned resource row AND its descendants.
    pub owned: Option<RowPredicate>,
    /// False rejects the entire atomic page; never converts it to an empty page.
    pub assertions: Vec<RowPredicate>,
    /// False excludes only child calls, preserving the producer inventory row.
    pub child: Option<ChildEligibility>,
}
