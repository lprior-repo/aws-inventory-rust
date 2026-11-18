// Pinned operation purposes: .local/aws-models/models/redshift-data/service/2019-12-20/redshift-data-2019-12-20.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Runs one or more SQL statements, which can be data manipulation language (DML) or data definition language (DDL). Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secrets Manager which has username and pass
    ("BatchExecuteStatement", Decision::Denied(DenyReason::Paid)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels a running query. To be canceled, a query must be running. For more information about the Amazon Redshift Data API and CLI usage examples, see Using the Amazon Redshift Data API in the Amazon Redshift Management Guide .
    ("CancelStatement", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeStatement",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Id, SecretArn, DbUser, Database, ClusterIdentifier, Duration, Error, Status, CreatedAt, UpdatedAt, RedshiftPid, HasResultSet, QueryString, ResultRows, ResultSize, RedshiftQueryId, RedshiftQueryUuid, QueryParameters, SubStatements, WorkgroupName, ResultFormat, SessionId, ExecutionMode. Pinned purpose: Describes the details about a specific instance when a query was run by the Amazon Redshift Data API. The information includes when the query started, when it finished, the query status, the number of rows returned, and the SQL statement. For more in",
        ),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Describes the detailed information about a table from metadata in the cluster. The information includes its columns. A token is returned to page through the column list. Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secrets Manager which has username an
    ("DescribeTable", Decision::Denied(DenyReason::Content)),
    // Executes billable analysis/inference/database or customer data access rather than metadata inventory. Pinned purpose: Runs an SQL statement, which can be data manipulation language (DML) or data definition language (DDL). This statement must be a single SQL statement. Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secret
    ("ExecuteStatement", Decision::Denied(DenyReason::Paid)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Fetches the temporarily cached result of an SQL statement in JSON format. The ExecuteStatement or BatchExecuteStatement operation that ran the SQL statement must have specified ResultFormat as JSON , or let the format default to JSON. A token is returned to page through the statement results. For more information about the Amazon Redshift Data API and CLI usage examples, see Using the Amazon Redsh
    ("GetStatementResult", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Fetches the temporarily cached result of an SQL statement in CSV format. The ExecuteStatement or BatchExecuteStatement operation that ran the SQL statement must have specified ResultFormat as CSV . A token is returned to page through the statement results. For more information about the Amazon Redshift Data API and CLI usage examples, see Using the Amazon Redshift Data API in the Amazon Redshift M
    (
        "GetStatementResultV2",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: List the databases in a cluster. A token is returned to page through the database list. Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secrets Manager which has username and password . The specified secret contains credentials to connect to the database
    ("ListDatabases", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Lists the schemas in a database. A token is returned to page through the schema list. Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secrets Manager which has username and password . The specified secret contains credentials to connect to the database yo
    ("ListSchemas", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Lists the sessions that the caller created in the last 24 hours. By default, only sessions with a status of AVAILABLE or BUSY are returned. You can filter the results by session status, compute target (cluster or serverless workgroup), or database. To retrieve the metadata for a single session, provide the SessionId parameter. Use NextToken to page through the session list. Returns only the sessio
    ("ListSessions", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: List of SQL statements. By default, only finished statements are shown. A token is returned to page through the statement list. When you use identity-enhanced role sessions to list statements, you must provide either the cluster-identifier or workgroup-name parameter. This ensures that the IdC user can only access the Amazon Redshift IdC applications they are assigned. For more information, see Tr
    ("ListStatements", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: List the tables in a database. If neither SchemaPattern nor TablePattern are specified, then all tables in the database are returned. A token is returned to page through the table list. Depending on the authorization method, use one of the following combinations of request parameters: Secrets Manager - when connecting to a cluster, provide the secret-arn of a secret stored in Secrets Manager which
    ("ListTables", Decision::Denied(DenyReason::Content)),
];
