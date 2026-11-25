use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CountClosedWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "CountOpenWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "CountPendingActivityTasks",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "CountPendingDecisionTasks",
        Decision::Denied(DenyReason::Content),
    ),
    ("DeleteActivityType", Decision::Denied(DenyReason::Mutating)),
    ("DeleteWorkflowType", Decision::Denied(DenyReason::Mutating)),
    (
        "DeprecateActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    ("DeprecateDomain", Decision::Denied(DenyReason::Content)),
    (
        "DeprecateWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "DescribeActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    ("DescribeDomain", Decision::Denied(DenyReason::Content)),
    (
        "DescribeWorkflowExecution",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "DescribeWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetWorkflowExecutionHistory",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "ListClosedWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "ListOpenWorkflowExecutions",
        Decision::Denied(DenyReason::Content),
    ),
    ("ListTagsForResource", Decision::Denied(DenyReason::Content)),
    ("PollForActivityTask", Decision::Denied(DenyReason::Content)),
    ("PollForDecisionTask", Decision::Denied(DenyReason::Content)),
    (
        "RecordActivityTaskHeartbeat",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "RegisterActivityType",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("RegisterDomain", Decision::Denied(DenyReason::Mutating)),
    (
        "RegisterWorkflowType",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RequestCancelWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "RespondActivityTaskCanceled",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "RespondActivityTaskCompleted",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "RespondActivityTaskFailed",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "RespondDecisionTaskCompleted",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "SignalWorkflowExecution",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "StartWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "TerminateWorkflowExecution",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UndeprecateActivityType",
        Decision::Denied(DenyReason::Content),
    ),
    ("UndeprecateDomain", Decision::Denied(DenyReason::Content)),
    (
        "UndeprecateWorkflowType",
        Decision::Denied(DenyReason::Content),
    ),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
