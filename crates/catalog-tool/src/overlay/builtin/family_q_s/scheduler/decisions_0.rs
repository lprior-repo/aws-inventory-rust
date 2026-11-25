use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateSchedule", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateScheduleGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteSchedule", Decision::Denied(DenyReason::Mutating)),
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
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateSchedule", Decision::Denied(DenyReason::Mutating)),
];
