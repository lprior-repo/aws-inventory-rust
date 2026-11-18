// Pinned source: .local/aws-models/models/cloudwatchomni/service/2025-01-01/cloudwatchomni-2025-01-01.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateAccessGrant",
            deferred(
                "Model requires domainId, spaceId, name, principal, permission. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AccessGrant that authorizes a principal to perform a set of actions on resources in a space. Optionally narrow the grant with scoped actions that limit it to specific resources and fields. Use ListAccessGrants and…",
            ),
        ),
        (
            "CreateAccessProfile",
            deferred(
                "Model requires spaceId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an access profile in a space. Use GetAccessProfile and ListAccessProfiles to retrieve profiles, and UpdateAccessProfile to modify one.",
            ),
        ),
        (
            "CreateAlert",
            deferred(
                "Model requires spaceId, profileId, name, rule. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new alert within a space. Use GetAlert and ListAlerts to retrieve alerts, UpdateAlert to modify one, and DeleteAlert to remove it.",
            ),
        ),
        (
            "CreateDomain",
            deferred(
                "Model requires name, identityProviders. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain with identity provider configuration. Use GetDomain to retrieve the domain, UpdateDomain to change its configuration, and CreateSpace to add spaces within it.",
            ),
        ),
        (
            "CreateDomainAccessGrantForOrganization",
            deferred(
                "Model requires domainId, name, principal, permission. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AccessGrant that authorizes a principal to administer an organization domain.",
            ),
        ),
        (
            "CreateDomainForOrganization",
            deferred(
                "Model requires name, identityProviders, domainAccessRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an organization-scoped domain for the caller's AWS Organization. Only the organization's management account can call this operation.",
            ),
        ),
        (
            "CreateIntegration",
            deferred(
                "Model requires integrationType, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an integration with a third-party provider. Returns the integration identifier and its initial status; when the provider requires interactive consent, an authorization URL is returned for the user to complete setup.",
            ),
        ),
        (
            "CreateOmniDashboard",
            deferred(
                "Model requires spaceId, name, body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new dashboard within a space. Use GetOmniDashboard and ListOmniDashboards to retrieve dashboards, UpdateOmniDashboard to modify one, and DeleteOmniDashboard to remove it.",
            ),
        ),
        (
            "CreateOneTimeDeepLinkCode",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates a one-time code for deep-link authentication. Direct the user's browser to the returned deepLinkUrl before it expires. The code is exchanged for an authenticated, domain-scoped session and can be used only once.",
            ),
        ),
        (
            "CreateSpace",
            deferred(
                "Model requires name, domainId, dataAccessRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a space in a domain. Use GetSpace to retrieve the space, ListSpaces to enumerate spaces, UpdateSpace to modify it, and DeleteSpace to remove it.",
            ),
        ),
        (
            "CreateView",
            deferred(
                "Model requires name, definition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new SQL view. A view is a named, reusable SQL query that can be referenced from telemetry queries. View names must be unique within the account and region. Only USER views can be created — MANAGED views are provisioned…",
            ),
        ),
        (
            "DeleteAccessGrant",
            deferred(
                "Model requires grantId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an existing AccessGrant, revoking the access it granted. A service-managed grant cannot be deleted.",
            ),
        ),
        (
            "DeleteAccessProfile",
            deferred(
                "Model requires spaceId, profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an access profile. An access profile cannot be deleted while access grants reference it.",
            ),
        ),
        (
            "DeleteAlert",
            deferred(
                "Model requires spaceId, alertId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an alert by its identifier. Idempotent: deleting an alert that has already been removed succeeds without error.",
            ),
        ),
        (
            "DeleteDomain",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a domain and all of its resources. Call this operation in the Region where the domain was created. A domain cannot be deleted while it contains spaces.",
            ),
        ),
        (
            "DeleteDomainAccessGrantForOrganization",
            deferred(
                "Model requires grantId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an existing organization access grant, revoking the access it granted. A service-managed grant cannot be deleted.",
            ),
        ),
        (
            "DeleteDomainForOrganization",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an organization domain and all of its resources. Call this operation in the Region where the domain was created. A domain cannot be deleted while it contains spaces.",
            ),
        ),
        (
            "DeleteIntegration",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an integration. Returns the resulting status.",
            ),
        ),
        (
            "DeleteOmniDashboard",
            deferred(
                "Model requires spaceId, dashboardId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a dashboard from a space.",
            ),
        ),
        (
            "DeleteSpace",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a space and all of its resources.",
            ),
        ),
        (
            "DeleteView",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified view. Queries that reference the view fail after it is deleted. Managed views cannot be deleted.",
            ),
        ),
        (
            "GetAccessGrant",
            deferred(
                "Model requires grantId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the full detail of a single AccessGrant by ID.",
            ),
        ),
        (
            "GetAccessProfile",
            deferred(
                "Model requires spaceId, profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an access profile by ID. The response indicates whether the calling principal is currently allowed to assume the profile.",
            ),
        ),
        (
            "GetAlert",
            deferred(
                "Model requires spaceId, alertId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a single alert by its identifier. Use ListAlerts to enumerate alerts in the space.",
            ),
        ),
        (
            "GetContextGraph",
            deferred(
                "Model requires startTime, endTime. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Queries the context graph with filtering, traversal, and pagination support. Pagination note: nodes and edges are returned together as a coherent subgraph. Pagination cursors advance over nodes (the primary collection); each page…",
            ),
        ),
        (
            "GetDomain",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a domain by ID.",
            ),
        ),
        (
            "GetDomainAccessGrantForOrganization",
            deferred(
                "Model requires grantId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the full detail of a single organization access grant by ID.",
            ),
        ),
        (
            "GetDomainForOrganization",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of an organization domain by ID.",
            ),
        ),
        (
            "GetIntegration",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the details of a single integration, identified by its identifier, Amazon Resource Name, or name.",
            ),
        ),
        // Model: Retrieves the intelligence configuration for the calling account. Account is identified via FAS (caller identity). Returns the default configuration if none exists yet.
        (
            "GetIntelligenceConfiguration",
            OperationOverlay {
                queryable: vec![path("accountId")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("accountId")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetOmniDashboard",
            deferred(
                "Model requires spaceId, dashboardId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a dashboard by ID within a space.",
            ),
        ),
        (
            "GetSpace",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the details of a space by ID.",
            ),
        ),
        (
            "GetSpaceCredentialsForOrganization",
            deferred(
                "Model requires context, credentialType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns temporary credentials for a space in an organization member account. The credentials are valid for one hour. The caller must be the organization's management account or a delegated administrator with access to the target…",
            ),
        ),
        (
            "GetSpaceDataAccess",
            deferred(
                "Model requires spaceId, targetScopes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns temporary credentials for working with data in a space, limited to the actions you request. The credentials allow only the requested actions that can be granted in the space and that your access grants allow. Requested…",
            ),
        ),
        (
            "GetTelemetryQueryResults",
            deferred(
                "Model requires queryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the results for the specified query.",
            ),
        ),
        (
            "GetView",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the definition and metadata of the specified view.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns AccessGrants, with optional filtering by domain, space, principal, or permission. A grant is returned only when it matches every filter supplied. With no filters, returns the grants for the…
        (
            "ListAccessGrants",
            OperationOverlay {
                queryable: vec![path("grantArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("grantArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListAccessProfiles",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the access profiles in a space.",
            ),
        ),
        (
            "ListAlerts",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists alerts within a space, optionally filtered by exact name(s), a single name prefix, or exact alertId(s), with pagination. Use GetAlert to retrieve a single alert's full detail.",
            ),
        ),
        (
            "ListDomainAccessGrantsForOrganization",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns organization-level domain access grants, with optional filtering by domain, principal, or permission. A grant is returned only when it matches every filter supplied. With no filters, returns the grants for the caller's…",
            ),
        ),
        // Model: Returns the caller's domains: the account-scoped domain and the organization-scoped domain, if either exists. At most two domains are returned.
        (
            "ListDomains",
            OperationOverlay {
                queryable: vec![path("domainArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("domainArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Lists the integrations in the account, optionally filtered by type, status, or name. Results are paginated.
        (
            "ListIntegrations",
            OperationOverlay {
                queryable: vec![path("integrationArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("integrationArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListOmniDashboards",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the dashboards in a space, optionally filtered by name prefix.",
            ),
        ),
        // Model: Returns the spaces in the account, optionally filtered by domain.
        (
            "ListSpaces",
            OperationOverlay {
                queryable: vec![path("spaceArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("spaceArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListSpacesForOrganization",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns the spaces across all member accounts in the organization.",
            ),
        ),
        (
            "ListTelemetryFields",
            deferred(
                "Model requires dataSetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists fields available for telemetry queries. Returns a list of fields included in the specified dataset, granular to telemetry type. Returned field names reflect the exact stored casing and are case-sensitive when referenced in…",
            ),
        ),
        (
            "ListTelemetryQuerySessions",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is sessions; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists telemetry query sessions. Returns a list of telemetry query sessions owned by the caller.",
            ),
        ),
        // Model: Lists the views in the caller's account and region. Returns a summary for each view, optionally filtered by view type. View definitions are not included — use GetView to retrieve them.
        (
            "ListViews",
            OperationOverlay {
                queryable: vec![path("name")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("name")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "PutIntelligenceConfiguration",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is accountId, kmsKeyArn, updatedAt, createdAt; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates or updates the intelligence configuration for the calling account. Account is identified via FAS (caller identity).",
            ),
        ),
        (
            "SearchPrincipals",
            deferred(
                "Model requires domainId, searchQuery. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Searches Identity Center for users and groups in a domain. The domain must be configured with Identity Center. To grant access to a result, pass its principalId to CreateAccessGrant with a principalType of IDC_USER for a user or…",
            ),
        ),
        (
            "StartTelemetryQuery",
            deferred(
                "Model requires queryString, sessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a telemetry query within a session. Submits the provided query string for execution in the specified session. Use GetTelemetryQueryResults to poll for results and check query status.",
            ),
        ),
        (
            "StartTelemetryQuerySession",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is sessionId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Starts a new telemetry query session. A session provides a logical grouping for one or more telemetry queries. The returned session ID is required when starting queries via StartTelemetryQuery.",
            ),
        ),
        (
            "StopTelemetryQuery",
            deferred(
                "Model requires queryId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a running telemetry query.",
            ),
        ),
        (
            "StopTelemetryQuerySession",
            deferred(
                "Model requires sessionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a telemetry query session. Terminates the specified session. After a session is stopped it cannot be reused.",
            ),
        ),
        (
            "UpdateAccessProfile",
            deferred(
                "Model requires spaceId, profileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name or description of an access profile. Only the provided fields are changed; omitted fields are left unchanged.",
            ),
        ),
        (
            "UpdateAlert",
            deferred(
                "Model requires spaceId, alertId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing alert. Only non-null fields overwrite existing values.",
            ),
        ),
        (
            "UpdateDomain",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a domain's name or identity provider configuration. Only the provided fields are changed; omitted fields are left unchanged. Renaming a domain also changes the endpoint URLs derived from its name.",
            ),
        ),
        (
            "UpdateDomainForOrganization",
            deferred(
                "Model requires domainId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an organization domain's name or identity provider configuration. Call this operation in the Region where the domain was created. Only the provided fields are changed; omitted fields are left unchanged. Renaming a domain…",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "UpdateIntegration",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing integration, identified by its id, ARN, or name. Only the fields you provide are changed.",
            ),
        ),
        (
            "UpdateOmniDashboard",
            deferred(
                "Model requires spaceId, dashboardId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing dashboard within a space. Only the provided fields are changed; omitted fields are left unchanged.",
            ),
        ),
        (
            "UpdateSpace",
            deferred(
                "Model requires spaceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a space. Only the provided fields are changed; omitted fields are left unchanged.",
            ),
        ),
        (
            "UpdateView",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing view's definition and/or description. Only the fields you provide are changed. Managed views cannot be updated.",
            ),
        ),
    ])
}
