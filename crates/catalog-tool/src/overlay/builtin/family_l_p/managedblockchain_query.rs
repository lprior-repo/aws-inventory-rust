use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[Group {
    operations: &[
        "BatchGetTokenBalance",
        "GetAssetContract",
        "GetTokenBalance",
        "GetTransaction",
        "ListAssetContracts",
        "ListFilteredTransactionEvents",
        "ListTokenBalances",
        "ListTransactionEvents",
        "ListTransactions",
    ],
    class: OverlayClass::Deny(DenyReason::Paid),
    reason: None,
}];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("managedblockchain-query", GROUPS, &[])
}
