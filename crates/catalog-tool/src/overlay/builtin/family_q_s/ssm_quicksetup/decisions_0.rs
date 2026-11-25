use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateConfigurationManager",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteConfigurationManager",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetConfiguration",
        Decision::Deferred(
            "Required selectors ConfigurationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Id, ManagerArn, ConfigurationDefinitionId, Type, TypeVersion, Account, Region, CreatedAt, LastModifiedAt, StatusSummaries, Parameters. Pinned purpose: Returns details about the specified configuration.",
        ),
    ),
    (
        "GetConfigurationManager",
        Decision::Deferred(
            "Required selectors ManagerArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ManagerArn, Description, Name, CreatedAt, LastModifiedAt, StatusSummaries, ConfigurationDefinitions, Tags. Pinned purpose: Returns a configuration manager.",
        ),
    ),
    (
        "GetServiceSettings",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ServiceSettings. Pinned request/ownership contract: Returns settings configured for Quick Setup in the requesting Amazon Web Services account and Amazon Web Services Region.",
        ),
    ),
    (
        "ListConfigurationManagers",
        Decision::Denied(DenyReason::Secret),
    ),
    ("ListConfigurations", Decision::Denied(DenyReason::Secret)),
    (
        "ListQuickSetupTypes",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Returns tags assigned to the resource.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateConfigurationDefinition",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateConfigurationManager",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateServiceSettings",
        Decision::Denied(DenyReason::Mutating),
    ),
];
