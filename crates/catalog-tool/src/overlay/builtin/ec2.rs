mod part_0;
mod part_1;
mod part_10;
mod part_11;
mod part_12;
mod part_13;
mod part_14;
mod part_2;
mod part_3;
mod part_4;
mod part_5;
mod part_6;
mod part_7;
mod part_8;
mod part_9;
use super::service;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let operations = [
        part_0::operations()?,
        part_1::operations()?,
        part_2::operations()?,
        part_3::operations()?,
        part_4::operations()?,
        part_5::operations()?,
        part_6::operations()?,
        part_7::operations()?,
        part_8::operations()?,
        part_9::operations()?,
        part_10::operations()?,
        part_11::operations()?,
        part_12::operations()?,
        part_13::operations()?,
        part_14::operations()?,
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut overlay = service("ec2", operations)?;
    overlay.reviewed = "2026-11-15".to_owned();
    overlay.reviewer = "Automated Sol (openai-codex/gpt-6.1-sol) model review; no human approval; not protocol/emulator verified".to_owned();
    Ok(overlay)
}
