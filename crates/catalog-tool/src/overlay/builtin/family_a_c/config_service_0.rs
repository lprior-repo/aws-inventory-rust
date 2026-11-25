use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateResourceTypes",
            deferred(
                "Model requires ConfigurationRecorderArn, ResourceTypes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds all resource types specified in the ResourceTypes list to the RecordingGroup of specified configuration recorder and includes those resource types when recording. For this operation, the specified configuration recorder must…",
            ),
        ),
        (
            "BatchGetAggregateResourceConfig",
            deferred(
                "Model requires ConfigurationAggregatorName, ResourceIdentifiers. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the current configuration items for resources that are present in your Config aggregator. The operation also returns a list of resources that are not processed in the current request. If there are no unprocessed…",
            ),
        ),
        (
            "BatchGetResourceConfig",
            deferred(
                "Model requires resourceKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the BaseConfigurationItem for one or more requested resources. The operation also returns a list of resources that are not processed in the current request. If there are no unprocessed resources, the operation returns an…",
            ),
        ),
        (
            "DeleteAggregationAuthorization",
            deferred(
                "Model requires AuthorizedAccountId, AuthorizedAwsRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the authorization granted to the specified configuration aggregator account in a specified region.",
            ),
        ),
        (
            "DeleteConfigRule",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified Config rule and all of its evaluation results. Config sets the state of a rule to DELETING until the deletion is complete. You cannot update a rule while it is in this state. If you make a PutConfigRule or…",
            ),
        ),
        (
            "DeleteConfigurationAggregator",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified configuration aggregator and the aggregated data associated with the aggregator.",
            ),
        ),
        (
            "DeleteConfigurationRecorder",
            deferred(
                "Model requires ConfigurationRecorderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the customer managed configuration recorder. This operation does not delete the configuration information that was previously recorded. You will be able to access the previously recorded information by using the…",
            ),
        ),
        (
            "DeleteConformancePack",
            deferred(
                "Model requires ConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified conformance pack and all the Config rules, remediation actions, and all evaluation results within that conformance pack. Config sets the conformance pack to DELETE_IN_PROGRESS until the deletion is complete.…",
            ),
        ),
        (
            "DeleteConnector",
            deferred(
                "Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified connector.",
            ),
        ),
        (
            "DeleteDeliveryChannel",
            deferred(
                "Model requires DeliveryChannelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the delivery channel. Before you can delete the delivery channel, you must stop the customer managed configuration recorder. You can use the StopConfigurationRecorder operation to stop the customer managed configuration…",
            ),
        ),
        (
            "DeleteEvaluationResults",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the evaluation results for the specified Config rule. You can specify one Config rule per request. After you delete the evaluation results, you can call the StartConfigRulesEvaluation API to start evaluating your Amazon…",
            ),
        ),
        (
            "DeleteOrganizationConfigRule",
            deferred(
                "Model requires OrganizationConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified organization Config rule and all of its evaluation results from all member accounts in that organization. Only a management account and a delegated administrator account can delete an organization Config…",
            ),
        ),
        (
            "DeleteOrganizationConformancePack",
            deferred(
                "Model requires OrganizationConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified organization conformance pack and all of the Config rules and remediation actions from all member accounts in that organization. Only a management account or a delegated administrator account can delete an…",
            ),
        ),
        (
            "DeletePendingAggregationRequest",
            deferred(
                "Model requires RequesterAccountId, RequesterAwsRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes pending authorization requests for a specified aggregator account in a specified region.",
            ),
        ),
        (
            "DeleteRemediationConfiguration",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the remediation configuration.",
            ),
        ),
        (
            "DeleteRemediationExceptions",
            deferred(
                "Model requires ConfigRuleName, ResourceKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes one or more remediation exceptions mentioned in the resource keys. Config generates a remediation exception when a problem occurs executing a remediation action to a specific resource. Remediation exceptions blocks…",
            ),
        ),
        (
            "DeleteResourceConfig",
            deferred(
                "Model requires ResourceType, ResourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Records the configuration state for a custom resource that has been deleted. This API records a new ConfigurationItem with a ResourceDeleted status. You can retrieve the ConfigurationItems recorded for this resource in your…",
            ),
        ),
        (
            "DeleteRetentionConfiguration",
            deferred(
                "Model requires RetentionConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the retention configuration.",
            ),
        ),
        (
            "DeleteServiceLinkedConfigurationRecorder",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is Arn, Name; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deletes an existing service-linked configuration recorder. This operation does not delete the configuration information that was previously recorded. You will be able to access the previously recorded information by using the…",
            ),
        ),
        (
            "DeleteStoredQuery",
            deferred(
                "Model requires QueryName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the stored query for a single Amazon Web Services account and a single Amazon Web Services Region.",
            ),
        ),
        (
            "DeliverConfigSnapshot",
            deferred(
                "Model requires deliveryChannelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Schedules delivery of a configuration snapshot to the Amazon S3 bucket in the specified delivery channel. After the delivery has started, Config sends the following notifications using an Amazon SNS topic that you have specified.…",
            ),
        ),
        (
            "DescribeAggregateComplianceByConfigRules",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of compliant and noncompliant rules with the number of resources for compliant and noncompliant rules. Does not display rules that do not have compliance results. The results can return an empty result page, but if…",
            ),
        ),
        (
            "DescribeAggregateComplianceByConformancePacks",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the existing and deleted conformance packs and their associated compliance status with the count of compliant and noncompliant Config rules within each conformance pack. Also returns the total rule count which…",
            ),
        ),
        (
            "DescribeAggregationAuthorizations",
            OperationOverlay {
                queryable: vec![path("AggregationAuthorizationArn")?],
                ..read(
                    "AggregationAuthorizations",
                    Identity::Arn(path("AggregationAuthorizationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeComplianceByConfigRule",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ComplianceByConfigRules; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Indicates whether the specified Config rules are compliant. If a rule is noncompliant, this operation returns the number of Amazon Web Services resources that do not comply with the rule. A rule is compliant if all of the…",
            ),
        ),
        (
            "DescribeComplianceByResource",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ComplianceByResources; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Indicates whether the specified Amazon Web Services resources are compliant. If a resource is noncompliant, this operation returns the number of Config rules that the resource does not comply with. A resource is compliant if it…",
            ),
        ),
        (
            "DescribeConfigRuleEvaluationStatus",
            OperationOverlay {
                queryable: vec![path("ConfigRuleArn")?],
                ..read(
                    "ConfigRulesEvaluationStatus",
                    Identity::Arn(path("ConfigRuleArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeConfigRules",
            OperationOverlay {
                queryable: vec![path("ConfigRuleArn")?],
                ..read(
                    "ConfigRules",
                    Identity::Arn(path("ConfigRuleArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeConfigurationAggregatorSourcesStatus",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns status information for sources within an aggregator. The status includes information about the last time Config verified authorization between the source account and an aggregator account. In case of a failure, the status…",
            ),
        ),
        (
            "DescribeConfigurationAggregators",
            OperationOverlay {
                queryable: vec![path("ConfigurationAggregatorArn")?],
                ..read(
                    "ConfigurationAggregators",
                    Identity::Arn(path("ConfigurationAggregatorArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeConfigurationRecorderStatus",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "ConfigurationRecordersStatus",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeConfigurationRecorders",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "ConfigurationRecorders",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeConformancePackCompliance",
            deferred(
                "Model requires ConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns compliance details for each rule in that conformance pack. You must provide exact rule names.",
            ),
        ),
        (
            "DescribeConformancePackStatus",
            OperationOverlay {
                queryable: vec![path("ConformancePackArn")?],
                ..read(
                    "ConformancePackStatusDetails",
                    Identity::Arn(path("ConformancePackArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeConformancePacks",
            OperationOverlay {
                queryable: vec![path("ConformancePackArn")?],
                ..read(
                    "ConformancePackDetails",
                    Identity::Arn(path("ConformancePackArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeDeliveryChannelStatus",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is DeliveryChannelsStatus; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the current status of the specified delivery channel. If a delivery channel is not specified, this operation returns the current status of all delivery channels associated with the account. Currently, you can specify only…",
            ),
        ),
        (
            "DescribeDeliveryChannels",
            OperationOverlay {
                queryable: vec![path("name")?],
                ..read(
                    "DeliveryChannels",
                    Identity::Key(vec![path("name")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeOrganizationConfigRuleStatuses",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is OrganizationConfigRuleStatuses; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Provides organization Config rule deployment status for an organization. The status is not considered successful until organization Config rule is successfully deployed in all the member accounts with an exception of excluded…",
            ),
        ),
        (
            "DescribeOrganizationConfigRules",
            OperationOverlay {
                queryable: vec![path("OrganizationConfigRuleArn")?],
                ..read(
                    "OrganizationConfigRules",
                    Identity::Arn(path("OrganizationConfigRuleArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribeOrganizationConformancePackStatuses",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is OrganizationConformancePackStatuses; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Provides organization conformance pack deployment status for an organization. The status is not considered successful until organization conformance pack is successfully deployed in all the member accounts with an exception of…",
            ),
        ),
        (
            "DescribeOrganizationConformancePacks",
            OperationOverlay {
                queryable: vec![path("OrganizationConformancePackArn")?],
                ..read(
                    "OrganizationConformancePacks",
                    Identity::Arn(path("OrganizationConformancePackArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DescribePendingAggregationRequests",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PendingAggregationRequests; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of all pending aggregation requests.",
            ),
        ),
        (
            "DescribeRemediationConfigurations",
            deferred(
                "Model requires ConfigRuleNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of one or more remediation configurations.",
            ),
        ),
        (
            "DescribeRemediationExceptions",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of one or more remediation exceptions. A detailed view of a remediation exception for a set of resources that includes an explanation of an exception and the time when the exception will be deleted. When you…",
            ),
        ),
        (
            "DescribeRemediationExecutionStatus",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Provides a detailed view of a Remediation Execution for a set of resources including state, timestamps for when steps for the remediation execution occur, and any error messages for steps that have failed. When you specify the…",
            ),
        ),
        (
            "DescribeRetentionConfigurations",
            OperationOverlay {
                queryable: vec![path("Name")?],
                ..read(
                    "RetentionConfigurations",
                    Identity::Key(vec![path("Name")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "DisassociateResourceTypes",
            deferred(
                "Model requires ConfigurationRecorderArn, ResourceTypes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes all resource types specified in the ResourceTypes list from the RecordingGroup of configuration recorder and excludes these resource types when recording. For this operation, the configuration recorder must use a…",
            ),
        ),
        (
            "GetAggregateComplianceDetailsByConfigRule",
            deferred(
                "Model requires ConfigurationAggregatorName, ConfigRuleName, AccountId, AwsRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the evaluation results for the specified Config rule for a specific resource in a rule. The results indicate which Amazon Web Services resources were evaluated by the rule, when each resource was last evaluated, and…",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetAggregateConfigRuleComplianceSummary",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the number of compliant and noncompliant rules for one or more accounts and regions in an aggregator. The results can return an empty result page, but if you have a nextToken, the results are displayed on the next page.",
            ),
        ),
        (
            "GetAggregateConformancePackComplianceSummary",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the count of compliant and noncompliant conformance packs across all Amazon Web Services accounts and Amazon Web Services Regions in an aggregator. You can filter based on Amazon Web Services account ID or Amazon Web…",
            ),
        ),
        (
            "GetAggregateDiscoveredResourceCounts",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the resource counts across accounts and regions that are present in your Config aggregator. You can request the resource counts by providing filters and GroupByKey. For example, if the input contains accountID 12345678910…",
            ),
        ),
        (
            "GetAggregateResourceConfig",
            deferred(
                "Model requires ConfigurationAggregatorName, ResourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns configuration item that is aggregated for your specific resource in a specific source account and region. The API does not return results for deleted resources.",
            ),
        ),
        (
            "GetComplianceDetailsByConfigRule",
            deferred(
                "Model requires ConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the evaluation results for the specified Config rule. The results indicate which Amazon Web Services resources were evaluated by the rule, when each resource was last evaluated, and whether each resource complies with the…",
            ),
        ),
        (
            "GetComplianceDetailsByResource",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is EvaluationResults; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the evaluation results for the specified Amazon Web Services resource. The results indicate which Config rules were used to evaluate the resource, when each rule was last invoked, and whether the resource complies with…",
            ),
        ),
        (
            "GetComplianceSummaryByConfigRule",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ComplianceSummary; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the number of Config rules that are compliant and noncompliant, up to a maximum of 25 for each.",
            ),
        ),
        (
            "GetComplianceSummaryByResourceType",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ComplianceSummariesByResourceType; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the number of resources that are compliant and the number that are noncompliant. You can specify one or more resource types to get these numbers for each resource type. The maximum number returned is 100.",
            ),
        ),
        (
            "GetConformancePackComplianceDetails",
            deferred(
                "Model requires ConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns compliance details of a conformance pack for all Amazon Web Services resources that are monitered by conformance pack.",
            ),
        ),
        (
            "GetConformancePackComplianceSummary",
            deferred(
                "Model requires ConformancePackNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns compliance details for the conformance pack based on the cumulative compliance results of all the rules in that conformance pack.",
            ),
        ),
        (
            "GetConnector",
            deferred(
                "Model requires Arn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of the specified connector.",
            ),
        ),
        (
            "GetCustomRulePolicy",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is PolicyText; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the policy definition containing the logic for your Config Custom Policy rule.",
            ),
        ),
        (
            "GetDiscoveredResourceCounts",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is totalDiscoveredResources, resourceCounts; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the resource types, the number of each resource type, and the total number of resources that Config is recording in this region for your Amazon Web Services account. Example Config is recording three resource types in the…",
            ),
        ),
        (
            "GetOrganizationConfigRuleDetailedStatus",
            deferred(
                "Model requires OrganizationConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed status for each member account within an organization for a given organization Config rule.",
            ),
        ),
        (
            "GetOrganizationConformancePackDetailedStatus",
            deferred(
                "Model requires OrganizationConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns detailed status for each member account within an organization for a given organization conformance pack.",
            ),
        ),
        (
            "GetOrganizationCustomRulePolicy",
            deferred(
                "Model requires OrganizationConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the policy definition containing the logic for your organization Config Custom Policy rule.",
            ),
        ),
        (
            "GetResourceConfigHistory",
            deferred(
                "Model requires resourceType, resourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: For accurate reporting on the compliance status, you must record the AWS::Config::ResourceCompliance resource type. For more information, see Recording Amazon Web Services Resources in the Config Resources Developer Guide .…",
            ),
        ),
        (
            "GetResourceEvaluationSummary",
            deferred(
                "Model requires ResourceEvaluationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a summary of resource evaluation for the specified resource evaluation ID from the proactive rules that were run. The results indicate which evaluation context was used to evaluate the rules, which resource details were…",
            ),
        ),
        (
            "GetStoredQuery",
            deferred(
                "Model requires QueryName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of a specific stored query.",
            ),
        ),
        (
            "ListAggregateDiscoveredResources",
            deferred(
                "Model requires ConfigurationAggregatorName, ResourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Accepts a resource type and returns a list of resource identifiers that are aggregated for a specific resource type across accounts and regions. A resource identifier includes the resource type, ID, (if available) the custom…",
            ),
        ),
        (
            "ListConfigurationRecorders",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "ConfigurationRecorderSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListConformancePackComplianceScores",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is ConformancePackComplianceScores; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns a list of conformance pack compliance scores. A compliance score is the percentage of the number of compliant rule-resource combinations in a conformance pack compared to the number of total possible rule-resource…",
            ),
        ),
        (
            "ListConnectors",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "ConnectorSummaries",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListDiscoveredResources",
            deferred(
                "Model requires resourceType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of resource resource identifiers for the specified resource types for the resources of that type. A resource identifier includes the resource type, ID, and (if available) the custom resource name. The results…",
            ),
        ),
        (
            "ListResourceEvaluations",
            OperationOverlay {
                queryable: vec![path("ResourceEvaluationId")?],
                ..read(
                    "ResourceEvaluations",
                    Identity::Key(vec![path("ResourceEvaluationId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListStoredQueries",
            OperationOverlay {
                queryable: vec![path("QueryArn")?],
                ..read(
                    "StoredQueryMetadata",
                    Identity::Arn(path("QueryArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the tags for Config resource.",
            ),
        ),
        (
            "PutAggregationAuthorization",
            deferred(
                "Model requires AuthorizedAccountId, AuthorizedAwsRegion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Authorizes the aggregator account and region to collect data from the source account and region. Tags are added at creation and cannot be updated with this operation PutAggregationAuthorization is an idempotent API. Subsequent…",
            ),
        ),
        (
            "PutConfigRule",
            deferred(
                "Model requires ConfigRule. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates an Config rule to evaluate if your Amazon Web Services resources comply with your desired configurations. For information on how many Config rules you can have per account, see Service Limits in the Config…",
            ),
        ),
        (
            "PutConfigurationAggregator",
            deferred(
                "Model requires ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and updates the configuration aggregator with the selected source accounts and regions. The source account can be individual account(s) or an organization. accountIds that are passed will be replaced with existing…",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutConfigurationRecorder",
            deferred(
                "Model requires ConfigurationRecorder. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates the customer managed configuration recorder. You can use this operation to create a new customer managed configuration recorder or to update the roleARN and the recordingGroup for an existing customer managed…",
            ),
        ),
        (
            "PutConformancePack",
            deferred(
                "Model requires ConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a conformance pack. A conformance pack is a collection of Config rules that can be easily deployed in an account and a region and across an organization. For information on how many conformance packs you can…",
            ),
        ),
        (
            "PutConnector",
            deferred(
                "Model requires ConnectorConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a connector that specifies the connection between a third-party cloud service provider and Config. A connector is required to create a service-linked configuration recorder for a third-party cloud service provider using…",
            ),
        ),
        (
            "PutDeliveryChannel",
            deferred(
                "Model requires DeliveryChannel. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a delivery channel to deliver configuration information and other compliance information. You can use this operation to create a new delivery channel or to update the Amazon S3 bucket and the Amazon SNS topic…",
            ),
        ),
        (
            "PutEvaluations",
            deferred(
                "Model requires ResultToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Used by an Lambda function to deliver evaluation results to Config. This operation is required in every Lambda function that is invoked by an Config rule.",
            ),
        ),
        (
            "PutExternalEvaluation",
            deferred(
                "Model requires ConfigRuleName, ExternalEvaluation. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add or updates the evaluations for process checks. This API checks if the rule is a process check when the name of the Config rule is provided.",
            ),
        ),
        (
            "PutOrganizationConfigRule",
            deferred(
                "Model requires OrganizationConfigRuleName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates an Config rule for your entire organization to evaluate if your Amazon Web Services resources comply with your desired configurations. For information on how many organization Config rules you can have per…",
            ),
        ),
        (
            "PutOrganizationConformancePack",
            deferred(
                "Model requires OrganizationConformancePackName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deploys conformance packs across member accounts in an Amazon Web Services Organization. For information on how many organization conformance packs and how many Config rules you can have per account, see Service Limits in the…",
            ),
        ),
        (
            "PutRemediationConfigurations",
            deferred(
                "Model requires RemediationConfigurations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates the remediation configuration with a specific Config rule with the selected target or action. The API creates the RemediationConfiguration object for the Config rule. The Config rule must already exist for you to…",
            ),
        ),
        (
            "PutRemediationExceptions",
            deferred(
                "Model requires ConfigRuleName, ResourceKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A remediation exception is when a specified resource is no longer considered for auto-remediation. This API adds a new exception or updates an existing exception for a specified resource with a specified Config rule. Exceptions…",
            ),
        ),
        (
            "PutResourceConfig",
            deferred(
                "Model requires ResourceType, SchemaVersionId, ResourceId, Configuration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Records the configuration state for the resource provided in the request. The configuration state of a resource is represented in Config as Configuration Items. Once this API records the configuration item, you can retrieve the…",
            ),
        ),
        (
            "PutRetentionConfiguration",
            deferred(
                "Model requires RetentionPeriodInDays. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates and updates the retention configuration with details about retention period (number of days) that Config stores your historical information. The API creates the RetentionConfiguration object and names the object as…",
            ),
        ),
        (
            "PutServiceLinkedConfigurationRecorder",
            deferred(
                "Model requires ServicePrincipal. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a service-linked configuration recorder that is linked to a specific Amazon Web Services service based on the ServicePrincipal you specify. The configuration recorder's name , recordingGroup , recordingMode , and…",
            ),
        ),
        (
            "PutStoredQuery",
            deferred(
                "Model requires StoredQuery. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Saves a new query or updates an existing saved query. The QueryName must be unique for a single Amazon Web Services account and a single Amazon Web Services Region. You can create upto 300 queries in a single Amazon Web Services…",
            ),
        ),
        (
            "PutThirdPartyServiceLinkedConfigurationRecorder",
            deferred(
                "Model requires ServicePrincipal, ConnectorArn, ScopeConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates or updates a service-linked configuration recorder that is linked to a third-party cloud service provider based on the ConnectorArn you specify. The configuration recorder's name , recordingGroup , recordingMode , and…",
            ),
        ),
        (
            "SelectAggregateResourceConfig",
            deferred(
                "Model requires Expression, ConfigurationAggregatorName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Accepts a structured query language (SQL) SELECT command and an aggregator to query configuration state of Amazon Web Services resources across multiple accounts and regions, performs the corresponding search, and returns…",
            ),
        ),
        (
            "SelectResourceConfig",
            deferred(
                "Model requires Expression. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Accepts a structured query language (SQL) SELECT command, performs the corresponding search, and returns resource configurations matching the properties. For more information about query components, see the Query Components…",
            ),
        ),
        (
            "StartConfigRulesEvaluation",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Runs an on-demand evaluation for the specified Config rules against the last known configuration state of the resources. Use StartConfigRulesEvaluation when you want to test that a rule you updated is working as expected.…",
            ),
        ),
        (
            "StartConfigurationRecorder",
            deferred(
                "Model requires ConfigurationRecorderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts the customer managed configuration recorder. The customer managed configuration recorder will begin recording configuration changes for the resource types you specify. You must have created a delivery channel to…",
            ),
        ),
        (
            "StartRemediationExecution",
            deferred(
                "Model requires ConfigRuleName, ResourceKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Runs an on-demand remediation for the specified Config rules against the last known remediation configuration. It runs an execution against the current state of your resources. Remediation execution is asynchronous. You can…",
            ),
        ),
        (
            "StartResourceEvaluation",
            deferred(
                "Model requires ResourceDetails, EvaluationMode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Runs an on-demand evaluation for the specified resource to determine whether the resource details will comply with configured Config rules. You can also use it for evaluation purposes. Config recommends using an evaluation…",
            ),
        ),
        (
            "StopConfigurationRecorder",
            deferred(
                "Model requires ConfigurationRecorderName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops the customer managed configuration recorder. The customer managed configuration recorder will stop recording configuration changes for the resource types you have specified.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified tags to a resource with the specified ResourceArn . If existing tags on a resource are not specified in the request parameters, they are not changed. If existing tags are specified, however, then their…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes specified tags from a resource.",
            ),
        ),
    ])
}
