// Pinned operation purposes: .local/aws-models/models/savingsplans/service/2019-06-28/savingsplans-2019-06-28.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates a Savings Plan.
    ("CreateSavingsPlan", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the queued purchase for the specified Savings Plan.
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
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Describes the offering rates for Savings Plans you might want to purchase.
    (
        "DescribeSavingsPlansOfferingRates",
        Decision::Denied(DenyReason::ReferenceData),
    ),
    // Public/service reference catalog or supported-capability enumeration, not account-owned resource inventory. Pinned purpose: Describes the offerings for the specified Savings Plans.
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
    // Changes service state rather than reading inventory. Pinned purpose: Adds the specified tags to the specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes the specified tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
];
