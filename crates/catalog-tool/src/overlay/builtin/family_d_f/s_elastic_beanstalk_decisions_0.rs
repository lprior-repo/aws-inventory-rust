pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "AbortEnvironmentUpdate",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Cancels in-progress environment configuration update or application version deployment.",
    ),
    (
        "ApplyEnvironmentManagedAction",
        "Requires ActionId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Applies a scheduled managed action immediately.",
    ),
    (
        "AssociateEnvironmentOperationsRole",
        "Requires EnvironmentName, OperationsRole; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The operations role feature of Elastic Beanstalk is in beta release and is subject to change.",
    ),
    (
        "CheckDNSAvailability",
        "Requires CNAMEPrefix; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Checks if the specified CNAME is available.",
    ),
    (
        "ComposeEnvironments",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Create or update a group of environments that each run a separate component of a single application.",
    ),
    (
        "CreateApplication",
        "Requires ApplicationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an application that has one configuration template named default and no application versions.",
    ),
    (
        "CreateApplicationVersion",
        "Requires ApplicationName, VersionLabel; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an application version for the specified application.",
    ),
    (
        "CreateConfigurationTemplate",
        "Requires ApplicationName, TemplateName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates an Elastic Beanstalk configuration template, associated with a specific Elastic Beanstalk application.",
    ),
    (
        "CreateEnvironment",
        "Requires ApplicationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Launches an Elastic Beanstalk environment for the specified application using the specified configuration.",
    ),
    (
        "CreatePlatformVersion",
        "Requires PlatformName, PlatformVersion, PlatformDefinitionBundle; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Create a new version of your custom platform.",
    ),
    (
        "CreateStorageLocation",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Creates a bucket in Amazon S3 to store application versions, logs, and other files used by Elastic Beanstalk environments.",
    ),
    (
        "DeleteApplication",
        "Requires ApplicationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified application along with all associated versions and configurations.",
    ),
    (
        "DeleteApplicationVersion",
        "Requires ApplicationName, VersionLabel; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified version from the specified application.",
    ),
    (
        "DeleteConfigurationTemplate",
        "Requires ApplicationName, TemplateName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified configuration template.",
    ),
    (
        "DeleteEnvironmentConfiguration",
        "Requires ApplicationName, EnvironmentName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the draft configuration associated with the running environment.",
    ),
    (
        "DeletePlatformVersion",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes the specified version of a custom platform.",
    ),
    (
        "DescribeAccountAttributes",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns attributes related to Elastic Beanstalk that are associated with the calling Amazon Web Services account.",
    ),
    (
        "DescribeConfigurationOptions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes the configuration options that are used in a particular configuration template or environment, or that a specified solution stack defines.",
    ),
    (
        "DescribeConfigurationSettings",
        "Requires ApplicationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Returns a description of the settings for the specified configuration set, that is, either a configuration template or the configuration set associated with a running environment.",
    ),
    (
        "DescribeEnvironmentHealth",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns information about the overall health of the specified environment.",
    ),
    (
        "DescribeEnvironmentManagedActionHistory",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists an environment's completed and failed managed actions.",
    ),
    (
        "DescribeEnvironmentManagedActions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists an environment's upcoming and in-progress managed actions.",
    ),
    (
        "DescribeEnvironmentResources",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns Amazon Web Services resources for this environment.",
    ),
    (
        "DescribeEvents",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns list of event descriptions matching criteria up to the last 6 weeks.",
    ),
    (
        "DescribeInstancesHealth",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Retrieves detailed information about the health of instances in your Elastic Beanstalk environments.",
    ),
    (
        "DescribePlatformVersion",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Describes a platform version.",
    ),
    (
        "DisassociateEnvironmentOperationsRole",
        "Requires EnvironmentName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The operations role feature of Elastic Beanstalk is in beta release and is subject to change.",
    ),
    (
        "ListAvailableSolutionStacks",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns a list of the available solution stack names, with the public version first and then in reverse chronological order.",
    ),
    (
        "ListPlatformBranches",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists the platform branches available for your account in an Amazon Web Services Region.",
    ),
    (
        "ListPlatformVersions",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Lists the platform versions available for your account in an Amazon Web Services Region.",
    ),
    (
        "ListTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Return the tags applied to an Elastic Beanstalk resource.",
    ),
    (
        "RebuildEnvironment",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Deletes and recreates all of the Amazon Web Services resources (for example: the Auto Scaling group, load balancer, etc.) for a specified environment and forces a restart.",
    ),
    (
        "RequestEnvironmentInfo",
        "Requires InfoType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Initiates a request to compile the specified type of information of the deployed environment.",
    ),
    (
        "RestartAppServer",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Causes the environment to restart the application container server running on each Amazon EC2 instance.",
    ),
    (
        "RetrieveEnvironmentInfo",
        "Requires InfoType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the compiled information from a RequestEnvironmentInfo request.",
    ),
];
