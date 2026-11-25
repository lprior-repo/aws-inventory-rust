pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateDevicePool",
        "Requires projectArn, name, rules; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a device pool.",
    ),
    (
        "CreateInstanceProfile",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a profile that can be applied to one or more private fleet device instances.",
    ),
    (
        "CreateNetworkProfile",
        "Requires projectArn, name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a network profile.",
    ),
    (
        "CreateProject",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a project.",
    ),
    (
        "CreateRemoteAccessSession",
        "Requires projectArn, deviceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Specifies and starts a remote access session.",
    ),
    (
        "CreateTestGridProject",
        "Requires name; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a Selenium testing project.",
    ),
    (
        "CreateUpload",
        "Requires projectArn, name, type; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Uploads an app or test scripts.",
    ),
    (
        "CreateVPCEConfiguration",
        "Requires vpceConfigurationName, vpceServiceName, serviceDnsName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a configuration record in Device Farm for your Amazon Virtual Private Cloud (VPC) endpoint.",
    ),
    (
        "DeleteDevicePool",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a device pool given the pool ARN.",
    ),
    (
        "DeleteInstanceProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a profile that can be applied to one or more private device instances.",
    ),
    (
        "DeleteNetworkProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a network profile.",
    ),
    (
        "DeleteProject",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an AWS Device Farm project, given the project ARN.",
    ),
    (
        "DeleteRemoteAccessSession",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a completed remote access session and its results.",
    ),
    (
        "DeleteRun",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the run, given the run ARN.",
    ),
    (
        "DeleteTestGridProject",
        "Requires projectArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a Selenium testing project and all content generated under it.",
    ),
    (
        "DeleteUpload",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes an upload given the upload ARN.",
    ),
    (
        "DeleteVPCEConfiguration",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a configuration for your Amazon Virtual Private Cloud (VPC) endpoint.",
    ),
    (
        "GetAccountSettings",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns the number of unmetered iOS or unmetered Android devices that have been purchased by the account.",
    ),
    (
        "GetDevice",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a unique device type.",
    ),
    (
        "GetDeviceInstance",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about a device instance that belongs to a private device fleet.",
    ),
    (
        "GetDevicePool",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a device pool.",
    ),
    (
        "GetDevicePoolCompatibility",
        "Requires devicePoolArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about compatibility with a device pool.",
    ),
    (
        "GetInstanceProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the specified instance profile.",
    ),
    (
        "GetJob",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a job.",
    ),
    (
        "GetNetworkProfile",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about a network profile.",
    ),
    (
        "GetOfferingStatus",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Gets the current status and future status of all offerings purchased by an AWS account.",
    ),
    (
        "GetRemoteAccessSession",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a link to a currently running remote access session.",
    ),
    (
        "GetRun",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a run.",
    ),
    (
        "GetSuite",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a suite.",
    ),
    (
        "GetTest",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about a test.",
    ),
    (
        "GetTestGridProject",
        "Requires projectArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about a Selenium testing project.",
    ),
    (
        "GetVPCEConfiguration",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns information about the configuration settings for your Amazon Virtual Private Cloud (VPC) endpoint.",
    ),
    (
        "InstallToRemoteAccessSession",
        "Requires remoteAccessSessionArn, appArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Installs an application to the device in a remote access session.",
    ),
    (
        "ListDevicePools",
        "Requires arn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Gets information about device pools.",
    ),
    (
        "ListDevices",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Gets information about unique device types.",
    ),
];
