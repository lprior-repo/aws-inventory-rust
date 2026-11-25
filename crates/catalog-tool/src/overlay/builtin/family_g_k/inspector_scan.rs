use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "inspector-scan",
        approved,
        DEFERRED,
        "ScanSbom performs billed vulnerability analysis on supplied software content; it does not enumerate inventory. No callable policy is approved.",
        &[(policy::DenyReason::Paid, PAID)],
    )
}

const DEFERRED: &[&str] = &[];

const PAID: &[&str] = &["ScanSbom"];
