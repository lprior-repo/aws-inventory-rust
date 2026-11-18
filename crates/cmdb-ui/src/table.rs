mod state;
mod widget;

use state::{
    apply_filter_enqueue_result, apply_graph_data, apply_selection_enqueue_result,
    apply_snapshot_data,
};
use widget::draw_empty;

use makepad_widgets::*;

use crate::data::{Data, Filter, Row};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.cmdb_col_account = 96
    mod.widgets.cmdb_col_region = 112
    mod.widgets.cmdb_col_kind = 96

    let TableRow = View{
        width: Fill height: 37
        flow: Down spacing: 0
        row_head := RoundedView{
            width: Fill height: 36
            flow: Right spacing: 0
            align: Align{y: 0.5}
            draw_bg.color: theme.color_u_hidden
            sel_mark := SolidView{width: 3 height: 36 draw_bg.color: theme.color_focus visible: false}
            hover_mark := SolidView{width: 2 height: 36 draw_bg.color: theme.color_u_3 visible: false}
            View{
                width: Fill height: Fill
                flow: Right spacing: 0
                align: Align{y: 0.5}
                padding: Inset{left: 19 right: 24 top: 0 bottom: 0}
                account := Label{width: mod.widgets.cmdb_col_account height: Fit max_lines: 1 text_overflow: Ellipsis text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_body_s}
                region := Label{width: mod.widgets.cmdb_col_region height: Fit max_lines: 1 text_overflow: Ellipsis text: "" draw_text.color: theme.color_text_meta draw_text.text_style: theme.font_body_s}
                kind := Label{width: mod.widgets.cmdb_col_kind height: Fit max_lines: 1 text_overflow: Ellipsis text: "" draw_text.color: theme.color_info draw_text.text_style: theme.font_body_s}
                View{
                    width: Fill height: Fit
                    padding: Inset{right: 12}
                    res_id := Label{width: Fill height: Fit max_lines: 1 text_overflow: Ellipsis text: "" draw_text.color: theme.color_text draw_text.text_style: theme.font_code}
                }
                name := Label{width: Fill height: Fit max_lines: 1 text_overflow: Ellipsis text: "" draw_text.color: theme.color_text_hl draw_text.text_style: theme.font_body_s}
            }
        }
        SolidView{width: Fill height: 1 draw_bg.color: theme.color_u_1}
    }

    let TableEmpty = View{
        width: Fill height: 96
        align: Center
            empty_text := Label{
                text: "Loading inventory…"
                draw_text.color: theme.color_text_meta
                draw_text.text_style: theme.font_body_s
            }
    }

    mod.widgets.ResourceTableBase = #(ResourceTable::register_widget(vm))
    mod.widgets.ResourceTable = set_type_default() do mod.widgets.ResourceTableBase{
        width: Fill height: Fill
        list := PortalList{
            width: Fill height: Fill
            spacing: 0
            scroll_bar: ScrollBar{}
            Row := CachedView{TableRow{}}
            Empty := CachedView{TableEmpty{}}
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct ResourceTable {
    #[deref]
    view: View,
    #[rust]
    data: Data,

    #[rust]
    pressed: Option<usize>,

    #[rust]
    hover: Option<usize>,
}

impl ResourceTable {
    pub fn apply_filter_result(
        &mut self,
        cx: &mut Cx,
        filter: Filter,
        result: Result<(), String>,
    ) -> bool {
        let accepted = apply_filter_enqueue_result(&mut self.data, filter, result);
        self.view.redraw(cx);
        accepted
    }

    pub fn clicked_row(&mut self, cx: &mut Cx, actions: &Actions) -> Option<Row> {
        self.track_hover(cx, actions);
        let index = self.click_index(cx, actions)?;
        let row = self.data.visible_row(index)?;
        (!self
            .data
            .selected_row()
            .is_some_and(|selected| selected.key == row.key))
        .then(|| row.clone())
    }

    fn click_index(&mut self, cx: &mut Cx, actions: &Actions) -> Option<usize> {
        let list = self.view.portal_list(cx, ids!(list));
        let mut clicked = None;
        for (index, item) in list.items_with_actions(actions) {
            let head = item.view(cx, ids!(row_head));
            if let Some(down) = head.finger_down(actions)
                && down.device.is_primary_hit()
            {
                self.pressed = Some(index);
            }
            if let Some(up) = head.finger_up(actions) {
                let pressed = self.pressed.take();
                if up.is_over && up.device.is_primary_hit() && pressed == Some(index) {
                    clicked = Some(index);
                }
            }
        }
        clicked
    }

    fn track_hover(&mut self, cx: &mut Cx, actions: &Actions) {
        let list = self.view.portal_list(cx, ids!(list));
        let mut changed = false;
        for (index, item) in list.items_with_actions(actions) {
            let head = item.view(cx, ids!(row_head));
            if head.finger_hover_in(actions).is_some() && self.hover != Some(index) {
                self.hover = Some(index);
                changed = true;
            }
            if head.finger_hover_out(actions).is_some() && self.hover == Some(index) {
                self.hover = None;
                changed = true;
            }
        }
        if changed {
            self.view.redraw(cx);
        }
    }

    pub fn filter(&self) -> crate::data::Filter {
        self.data.filter().clone()
    }
    pub fn summary(&self) -> String {
        self.data.summary()
    }

    pub fn status_line(&self) -> String {
        self.data.status_line()
    }

    pub fn graph_status(&self) -> String {
        self.data.graph_status()
    }

    pub fn visible_len(&self) -> usize {
        self.data.visible_len()
    }

    pub fn selected_row(&self) -> Option<Row> {
        self.data.selected_row().cloned()
    }

    pub fn account_filter(&self) -> Option<&str> {
        self.data.account_filter()
    }

    pub fn region_filter(&self) -> Option<&str> {
        self.data.region_filter()
    }

    pub fn kind_filter(&self) -> Option<&str> {
        self.data.kind_filter()
    }

    pub fn options(&self, facet: crate::data::Facet) -> Vec<String> {
        self.data.options(facet)
    }

    pub fn graph(&self) -> crate::data::Graph {
        self.data.visual_graph()
    }

    pub fn apply_snapshot(
        &mut self,
        cx: &mut Cx,
        counts: crate::data::Counts,
        rows: Vec<Row>,
        total_matching: u64,
        error: Option<String>,
    ) {
        apply_snapshot_data(&mut self.data, counts, rows, total_matching, error);
        self.pressed = None;
        self.hover = None;
        self.view.redraw(cx);
    }

    pub fn apply_graph(
        &mut self,
        cx: &mut Cx,
        graph: crate::data::Graph,
        error: Option<String>,
    ) -> bool {
        let accepted = apply_graph_data(&mut self.data, graph, error);
        if accepted {
            self.view.redraw(cx);
        }
        accepted
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.data.set_loading(loading);
    }

    pub fn row_for_id(&self, id: &str) -> Option<Row> {
        self.data.row_for_key(id).cloned()
    }

    pub fn apply_selection_result(
        &mut self,
        cx: &mut Cx,
        row: Row,
        result: Result<(), String>,
    ) -> bool {
        let changed = apply_selection_enqueue_result(&mut self.data, row, result);
        self.view.redraw(cx);
        changed
    }

    pub fn graph_request_failed(&mut self, cx: &mut Cx, error: String) {
        self.data.set_graph_error(Some(error));
        self.view.redraw(cx);
    }

    pub fn clear_selection(&mut self, cx: &mut Cx) -> bool {
        if !self.data.clear_selection() {
            return false;
        }
        self.view.redraw(cx);
        true
    }

    fn draw_list(&mut self, cx: &mut Cx2d, list: &mut PortalList) {
        if self.data.visible_len() == 0 {
            draw_empty(cx, list, &self.data.status_line());
        } else {
            self.draw_rows(cx, list);
        }
    }

    fn draw_rows(&self, cx: &mut Cx2d, list: &mut PortalList) {
        list.set_item_range(cx, 0, self.data.visible_len());
        while let Some(item_id) = list.next_visible_item(cx) {
            let Some(row) = self.data.visible_row(item_id) else {
                continue;
            };
            let item = list.item(cx, item_id, id!(Row));
            self.fill_row(cx, &item, item_id, row);
            item.draw_all_unscoped(cx);
        }
    }

    fn fill_row(&self, cx: &mut Cx2d, item: &WidgetRef, index: usize, row: &Row) {
        item.label(cx, ids!(account)).set_text(cx, &row.account);
        item.label(cx, ids!(region)).set_text(cx, &row.region);
        item.label(cx, ids!(kind)).set_text(cx, &row.kind);
        item.label(cx, ids!(res_id)).set_text(cx, &row.id);
        item.label(cx, ids!(name)).set_text(cx, &row.name);

        let selected = self.data.selected_row().is_some_and(|it| it.key == row.key);
        let hovered = self.hover == Some(index) && !selected;
        item.view(cx, ids!(sel_mark)).set_visible(cx, selected);
        item.view(cx, ids!(hover_mark)).set_visible(cx, hovered);
        self.tint_row(cx, item, selected, hovered);
    }

    fn tint_row(&self, cx: &mut Cx2d, item: &WidgetRef, selected: bool, hovered: bool) {
        let mut head = item.view(cx, ids!(row_head));
        if selected {
            script_apply_eval!(cx, head, { draw_bg +: { color: mod.theme.color_u_2 } });
        } else if hovered {
            script_apply_eval!(cx, head, { draw_bg +: { color: mod.theme.color_u_1 } });
        } else {
            script_apply_eval!(cx, head, { draw_bg +: { color: mod.theme.color_u_hidden } });
        }
    }
}
