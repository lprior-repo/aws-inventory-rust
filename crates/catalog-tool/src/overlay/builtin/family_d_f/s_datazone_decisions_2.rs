pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DisassociateEnvironmentRole",
        "Requires domainIdentifier, environmentIdentifier, environmentRoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates the environment role in Amazon DataZone.",
    ),
    (
        "DisassociateGovernedTerms",
        "Requires domainIdentifier, entityIdentifier, entityType, governedGlossaryTerms; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disassociates restricted terms from an asset.",
    ),
    (
        "GetAccountPool",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of the account pool.",
    ),
    (
        "GetAsset",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone asset.",
    ),
    (
        "GetAssetFilter",
        "Requires domainIdentifier, assetIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an asset filter.",
    ),
    (
        "GetAssetType",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone asset type.",
    ),
    (
        "GetConnection",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a connection.",
    ),
    (
        "GetDataExportConfiguration",
        "Requires domainIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets data export configuration details.",
    ),
    (
        "GetDataProduct",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the data product.",
    ),
    (
        "GetDataSource",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone data source.",
    ),
    (
        "GetDataSourceRun",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone data source run.",
    ),
    (
        "GetDesignation",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of a designation, including its Cedar permit policy.",
    ),
    (
        "GetDomain",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone domain.",
    ),
    (
        "GetDomainUnit",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of the specified domain unit.",
    ),
    (
        "GetEnvironment",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone environment.",
    ),
    (
        "GetEnvironmentAction",
        "Requires domainIdentifier, environmentIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the specified environment action.",
    ),
    (
        "GetEnvironmentBlueprint",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an Amazon DataZone blueprint.",
    ),
    (
        "GetEnvironmentBlueprintConfiguration",
        "Requires domainIdentifier, environmentBlueprintIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the blueprint configuration in Amazon DataZone.",
    ),
    (
        "GetEnvironmentCredentials",
        "Requires domainIdentifier, environmentIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the credentials of an environment in Amazon DataZone.",
    ),
    (
        "GetEnvironmentProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets an evinronment profile in Amazon DataZone.",
    ),
    (
        "GetFormType",
        "Requires domainIdentifier, formTypeIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a metadata form type in Amazon DataZone.",
    ),
    (
        "GetGlossary",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a business glossary in Amazon DataZone.",
    ),
    (
        "GetGlossaryTerm",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a business glossary term in Amazon DataZone.",
    ),
    (
        "GetGroupProfile",
        "Requires domainIdentifier, groupIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a group profile in Amazon DataZone.",
    ),
    (
        "GetIamPortalLoginUrl",
        "Requires domainIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the data portal URL for the specified Amazon DataZone domain.",
    ),
    (
        "GetJobRun",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The details of the job run.",
    ),
    (
        "GetLineageEvent",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the lineage event.",
    ),
    (
        "GetLineageNode",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the data lineage node.",
    ),
    (
        "GetListing",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a listing (a record of an asset at a given time).",
    ),
    (
        "GetMetadataGenerationRun",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a metadata generation run in Amazon DataZone.",
    ),
    (
        "GetNotebook",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of a notebook in Amazon SageMaker Unified Studio.",
    ),
    (
        "GetNotebookExport",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of a notebook export in Amazon SageMaker Unified Studio.",
    ),
    (
        "GetNotebookRun",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of a notebook run in Amazon SageMaker Unified Studio.",
    ),
    (
        "GetProject",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets a project in Amazon DataZone.",
    ),
    (
        "GetProjectProfile",
        "Requires domainIdentifier, identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The details of the project profile.",
    ),
];
