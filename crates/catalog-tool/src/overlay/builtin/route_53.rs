mod part_0;
mod part_1;
mod part_2;
mod part_3;
use super::service;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [
        part_0::operations()?,
        part_1::operations()?,
        part_2::operations()?,
        part_3::operations()?,
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut overlay = service("route-53", operations)?;
    overlay.reviewed = "2026-11-15".to_owned();
    overlay.reviewer = "Automated Sol model review; no human approval".to_owned();
    Ok(overlay)
}
