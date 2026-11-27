use super::super::{name, path};
use super::policy::{Group, Id, Resource, compile};
use crate::classes::DenyReason;
use crate::overlay::{ExecutionBranch, FixedValue, NameError, OverlayClass, Scope, ServiceOverlay};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptAgreementCancellationRequest",
            "AcceptAgreementPaymentRequest",
            "AcceptAgreementRequest",
            "BatchCreateBillingAdjustmentRequest",
            "CancelAgreement",
            "CancelAgreementCancellationRequest",
            "CancelAgreementPaymentRequest",
            "CreateAgreementRequest",
            "RejectAgreementCancellationRequest",
            "RejectAgreementPaymentRequest",
            "SendAgreementCancellationRequest",
            "SendAgreementPaymentRequest",
            "UpdatePurchaseOrders",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeAgreement",
            "GetAgreementEntitlements",
            "GetAgreementTerms",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAgreementCancellationRequest"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (agreementCancellationRequestId, agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAgreementPaymentRequest"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (paymentRequestId, agreementId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetBillingAdjustmentRequest"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (agreementId, billingAdjustmentRequestId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListAgreementCancellationRequests",
            "ListAgreementPaymentRequests",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (partyType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListAgreementCharges", "ListBillingAdjustmentRequests"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["ListAgreementInvoiceLineItems"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (agreementId, groupBy); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[Resource {
    operation: "SearchAgreements",
    items: "agreementViewSummaries",
    id: Id::Key(&["agreementId"]),
    scope: Scope::Account,
    tokens: Some(("nextToken", "nextToken")),
    query: &[
        "agreementId",
        "agreementType",
        "status",
        "initialAgreementId",
        "endTimeBehaviorType",
    ],
    tags: None,
    bindings: &[],
    fixed: &[],
}];

fn filter(key: &str, value: &str) -> Result<FixedValue, NameError> {
    FixedValue::structure([
        ("name", FixedValue::String(key.to_owned())),
        ("values", FixedValue::Strings(vec![value.to_owned()])),
    ])
}

fn party_branch(party: &str) -> Result<ExecutionBranch, NameError> {
    let filters = FixedValue::list([
        filter("AgreementType", "PurchaseAgreement")?,
        filter("PartyType", party)?,
    ]);
    Ok(ExecutionBranch {
        discriminator: party.to_ascii_lowercase(),
        fixed: [(path("filters")?, filters)].into_iter().collect(),
        bind: Default::default(),
        emission: None,
        row_policy: None,
        scope: Scope::Account,
        request_region: None,
    })
}

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut overlay = compile("marketplace-agreement", GROUPS, RESOURCES)?;
    let operation = overlay
        .operations
        .get_mut(&name("SearchAgreements")?)
        .ok_or_else(|| NameError {
            value: "SearchAgreements".to_owned(),
        })?;
    operation.branches = vec![party_branch("Proposer")?, party_branch("Acceptor")?];
    Ok(overlay)
}
