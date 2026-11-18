mod part_0;
mod part_1;
use super::service;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [part_0::operations()?, part_1::operations()?]
        .into_iter()
        .flatten()
        .collect();
    let mut overlay = service("sfn", operations)?;
    overlay.reviewed = "2026-11-15".to_owned();
    overlay.reviewer = "automated Sol model review".to_owned();
    Ok(overlay)
}
