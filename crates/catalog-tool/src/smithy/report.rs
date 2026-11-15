use std::fmt::{self, Write};

use super::totals::Totals;

pub(super) fn render_report(totals: &Totals) -> Result<String, fmt::Error> {
    let mut text = String::new();
    fields(totals)
        .into_iter()
        .try_for_each(|(name, count)| writeln!(text, "{name} {count}"))?;
    totals
        .protocol_counts
        .iter()
        .try_for_each(|(id, count)| writeln!(text, "protocol {id} {count}"))?;
    Ok(text)
}

fn fields(totals: &Totals) -> [(&'static str, usize); 15] {
    let c = &totals.counts;
    [
        ("services", totals.services),
        ("operations", c.operations),
        ("read_named", c.read_named),
        ("readonly", c.readonly),
        ("list_describe", c.list_describe),
        ("list_describe_root", c.list_describe_root),
        ("services_with_root", totals.services_with_root),
        ("paginated", c.paginated),
        ("resources", c.resources),
        ("arn_templates", c.arn_templates),
        ("members", c.members),
        ("distinct_member_names", totals.distinct_member_names.len()),
        ("json_services", totals.json_services),
        ("xml_only_services", totals.xml_only_services),
        ("cbor_services", totals.cbor_services),
    ]
}
