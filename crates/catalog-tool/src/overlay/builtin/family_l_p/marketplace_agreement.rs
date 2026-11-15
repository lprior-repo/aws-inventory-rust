// Pinned Smithy source: .local/aws-models/models/marketplace-agreement/service/2020-03-01/marketplace-agreement-2020-03-01.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptAgreementCancellationRequest", "AcceptAgreementPaymentRequest", 
            "AcceptAgreementRequest", "BatchCreateBillingAdjustmentRequest", "CancelAgreement", 
            "CancelAgreementCancellationRequest", "CancelAgreementPaymentRequest", 
            "CreateAgreementRequest", "RejectAgreementCancellationRequest", 
            "RejectAgreementPaymentRequest", "SendAgreementCancellationRequest", 
            "SendAgreementPaymentRequest", "UpdatePurchaseOrders",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeAgreement", "GetAgreementEntitlements", "GetAgreementTerms",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetAgreementCancellationRequest",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (agreementCancellationRequestId, agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetAgreementPaymentRequest",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (paymentRequestId, agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetBillingAdjustmentRequest",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (agreementId, billingAdjustmentRequestId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAgreementCancellationRequests", "ListAgreementPaymentRequests",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (partyType); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListAgreementCharges", "ListBillingAdjustmentRequests",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "ListAgreementInvoiceLineItems",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (agreementId, groupBy); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Searches across all agreements that a proposer or an acceptor has in AWS Marketplace. The search returns a list of agreements with basic agreement information.
    Resource {
        operation: "SearchAgreements",
        items: "agreementViewSummaries", id: Id::Key(&["agreementId"]),
        scope: Scope::Account, tokens: Some(("nextToken", "nextToken")),
        query: &["agreementId", "agreementType", "status", "initialAgreementId", "endTimeBehaviorType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("marketplace-agreement", GROUPS, RESOURCES)
}
