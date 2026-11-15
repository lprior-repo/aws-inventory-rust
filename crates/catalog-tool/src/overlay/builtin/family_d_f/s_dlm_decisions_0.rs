// Explicit dispositions from .local/aws-models/models/dlm/service/2018-01-12/dlm-2018-01-12.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    ("CreateLifecyclePolicy", "Requires ExecutionRoleArn, Description, State; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Amazon Data Lifecycle Manager lifecycle policy."),
    ("DeleteLifecyclePolicy", "Requires PolicyId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified lifecycle policy and halts the automated operations that the policy specified."),
    ("GetLifecyclePolicy", "Policy singleton enrichment shares identity with summary enumeration; no merge/enrichment policy is registered. Summary resource inventory remains callable. Pinned operation: Gets detailed information about the specified lifecycle policy."),
    ("ListTagsForResource", "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags for the specified resource."),
    ("TagResource", "Requires ResourceArn, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds the specified tags to the specified resource."),
    ("UntagResource", "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified tags from the specified resource."),
    ("UpdateLifecyclePolicy", "Requires PolicyId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified lifecycle policy."),
];
