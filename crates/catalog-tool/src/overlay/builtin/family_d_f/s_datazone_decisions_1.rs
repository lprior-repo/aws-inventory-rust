pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateRule",
        "Requires domainIdentifier, name, target, action, scope, detail; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a rule in Amazon DataZone.",
    ),
    (
        "CreateSubscriptionGrant",
        "Requires domainIdentifier, environmentIdentifier, grantedEntity; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a subsscription grant in Amazon DataZone.",
    ),
    (
        "CreateSubscriptionRequest",
        "Requires domainIdentifier, subscribedPrincipals, subscribedListings, requestReason; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a subscription request in Amazon DataZone.",
    ),
    (
        "CreateSubscriptionTarget",
        "Requires domainIdentifier, environmentIdentifier, name, type, subscriptionTargetConfig, authorizedPrincipals, manageAccessRole, applicableAssetTypes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a subscription target in Amazon DataZone.",
    ),
    (
        "CreateUserProfile",
        "Requires domainIdentifier, userIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a user profile in Amazon DataZone.",
    ),
    (
        "DeleteAccountPool",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an account pool.",
    ),
    (
        "DeleteAsset",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an asset in Amazon DataZone.",
    ),
    (
        "DeleteAssetFilter",
        "Requires domainIdentifier, assetIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an asset filter.",
    ),
    (
        "DeleteAssetType",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an asset type in Amazon DataZone.",
    ),
    (
        "DeleteConnection",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes and connection.",
    ),
    (
        "DeleteDataExportConfiguration",
        "Requires domainIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes data export configuration for a domain.",
    ),
    (
        "DeleteDataProduct",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a data product in Amazon DataZone.",
    ),
    (
        "DeleteDataSource",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a data source in Amazon DataZone.",
    ),
    (
        "DeleteDesignation",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a user-created designation.",
    ),
    (
        "DeleteDomain",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a Amazon DataZone domain.",
    ),
    (
        "DeleteDomainUnit",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a domain unit.",
    ),
    (
        "DeleteEnvironment",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an environment in Amazon DataZone.",
    ),
    (
        "DeleteEnvironmentAction",
        "Requires domainIdentifier, environmentIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an action for the environment, for example, deletes a console link for an analytics tool that is available in this environment.",
    ),
    (
        "DeleteEnvironmentBlueprint",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a blueprint in Amazon DataZone.",
    ),
    (
        "DeleteEnvironmentBlueprintConfiguration",
        "Requires domainIdentifier, environmentBlueprintIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the blueprint configuration in Amazon DataZone.",
    ),
    (
        "DeleteEnvironmentProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an environment profile in Amazon DataZone.",
    ),
    (
        "DeleteFormType",
        "Requires domainIdentifier, formTypeIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes and metadata form type in Amazon DataZone.",
    ),
    (
        "DeleteGlossary",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a business glossary in Amazon DataZone.",
    ),
    (
        "DeleteGlossaryTerm",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a business glossary term in Amazon DataZone.",
    ),
    (
        "DeleteLineageEvent",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified lineage event.",
    ),
    (
        "DeleteListing",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a listing (a record of an asset at a given time).",
    ),
    (
        "DeleteNotebook",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a notebook in Amazon SageMaker Unified Studio.",
    ),
    (
        "DeleteProject",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a project in Amazon DataZone.",
    ),
    (
        "DeleteProjectMembership",
        "Requires domainIdentifier, projectIdentifier, member; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes project membership in Amazon DataZone.",
    ),
    (
        "DeleteProjectProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a project profile.",
    ),
    (
        "DeleteRule",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a rule in Amazon DataZone.",
    ),
    (
        "DeleteSubscriptionGrant",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes and subscription grant in Amazon DataZone.",
    ),
    (
        "DeleteSubscriptionRequest",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a subscription request in Amazon DataZone.",
    ),
    (
        "DeleteSubscriptionTarget",
        "Requires domainIdentifier, environmentIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a subscription target in Amazon DataZone.",
    ),
    (
        "DeleteTimeSeriesDataPoints",
        "Requires domainIdentifier, entityIdentifier, entityType, formName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified time series form for the specified asset.",
    ),
];
