// Explicit dispositions from .local/aws-models/models/datazone/service/2018-05-10/datazone-2018-05-10.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "UpdateEnvironmentAction",
        "Requires domainIdentifier, environmentIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an environment action.",
    ),
    (
        "UpdateEnvironmentBlueprint",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an environment blueprint in Amazon DataZone.",
    ),
    (
        "UpdateEnvironmentProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified environment profile in Amazon DataZone.",
    ),
    (
        "UpdateGlossary",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the business glossary in Amazon DataZone.",
    ),
    (
        "UpdateGlossaryTerm",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a business glossary term in Amazon DataZone.",
    ),
    (
        "UpdateGroupProfile",
        "Requires domainIdentifier, groupIdentifier, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified group profile in Amazon DataZone.",
    ),
    (
        "UpdateNotebook",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a notebook in Amazon SageMaker Unified Studio.",
    ),
    (
        "UpdateProject",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified project in Amazon DataZone.",
    ),
    (
        "UpdateProjectProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a project profile.",
    ),
    (
        "UpdateRootDomainUnitOwner",
        "Requires domainIdentifier, currentOwner, newOwner; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the owner of the root domain unit.",
    ),
    (
        "UpdateRule",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a rule.",
    ),
    (
        "UpdateSubscriptionGrantStatus",
        "Requires domainIdentifier, identifier, assetIdentifier, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the status of the specified subscription grant status in Amazon DataZone.",
    ),
    (
        "UpdateSubscriptionRequest",
        "Requires domainIdentifier, identifier, requestReason; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a specified subscription request in Amazon DataZone.",
    ),
    (
        "UpdateSubscriptionTarget",
        "Requires domainIdentifier, environmentIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified subscription target in Amazon DataZone.",
    ),
    (
        "UpdateUserProfile",
        "Requires domainIdentifier, userIdentifier, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified user profile in Amazon DataZone.",
    ),
];
