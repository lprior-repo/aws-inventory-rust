use std::path::Path;

use super::{Error, build_report};
use crate::{ReportMode, report::publish};

const REPORT_PATH: &str = "catalog/aws-endpoints.report";

pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let text = build_report(source)?;
    publish(&root.join(REPORT_PATH), &text, mode)?;
    Ok(())
}
