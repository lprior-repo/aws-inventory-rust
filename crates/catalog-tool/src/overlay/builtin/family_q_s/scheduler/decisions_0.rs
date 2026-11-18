// Pinned operation purposes: .local/aws-models/models/scheduler/service/2021-06-30/scheduler-2021-06-30.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates the specified schedule.
    ("CreateSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates the specified schedule group.
    (
        "CreateScheduleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified schedule.
    ("DeleteSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the specified schedule group. Deleting a schedule group results in EventBridge Scheduler deleting all schedules associated with the group. When you delete a group, it remains in a DELETING state until all of its associated schedules are deleted. Schedules associated with the group that are set to run while the schedule group is in the process of being deleted might continue to invoke their
    (
        "DeleteScheduleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetSchedule",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, GroupName, Name, ScheduleExpression, StartDate, EndDate, Description, ScheduleExpressionTimezone, State, CreationDate, LastModificationDate, KmsKeyArn, Target, FlexibleTimeWindow, ActionAfterCompletion. Pinned purpose: Retrieves the specified schedule.",
        ),
    ),
    (
        "GetScheduleGroup",
        Decision::Deferred(
            "Required selectors Name are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Arn, Name, State, CreationDate, LastModificationDate. Pinned purpose: Retrieves the specified schedule group.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceArn. Pinned purpose: Lists the tags associated with the Scheduler resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags (key-value pairs) to the specified EventBridge Scheduler resource. You can only assign tags to schedule groups.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from the specified EventBridge Scheduler schedule group.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the specified schedule. When you call UpdateSchedule , EventBridge Scheduler uses all values, including empty values, specified in the request and overrides the existing schedule. This is by design. This means that if you do not set an optional field in your request, that field will be set to its system-default value after the update. Before calling this operation, we recommend that you ca
    ("UpdateSchedule", Decision::Denied(DenyReason::Mutating)),
];
