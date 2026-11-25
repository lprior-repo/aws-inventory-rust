use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateEndpoint", Decision::Denied(DenyReason::Mutating)),
    ("DeleteEndpoint", Decision::Denied(DenyReason::Mutating)),
    (
        "ListSharedEndpoints",
        Decision::Deferred(
            "Required selectors OutpostId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Endpoints, NextToken. Pinned purpose: Lists all endpoints associated with an Outpost that has been shared by Amazon Web Services Resource Access Manager (RAM). Related actions include: CreateEndpoint DeleteEndpoint",
        ),
    ),
];
