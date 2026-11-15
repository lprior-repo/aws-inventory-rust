// Pinned operation purposes: .local/aws-models/models/ssm-quicksetup/service/2018-05-10/ssm-quicksetup-2018-05-10.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Quick Setup configuration manager resource. This object is a collection of desired state configurations for multiple configuration definitions and summaries describing the deployments of those definitions.
    ("CreateConfigurationManager", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a configuration manager.
    ("DeleteConfigurationManager", Decision::Denied(DenyReason::Mutating)),
    ("GetConfiguration", Decision::Deferred("Required selectors ConfigurationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Id, ManagerArn, ConfigurationDefinitionId, Type, TypeVersion, Account, Region, CreatedAt, LastModifiedAt, StatusSummaries, Parameters. Pinned purpose: Returns details about the specified configuration.")),
    ("GetConfigurationManager", Decision::Deferred("Required selectors ManagerArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ManagerArn, Description, Name, CreatedAt, LastModifiedAt, StatusSummaries, ConfigurationDefinitions, Tags. Pinned purpose: Returns a configuration manager.")),
    ("GetServiceSettings", Decision::Deferred("The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members ServiceSettings. Pinned request/ownership contract: Returns settings configured for Quick Setup in the requesting Amazon Web Services account and Amazon Web Services Region.")),
    // ConfigurationDefinitionSummaries include FirstClassParameters, arbitrary deployment parameter values that can contain secrets. Fetching the full response is not justified by projecting ManagerArn.
    ("ListConfigurationManagers", Decision::Denied(DenyReason::Secret)),
    // Results include Account and Region for deployed target configurations plus FirstClassParameters. Per-item target scope and secret-safe parameter semantics are required; local-region projection is unsafe.
    ("ListConfigurations", Decision::Denied(DenyReason::Secret)),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Returns the available Quick Setup types.
    ("ListQuickSetupTypes", Decision::Denied(DenyReason::ReferenceData)),
    ("ListTagsForResource", Decision::Deferred("Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Returns tags assigned to the resource.")),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns key-value pairs of metadata to Amazon Web Services resources.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a Quick Setup configuration definition.
    ("UpdateConfigurationDefinition", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a Quick Setup configuration manager.
    ("UpdateConfigurationManager", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates settings configured for Quick Setup.
    ("UpdateServiceSettings", Decision::Denied(DenyReason::Mutating)),
];
