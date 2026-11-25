use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateBillOfMaterialsImportJob",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDataIntegrationFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDataLakeDataset",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreateDataLakeNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteDataIntegrationFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteDataLakeDataset",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DeleteDataLakeNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "GetBillOfMaterialsImportJob",
        Decision::Deferred(
            "Required selectors instanceId, jobId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members job. Pinned purpose: Get status and details of a BillOfMaterialsImportJob.",
        ),
    ),
    (
        "GetDataIntegrationEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetDataIntegrationFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetDataIntegrationFlowExecution",
        Decision::Deferred(
            "Required selectors instanceId, flowName, executionId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowExecution. Pinned purpose: Get the flow execution.",
        ),
    ),
    ("GetDataLakeDataset", Decision::Denied(DenyReason::Mutating)),
    (
        "GetDataLakeNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("GetInstance", Decision::Denied(DenyReason::Mutating)),
    (
        "ListDataIntegrationEvents",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListDataIntegrationFlowExecutions",
        Decision::Deferred(
            "Required selectors instanceId, flowName are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members flowExecutions, nextToken. Pinned purpose: List flow executions.",
        ),
    ),
    (
        "ListDataIntegrationFlows",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: List all the tags for an Amazon Web ServicesSupply Chain resource. You can list all the tags added to a resource. By listing the tags, developers can view the tag level information on a resource and perform actions such as, deleting a resource associated with a particular tag.",
        ),
    ),
    (
        "SendDataIntegrationEvent",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateDataIntegrationFlow",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDataLakeDataset",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateDataLakeNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateInstance", Decision::Denied(DenyReason::Mutating)),
];
