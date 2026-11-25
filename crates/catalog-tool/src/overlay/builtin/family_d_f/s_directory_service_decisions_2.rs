pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "UpdateNumberOfDomainControllers",
        "Requires DirectoryId, DesiredNumber; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds or removes domain controllers to or from the directory.",
    ),
    (
        "UpdateRadius",
        "Requires DirectoryId, RadiusSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the Remote Authentication Dial In User Service (RADIUS) server information for an AD Connector or Microsoft AD directory.",
    ),
    (
        "UpdateSettings",
        "Requires DirectoryId, Settings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the configurable settings for the specified directory.",
    ),
    (
        "UpdateTrust",
        "Requires TrustId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the trust that has been set up between your Managed Microsoft AD directory and an self-managed Active Directory.",
    ),
    (
        "VerifyTrust",
        "Requires TrustId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Directory Service for Microsoft Active Directory allows you to configure and verify trust relationships.",
    ),
];
