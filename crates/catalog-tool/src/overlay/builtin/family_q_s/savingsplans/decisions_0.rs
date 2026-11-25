use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    ("CreateSavingsPlan", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteQueuedSavingsPlan",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DescribeSavingsPlanRates",
        Decision::Deferred(
            "Required selectors savingsPlanId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members savingsPlanId, searchResults, nextToken. Pinned purpose: Describes the rates for a specific, existing Savings Plan.",
        ),
    ),
    (
        "DescribeSavingsPlansOfferingRates",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "DescribeSavingsPlansOfferings",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists the tags for the specified resource.",
        ),
    ),
    (
        "ReturnSavingsPlan",
        Decision::Deferred(
            "Required selectors savingsPlanId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members savingsPlanId. Pinned purpose: Returns the specified Savings Plan.",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
