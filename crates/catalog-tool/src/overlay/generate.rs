mod engine;
mod output;
mod publish;
pub(in crate::overlay) mod selection;
mod derive;
mod identity;
mod items;
mod pagination;
mod render;
mod service;

pub const GENERATED_RULES_DATE: &str = "2026-11-15";

pub use engine::{Generated, Generator};

use crate::{ReportMode, classes::DenyList};
use std::{collections::BTreeSet, path::Path};

type Failure = Box<dyn std::error::Error>;

pub fn run(
    root: &Path,
    source: &Path,
    services: &BTreeSet<String>,
    mode: ReportMode,
) -> Result<(), Failure> {
    let snapshot = publish::Snapshot::read(root)?;
    let generator = Generator::read(source)?;
    let selected = selection::services(root, services, generator.values())?;
    let text = selection::deny_text(root)?;
    let deny = DenyList::parse(&text)?;
    let generated = generator.generate(&selected, &deny)?;
    snapshot.publish(root, &generated, mode)
}

use super::OperationOverlay;

pub(crate) type Derived = Result<OperationOverlay, String>;

#[cfg(test)]
mod tests;
