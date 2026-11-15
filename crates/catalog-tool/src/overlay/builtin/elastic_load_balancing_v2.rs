mod part_0;
mod part_1;
mod part_2;
use crate::overlay::{NameError, ServiceOverlay};
use super::service;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [part_0::operations()?, part_1::operations()?, part_2::operations()?].into_iter().flatten().collect();
    let mut overlay = service("elastic-load-balancing-v2", operations)?;
    overlay.reviewed = "2026-11-15".to_owned();
    overlay.reviewer = "Automated Sol model review".to_owned();
    Ok(overlay)
}
