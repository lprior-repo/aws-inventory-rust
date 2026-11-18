// Explicit dispositions from .local/aws-models/models/elastic-beanstalk/service/2010-12-01/elastic-beanstalk-2010-12-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "SwapEnvironmentCNAMEs",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Swaps the CNAMEs of two environments.",
    ),
    (
        "TerminateEnvironment",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Terminates the specified environment.",
    ),
    (
        "UpdateApplication",
        "Requires ApplicationName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified application to have the specified properties.",
    ),
    (
        "UpdateApplicationResourceLifecycle",
        "Requires ApplicationName, ResourceLifecycleConfig; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies lifecycle settings for an application.",
    ),
    (
        "UpdateApplicationVersion",
        "Requires ApplicationName, VersionLabel; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified application version to have the specified properties.",
    ),
    (
        "UpdateConfigurationTemplate",
        "Requires ApplicationName, TemplateName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates the specified configuration template to have the specified properties or configuration option values.",
    ),
    (
        "UpdateEnvironment",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Updates the environment description, deploys a new application version, updates the configuration settings to an entirely new configuration template, or updates select configuration option values in t",
    ),
    (
        "UpdateTagsForResource",
        "Requires ResourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Update the list of tags applied to an Elastic Beanstalk resource.",
    ),
    (
        "ValidateConfigurationSettings",
        "Requires ApplicationName, OptionSettings; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Takes a set of configuration settings and either a configuration template or environment, and determines whether those values are valid.",
    ),
];
