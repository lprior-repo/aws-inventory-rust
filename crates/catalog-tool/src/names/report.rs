use crate::{ReportMode, kind_map::{KindMap, KindTarget}, report};
use std::path::Path;

pub(crate) fn run(
    root: &Path,
    kinds: &KindMap,
    mode: ReportMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut types: Vec<_> = kinds.entries().iter()
        .filter(|entry| matches!(entry.target, KindTarget::Mapped { .. }))
        .map(|entry| entry.cfn_type.as_str())
        .chain(kinds.api_only().iter().map(|entry| entry.kind.as_str()))
        .collect();
    types.sort_unstable();
    let text = super::render::render_report(&types, &super::Exceptions::builtin())?;
    report::publish(&root.join("catalog/aws-names.report"), &text, mode)?;
    Ok(())
}
