// Explicit dispositions from .local/aws-models/models/directory-service/service/2015-04-16/directory-service-2015-04-16.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AcceptSharedDirectory",
        "Requires SharedDirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Accepts a directory sharing request that was sent from the directory owner account.",
    ),
    (
        "AddIpRoutes",
        "Requires DirectoryId, IpRoutes; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: If the DNS server for your self-managed domain uses a publicly addressable IP address, you must add a CIDR address block to correctly route traffic to and from your Microsoft AD on Amazon Web Services",
    ),
    (
        "AddRegion",
        "Requires DirectoryId, RegionName, VPCSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds two domain controllers in the specified Region for the specified directory.",
    ),
    (
        "AddTagsToResource",
        "Requires ResourceId, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds or overwrites one or more tags for the specified directory.",
    ),
    (
        "CancelSchemaExtension",
        "Requires DirectoryId, SchemaExtensionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Cancels an in-progress schema extension to a Microsoft AD directory.",
    ),
    (
        "ConnectDirectory",
        "Requires Name, Password, Size, ConnectSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an AD Connector to connect to a self-managed directory.",
    ),
    (
        "CreateAlias",
        "Requires DirectoryId, Alias; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an alias for a directory and assigns the alias to the directory.",
    ),
    (
        "CreateComputer",
        "Requires DirectoryId, ComputerName, Password; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Active Directory computer object in the specified directory.",
    ),
    (
        "CreateConditionalForwarder",
        "Requires DirectoryId, RemoteDomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a conditional forwarder associated with your Amazon Web Services directory.",
    ),
    (
        "CreateDirectory",
        "Requires Name, Password, Size; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Simple AD directory.",
    ),
    (
        "CreateHybridAD",
        "Requires SecretArn, AssessmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a hybrid directory that connects your self-managed Active Directory (AD) infrastructure and Amazon Web Services.",
    ),
    (
        "CreateLogSubscription",
        "Requires DirectoryId, LogGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a subscription to forward real-time Directory Service domain controller security logs to the specified Amazon CloudWatch log group in your Amazon Web Services account.",
    ),
    (
        "CreateMicrosoftAD",
        "Requires Name, Password, VpcSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Microsoft AD directory in the Amazon Web Services Cloud.",
    ),
    (
        "CreateSnapshot",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a snapshot of a Simple AD or Microsoft AD directory in the Amazon Web Services cloud.",
    ),
    (
        "CreateTrust",
        "Requires DirectoryId, RemoteDomainName, TrustPassword, TrustDirection; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Directory Service for Microsoft Active Directory allows you to configure trust relationships.",
    ),
    (
        "DeleteADAssessment",
        "Requires AssessmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a directory assessment and all associated data.",
    ),
    (
        "DeleteConditionalForwarder",
        "Requires DirectoryId, RemoteDomainName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a conditional forwarder that has been set up for your Amazon Web Services directory.",
    ),
    (
        "DeleteDirectory",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an Directory Service directory.",
    ),
    (
        "DeleteLogSubscription",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified log subscription.",
    ),
    (
        "DeleteSnapshot",
        "Requires SnapshotId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a directory snapshot.",
    ),
    (
        "DeleteTrust",
        "Requires TrustId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an existing trust relationship between your Managed Microsoft AD directory and an external domain.",
    ),
    (
        "DeregisterCertificate",
        "Requires DirectoryId, CertificateId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes from the system the certificate that was registered for secure LDAP or client certificate authentication.",
    ),
    (
        "DeregisterEventTopic",
        "Requires DirectoryId, TopicName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the specified directory as a publisher to the specified Amazon SNS topic.",
    ),
    (
        "DescribeADAssessment",
        "Requires AssessmentId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves detailed information about a directory assessment, including its current status, validation results, and configuration details.",
    ),
    (
        "DescribeCAEnrollmentPolicy",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves detailed information about the certificate authority (CA) enrollment policy for the specified directory.",
    ),
    (
        "DescribeCertificate",
        "Requires DirectoryId, CertificateId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays information about the certificate registered for secure LDAP or client certificate authentication.",
    ),
    (
        "DescribeClientAuthenticationSettings",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about the type of client authentication for the specified directory, if the type is specified.",
    ),
    (
        "DescribeConditionalForwarders",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Obtains information about the conditional forwarders for this account.",
    ),
    (
        "DescribeDirectoryDataAccess",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Obtains status of directory data access enablement through the Directory Service Data API for the specified directory.",
    ),
    (
        "DescribeDomainControllers",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provides information about any domain controllers in your directory.",
    ),
    (
        "DescribeHybridADUpdate",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about update activities for a hybrid directory.",
    ),
    (
        "DescribeLDAPSSettings",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Describes the status of LDAP security for the specified directory.",
    ),
    (
        "DescribeRegions",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Provides information about the Regions that are configured for multi-Region replication.",
    ),
    (
        "DescribeSettings",
        "Requires DirectoryId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about the configurable settings for the specified directory.",
    ),
];
