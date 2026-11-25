use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BuildSuggesters",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Indexes the search suggestions. For more information, see Configuring Suggesters in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "CreateDomain",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new search domain. For more information, see Creating a Search Domain in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DefineAnalysisScheme",
            deferred(
                "Model requires DomainName, AnalysisScheme. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures an analysis scheme that can be applied to a text or text-array field to define language-specific text processing options. For more information, see Configuring Analysis Schemes in the Amazon CloudSearch Developer Guide…",
            ),
        ),
        (
            "DefineExpression",
            deferred(
                "Model requires DomainName, Expression. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures an Expression for the search domain. Used to create new expressions and modify existing ones. If the expression exists, the new configuration replaces the old one. For more information, see Configuring Expressions in…",
            ),
        ),
        (
            "DefineIndexField",
            deferred(
                "Model requires DomainName, IndexField. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures an IndexField for the search domain. Used to create new fields and modify existing ones. You must specify the name of the domain you are configuring and an index field configuration. The index field configuration…",
            ),
        ),
        (
            "DefineSuggester",
            deferred(
                "Model requires DomainName, Suggester. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures a suggester for a domain. A suggester enables you to display possible matches before users finish typing their queries. When you configure a suggester, you must specify the name of the text field you want to search for…",
            ),
        ),
        (
            "DeleteAnalysisScheme",
            deferred(
                "Model requires DomainName, AnalysisSchemeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an analysis scheme. For more information, see Configuring Analysis Schemes in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DeleteDomain",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Permanently deletes a search domain and all of its data. Once a domain has been deleted, it cannot be recovered. For more information, see Deleting a Search Domain in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DeleteExpression",
            deferred(
                "Model requires DomainName, ExpressionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an Expression from the search domain. For more information, see Configuring Expressions in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DeleteIndexField",
            deferred(
                "Model requires DomainName, IndexFieldName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an IndexField from the search domain. For more information, see Configuring Index Fields in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DeleteSuggester",
            deferred(
                "Model requires DomainName, SuggesterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a suggester. For more information, see Getting Search Suggestions in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DescribeAnalysisSchemes",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the analysis schemes configured for a domain. An analysis scheme defines language-specific text processing options for a text field. Can be limited to specific analysis schemes by name. By default, shows all analysis schemes…",
            ),
        ),
        (
            "DescribeAvailabilityOptions",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the availability options configured for a domain. By default, shows the configuration with any pending changes. Set the Deployed option to true to show the active configuration and exclude pending changes. For more…",
            ),
        ),
        (
            "DescribeDomainEndpointOptions",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the domain's endpoint options, specifically whether all requests to the domain must arrive over HTTPS. For more information, see Configuring Domain Endpoint Options in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "DescribeDomains",
            OperationOverlay {
                queryable: vec![path("ARN")?],
                ..read(
                    "DomainStatusList",
                    Identity::Arn(path("ARN")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeExpressions",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the expressions configured for the search domain. Can be limited to specific expressions by name. By default, shows all expressions and includes any pending changes to the configuration. Set the Deployed option to true to…",
            ),
        ),
        (
            "DescribeIndexFields",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the index fields configured for the search domain. Can be limited to specific fields by name. By default, shows all fields and includes any pending changes to the configuration. Set the Deployed option to…",
            ),
        ),
        (
            "DescribeScalingParameters",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the scaling parameters configured for a domain. A domain's scaling parameters specify the desired search instance type and replication count. For more information, see Configuring Scaling Options in the Amazon CloudSearch…",
            ),
        ),
        (
            "DescribeServiceAccessPolicies",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the access policies that control access to the domain's document and search endpoints. By default, shows the configuration with any pending changes. Set the Deployed option to true to show the active…",
            ),
        ),
        (
            "DescribeSuggesters",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the suggesters configured for a domain. A suggester enables you to display possible matches before users finish typing their queries. Can be limited to specific suggesters by name. By default, shows all suggesters and…",
            ),
        ),
        (
            "IndexDocuments",
            deferred(
                "Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tells the search domain to start indexing its documents using the latest indexing options. This operation must be invoked to activate options whose OptionStatus is RequiresIndexDocuments .",
            ),
        ),
        (
            "ListDomainNames",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is DomainNames; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists all search domains owned by an account.",
            ),
        ),
        (
            "UpdateAvailabilityOptions",
            deferred(
                "Model requires DomainName, MultiAZ. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures the availability options for a domain. Enabling the Multi-AZ option expands an Amazon CloudSearch domain to an additional Availability Zone in the same Region to increase fault tolerance in the event of a service…",
            ),
        ),
        (
            "UpdateDomainEndpointOptions",
            deferred(
                "Model requires DomainName, DomainEndpointOptions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the domain's endpoint options, specifically whether all requests to the domain must arrive over HTTPS. For more information, see Configuring Domain Endpoint Options in the Amazon CloudSearch Developer Guide .",
            ),
        ),
        (
            "UpdateScalingParameters",
            deferred(
                "Model requires DomainName, ScalingParameters. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures scaling parameters for a domain. A domain's scaling parameters specify the desired search instance type and replication count. Amazon CloudSearch will still automatically scale your domain based on the volume of data…",
            ),
        ),
        (
            "UpdateServiceAccessPolicies",
            deferred(
                "Model requires DomainName, AccessPolicies. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Configures the access rules that control access to the domain's document and search endpoints. For more information, see Configuring Access for an Amazon CloudSearch Domain .",
            ),
        ),
    ])
}
