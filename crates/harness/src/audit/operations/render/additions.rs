//! Markdown section for reviewed additive operations.

use super::super::CoverageReport;

pub(super) fn render(out: &mut String, report: &CoverageReport) {
    if report.additions.is_empty() {
        return;
    }
    out.push_str(
        "## Reviewed additive operations (not exercised)\n\nThese reviewed additions are not \
part of the frozen historical operation matrix and are not counted as exercised. Their status \
records no emulator outcome; it is not evidence of runtime behavior.\n\n\
| Service | SDK method | API operation | Status | Reviewed callsites |\n\
| --- | --- | --- | --- | --- |\n",
    );
    for row in &report.additions {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} |\n",
            row.service,
            row.module,
            row.api_operation,
            row.status,
            callsites(row)
        ));
    }
    out.push('\n');
}

fn callsites(row: &super::super::AdditiveRow) -> String {
    let sites: Vec<String> = row
        .sites
        .iter()
        .map(|site| format!("`{}:{}`", site.path, site.function))
        .collect();
    sites.join(", ")
}
