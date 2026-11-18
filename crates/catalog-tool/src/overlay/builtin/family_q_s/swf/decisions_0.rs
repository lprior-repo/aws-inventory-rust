// Pinned operation purposes: .local/aws-models/models/swf/service/2012-01-25/swf-2012-01-25.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the number of closed workflow executions within the given domain that meet the specified filtering criteria. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to lim
    (
        "CountClosedWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the number of open workflow executions within the given domain that meet the specified filtering criteria. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit
    (
        "CountOpenWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the estimated number of activity tasks in the specified task list. The count returned is an approximation and isn't guaranteed to be exact. If you specify a task list that no activity task was ever scheduled in then 0 is returned. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit
    (
        "CountPendingActivityTasks",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the estimated number of decision tasks in the specified task list. The count returned is an approximation and isn't guaranteed to be exact. If you specify a task list that no decision task was ever scheduled in then 0 is returned. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit
    (
        "CountPendingDecisionTasks",
        Decision::Denied(DenyReason::Content),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified activity type . Note: Prior to deletion, activity types must first be deprecated . After an activity type has been deleted, you cannot schedule new activities of that type. Activities that started before the type was deleted will continue to run. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element
    ("DeleteActivityType", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified workflow type . Note: Prior to deletion, workflow types must first be deprecated . After a workflow type has been deleted, you cannot create new executions of that type. Executions that started before the type was deleted will continue to run. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element wit
    ("DeleteWorkflowType", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Deprecates the specified activity type . After an activity type has been deprecated, you cannot create new tasks of that activity type. Tasks of this type that were scheduled before the type was deprecated continue to run. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to on
    (
        "DeprecateActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Deprecates the specified domain. After a domain has been deprecated it cannot be used to create new workflow executions or register new types. However, you can still use visibility actions on this domain. Deprecating a domain also deprecates all activity and workflow types registered in the domain. Executions that were started before the domain was deprecated continues to run. This operation is ev
    ("DeprecateDomain", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Deprecates the specified workflow type . After a workflow type has been deprecated, you cannot create new executions of that type. Executions that were started before the type was deprecated continues to run. A deprecated workflow type may still be used when calling visibility actions. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates a
    (
        "DeprecateWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about the specified activity type. This includes configuration settings provided when the type was registered and other general information about the type. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to only specified domains. Use an Action element to
    (
        "DescribeActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about the specified domain, including description and status. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to only specified domains. Use an Action element to allow or deny permission to call this action. You cannot use an IAM policy to constrain this a
    ("DescribeDomain", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about the specified workflow execution including its type and some statistics. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to o
    (
        "DescribeWorkflowExecution",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns information about the specified workflow type . This includes configuration settings specified when the type was registered and other information such as creation date, current status, etc. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to only specified domains. Use
    (
        "DescribeWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns the history of the specified workflow execution. The results may be split into multiple pages. To retrieve subsequent pages, make the call again using the nextPageToken returned by the initial call. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's acc
    (
        "GetWorkflowExecutionHistory",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns a list of closed workflow executions in the specified domain that meet the filtering criteria. The results may be split into multiple pages. To retrieve subsequent pages, make the call again using the nextPageToken returned by the initial call. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can
    (
        "ListClosedWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Returns a list of open workflow executions in the specified domain that meet the filtering criteria. The results may be split into multiple pages. To retrieve subsequent pages, make the call again using the nextPageToken returned by the initial call. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can u
    (
        "ListOpenWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: List tags for a given domain.
    ("ListTagsForResource", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by workers to get an ActivityTask from the specified activity taskList . This initiates a long poll, where the service holds the HTTP connection open and responds as soon as a task becomes available. The maximum time the service holds on to the request before responding is 60 seconds. If no task is available within 60 seconds, the poll returns an empty result. An empty result, in this context
    ("PollForActivityTask", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by deciders to get a DecisionTask from the specified decision taskList . A decision task may be returned for any open workflow execution that is using the specified task list. The task includes a paginated view of the history of the workflow execution. The decider should use the workflow type and the history to determine how to properly handle the task. This action initiates a long poll, wher
    ("PollForDecisionTask", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by activity workers to report to the service that the ActivityTask represented by the specified taskToken is still making progress. The worker can also specify details of the progress, for example percent complete, using the details parameter. This action can also be used by the worker as a mechanism to check if cancellation is being requested for the activity task. If a cancellation is being
    (
        "RecordActivityTaskHeartbeat",
        Decision::Denied(DenyReason::Content),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a new activity type along with its configuration settings in the specified domain. A TypeAlreadyExists fault is returned if the type already exists in the domain. You cannot change any configuration settings of the type after its registration, and it must be registered as a new version. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as fol
    (
        "RegisterActivityType",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a new domain. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: You cannot use an IAM policy to control domain access for this action. The name of the domain being registered is available as the resource of this action. Use an Action element to allow or deny permission to call this action. You cannot use an IAM policy to constrain
    ("RegisterDomain", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Registers a new workflow type and its configuration settings in the specified domain. The retention period for the workflow history is set by the RegisterDomain action. If the type already exists, then a TypeAlreadyExists fault is returned. You cannot change the configuration settings of a workflow type once it is registered and it must be registered as a new version. Access Control You can use IA
    (
        "RegisterWorkflowType",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Records a WorkflowExecutionCancelRequested event in the currently running workflow execution identified by the given domain, workflowId, and runId. This logically requests the cancellation of the workflow execution as a whole. It is up to the decider to take appropriate actions when it receives an execution history with this event. If the runId isn't specified, the WorkflowExecutionCancelRequested
    (
        "RequestCancelWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by workers to tell the service that the ActivityTask identified by the taskToken was successfully canceled. Additional details can be provided using the details argument. These details (if provided) appear in the ActivityTaskCanceled event added to the workflow history. Only use this operation if the canceled flag of a RecordActivityTaskHeartbeat request returns true and if the activity can b
    (
        "RespondActivityTaskCanceled",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by workers to tell the service that the ActivityTask identified by the taskToken completed successfully with a result (if provided). The result appears in the ActivityTaskCompleted event in the workflow history. If the requested task doesn't complete successfully, use RespondActivityTaskFailed instead. If the worker finds that the task is canceled through the canceled flag returned by RecordA
    (
        "RespondActivityTaskCompleted",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by workers to tell the service that the ActivityTask identified by the taskToken has failed with reason (if specified). The reason and details appear in the ActivityTaskFailed event added to the workflow history. A task is considered open from the time that it is scheduled until it is closed. Therefore a task is reported as open while a worker is processing it. A task is closed after it has b
    (
        "RespondActivityTaskFailed",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Used by deciders to tell the service that the DecisionTask identified by the taskToken has successfully completed. The decisions argument specifies the list of decisions made while processing the task. A DecisionTaskCompleted event is added to the workflow history. The executionContext specified is attached to the event in the workflow execution history. Access Control If an IAM policy grants perm
    (
        "RespondDecisionTaskCompleted",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Records a WorkflowExecutionSignaled event in the workflow execution history and creates a decision task for the workflow execution identified by the given domain, workflowId and runId. The event is recorded with the specified user defined signalName and input (if provided). If a runId isn't specified, then the WorkflowExecutionSignaled event is recorded in the history of the current open workflow
    (
        "SignalWorkflowExecution",
        Decision::Denied(DenyReason::Content),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Starts an execution of the workflow type in the specified domain using the provided workflowId and input data. This action returns the newly started workflow execution. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element with the domain name to limit the action to only specified domains. Use an Action element to allow o
    (
        "StartWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Add a tag to a Amazon SWF domain. Amazon SWF supports a maximum of 50 tags per resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Records a WorkflowExecutionTerminated event and forces closure of the workflow execution identified by the given domain, runId, and workflowId. The child policy, registered with the workflow type or specified when starting this execution, is applied to any open child workflow executions of this workflow execution. If the identified workflow execution was in progress, it is terminated immediately.
    (
        "TerminateWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Undeprecates a previously deprecated activity type . After an activity type has been undeprecated, you can create new tasks of that activity type. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource eleme
    (
        "UndeprecateActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Undeprecates a previously deprecated domain. After a domain has been undeprecated it can be used to create new workflow executions or register new types. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resourc
    ("UndeprecateDomain", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Undeprecates a previously deprecated workflow type . After a workflow type has been undeprecated, you can create new executions of that type. This operation is eventually consistent. The results are best effort and may not exactly reflect recent updates and changes. Access Control You can use IAM policies to control this action's access to Amazon SWF resources as follows: Use a Resource element wi
    (
        "UndeprecateWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Remove a tag from a Amazon SWF domain.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
