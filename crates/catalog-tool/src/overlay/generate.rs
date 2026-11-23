mod derive;
mod identity;
mod items;
mod pagination;
mod render;
mod service;

pub const GENERATED_RULES_DATE: &str = "2026-11-15";

use super::OperationOverlay;

pub(crate) type Derived = Result<OperationOverlay, String>;

#[cfg(test)]
mod tests;
