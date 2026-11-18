// Pinned source: .local/aws-models/models/codecatalyst/service/2022-09-28/codecatalyst-2022-09-28.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateAccessToken",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Creates a personal access token (PAT) for the current user. A personal access token (PAT) is similar to a password. It is associated with your user identity for use across all spaces and projects in Amazon CodeCatalyst. You use…",
            ),
        ),
        (
            "CreateDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Creates a Dev Environment in Amazon CodeCatalyst, a cloud-based development environment that you can use to quickly work on the code stored in the source repositories of your project. When created in the Amazon CodeCatalyst…",
            ),
        ),
        (
            "CreateProject",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Creates a project in a specified space.",
            ),
        ),
        (
            "CreateSourceRepository",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Creates an empty Git-based source repository in a specified project. The repository is created with an initial empty commit with a default branch named main .",
            ),
        ),
        (
            "CreateSourceRepositoryBranch",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Creates a branch in a specified source repository in Amazon CodeCatalyst. This API only creates a branch in a source repository hosted in Amazon CodeCatalyst. You cannot use this API to create a branch in a linked repository.",
            ),
        ),
        (
            "DeleteAccessToken",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Deletes a specified personal access token (PAT). A personal access token can only be deleted by the user who created it.",
            ),
        ),
        (
            "DeleteDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Deletes a Dev Environment.",
            ),
        ),
        (
            "DeleteProject",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Deletes a project in a space.",
            ),
        ),
        (
            "DeleteSourceRepository",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Deletes a source repository in Amazon CodeCatalyst. You cannot use this API to delete a linked repository. It can only be used to delete a Amazon CodeCatalyst source repository.",
            ),
        ),
        (
            "DeleteSpace",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Deletes a space. Deleting a space cannot be undone. Additionally, since space names must be unique across Amazon CodeCatalyst, you cannot reuse names of deleted spaces.",
            ),
        ),
        (
            "GetDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a Dev Environment for a source repository in a project. Dev Environments are specific to the user who creates them.",
            ),
        ),
        (
            "GetProject",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a project.",
            ),
        ),
        (
            "GetSourceRepository",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a source repository.",
            ),
        ),
        (
            "GetSourceRepositoryCloneUrls",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about the URLs that can be used with a Git client to clone a source repository.",
            ),
        ),
        (
            "GetSpace",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about an space.",
            ),
        ),
        (
            "GetSubscription",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about the Amazon Web Services account used for billing purposes and the billing plan for the space.",
            ),
        ),
        (
            "GetUserDetails",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a user.",
            ),
        ),
        (
            "GetWorkflow",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a workflow.",
            ),
        ),
        (
            "GetWorkflowRun",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Returns information about a specified run of a workflow.",
            ),
        ),
        (
            "ListAccessTokens",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Lists all personal access tokens (PATs) associated with the user who calls the API. You can only list PATs associated with your Amazon Web Services Builder ID.",
            ),
        ),
        (
            "ListDevEnvironmentSessions",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of active sessions for a Dev Environment in a project.",
            ),
        ),
        (
            "ListDevEnvironments",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of Dev Environments in a project.",
            ),
        ),
        (
            "ListEventLogs",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of events that occurred during a specific time in a space. You can use these events to audit user and system activity in a space. For more information, see Monitoring in the Amazon CodeCatalyst User Guide .…",
            ),
        ),
        (
            "ListProjects",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of projects.",
            ),
        ),
        (
            "ListSourceRepositories",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of source repositories in a project.",
            ),
        ),
        (
            "ListSourceRepositoryBranches",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of branches in a specified source repository.",
            ),
        ),
        (
            "ListSpaces",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of spaces.",
            ),
        ),
        (
            "ListWorkflowRuns",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of workflow runs of a specified workflow.",
            ),
        ),
        (
            "ListWorkflows",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Retrieves a list of workflows in a specified project.",
            ),
        ),
        (
            "StartDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Starts a specified Dev Environment and puts it into an active state.",
            ),
        ),
        (
            "StartDevEnvironmentSession",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Starts a session for a specified Dev Environment.",
            ),
        ),
        (
            "StartWorkflowRun",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Begins a run of a specified workflow.",
            ),
        ),
        (
            "StopDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Pauses a specified Dev Environment and places it in a non-running state. Stopped Dev Environments do not consume compute minutes.",
            ),
        ),
        (
            "StopDevEnvironmentSession",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Stops a session for a specified Dev Environment.",
            ),
        ),
        (
            "UpdateDevEnvironment",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Changes one or more values for a Dev Environment. Updating certain values of the Dev Environment will cause a restart.",
            ),
        ),
        (
            "UpdateProject",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Changes one or more values for a project.",
            ),
        ),
        (
            "UpdateSpace",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Changes one or more values for a space.",
            ),
        ),
        (
            "VerifySession",
            deferred(
                "CodeCatalyst requires AWS Builder ID bearer authentication and a space/project context; the current SigV4 inventory runtime has no reviewed bearer-auth credential source. Model: Verifies whether the calling user has a valid Amazon CodeCatalyst login and session. If successful, this returns the ID of the user in Amazon CodeCatalyst.",
            ),
        ),
    ])
}
