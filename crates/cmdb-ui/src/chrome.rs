//! Projection of the table's state onto the title bar, filter rail and the
//! detail panel. No filtering happens here: `Data` already decided.

use makepad_widgets::*;

use crate::data::{ACCOUNTS, KINDS, REGIONS, Row};
use crate::table::ResourceTable;

struct ChromeState {
    summary: String,
    status: String,
    visible_len: usize,
    selected: Option<Row>,
    account: Option<&'static str>,
    region: Option<&'static str>,
    kind: Option<&'static str>,
}

fn read_state(ui: &WidgetRef, cx: &mut Cx) -> Option<ChromeState> {
    let table_ref = ui.widget(cx, ids!(table));
    let table = table_ref.borrow::<ResourceTable>()?;
    Some(ChromeState {
        summary: table.summary(),
        status: table.status_line(),
        visible_len: table.visible_len(),
        selected: table.selected_row(),
        account: table.account_filter(),
        region: table.region_filter(),
        kind: table.kind_filter(),
    })
}

pub(crate) fn sync_chrome(ui: &WidgetRef, cx: &mut Cx) {
    let Some(state) = read_state(ui, cx) else {
        return;
    };
    ui.label(cx, ids!(summary)).set_text(cx, &state.summary);
    ui.label(cx, ids!(table_status)).set_text(cx, &state.status);

    write_marks(ui, cx, state.account, state.region, state.kind);
    write_kind_counts(ui, cx);
    match state.selected {
        Some(row) => show_details(ui, cx, row),
        None => hide_details(ui, cx),
    }
    let visible_len = state.visible_len;
    script_eval!(cx, { mod.state.visible_count = #(visible_len) });
}

fn write_marks(
    ui: &WidgetRef,
    cx: &mut Cx,
    account: Option<&'static str>,
    region: Option<&'static str>,
    kind: Option<&'static str>,
) {
    let none = account.is_none() && region.is_none() && kind.is_none();
    ui.label(cx, ids!(all_mark)).set_visible(cx, none);
    write_account_marks(ui, cx, account);
    write_region_marks(ui, cx, region);
    write_kind_marks(ui, cx, kind);
}

fn write_account_marks(ui: &WidgetRef, cx: &mut Cx, account: Option<&'static str>) {
    let [prod, staging, sandbox] = ACCOUNTS;
    ui.label(cx, ids!(prod_mark))
        .set_visible(cx, account == Some(prod));
    ui.label(cx, ids!(staging_mark))
        .set_visible(cx, account == Some(staging));
    ui.label(cx, ids!(sandbox_mark))
        .set_visible(cx, account == Some(sandbox));
}

fn write_region_marks(ui: &WidgetRef, cx: &mut Cx, region: Option<&'static str>) {
    let [us_east, eu_west] = REGIONS;
    ui.label(cx, ids!(useast_mark))
        .set_visible(cx, region == Some(us_east));
    ui.label(cx, ids!(euwest_mark))
        .set_visible(cx, region == Some(eu_west));
}

fn write_kind_marks(ui: &WidgetRef, cx: &mut Cx, kind: Option<&'static str>) {
    let [ec2, s3, rds, lambda] = KINDS;
    ui.label(cx, ids!(ec2_mark))
        .set_visible(cx, kind == Some(ec2));
    ui.label(cx, ids!(s3_mark))
        .set_visible(cx, kind == Some(s3));
    ui.label(cx, ids!(rds_mark))
        .set_visible(cx, kind == Some(rds));
    ui.label(cx, ids!(lambda_mark))
        .set_visible(cx, kind == Some(lambda));
}

fn write_kind_counts(ui: &WidgetRef, cx: &mut Cx) {
    let table_ref = ui.widget(cx, ids!(table));
    let Some(table) = table_ref.borrow::<ResourceTable>() else {
        return;
    };
    let [ec2, s3, rds, lambda] = KINDS;
    ui.label(cx, ids!(ec2_count))
        .set_text(cx, &table.count(ec2).to_string());
    ui.label(cx, ids!(s3_count))
        .set_text(cx, &table.count(s3).to_string());
    ui.label(cx, ids!(rds_count))
        .set_text(cx, &table.count(rds).to_string());
    ui.label(cx, ids!(lambda_count))
        .set_text(cx, &table.count(lambda).to_string());
}

fn show_details(ui: &WidgetRef, cx: &mut Cx, row: Row) {
    ui.view(cx, ids!(detail_body)).set_visible(cx, true);
    ui.label(cx, ids!(detail_empty)).set_visible(cx, false);
    let detail = ui.widget(cx, ids!(detail_body));
    detail.label(cx, ids!(detail_name)).set_text(cx, row.name);
    detail
        .label(cx, ids!(detail_account))
        .set_text(cx, row.account);
    detail
        .label(cx, ids!(detail_region))
        .set_text(cx, row.region);
    detail.label(cx, ids!(detail_kind)).set_text(cx, row.kind);
    detail.label(cx, ids!(detail_id)).set_text(cx, row.id);
    detail.label(cx, ids!(detail_tags)).set_text(cx, row.tags());
    detail.label(cx, ids!(detail_size)).set_text(cx, row.size());
}

fn hide_details(ui: &WidgetRef, cx: &mut Cx) {
    ui.view(cx, ids!(detail_body)).set_visible(cx, false);
    ui.label(cx, ids!(detail_empty)).set_visible(cx, true);
}
