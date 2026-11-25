use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CancelTask", Decision::Denied(DenyReason::Mutating)),
    ("CreateTask", Decision::Denied(DenyReason::Mutating)),
    (
        "DescribeDevice",
        Decision::Deferred(
            "Required selectors managedDeviceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members lastReachedOutAt, lastUpdatedAt, tags, managedDeviceId, managedDeviceArn, deviceType, associatedWithJob, deviceState, physicalNetworkInterfaces, deviceCapacities, software. Pinned purpose: Checks device-specific information, such as the device type, software version, IP addresses, and lock status.",
        ),
    ),
    (
        "DescribeDeviceEc2Instances",
        Decision::Deferred(
            "Required selectors managedDeviceId, instanceIds are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members instances. Pinned purpose: Checks the current state of the Amazon EC2 instances. The output is similar to describeDevice , but the results are sourced from the device cache in the Amazon Web Services Cloud and include a subset of the available fields.",
        ),
    ),
    (
        "DescribeExecution",
        Decision::Deferred(
            "Required selectors taskId, managedDeviceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members taskId, executionId, managedDeviceId, state, startedAt, lastUpdatedAt. Pinned purpose: Checks the status of a remote task running on one or more target devices.",
        ),
    ),
    (
        "DescribeTask",
        Decision::Deferred(
            "Required selectors taskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members taskId, taskArn, targets, state, createdAt, lastUpdatedAt, completedAt, description, tags. Pinned purpose: Checks the metadata for a given task on a device.",
        ),
    ),
    (
        "ListDeviceResources",
        Decision::Deferred(
            "Required selectors managedDeviceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resources, nextToken. Pinned purpose: Returns a list of the Amazon Web Services resources available for a device. Currently, Amazon EC2 instances are the only supported resource type.",
        ),
    ),
    (
        "ListExecutions",
        Decision::Deferred(
            "Required selectors taskId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members executions, nextToken. Pinned purpose: Returns the status of tasks for one or more target devices.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Returns a list of tags for a managed device or task.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
