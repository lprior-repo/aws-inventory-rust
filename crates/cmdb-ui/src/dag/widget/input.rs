use makepad_widgets::event::DigitId;
use makepad_widgets::*;

use super::{DagSelectAction, DagView, DragState};
use crate::dag::view::{ViewTransform, f32_of};

const CLICK_SLOP: f64 = 4.0;

pub(super) fn handle_event(view: &mut DagView, cx: &mut Cx, event: &Event) {
    let area = view.draw_bg.area();
    match event.hits_with_capture_overload(cx, area, true) {
        Hit::FingerDown(fe) if fe.is_primary_hit() => press(view, cx, fe.abs, fe.digit_id),
        Hit::FingerMove(fe) => moved(view, cx, fe.abs),
        Hit::FingerUp(fe) => release(view, cx, fe.abs),
        Hit::FingerHoverIn(_) => cx.set_cursor(MouseCursor::Grab),
        Hit::FingerHoverOut(_) => leave(view, cx),
        Hit::FingerScroll(fs) => scroll(view, cx, event, fs.scroll, fs.abs, area),
        _ => {}
    }
}

fn press(view: &mut DagView, cx: &mut Cx, abs: DVec2, digit: DigitId) {
    view.drag = Some(DragState {
        start_abs: abs,
        start_offset: view.view.offset,
        moved: 0.0,
    });
    cx.claim_finger_drag(digit);
    cx.set_cursor(MouseCursor::Grabbing);
}

fn moved(view: &mut DagView, cx: &mut Cx, abs: DVec2) {
    update_hover(view, cx, abs);
    let Some(mut drag) = view.drag else {
        return;
    };
    let delta = dvec2(abs.x - drag.start_abs.x, abs.y - drag.start_abs.y);
    drag.moved = drag.moved.max(delta.x.hypot(delta.y));
    view.drag = Some(drag);
    view.view = ViewTransform {
        scale: view.view.scale,
        offset: drag.start_offset,
    }
    .panned(delta);
    view.redraw(cx);
}

fn release(view: &mut DagView, cx: &mut Cx, abs: DVec2) {
    if let Some(drag) = view.drag.take()
        && drag.moved <= CLICK_SLOP
    {
        select_node(view, cx, hit_node(view, abs));
    }
    cx.set_cursor(MouseCursor::Grab);
}

fn leave(view: &mut DagView, cx: &mut Cx) {
    if view.hover.take().is_some() {
        view.redraw(cx);
    }
}

fn scroll(view: &mut DagView, cx: &mut Cx, event: &Event, scroll: DVec2, abs: DVec2, area: Area) {
    let owned = matches!(event, Event::Scroll(se)
        if cx.is_scroll_owner(se, area, Vec2Index::Y, true)
            || cx.is_scroll_owner(se, area, Vec2Index::X, true));
    if !owned {
        return;
    }
    let delta = if scroll.y.abs() > f64::EPSILON {
        scroll.y
    } else {
        scroll.x
    };
    zoom_at(view, cx, delta, abs);
}

fn hit_node(view: &DagView, abs: DVec2) -> Option<usize> {
    let (x, y) = view.view.to_world(view.rect, abs);
    view.dag.hit_test(x, y)
}

fn update_hover(view: &mut DagView, cx: &mut Cx, abs: DVec2) {
    let node = hit_node(view, abs);
    if node != view.hover {
        view.hover = node;
        view.redraw(cx);
    }
}

fn select_node(view: &mut DagView, cx: &mut Cx, node: Option<usize>) {
    if view.selection == node {
        return;
    }
    view.selection = node;
    view.redraw(cx);
    emit_selection(view, cx);
}

fn emit_selection(view: &mut DagView, cx: &mut Cx) {
    let Some(index) = view.selection else {
        cx.widget_action(view.uid, DagSelectAction { id: None });
        return;
    };
    let is_resource = view
        .graph
        .node(index)
        .is_some_and(|node| node.kind().is_resource());
    if !is_resource {
        return;
    }
    let id = view.selected_id().map(str::to_owned);
    cx.widget_action(view.uid, DagSelectAction { id });
}

fn zoom_at(view: &mut DagView, cx: &mut Cx, scroll: f64, anchor: DVec2) {
    if scroll.abs() <= f64::EPSILON {
        return;
    }
    let steps = (scroll.abs() / 64.0).clamp(0.05, 1.0);
    let base = if scroll > 0.0 { 0.9_f64 } else { 1.0 / 0.9 };
    let factor = f32_of(base.powf(steps));
    view.view = view.view.zoomed(factor, anchor, view.rect);
    view.redraw(cx);
}
