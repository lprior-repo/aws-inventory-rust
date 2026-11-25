use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "ListJobs",
        Decision::Deferred(
            "The modeled US endpoints return jobs from all US Regions; regional persistence would misattribute and duplicate them. A geography-aware identity/scope contract is required.",
        ),
    ),
    ("CancelCluster", Decision::Denied(DenyReason::Mutating)),
    ("CancelJob", Decision::Denied(DenyReason::Mutating)),
    ("CreateAddress", Decision::Denied(DenyReason::Mutating)),
    ("CreateCluster", Decision::Denied(DenyReason::Mutating)),
    ("CreateJob", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateLongTermPricing",
        Decision::Denied(DenyReason::Mutating),
    ),
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
    ("UpdateCluster", Decision::Denied(DenyReason::Mutating)),
    ("UpdateJob", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateJobShipmentState",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateLongTermPricing",
        Decision::Denied(DenyReason::Mutating),
    ),
];
