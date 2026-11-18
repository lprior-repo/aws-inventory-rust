// Pinned operation purposes: .local/aws-models/models/resiliencehub/service/2020-04-30/resiliencehub-2020-04-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Accepts the resource grouping recommendations suggested by Resilience Hub for your application.
    (
        "AcceptResourceGroupingRecommendations",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds the source of resource-maps to the draft version of an application. During assessment, Resilience Hub will use these resource-maps to resolve the latest physical ID for each resource in the application template. For more information about different types of resources supported by Resilience Hub and how to add them in your application, see Step 2: How is your application managed? in the Resili
    (
        "AddDraftAppVersionResourceMappings",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Enables you to include or exclude one or more operational recommendations.
    (
        "BatchUpdateRecommendationStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates an Resilience Hub application. An Resilience Hub application is a collection of Amazon Web Services resources structured to prevent and recover Amazon Web Services application disruptions. To describe a Resilience Hub application, you provide an application name, resources from one or more CloudFormation stacks, Resource Groups, Terraform state files, AppRegistry applications, and an appro
    ("CreateApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new Application Component in the Resilience Hub application. This API updates the Resilience Hub application draft version. To use this Application Component for running assessments, you must publish the Resilience Hub application using the PublishAppVersion API.
    (
        "CreateAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Adds a resource to the Resilience Hub application and assigns it to the specified Application Components. If you specify a new Application Component, Resilience Hub will automatically create the Application Component. This action has no effect outside Resilience Hub. This API updates the Resilience Hub application draft version. To use this resource for running resiliency assessments, you must pub
    (
        "CreateAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new recommendation template for the Resilience Hub application.
    (
        "CreateRecommendationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a resiliency policy for an application. Resilience Hub allows you to provide a value of zero for rtoInSecs and rpoInSecs of your resiliency policy. But, while assessing your application, the lowest possible assessment result is near zero. Hence, if you provide value zero for rtoInSecs and rpoInSecs , the estimated workload RTO and estimated workload RPO result will be near zero and the Com
    (
        "CreateResiliencyPolicy",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Resilience Hub application. This is a destructive action that can't be undone.
    ("DeleteApp", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Resilience Hub application assessment. This is a destructive action that can't be undone.
    (
        "DeleteAppAssessment",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the input source and all of its imported resources from the Resilience Hub application.
    (
        "DeleteAppInputSource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an Application Component from the Resilience Hub application. This API updates the Resilience Hub application draft version. To use this Application Component for running assessments, you must publish the Resilience Hub application using the PublishAppVersion API. You will not be able to delete an Application Component if it has resources associated with it.
    (
        "DeleteAppVersionAppComponent",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a resource from the Resilience Hub application. You can only delete a manually added resource. To exclude non-manually added resources, use the UpdateAppVersionResource API. This action has no effect outside Resilience Hub. This API updates the Resilience Hub application draft version. To use this resource for running resiliency assessments, you must publish the Resilience Hub application
    (
        "DeleteAppVersionResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a recommendation template. This is a destructive action that can't be undone.
    (
        "DeleteRecommendationTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a resiliency policy. This is a destructive action that can't be undone.
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
    // Changes service state rather than reading inventory. Pinned purpose: Imports resources to Resilience Hub application draft version from different input sources. For more information about the input sources supported by Resilience Hub, see Discover the structure and describe your Resilience Hub application .
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
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Lists the suggested resiliency policies for the Resilience Hub applications.
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds or updates the app template for an Resilience Hub application draft version.
    (
        "PutDraftAppVersionTemplate",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Rejects resource grouping recommendations.
    (
        "RejectResourceGroupingRecommendations",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Removes resource mappings from a draft application version.
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
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new application assessment for an application.
    ("StartAppAssessment", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Initiates the export task of metrics.
    ("StartMetricsExport", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Starts grouping recommendation task.
    (
        "StartResourceGroupingRecommendationTask",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Applies one or more tags to a resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from a resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an application.
    ("UpdateApp", Decision::Denied(DenyReason::Mutating)),
];
