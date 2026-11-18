mod part_0;
use super::service;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [part_0::operations()?].into_iter().flatten().collect();
    let mut overlay = service("sts", operations)?;
    overlay.reviewed = "2026-11-14".to_owned();
    overlay.reviewer = "Automated model review; no human approval".to_owned();
    Ok(overlay)
}
