// Explicit dispositions from .local/aws-models/models/endusermessaging/service/2026-09-21/endusermessaging-2026-09-21.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateBrandProfile",
        "Requires brandProfileName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a brand profile.",
    ),
    (
        "CreateBrandProfileAttributes",
        "Requires brandProfileId, attributes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates up to 10 attributes for a brand profile in a single request.",
    ),
    (
        "CreateBrandProfileFromRegistration",
        "Requires registrationId, brandProfileName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a brand profile and populates its attributes from an existing registration.",
    ),
    (
        "CreateNotifyCodeConfiguration",
        "Requires notifyCodeConfigurationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a notify code configuration.",
    ),
    (
        "CreateRegistrationsFromBrandProfile",
        "Requires brandProfileId, registrationTypes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates one or more registrations in the DRAFT state and prefills their fields from the attributes of a brand profile.",
    ),
    (
        "DeleteBrandProfile",
        "Requires brandProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a brand profile.",
    ),
    (
        "DeleteBrandProfileAttribute",
        "Requires brandProfileId, attributeName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a brand profile attribute.",
    ),
    (
        "DeleteNotifyCodeConfiguration",
        "Requires notifyCodeConfigurationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a notify code configuration.",
    ),
    (
        "GetBrandProfile",
        "Requires brandProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the metadata for a brand profile, including its name, status, deletion protection setting, and timestamps.",
    ),
    (
        "GetBrandProfileAttribute",
        "Requires brandProfileId, attributeName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves a single brand profile attribute.",
    ),
    (
        "GetJob",
        "Requires jobId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the current state of an asynchronous job, including its status and any resources that it created or updated.",
    ),
    (
        "ListBrandProfileAttributes",
        "Requires brandProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves a paginated list of the attributes for a brand profile.",
    ),
    (
        "ListRegistrationsFromBrandProfile",
        "Requires brandProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves a paginated list of the registrations that were created from a brand profile through the synchronization operations.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the tags that are associated with a resource.",
    ),
    (
        "SendNotifyCodeVerification",
        "Requires channel, destinationIdentity, originationIdentity; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Generates a one-time passcode and delivers it to a recipient over the requested channel.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds or overwrites the tags on a resource.",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified tags from a resource.",
    ),
    (
        "UpdateBrandProfile",
        "Requires brandProfileId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the name or the deletion protection setting of a brand profile.",
    ),
    (
        "UpdateBrandProfileAttribute",
        "Requires brandProfileId, attributeName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the value, description, or category of an existing brand profile attribute.",
    ),
    (
        "UpdateBrandProfileFromRegistration",
        "Requires brandProfileId, registrationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Imports or refreshes the attributes of an existing brand profile from an existing registration.",
    ),
    (
        "UpdateNotifyCodeConfiguration",
        "Requires notifyCodeConfigurationId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the mutable fields of a notify code configuration.",
    ),
    (
        "UpdateRegistrationsFromBrandProfile",
        "Requires brandProfileId, registrationIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Repushes the attributes of a brand profile into existing DRAFT registrations.",
    ),
    (
        "ValidateNotifyCodeVerification",
        "Requires destinationIdentity, code; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Validates a one-time passcode that a recipient submitted.",
    ),
];
