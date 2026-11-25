use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AcceptResourceGroupingRecommendations",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "AddDraftAppVersionResourceMappings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "BatchUpdateRecommendationStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateApp", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateRecommendationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateResiliencyPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAppAssessment",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAppInputSource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteRecommendationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteResiliencyPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeApp",
        Decision::Deferred(
            "Required selectors appArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members app. Pinned purpose: Describes an Resilience Hub application.",
        ),
    ),
    (
        "DescribeAppAssessment",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members assessment. Pinned purpose: Describes an assessment for an Resilience Hub application.",
        ),
    ),
    (
        "DescribeAppVersion",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, additionalInfo. Pinned purpose: Describes the Resilience Hub application version.",
        ),
    ),
    (
        "DescribeAppVersionAppComponent",
        Decision::Deferred(
            "Required selectors appArn, appVersion, id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, appComponent. Pinned purpose: Describes an Application Component in the Resilience Hub application.",
        ),
    ),
    (
        "DescribeAppVersionResource",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, physicalResource. Pinned purpose: Describes a resource of the Resilience Hub application. This API accepts only one of the following parameters to describe the resource: resourceName logicalResourceId physicalResourceId (Along with physicalResourceId , you can also provide awsAccount",
        ),
    ),
    (
        "DescribeAppVersionResourcesResolutionStatus",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, resolutionId, status, errorMessage. Pinned purpose: Returns the resolution status for the specified resolution identifier for an application version. If resolutionId is not specified, the current resolution status is returned.",
        ),
    ),
    (
        "DescribeAppVersionTemplate",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, appTemplateBody. Pinned purpose: Describes details about an Resilience Hub application.",
        ),
    ),
    (
        "DescribeDraftAppVersionResourcesImportStatus",
        Decision::Deferred(
            "Required selectors appArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, status, statusChangeTime, errorMessage, errorDetails. Pinned purpose: Describes the status of importing resources to an application version. If you get a 404 error with ResourceImportStatusNotFoundAppMetadataException , you must call importResourcesToDraftAppVersion after creating the application and before calling des",
        ),
    ),
    (
        "DescribeMetricsExport",
        Decision::Deferred(
            "Required selectors metricsExportId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members metricsExportId, status, exportLocation, errorMessage. Pinned purpose: Describes the metrics of the application configuration being exported.",
        ),
    ),
    (
        "DescribeResiliencyPolicy",
        Decision::Deferred(
            "Required selectors policyArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members policy. Pinned purpose: Describes a specified resiliency policy for an Resilience Hub application. The returned policy object includes creation time, data location constraints, the Amazon Resource Name (ARN) for the policy, tags, tier, and more.",
        ),
    ),
    (
        "DescribeResourceGroupingRecommendationTask",
        Decision::Deferred(
            "Required selectors appArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members groupingId, status, errorMessage. Pinned purpose: Describes the resource grouping recommendation tasks run by Resilience Hub for your application.",
        ),
    ),
    (
        "ImportResourcesToDraftAppVersion",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListAlarmRecommendations",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members alarmRecommendations, nextToken. Pinned purpose: Lists the alarm recommendations for an Resilience Hub application.",
        ),
    ),
    (
        "ListAppAssessmentComplianceDrifts",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members complianceDrifts, nextToken. Pinned purpose: List of compliance drifts that were detected while running an assessment.",
        ),
    ),
    (
        "ListAppAssessmentResourceDrifts",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourceDrifts, nextToken. Pinned purpose: List of resource drifts that were detected while running an assessment.",
        ),
    ),
    (
        "ListAppComponentCompliances",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members componentCompliances, nextToken. Pinned purpose: Lists the compliances for an Resilience Hub Application Component.",
        ),
    ),
    (
        "ListAppComponentRecommendations",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members componentRecommendations, nextToken. Pinned purpose: Lists the recommendations for an Resilience Hub Application Component.",
        ),
    ),
    (
        "ListAppInputSources",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appInputSources, nextToken. Pinned purpose: Lists all the input sources of the Resilience Hub application. For more information about the input sources supported by Resilience Hub, see Discover the structure and describe your Resilience Hub application .",
        ),
    ),
    (
        "ListAppVersionAppComponents",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, appComponents, nextToken. Pinned purpose: Lists all the Application Components in the Resilience Hub application.",
        ),
    ),
    (
        "ListAppVersionResourceMappings",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resourceMappings, nextToken. Pinned purpose: Lists how the resources in an application version are mapped/sourced from. Mappings can be physical resource identifiers, CloudFormation stacks, resource-groups, or an application registry app.",
        ),
    ),
    (
        "ListAppVersionResources",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members physicalResources, resolutionId, nextToken. Pinned purpose: Lists all the resources in an Resilience Hub application.",
        ),
    ),
    (
        "ListAppVersions",
        Decision::Deferred(
            "Required selectors appArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appVersions, nextToken. Pinned purpose: Lists the different versions for the Resilience Hub applications.",
        ),
    ),
    (
        "ListMetrics",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members rows, nextToken. Pinned request/ownership contract: Lists the metrics that can be exported.",
        ),
    ),
    (
        "ListResourceGroupingRecommendations",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members groupingRecommendations, nextToken. Pinned request/ownership contract: Lists the resource grouping recommendations suggested by Resilience Hub for your application.",
        ),
    ),
    (
        "ListSopRecommendations",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, sopRecommendations. Pinned purpose: Lists the standard operating procedure (SOP) recommendations for the Resilience Hub applications.",
        ),
    ),
    (
        "ListSuggestedResiliencyPolicies",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags for your resources in your Resilience Hub applications.",
        ),
    ),
    (
        "ListTestRecommendations",
        Decision::Deferred(
            "Required selectors assessmentArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members nextToken, testRecommendations. Pinned purpose: Lists the test recommendations for the Resilience Hub application.",
        ),
    ),
    (
        "ListUnsupportedAppVersionResources",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members unsupportedResources, resolutionId, nextToken. Pinned purpose: Lists the resources that are not currently supported in Resilience Hub. An unsupported resource is a resource that exists in the object that was used to create an app, but is not supported by Resilience Hub.",
        ),
    ),
    (
        "PublishAppVersion",
        Decision::Deferred(
            "Required selectors appArn are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, identifier, versionName. Pinned purpose: Publishes a new version of a specific Resilience Hub application.",
        ),
    ),
    (
        "PutDraftAppVersionTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RejectResourceGroupingRecommendations",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RemoveDraftAppVersionResourceMappings",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ResolveAppVersionResources",
        Decision::Deferred(
            "Required selectors appArn, appVersion are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members appArn, appVersion, resolutionId, status. Pinned purpose: Resolves the resources for an application version.",
        ),
    ),
    ("StartAppAssessment", Decision::Denied(DenyReason::Mutating)),
    ("StartMetricsExport", Decision::Denied(DenyReason::Mutating)),
    (
        "StartResourceGroupingRecommendationTask",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateApp", Decision::Denied(DenyReason::Mutating)),
];
