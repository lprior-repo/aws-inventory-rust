use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("BatchExecuteStatement", Decision::Denied(DenyReason::Paid)),
    ("BeginTransaction", Decision::Denied(DenyReason::Mutating)),
    (
        "CommitTransaction",
        Decision::Deferred(
            "Required selectors resourceArn, secretArn, transactionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members transactionStatus. Pinned purpose: Ends a SQL transaction started with the BeginTransaction operation and commits the changes.",
        ),
    ),
    ("ExecuteSql", Decision::Denied(DenyReason::Paid)),
    ("ExecuteStatement", Decision::Denied(DenyReason::Paid)),
    (
        "RollbackTransaction",
        Decision::Deferred(
            "Required selectors resourceArn, secretArn, transactionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members transactionStatus. Pinned purpose: Performs a rollback of a transaction. Rolling back a transaction cancels its changes.",
        ),
    ),
];
