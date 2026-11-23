mod compatibility;
mod derived;
mod evaluation;
mod identity;
mod pagination;
mod pagination_none;
mod pagination_tokens;
mod paths;
mod row_policy;

pub(in crate::overlay) use evaluation::{StateArgs, operation};

use super::{CallableOperation, Violation, ViolationRule};

pub(in crate::overlay) struct OperationResult {
    pub callable: Option<CallableOperation>,
    pub effective: &'static str,
    pub violations: Vec<Violation>,
}
