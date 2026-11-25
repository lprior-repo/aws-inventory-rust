use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociatePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchDeleteDocument",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("BatchPutDocument", Decision::Denied(DenyReason::Mutating)),
    ("CancelSubscription", Decision::Denied(DenyReason::Mutating)),
    ("Chat", Decision::Denied(DenyReason::Mutating)),
    ("ChatSync", Decision::Denied(DenyReason::Mutating)),
    (
        "CheckDocumentAccess",
        Decision::Deferred(
            "Required selectors applicationId, indexId, userId, documentId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members userGroups, userAliases, hasAccess, documentAcl. Pinned purpose: Verifies if a user has access permissions for a specified document and returns the actual ACL attached to the document. Resolves user access on the document via user aliases and groups when verifying user access.",
        ),
    ),
    (
        "CreateAnonymousWebExperienceUrl",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateChatResponseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateDataAccessor", Decision::Denied(DenyReason::Mutating)),
    ("CreateDataSource", Decision::Denied(DenyReason::Mutating)),
    ("CreateIndex", Decision::Denied(DenyReason::Mutating)),
    ("CreatePlugin", Decision::Denied(DenyReason::Mutating)),
    ("CreateRetriever", Decision::Denied(DenyReason::Mutating)),
    ("CreateSubscription", Decision::Denied(DenyReason::Mutating)),
    ("CreateUser", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateWebExperience",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    ("DeleteAttachment", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteChatControlsConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteChatResponseConfiguration",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteConversation", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDataAccessor", Decision::Denied(DenyReason::Mutating)),
    ("DeleteDataSource", Decision::Denied(DenyReason::Mutating)),
    ("DeleteGroup", Decision::Denied(DenyReason::Mutating)),
    ("DeleteIndex", Decision::Denied(DenyReason::Mutating)),
    ("DeletePlugin", Decision::Denied(DenyReason::Mutating)),
    ("DeleteRetriever", Decision::Denied(DenyReason::Mutating)),
    ("DeleteUser", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteWebExperience",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociatePermission",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetApplication",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members displayName, applicationId, applicationArn, identityType, iamIdentityProviderArn, identityCenterApplicationArn, roleArn, status, description, encryptionConfiguration, createdAt, updatedAt, error, attachmentsConfiguration, qAppsConfiguration, personalizationConfiguration, autoSubscriptionConfiguration, clientIdsForOIDC, quickSightConfiguration. Pinned purpose: Gets information about an existing Amazon Q Business application.",
        ),
    ),
    (
        "GetChatControlsConfiguration",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members responseScope, orchestrationConfiguration, blockedPhrases, topicConfigurations, creatorModeConfiguration, nextToken, hallucinationReductionConfiguration. Pinned purpose: Gets information about chat controls configured for an existing Amazon Q Business application.",
        ),
    ),
    (
        "GetChatResponseConfiguration",
        Decision::Deferred(
            "Required selectors applicationId, chatResponseConfigurationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members chatResponseConfigurationId, chatResponseConfigurationArn, displayName, createdAt, inUseConfiguration, lastUpdateConfiguration. Pinned purpose: Retrieves detailed information about a specific chat response configuration from an Amazon Q Business application. This operation returns the complete configuration settings and metadata.",
        ),
    ),
    (
        "GetDataAccessor",
        Decision::Deferred(
            "Required selectors applicationId, dataAccessorId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members displayName, dataAccessorId, dataAccessorArn, applicationId, idcApplicationArn, principal, actionConfigurations, authenticationDetail, createdAt, updatedAt. Pinned purpose: Retrieves information about a specified data accessor. This operation returns details about the data accessor, including its display name, unique identifier, Amazon Resource Name (ARN), the associated Amazon Q Business application and IAM Identity Ce",
        ),
    ),
    (
        "GetDataSource",
        Decision::Deferred(
            "Required selectors applicationId, indexId, dataSourceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, indexId, dataSourceId, dataSourceArn, displayName, type, configuration, vpcConfiguration, createdAt, updatedAt, description, status, syncSchedule, roleArn, error, documentEnrichmentConfiguration, mediaExtractionConfiguration. Pinned purpose: Gets information about an existing Amazon Q Business data source connector.",
        ),
    ),
    ("GetDocumentContent", Decision::Denied(DenyReason::Content)),
    (
        "GetGroup",
        Decision::Deferred(
            "Required selectors applicationId, indexId, groupName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members status, statusHistory. Pinned purpose: Describes a group by group name.",
        ),
    ),
    (
        "GetIndex",
        Decision::Deferred(
            "Required selectors applicationId, indexId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, indexId, displayName, indexArn, status, type, description, createdAt, updatedAt, capacityConfiguration, documentAttributeConfigurations, error, indexStatistics. Pinned purpose: Gets information about an existing Amazon Q Business index.",
        ),
    ),
    ("GetMedia", Decision::Denied(DenyReason::Content)),
    (
        "GetPlugin",
        Decision::Deferred(
            "Required selectors applicationId, pluginId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, pluginId, displayName, type, serverUrl, authConfiguration, customPluginConfiguration, buildStatus, pluginArn, state, createdAt, updatedAt. Pinned purpose: Gets information about an existing Amazon Q Business plugin.",
        ),
    ),
    (
        "GetPolicy",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members policy. Pinned purpose: Retrieves the current permission policy for a Amazon Q Business application. The policy is returned as a JSON-formatted string and defines the IAM actions that are allowed or denied for the application's resources.",
        ),
    ),
    (
        "GetRetriever",
        Decision::Deferred(
            "Required selectors applicationId, retrieverId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, retrieverId, retrieverArn, type, status, displayName, configuration, roleArn, createdAt, updatedAt. Pinned purpose: Gets information about an existing retriever used by an Amazon Q Business application.",
        ),
    ),
    (
        "GetUser",
        Decision::Deferred(
            "Required selectors applicationId, userId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members userAliases. Pinned purpose: Describes the universally unique identifier (UUID) associated with a local user in a data source.",
        ),
    ),
    (
        "GetWebExperience",
        Decision::Deferred(
            "Required selectors applicationId, webExperienceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationId, webExperienceId, webExperienceArn, defaultEndpoint, status, createdAt, updatedAt, title, subtitle, welcomeMessage, samplePromptsControlMode, origins, roleArn, identityProviderConfiguration, authenticationConfiguration, error, browserExtensionConfiguration, customizationConfiguration. Pinned purpose: Gets information about an existing Amazon Q Business web experience.",
        ),
    ),
    (
        "ListAttachments",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members attachments, nextToken. Pinned purpose: Gets a list of attachments associated with an Amazon Q Business web experience or a list of attachements associated with a specific Amazon Q Business conversation.",
        ),
    ),
    (
        "ListChatResponseConfigurations",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members chatResponseConfigurations, nextToken. Pinned purpose: Retrieves a list of all chat response configurations available in a specified Amazon Q Business application. This operation returns summary information about each configuration to help administrators manage and select appropriate response settings.",
        ),
    ),
    (
        "ListConversations",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, conversations. Pinned purpose: Lists one or more Amazon Q Business conversations.",
        ),
    ),
    (
        "ListDataAccessors",
        Decision::Deferred(
            "Required selectors applicationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members dataAccessors, nextToken. Pinned purpose: Lists the data accessors for a Amazon Q Business application. This operation returns a paginated list of data accessor summaries, including the friendly name, unique identifier, ARN, associated IAM role, and creation/update timestamps for each data a",
        ),
    ),
    (
        "ListDataSourceSyncJobs",
        Decision::Deferred(
            "Required selectors dataSourceId, applicationId, indexId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members history, nextToken. Pinned purpose: Get information about an Amazon Q Business data source connector synchronization.",
        ),
    ),
    (
        "ListDocuments",
        Decision::Deferred(
            "Required selectors applicationId, indexId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members documentDetailList, nextToken. Pinned purpose: A list of documents attached to an index.",
        ),
    ),
    (
        "ListGroups",
        Decision::Deferred(
            "Required selectors applicationId, indexId, updatedEarlierThan are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, items. Pinned purpose: Provides a list of groups that are mapped to users.",
        ),
    ),
];
