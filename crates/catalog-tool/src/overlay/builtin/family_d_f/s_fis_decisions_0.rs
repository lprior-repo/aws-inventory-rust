pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateExperimentTemplate",
        "Requires clientToken, description, stopConditions, actions, roleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an experiment template.",
    ),
    (
        "CreateTargetAccountConfiguration",
        "Requires experimentTemplateId, accountId, roleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a target account configuration for the experiment template.",
    ),
    (
        "DeleteExperimentTemplate",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified experiment template.",
    ),
    (
        "DeleteTargetAccountConfiguration",
        "Requires experimentTemplateId, accountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified target account configuration of the experiment template.",
    ),
    (
        "GetAction",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified FIS action.",
    ),
    (
        "GetExperiment",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified experiment.",
    ),
    (
        "GetExperimentTargetAccountConfiguration",
        "Requires experimentId, accountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified target account configuration of the experiment.",
    ),
    (
        "GetExperimentTemplate",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified experiment template.",
    ),
    (
        "GetSafetyLever",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified safety lever.",
    ),
    (
        "GetTargetAccountConfiguration",
        "Requires experimentTemplateId, accountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified target account configuration of the experiment template.",
    ),
    (
        "GetTargetResourceType",
        "Requires resourceType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about the specified resource type.",
    ),
    (
        "ListActions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists the available FIS actions.",
    ),
    (
        "ListExperimentResolvedTargets",
        "Requires experimentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the resolved targets information of the specified experiment.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags for the specified resource.",
    ),
    (
        "ListTargetResourceTypes",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists the target resource types.",
    ),
    (
        "StartExperiment",
        "Requires clientToken, experimentTemplateId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts running an experiment from the specified experiment template.",
    ),
    (
        "StopExperiment",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the specified experiment.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Applies the specified tags to the specified resource.",
    ),
    (
        "UntagResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified tags from the specified resource.",
    ),
    (
        "UpdateExperimentTemplate",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified experiment template.",
    ),
    (
        "UpdateSafetyLeverState",
        "Requires id, state; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified safety lever state.",
    ),
    (
        "UpdateTargetAccountConfiguration",
        "Requires experimentTemplateId, accountId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the target account configuration for the specified experiment template.",
    ),
];
