use super::*;

pub(super) fn draw_empty(cx: &mut Cx2d, list: &mut PortalList, status: &str) {
    list.set_item_range(cx, 0, 1);
    while let Some(item_id) = list.next_visible_item(cx) {
        let item = list.item(cx, item_id, id!(Empty));
        item.label(cx, ids!(empty_text)).set_text(cx, status);
        item.draw_all_unscoped(cx);
    }
}

impl Widget for ResourceTable {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = step.as_portal_list().borrow_mut() {
                self.draw_list(cx, &mut list);
            }
        }
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}
