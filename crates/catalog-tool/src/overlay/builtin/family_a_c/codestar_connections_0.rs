// Pinned source: .local/aws-models/models/codestar-connections/service/2019-12-01/codestar-connections-2019-12-01.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateConnection", deferred("Model requires ConnectionName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a connection that can then be given to other Amazon Web Services services like CodePipeline so that it can access third-party code repositories. The connection is in pending status until the third-party connection…")),
        ("CreateHost", deferred("Model requires Name, ProviderType, ProviderEndpoint. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a resource that represents the infrastructure where a third-party provider is installed. The host is used when you create connections to an installed third-party provider type, such as GitHub Enterprise Server. You create…")),
        ("CreateRepositoryLink", deferred("Model requires ConnectionArn, OwnerId, RepositoryName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a link to a specified external Git repository. A repository link allows Git sync to monitor and sync changes to files in a specified Git repository.")),
        ("CreateSyncConfiguration", deferred("Model requires Branch, ConfigFile, RepositoryLinkId, ResourceName, RoleArn, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a sync configuration which allows Amazon Web Services to sync content from a Git repository to update a specified Amazon Web Services resource. Parameters for the sync configuration are determined by the sync type.")),
        ("DeleteConnection", deferred("Model requires ConnectionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The connection to be deleted.")),
        ("DeleteHost", deferred("Model requires HostArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The host to be deleted. Before you delete a host, all connections associated to the host must be deleted. A host cannot be deleted if it is in the VPC_CONFIG_INITIALIZING or VPC_CONFIG_DELETING state.")),
        ("DeleteRepositoryLink", deferred("Model requires RepositoryLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the association between your connection and a specified external Git repository.")),
        ("DeleteSyncConfiguration", deferred("Model requires SyncType, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the sync configuration for a specified repository and connection.")),
        // Model: Returns the connection ARN and details such as status, owner, and provider type.
        ("GetConnection", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("ConnectionArn")?, Binding::From { operation: name("ListConnections")?, path: path("ConnectionArn")? })].into_iter().collect(),
            queryable: vec![path("ConnectionArn")?],
            ..read("Connection", Identity::Arn(path("ConnectionArn")?), Scope::Region, Paginator::None)?
        }),
        ("GetHost", deferred("Host metadata response does not repeat HostArn. Current selected-record identities cannot use the bound HostArn; ListHosts already supplies stable ARN summaries.")),
        // Model: Returns details about a repository link. A repository link allows Git sync to monitor and sync changes from files in a specified Git repository.
        ("GetRepositoryLink", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("RepositoryLinkId")?, Binding::From { operation: name("ListRepositoryLinks")?, path: path("RepositoryLinkId")? })].into_iter().collect(),
            queryable: vec![path("RepositoryLinkArn")?],
            ..read("RepositoryLinkInfo", Identity::Arn(path("RepositoryLinkArn")?), Scope::Region, Paginator::None)?
        }),
        ("GetRepositorySyncStatus", deferred("Model requires Branch, RepositoryLinkId, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns details about the sync status for a repository. A repository sync uses Git sync to push and pull changes from your remote repository.")),
        ("GetResourceSyncStatus", deferred("Model requires ResourceName, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the status of the sync with the Git repository for a specific Amazon Web Services resource.")),
        ("GetSyncBlockerSummary", deferred("Model requires SyncType, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the most recent sync blockers.")),
        ("GetSyncConfiguration", deferred("Model requires SyncType, ResourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns details about a sync configuration, including the sync type and resource name. A sync configuration allows the configuration to sync (push and pull) changes from the remote repository for a specified branch in a Git…")),
        // Model: Lists the connections associated with your account.
        ("ListConnections", OperationOverlay {
            queryable: vec![path("ConnectionArn")?],
            ..read("Connections", Identity::Arn(path("ConnectionArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Lists the hosts associated with your account.
        ("ListHosts", OperationOverlay {
            queryable: vec![path("HostArn")?],
            ..read("Hosts", Identity::Arn(path("HostArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Model: Lists the repository links created for connections in your account.
        ("ListRepositoryLinks", OperationOverlay {
            queryable: vec![path("RepositoryLinkArn")?],
            ..read("RepositoryLinks", Identity::Arn(path("RepositoryLinkArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("ListRepositorySyncDefinitions", deferred("Model requires RepositoryLinkId, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the repository sync definitions for repository links in your account.")),
        ("ListSyncConfigurations", deferred("Model requires RepositoryLinkId, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of sync configurations for a specified repository.")),
        ("ListTagsForResource", deferred("Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the set of key-value pairs (metadata) that are used to manage the resource.")),
        ("TagResource", deferred("Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds to or modifies the tags of the given resource. Tags are metadata that can be used to manage a resource.")),
        ("UntagResource", deferred("Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from an Amazon Web Services resource.")),
        ("UpdateHost", deferred("Model requires HostArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a specified host with the provided configurations.")),
        ("UpdateRepositoryLink", deferred("Model requires RepositoryLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the association between your connection and a specified external Git repository. A repository link allows Git sync to monitor and sync changes to files in a specified Git repository.")),
        ("UpdateSyncBlocker", deferred("Model requires Id, SyncType, ResourceName, ResolvedReason. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows you to update the status of a sync blocker, resolving the blocker and allowing syncing to continue.")),
        ("UpdateSyncConfiguration", deferred("Model requires ResourceName, SyncType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the sync configuration for your connection and a specified external Git repository.")),
    ])
}
