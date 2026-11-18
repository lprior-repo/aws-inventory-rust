// Explicit dispositions from .local/aws-models/models/frauddetector/service/2019-11-15/frauddetector-2019-11-15.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "GetKMSEncryptionKey",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Gets the encryption key if a KMS key has been specified to be used to encrypt content in Amazon Fraud Detector.",
    ),
    (
        "GetModelVersion",
        "Requires modelId, modelType, modelVersionNumber; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets the details of the specified model version.",
    ),
    (
        "GetVariables",
        "Variable.defaultValue may contain customer content; variable configuration is not an authorization to read stored default data. Pinned operation: Gets all of the variables or the specific variable.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceARN; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all tags associated with the resource.",
    ),
    (
        "PutDetector",
        "Requires detectorId, eventTypeName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates a detector.",
    ),
    (
        "PutEntityType",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an entity type.",
    ),
    (
        "PutEventType",
        "Requires name, eventVariables, entityTypes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an event type.",
    ),
    (
        "PutExternalModel",
        "Requires modelEndpoint, modelSource, invokeModelEndpointRoleArn, inputConfiguration, outputConfiguration, modelEndpointStatus; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an Amazon SageMaker model endpoint.",
    ),
    (
        "PutKMSEncryptionKey",
        "Requires kmsEncryptionKeyArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Specifies the KMS key to be used to encrypt content in Amazon Fraud Detector.",
    ),
    (
        "PutLabel",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates label.",
    ),
    (
        "PutOutcome",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates or updates an outcome.",
    ),
    (
        "SendEvent",
        "Requires eventId, eventTypeName, eventTimestamp, eventVariables, entities; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stores events in Amazon Fraud Detector without generating fraud predictions for those events.",
    ),
    (
        "TagResource",
        "Requires resourceARN, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Assigns tags to a resource.",
    ),
    (
        "UntagResource",
        "Requires resourceARN, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from a resource.",
    ),
    (
        "UpdateDetectorVersion",
        "Requires detectorId, detectorVersionId, externalModelEndpoints, rules; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a detector version.",
    ),
    (
        "UpdateDetectorVersionMetadata",
        "Requires detectorId, detectorVersionId, description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the detector version's description.",
    ),
    (
        "UpdateDetectorVersionStatus",
        "Requires detectorId, detectorVersionId, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the detector version’s status.",
    ),
    (
        "UpdateEventLabel",
        "Requires eventId, eventTypeName, assignedLabel, labelTimestamp; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified event with a new label.",
    ),
    (
        "UpdateList",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a list.",
    ),
    (
        "UpdateModel",
        "Requires modelId, modelType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates model description.",
    ),
    (
        "UpdateModelVersion",
        "Requires modelId, modelType, majorVersionNumber; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a model version.",
    ),
    (
        "UpdateModelVersionStatus",
        "Requires modelId, modelType, modelVersionNumber, status; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the status of a model version.",
    ),
    (
        "UpdateRuleMetadata",
        "Requires rule, description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a rule's metadata.",
    ),
    (
        "UpdateRuleVersion",
        "Requires rule, expression, language, outcomes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a rule version resulting in a new rule version.",
    ),
    (
        "UpdateVariable",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a variable.",
    ),
];
