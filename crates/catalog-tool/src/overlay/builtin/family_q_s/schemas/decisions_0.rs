use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateDiscoverer", Decision::Denied(DenyReason::Mutating)),
    ("CreateRegistry", Decision::Denied(DenyReason::Mutating)),
    ("CreateSchema", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDiscoverer", Decision::Denied(DenyReason::Mutating)),
    ("DeleteRegistry", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteResourcePolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSchema", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteSchemaVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeCodeBinding",
        Decision::Deferred(
            "Required selectors Language, RegistryName, SchemaName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members CreationDate, LastModified, SchemaVersion, Status. Pinned purpose: Describe the code binding URI.",
        ),
    ),
    (
        "DescribeDiscoverer",
        Decision::Deferred(
            "Required selectors DiscovererId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Description, DiscovererArn, DiscovererId, SourceArn, State, CrossAccount, Tags. Pinned purpose: Describes the discoverer.",
        ),
    ),
    (
        "DescribeRegistry",
        Decision::Deferred(
            "Required selectors RegistryName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Description, RegistryArn, RegistryName, Tags. Pinned purpose: Describes the registry.",
        ),
    ),
    (
        "DescribeSchema",
        Decision::Deferred(
            "Required selectors RegistryName, SchemaName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Content, Description, LastModified, SchemaArn, SchemaName, SchemaVersion, Tags, Type, VersionCreatedDate. Pinned purpose: Retrieve the schema definition.",
        ),
    ),
    ("ExportSchema", Decision::Denied(DenyReason::Mutating)),
    (
        "GetCodeBindingSource",
        Decision::Deferred(
            "Required selectors Language, RegistryName, SchemaName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Body. Pinned purpose: Get the code binding source URI.",
        ),
    ),
    (
        "GetDiscoveredSchema",
        Decision::Deferred(
            "Required selectors Events, Type are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Content. Pinned purpose: Get the discovered schema that was generated based on sampled events.",
        ),
    ),
    (
        "GetResourcePolicy",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Policy, RevisionId. Pinned request/ownership contract: Retrieves the resource-based policy attached to a given registry.",
        ),
    ),
    (
        "ListSchemaVersions",
        Decision::Deferred(
            "Required selectors RegistryName, SchemaName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, SchemaVersions. Pinned purpose: Provides a list of the schema versions and related information.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Get tags for resource.",
        ),
    ),
    ("PutCodeBinding", Decision::Denied(DenyReason::Mutating)),
    ("PutResourcePolicy", Decision::Denied(DenyReason::Mutating)),
    (
        "SearchSchemas",
        Decision::Deferred(
            "Required selectors Keywords, RegistryName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members NextToken, Schemas. Pinned purpose: Search the schemas",
        ),
    ),
    ("StartDiscoverer", Decision::Denied(DenyReason::Mutating)),
    ("StopDiscoverer", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateDiscoverer", Decision::Denied(DenyReason::Mutating)),
    ("UpdateRegistry", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSchema", Decision::Denied(DenyReason::Mutating)),
];
