use makepad_widgets::*;

use crate::data::{Facet, Row};
use crate::table::ResourceTable;

struct ChromeState {
    summary: String,
    status: String,
    visible_len: usize,
    filters_clear: bool,
    selected: Option<Row>,
    account: Option<String>,
    region: Option<String>,
    kind: Option<String>,
    accounts: Vec<String>,
    regions: Vec<String>,
    kinds: Vec<String>,
}

fn read_state(ui: &WidgetRef, cx: &mut Cx) -> Option<ChromeState> {
    let table_widget = ui.widget(cx, ids!(table));
    let table = table_widget.borrow::<ResourceTable>()?;
    let filter = table.filter();
    Some(ChromeState {
        summary: table.summary(),
        status: table.status_line(),
        visible_len: table.visible_len(),
        filters_clear: filter.account.is_none()
            && filter.region.is_none()
            && filter.kind.is_none()
            && filter.query.is_empty()
            && filter.tag.is_none(),
        selected: table.selected_row(),
        account: table.account_filter().map(str::to_owned),
        region: table.region_filter().map(str::to_owned),
        kind: table.kind_filter().map(str::to_owned),
        accounts: table.options(Facet::Account),
        regions: table.options(Facet::Region),
        kinds: table.options(Facet::Kind),
    })
}

