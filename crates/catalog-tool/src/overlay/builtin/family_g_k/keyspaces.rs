//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/keyspaces/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{Identity, IdentityField, MemberPath, NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListKeyspaces",
            policy::query(
                policy::enumerate(
                    "keyspaces",
                    policy::arn("resourceArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["resourceArn", "keyspaceName"],
            )?,
        ),
        (
            "ListTables",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "tables",
                        policy::arn("resourceArn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "keyspaceName",
                    "ListKeyspaces",
                    "keyspaceName",
                )?,
                &["resourceArn", "keyspaceName"],
            )?,
        ),
        (
            "GetTable",
            policy::query(
                policy::bind(
                    policy::bind(
                        policy::enumerate("$", policy::arn("resourceArn")?, None)?,
                        "keyspaceName",
                        "ListTables",
                        "keyspaceName",
                    )?,
                    "tableName",
                    "ListTables",
                    "tableName",
                )?,
                &["resourceArn"],
            )?,
        ),
        (
            "ListTypes",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "types",
                        Identity::CorrelatedKey(vec![
                            IdentityField::Request(MemberPath::new("keyspaceName")?),
                            IdentityField::Selected(MemberPath::selected("$")?),
                        ]),
                        Some(("nextToken", "nextToken")),
                    )?,
                    "keyspaceName",
                    "ListKeyspaces",
                    "keyspaceName",
                )?,
                &["$"],
            )?,
        ),
    ];
    policy::finish(
        "keyspaces",
        approved,
        DEFERRED,
        "Keyspace enumeration drives tables and keyspace-qualified scalar UDT names. Select permission on system metadata is not table-row collection. Type definition details and tags remain separately unapproved.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetKeyspace",
    "GetTableAutoScalingSettings",
    "GetType",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateKeyspace",
    "CreateTable",
    "CreateType",
    "DeleteKeyspace",
    "DeleteTable",
    "DeleteType",
    "RestoreTable",
    "TagResource",
    "UntagResource",
    "UpdateKeyspace",
    "UpdateTable",
];
