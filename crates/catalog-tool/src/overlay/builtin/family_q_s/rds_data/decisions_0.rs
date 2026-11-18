// Pinned operation purposes: .local/aws-models/models/rds-data/service/2018-08-01/rds-data-2018-08-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Runs a batch SQL statement over an array of data. You can run bulk update and insert operations for multiple records using a DML statement with different parameter sets. Bulk operations can provide a significant performance improvement over individual insert and update operations. If a call isn't part of a transaction because it doesn't include the
    ("BatchExecuteStatement", Decision::Denied(DenyReason::Paid)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts a SQL transaction. A transaction can run for a maximum of 24 hours. A transaction is terminated and rolled back automatically after 24 hours. A transaction times out if no calls use its transaction ID in three minutes. If a transaction times out before it's committed, it's rolled back automatically. For Aurora MySQL, DDL statements inside a transaction cause an implicit commit. We recommend
    ("BeginTransaction", Decision::Denied(DenyReason::Mutating)),
    (
        "CommitTransaction",
        Decision::Deferred(
            "Required selectors resourceArn, secretArn, transactionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members transactionStatus. Pinned purpose: Ends a SQL transaction started with the BeginTransaction operation and commits the changes.",
        ),
    ),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Runs one or more SQL statements. This operation is deprecated. Please use the BatchExecuteStatement or ExecuteStatement operation.
    ("ExecuteSql", Decision::Denied(DenyReason::Paid)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Runs a SQL statement against a database. If a call isn't part of a transaction because it doesn't include the transactionID parameter, changes that result from the call are committed automatically. If the binary response data from the database is more than 1 MB, the call is terminated.
    ("ExecuteStatement", Decision::Denied(DenyReason::Paid)),
    (
        "RollbackTransaction",
        Decision::Deferred(
            "Required selectors resourceArn, secretArn, transactionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members transactionStatus. Pinned purpose: Performs a rollback of a transaction. Rolling back a transaction cancels its changes.",
        ),
    ),
];
