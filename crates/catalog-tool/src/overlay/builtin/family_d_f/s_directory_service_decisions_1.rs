// Explicit dispositions from .local/aws-models/models/directory-service/service/2015-04-16/directory-service-2015-04-16.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "DescribeSharedDirectories",
        "Requires OwnerDirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the shared directories in your account.",
    ),
    (
        "DescribeUpdateDirectory",
        "Requires DirectoryId, UpdateType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the updates of a directory for a particular update type.",
    ),
    (
        "DisableCAEnrollmentPolicy",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables the certificate authority (CA) enrollment policy for the specified directory.",
    ),
    (
        "DisableClientAuthentication",
        "Requires DirectoryId, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables alternative client authentication methods for the specified directory.",
    ),
    (
        "DisableDirectoryDataAccess",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deactivates access to directory data via the Directory Service Data API for the specified directory.",
    ),
    (
        "DisableLDAPS",
        "Requires DirectoryId, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deactivates LDAP secure calls for the specified directory.",
    ),
    (
        "DisableRadius",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables multi-factor authentication (MFA) with the Remote Authentication Dial In User Service (RADIUS) server for an AD Connector or Microsoft AD directory.",
    ),
    (
        "DisableSso",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Disables single-sign on for a directory.",
    ),
    (
        "EnableCAEnrollmentPolicy",
        "Requires DirectoryId, PcaConnectorArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables certificate authority (CA) enrollment policy for the specified directory.",
    ),
    (
        "EnableClientAuthentication",
        "Requires DirectoryId, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables alternative client authentication methods for the specified directory.",
    ),
    (
        "EnableDirectoryDataAccess",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables access to directory data via the Directory Service Data API for the specified directory.",
    ),
    (
        "EnableLDAPS",
        "Requires DirectoryId, Type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Activates the switch for the specific directory to always use LDAP secure calls.",
    ),
    (
        "EnableRadius",
        "Requires DirectoryId, RadiusSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables multi-factor authentication (MFA) with the Remote Authentication Dial In User Service (RADIUS) server for an AD Connector or Microsoft AD directory.",
    ),
    (
        "EnableSso",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Enables single sign-on for a directory.",
    ),
    (
        "GetDirectoryLimits",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Obtains directory limit information for the current Region.",
    ),
    (
        "GetSnapshotLimits",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Obtains the manual snapshot limits for a directory.",
    ),
    (
        "ListCertificates",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: For the specified directory, lists all the certificates registered for a secure LDAP or client certificate authentication.",
    ),
    (
        "ListIpRoutes",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the address blocks that you have added to a directory.",
    ),
    (
        "ListSchemaExtensions",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all schema extensions applied to a Microsoft AD Directory.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all tags on a directory.",
    ),
    (
        "RegisterCertificate",
        "Requires DirectoryId, CertificateData; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Registers a certificate for a secure LDAP or client certificate authentication.",
    ),
    (
        "RegisterEventTopic",
        "Requires DirectoryId, TopicName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a directory with an Amazon SNS topic.",
    ),
    (
        "RejectSharedDirectory",
        "Requires SharedDirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Rejects a directory sharing request that was sent from the directory owner account.",
    ),
    (
        "RemoveIpRoutes",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes IP address blocks from a directory.",
    ),
    (
        "RemoveRegion",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops all replication and removes the domain controllers from the specified Region.",
    ),
    (
        "RemoveTagsFromResource",
        "Requires ResourceId, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from a directory.",
    ),
    (
        "ResetUserPassword",
        "Requires DirectoryId, UserName, NewPassword; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Resets the password for any user in your Managed Microsoft AD or Simple AD directory.",
    ),
    (
        "RestoreFromSnapshot",
        "Requires SnapshotId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Restores a directory using an existing directory snapshot.",
    ),
    (
        "ShareDirectory",
        "Requires DirectoryId, ShareTarget, ShareMethod; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Shares a specified directory ( DirectoryId ) in your Amazon Web Services account (directory owner) with another Amazon Web Services account (directory consumer).",
    ),
    (
        "StartADAssessment",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Initiates a directory assessment to validate your self-managed AD environment for hybrid domain join.",
    ),
    (
        "StartSchemaExtension",
        "Requires DirectoryId, CreateSnapshotBeforeSchemaExtension, LdifContent, Description; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Applies a schema extension to a Microsoft AD directory.",
    ),
    (
        "UnshareDirectory",
        "Requires DirectoryId, UnshareTarget; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Stops the directory sharing between the directory owner and consumer accounts.",
    ),
    (
        "UpdateConditionalForwarder",
        "Requires DirectoryId, RemoteDomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a conditional forwarder that has been set up for your Amazon Web Services directory.",
    ),
    (
        "UpdateDirectorySetup",
        "Requires DirectoryId, UpdateType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates directory configuration for the specified update type.",
    ),
    (
        "UpdateHybridAD",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the configuration of an existing hybrid directory.",
    ),
];
