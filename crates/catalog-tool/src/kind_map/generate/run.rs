use super::{Failure, Generated};
use crate::{ReportMode, classes::DenyList, kind_map, overlay, report, smithy::fields::publish};
use std::path::Path;

pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Failure> {
    let path = root.join("catalog/kinds.generated.tsv");
    let original = publish::read_bytes(&path, 2_097_152)?;
    let cfn = report::read_text(&root.join("catalog/cloudcontrol-universal.json"))?;
    let hand = kind_map::builtin::reviewed()?;
    let values = overlay::read_model_values(source)?;
    let mut policies = overlay::load(root)?;
    policies.retain(|policy| values.contains_key(policy.service()));
    let deny_text = overlay::deny_text(root)?;
    let checked = overlay::check_model_policies(&values, &policies, &DenyList::parse(&deny_text)?)?;
    let generated = super::generate(&values, &checked, &hand, cfn.as_bytes())?;
    kind_map::load_with_cfn(cfn.as_bytes(), generated.text.as_bytes())?;
    publish_output(root, original.as_deref(), &generated, mode)
}

fn publish_output(
    root: &Path,
    original: Option<&[u8]>,
    generated: &Generated,
    mode: ReportMode,
) -> Result<(), Failure> {
    let path = root.join("catalog/kinds.generated.tsv");
    match mode {
        ReportMode::Write => publish::publish_bytes(&path, original, generated.text.as_bytes(), 2_097_152)?,
        ReportMode::Check if original != Some(generated.text.as_bytes()) => {
            return Err("generated kinds differ; run catalog-tool kinds generate --write".into());
        }
        ReportMode::Print => report::write_stdout(&generated.text)?,
        _ => {}
    }
    report::publish(&root.join("catalog/aws-kinds-generated.report"), &generated.report()?, mode)?;
    Ok(())
}
