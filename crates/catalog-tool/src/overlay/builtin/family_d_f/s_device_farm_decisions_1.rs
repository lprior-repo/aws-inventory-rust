// Explicit dispositions from .local/aws-models/models/device-farm/service/2015-06-23/device-farm-2015-06-23.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "ListJobs",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about jobs for a given test run.",
    ),
    (
        "ListNetworkProfiles",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns the list of available network profiles.",
    ),
    (
        "ListOfferingPromotions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns a list of offering promotions.",
    ),
    (
        "ListOfferingTransactions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns a list of all historical purchases, renewals, and system renewal transactions for an AWS account.",
    ),
    (
        "ListOfferings",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns a list of products or offerings that the user can manage through the API.",
    ),
    (
        "ListRemoteAccessSessions",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a list of all currently running remote access sessions.",
    ),
    (
        "ListRuns",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about runs, given an AWS Device Farm project ARN.",
    ),
    (
        "ListSamples",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about samples, given an AWS Device Farm job ARN.",
    ),
    (
        "ListSuites",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about test suites for a given job.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceARN; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List the tags for an AWS Device Farm resource.",
    ),
    (
        "ListTestGridSessionActions",
        "Requires sessionArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a list of the actions taken in a TestGridSession .",
    ),
    (
        "ListTestGridSessionArtifacts",
        "Requires sessionArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves a list of artifacts created during the session.",
    ),
    (
        "ListTests",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about tests in a given test suite.",
    ),
    (
        "ListUniqueProblems",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about unique problems, such as exceptions or crashes.",
    ),
    (
        "PurchaseOffering",
        "Requires offeringId, quantity; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Immediately purchases offerings for an AWS account.",
    ),
    (
        "RenewOffering",
        "Requires offeringId, quantity; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Explicitly sets the quantity of devices to renew for an offering, starting from the effectiveDate of the next period.",
    ),
    (
        "ScheduleRun",
        "Requires projectArn, test; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Schedules a run.",
    ),
    (
        "StopJob",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Initiates a stop request for the current job.",
    ),
    (
        "StopRemoteAccessSession",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Ends a specified remote access session.",
    ),
    (
        "StopRun",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Initiates a stop request for the current test run.",
    ),
    (
        "TagResource",
        "Requires ResourceARN, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates the specified tags to a resource with the specified resourceArn .",
    ),
    (
        "UntagResource",
        "Requires ResourceARN, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified tags from a resource.",
    ),
    (
        "UpdateDeviceInstance",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates information about a private device instance.",
    ),
    (
        "UpdateDevicePool",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the name, description, and rules in a device pool given the attributes and the pool ARN.",
    ),
    (
        "UpdateInstanceProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates information about an existing private device instance profile.",
    ),
    (
        "UpdateNetworkProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the network profile.",
    ),
    (
        "UpdateProject",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the specified project name, given the project ARN and a new name.",
    ),
    (
        "UpdateTestGridProject",
        "Requires projectArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Change details of a project.",
    ),
    (
        "UpdateUpload",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates an uploaded test spec.",
    ),
    (
        "UpdateVPCEConfiguration",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates information about an Amazon Virtual Private Cloud (VPC) endpoint configuration.",
    ),
];
