mod compatibility;
mod derived;
mod evaluation;
mod identity;
mod pagination;
mod pagination_none;
mod pagination_tokens;
mod paths;
mod row_policy;

pub(super) use evaluation::{StateArgs, operation};

use super::{CallableOperation, Violation, ViolationRule};

pub(super) struct OperationResult {
    pub callable: Option<CallableOperation>,
    pub effective: &'static str,
    pub violations: Vec<Violation>,
}
