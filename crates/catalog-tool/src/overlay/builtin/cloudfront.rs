mod part_0;
mod part_1;
mod part_2;
mod part_3;
mod part_4;
use crate::overlay::{NameError, ServiceOverlay};
use super::service;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [part_0::operations()?, part_1::operations()?, part_2::operations()?, part_3::operations()?, part_4::operations()?].into_iter().flatten().collect();
    let mut overlay = service("cloudfront", operations)?;
    overlay.reviewed = "2026-11-15".to_owned();
    overlay.reviewer = "Automated Sol model review; no human approval".to_owned();
    Ok(overlay)
}
