pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AcceptDataGrant",
        "Requires DataGrantArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation accepts a data grant.",
    ),
    (
        "CancelJob",
        "Requires JobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation cancels a job.",
    ),
    (
        "CreateDataGrant",
        "Requires Name, GrantDistributionScope, ReceiverPrincipal, SourceDataSetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation creates a data grant.",
    ),
    (
        "CreateDataSet",
        "Requires AssetType, Description, Name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation creates a data set.",
    ),
    (
        "CreateEventAction",
        "Requires Action, Event; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation creates an event action.",
    ),
    (
        "CreateJob",
        "Requires Details, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation creates a job.",
    ),
    (
        "CreateRevision",
        "Requires DataSetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation creates a revision for a data set.",
    ),
    (
        "DeleteAsset",
        "Requires AssetId, DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation deletes an asset.",
    ),
    (
        "DeleteDataGrant",
        "Requires DataGrantId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation deletes a data grant.",
    ),
    (
        "DeleteDataSet",
        "Requires DataSetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation deletes a data set.",
    ),
    (
        "DeleteEventAction",
        "Requires EventActionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation deletes the event action.",
    ),
    (
        "DeleteRevision",
        "Requires DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation deletes a revision.",
    ),
    (
        "GetAsset",
        "Requires AssetId, DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about an asset.",
    ),
    (
        "GetDataGrant",
        "Requires DataGrantId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about a data grant.",
    ),
    (
        "GetDataSet",
        "Requires DataSetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about a data set.",
    ),
    (
        "GetEventAction",
        "Requires EventActionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation retrieves information about an event action.",
    ),
    (
        "GetJob",
        "Requires JobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about a job.",
    ),
    (
        "GetReceivedDataGrant",
        "Requires DataGrantArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about a received data grant.",
    ),
    (
        "GetRevision",
        "Requires DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation returns information about a revision.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation lists the tags on the resource.",
    ),
    (
        "RevokeRevision",
        "Requires DataSetId, RevisionId, RevocationComment; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation revokes subscribers' access to a revision.",
    ),
    (
        "SendApiAsset",
        "Requires AssetId, DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation invokes an API Gateway API asset.",
    ),
    (
        "SendDataSetNotification",
        "Requires DataSetId, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The type of event associated with the data set.",
    ),
    (
        "StartJob",
        "Requires JobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation starts a job.",
    ),
    (
        "TagResource",
        "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation tags a resource.",
    ),
    (
        "UntagResource",
        "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation removes one or more tags from a resource.",
    ),
    (
        "UpdateAsset",
        "Requires AssetId, DataSetId, Name, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation updates an asset.",
    ),
    (
        "UpdateDataSet",
        "Requires DataSetId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation updates a data set.",
    ),
    (
        "UpdateEventAction",
        "Requires EventActionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation updates the event action.",
    ),
    (
        "UpdateRevision",
        "Requires DataSetId, RevisionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: This operation updates a revision.",
    ),
];