pub(crate) fn sync_chrome(ui: &WidgetRef, cx: &mut Cx) {
    let Some(state) = read_state(ui, cx) else {
        return;
    };
    write_summary(ui, cx, &state);
    write_account_chips(ui, cx, &state);
    write_region_chips(ui, cx, &state);
    write_kind_chips(ui, cx, &state);
    write_details(ui, cx, state.selected);
    script_eval!(cx, { mod.state.visible_count = #(state.visible_len) });
}

fn write_summary(ui: &WidgetRef, cx: &mut Cx, state: &ChromeState) {
    ui.label(cx, ids!(summary)).set_text(cx, &state.summary);
    ui.label(cx, ids!(table_status)).set_text(cx, &state.status);
    ui.label(cx, ids!(all_mark))
        .set_visible(cx, state.filters_clear);
    [
        ids!(ec2_count),
        ids!(s3_count),
        ids!(rds_count),
        ids!(lambda_count),
    ]
    .iter()
    .for_each(|id| ui.label(cx, *id).set_visible(cx, false));
}

fn write_account_chips(ui: &WidgetRef, cx: &mut Cx, state: &ChromeState) {
    let active = state.account.as_deref();
    let chips = [
        (ids!(prod_button), ids!(prod_mark), state.accounts.first()),
        (
            ids!(staging_button),
            ids!(staging_mark),
            state.accounts.get(1),
        ),
        (
            ids!(sandbox_button),
            ids!(sandbox_mark),
            state.accounts.get(2),
        ),
    ];
    for (button, mark, value) in chips {
        write_chip(
            ui.widget(cx, button),
            ui.widget(cx, mark),
            cx,
            value,
            active,
        );
    }
}

fn write_region_chips(ui: &WidgetRef, cx: &mut Cx, state: &ChromeState) {
    let button = ui.widget(cx, ids!(useast_button));
    let mark = ui.widget(cx, ids!(useast_mark));
    write_chip(
        button,
        mark,
        cx,
        state.regions.first(),
        state.region.as_deref(),
    );
    let button = ui.widget(cx, ids!(euwest_button));
    let mark = ui.widget(cx, ids!(euwest_mark));
    write_chip(
        button,
        mark,
        cx,
        state.regions.get(1),
        state.region.as_deref(),
    );
}

fn write_kind_chips(ui: &WidgetRef, cx: &mut Cx, state: &ChromeState) {
    let button = ui.widget(cx, ids!(ec2_button));
    let mark = ui.widget(cx, ids!(ec2_mark));
    write_chip(button, mark, cx, state.kinds.first(), state.kind.as_deref());
    let button = ui.widget(cx, ids!(s3_button));
    let mark = ui.widget(cx, ids!(s3_mark));
    write_chip(button, mark, cx, state.kinds.get(1), state.kind.as_deref());
    let button = ui.widget(cx, ids!(rds_button));
    let mark = ui.widget(cx, ids!(rds_mark));
    write_chip(button, mark, cx, state.kinds.get(2), state.kind.as_deref());
    let button = ui.widget(cx, ids!(lambda_button));
    let mark = ui.widget(cx, ids!(lambda_mark));
    write_chip(button, mark, cx, state.kinds.get(3), state.kind.as_deref());
}

fn write_chip(
    button: WidgetRef,
    mark: WidgetRef,
    cx: &mut Cx,
    value: Option<&String>,
    active: Option<&str>,
) {
    match value {
        Some(value) => {
            button.set_text(cx, value);
            button.set_visible(cx, true);
            mark.set_visible(cx, active == Some(value));
        }
        None => {
            button.set_visible(cx, false);
            mark.set_visible(cx, false);
        }
    }
}

fn write_details(ui: &WidgetRef, cx: &mut Cx, selected: Option<Row>) {
    match selected {
        Some(row) => show_details(ui, cx, row),
        None => hide_details(ui, cx),
    }
}

fn show_details(ui: &WidgetRef, cx: &mut Cx, row: Row) {
    ui.view(cx, ids!(detail_body)).set_visible(cx, true);
    ui.label(cx, ids!(detail_empty)).set_visible(cx, false);
    let detail = ui.widget(cx, ids!(detail_body));
    detail.label(cx, ids!(detail_name)).set_text(cx, &row.name);
    detail
        .label(cx, ids!(detail_account))
        .set_text(cx, &row.account);
    detail
        .label(cx, ids!(detail_region))
        .set_text(cx, &row.region);
    detail.label(cx, ids!(detail_kind)).set_text(cx, &row.kind);
    detail.label(cx, ids!(detail_id)).set_text(cx, &row.id);
    let record = row.record_bytes.map_or_else(
        || "Graph neighbor · record not loaded".to_owned(),
        |bytes| format!("{bytes} bytes"),
    );
    let hash = row.content_hash.as_deref().map_or_else(
        || "not loaded".to_owned(),
        |value| format!("BLAKE3 {value}"),
    );
    detail.label(cx, ids!(detail_record)).set_text(cx, &record);
    detail.label(cx, ids!(detail_hash)).set_text(cx, &hash);
    ui.label(cx, ids!(detail_tags))
        .set_text(cx, &tag_summary(&row));
}

fn hide_details(ui: &WidgetRef, cx: &mut Cx) {
    ui.view(cx, ids!(detail_body)).set_visible(cx, false);
    ui.label(cx, ids!(detail_empty)).set_visible(cx, true);
    ui.label(cx, ids!(detail_tags)).set_text(cx, "");
}

fn tag_summary(row: &Row) -> String {
    use std::fmt::Write;

    if row.record_bytes.is_none() {
        return "Tags not loaded from SQLite".to_owned();
    }
    let mut text = tag_observation_summary(row);
    if row.tags.is_empty() && !row.tags_truncated {
        text.push_str("\nNo known persisted tag associations");
        return text;
    }
    text.push('\n');
    let result = row
        .tags
        .iter()
        .take(8)
        .try_for_each(|tag| writeln!(&mut text, "{:?} = {:?}", tag.key, tag.value));
    if let Err(error) = result {
        return format!("Tag display unavailable · {error}");
    }
    let remaining = row.tags.len().saturating_sub(8);
    if remaining > 0 {
        text.push_str(&format!("… {remaining} more tags\n"));
    }
    if row.tags_truncated {
        text.push_str("Tag list truncated by SQLite read limit\n");
    }
    text
}

fn tag_observation_summary(row: &Row) -> String {
    let Some(observation) = row.tag_observation else {
        return "Observation completeness unknown".to_owned();
    };
    let coverage = match observation.coverage {
        cmdb_view::TagCoverage::Complete => "Complete tag observation",
        cmdb_view::TagCoverage::Partial => "Partial observation · retained known tags",
        cmdb_view::TagCoverage::Failed => "Failed observation · retained known tags",
        cmdb_view::TagCoverage::Unobserved => "Not observed · retained known tags",
    };
    let mut text = format!("{coverage} · at {} ms UTC", observation.observed_at_ms);
    if observation.coverage != cmdb_view::TagCoverage::Complete
        && let Some(last_complete) = observation.last_complete_at_ms
    {
        text.push_str(&format!("\nLast complete at {last_complete} ms UTC"));
    }
    text
}
