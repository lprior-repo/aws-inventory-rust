mod derived;
mod evaluation;
mod identity;
mod pagination;
mod paths;

pub(super) use evaluation::{StateArgs, operation};

use super::{CallableOperation, Violation, ViolationRule};

pub(super) struct OperationResult {
    pub callable: Option<CallableOperation>,
    pub effective: &'static str,
    pub violations: Vec<Violation>,
}
