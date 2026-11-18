// Pinned operation purposes: .local/aws-models/models/snowball/service/2016-06-30/snowball-2016-06-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Cancels a cluster job. You can only cancel a cluster job while it's in the AwaitingQuorum status. You'll have at least an hour after creating a cluster job to cancel it.
    ("CancelCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Cancels the specified job. You can only cancel a job before its JobState value changes to PreparingAppliance . Requesting the ListJobs or DescribeJob action returns a job's JobState as part of the response element data returned.
    ("CancelJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an address for a Snow device to be shipped to. In most regions, addresses are validated at the time of creation. The address you provide must be located within the serviceable area of your region. If the address is invalid or unsupported, then an exception is thrown. If providing an address as a JSON file through the cli-input-json option, include the full file path. For example, --cli-inp
    ("CreateAddress", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an empty cluster. Each cluster supports five nodes. You use the CreateJob action separately to create the jobs for each of these nodes. The cluster does not ship until these five node jobs have been created.
    ("CreateCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a job to import or export data between Amazon S3 and your on-premises data center. Your Amazon Web Services account must have the right trust policies and permissions in place to create a job for a Snow device. If you're creating a job for a node in a cluster, you only need to provide the clusterId value; the other job attributes are inherited from the cluster. Only the Snowball; Edge devi
    ("CreateJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a job with the long-term usage option for a device. The long-term usage is a 1-year or 3-year long-term pricing type for the device. You are billed upfront, and Amazon Web Services provides discounts for long-term pricing.
    (
        "CreateLongTermPricing",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a shipping label that will be used to return the Snow device to Amazon Web Services.
    (
        "CreateReturnShippingLabel",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeAddress",
        Decision::Deferred(
            "Required selectors AddressId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Address. Pinned purpose: Takes an AddressId and returns specific details about that address in the form of an Address object.",
        ),
    ),
    // Returns human postal contact/shipping addresses, not cloud resource inventory.
    ("DescribeAddresses", Decision::Denied(DenyReason::Content)),
    (
        "DescribeCluster",
        Decision::Deferred(
            "Required selectors ClusterId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ClusterMetadata. Pinned purpose: Returns information about a specific cluster including shipping information, cluster status, and other important metadata.",
        ),
    ),
    (
        "DescribeJob",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobMetadata, SubJobMetadata. Pinned purpose: Returns information about a specific job including shipping information, job status, and other important metadata.",
        ),
    ),
    (
        "DescribeReturnShippingLabel",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, ExpirationDate, ReturnShippingLabelURI. Pinned purpose: Information on the shipping label of a Snow device that is being returned to Amazon Web Services.",
        ),
    ),
    (
        "GetJobManifest",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ManifestURI. Pinned purpose: Returns a link to an Amazon S3 presigned URL for the manifest file associated with the specified JobId value. You can access the manifest file for up to 60 minutes after this request has been made. To access the manifest file after 60 minutes have pa",
        ),
    ),
    (
        "GetJobUnlockCode",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UnlockCode. Pinned purpose: Returns the UnlockCode code value for the specified job. A particular UnlockCode value can be accessed for up to 360 days after the associated job has been created. The UnlockCode value is a 29-character code with 25 alphanumeric characters and 4 hyp",
        ),
    ),
    (
        "GetSnowballUsage",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members SnowballLimit, SnowballsInUse. Pinned request/ownership contract: Returns information about the Snow Family service limit for your account, and also the number of Snow devices your account has in use. The default service limit for the number of Snow devices that you can have at one time is 1. If you want to increase your service limit, contact Amazon Web Services Support.",
        ),
    ),
    (
        "GetSoftwareUpdates",
        Decision::Deferred(
            "Required selectors JobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members UpdatesURI. Pinned purpose: Returns an Amazon S3 presigned URL for an update file associated with a specified JobId .",
        ),
    ),
    (
        "ListClusterJobs",
        Decision::Deferred(
            "Required selectors ClusterId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members JobListEntries, NextToken. Pinned purpose: Returns an array of JobListEntry objects of the specified length. Each JobListEntry object is for a job in the specified cluster and contains a job's state, a job's ID, and other information.",
        ),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: This action returns a list of the different Amazon EC2-compatible Amazon Machine Images (AMIs) that are owned by your Amazon Web Services accountthat would be supported for use on a Snow device. Currently, supported AMIs are based on the Amazon Linux-2, Ubuntu 20.04 LTS - Focal, or Ubuntu 22.04 LTS - Jammy images, available on the Amazon Web Servic
    (
        "ListCompatibleImages",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListPickupLocations",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members Addresses, NextToken. Pinned request/ownership contract: A list of locations from which the customer can choose to pickup a device.",
        ),
    ),
    (
        "ListServiceVersions",
        Decision::Deferred(
            "Required selectors ServiceName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceVersions, ServiceName, DependentServices, NextToken. Pinned purpose: Lists all supported versions for Snow on-device services. Returns an array of ServiceVersion object containing the supported versions for a particular service.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: While a cluster's ClusterState value is in the AwaitingQuorum state, you can update some of the information associated with a cluster. Once the cluster changes to a different job state, usually 60 minutes after the cluster being created, this action is no longer available.
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: While a job's JobState value is New , you can update some of the information associated with a job. Once the job changes to a different job state, usually within 60 minutes of the job being created, this action is no longer available.
    ("UpdateJob", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the state when a shipment state changes to a different state.
    (
        "UpdateJobShipmentState",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the long-term pricing type.
    (
        "UpdateLongTermPricing",
        Decision::Denied(DenyReason::Mutating),
    ),
];
