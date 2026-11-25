use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("BatchExecuteStatement", Decision::Denied(DenyReason::Paid)),
    ("CancelStatement", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeStatement",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Id, SecretArn, DbUser, Database, ClusterIdentifier, Duration, Error, Status, CreatedAt, UpdatedAt, RedshiftPid, HasResultSet, QueryString, ResultRows, ResultSize, RedshiftQueryId, RedshiftQueryUuid, QueryParameters, SubStatements, WorkgroupName, ResultFormat, SessionId, ExecutionMode. Pinned purpose: Describes the details about a specific instance when a query was run by the Amazon Redshift Data API. The information includes when the query started, when it finished, the query status, the number of rows returned, and the SQL statement. For more in",
        ),
    ),
    ("DescribeTable", Decision::Denied(DenyReason::Content)),
    ("ExecuteStatement", Decision::Denied(DenyReason::Paid)),
    ("GetStatementResult", Decision::Denied(DenyReason::Content)),
    (
        "GetStatementResultV2",
        Decision::Denied(DenyReason::Content),
    ),
    ("ListDatabases", Decision::Denied(DenyReason::Content)),
    ("ListSchemas", Decision::Denied(DenyReason::Content)),
    ("ListSessions", Decision::Denied(DenyReason::Content)),
    ("ListStatements", Decision::Denied(DenyReason::Content)),
    ("ListTables", Decision::Denied(DenyReason::Content)),
];
