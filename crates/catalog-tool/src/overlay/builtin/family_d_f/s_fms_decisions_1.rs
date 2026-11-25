pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "PutPolicy",
        "Requires Policy; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Firewall Manager policy.",
    ),
    (
        "PutProtocolsList",
        "Requires ProtocolsList; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Firewall Manager protocols list.",
    ),
    (
        "PutResourceSet",
        "Requires ResourceSet; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates the resource set.",
    ),
    (
        "TagResource",
        "Requires ResourceArn, TagList; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds one or more tags to an Amazon Web Services resource.",
    ),
    (
        "UntagResource",
        "Requires ResourceArn, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes one or more tags from an Amazon Web Services resource.",
    ),
];
