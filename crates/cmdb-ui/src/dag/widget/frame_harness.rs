use super::{DagView, input};
use crate::dag::layout::LayoutRect;
use crate::dag::view::{FIT_MARGIN, LABEL_MIN_SCALE, MAX_SCALE, ViewTransform, f32_of};
use crate::frame_harness::{path::PathState, report::Scene};
use makepad_widgets::{Cx, dvec2};

impl DagView {
    pub fn harness_path(&mut self, cx: &mut Cx, state: PathState) {
        let fit = self.harness_fit_scale();
        let pan_zoom = (state.pan * 5.0).min(1.0);
        let scale = fit + (f64::from(MAX_SCALE) - fit) * state.zoom + (fit.max(1.0) - fit) * pan_zoom;
        let (x, y) = self.harness_center(state, pan_zoom);
        self.view = ViewTransform {
            scale: f32_of(scale),
            offset: dvec2(self.rect.size.x * 0.5 - x * scale, self.rect.size.y * 0.5 - y * scale),
        };
        self.redraw(cx);
    }

    fn harness_fit_scale(&self) -> f64 {
        let margin = f64::from(FIT_MARGIN) * 2.0;
        ((self.rect.size.x - margin).max(1.0) / f64::from(self.dag.width().max(1.0)))
            .min((self.rect.size.y - margin).max(1.0) / f64::from(self.dag.height().max(1.0)))
            .min(f64::from(MAX_SCALE))
    }

    fn harness_center(&self, state: PathState, pan_zoom: f64) -> (f64, f64) {
        let center = (f64::from(self.dag.width()) * 0.5, f64::from(self.dag.height()) * 0.5);
        let first = self.dag.order().first().and_then(|&index| self.dag.rect(index));
        let last = self.dag.order().last().and_then(|&index| self.dag.rect(index));
        let target = first.map_or(center, layout_center);
        let end = last.map_or(center, layout_center);
        let x = center.0 + (target.0 - center.0) * state.zoom;
        let y = center.1 + (target.1 - center.1) * state.zoom;
        (
            x + (target.0 + (end.0 - target.0) * state.pan - center.0) * pan_zoom,
            y + (target.1 + (end.1 - target.1) * state.pan - center.1) * pan_zoom,
        )
    }

    pub fn harness_click(&mut self, cx: &mut Cx, ordinal: u8) -> bool {
        let bounds = self.harness_bounds(0.0);
        let candidate = self.dag.order().iter().filter_map(|&index| {
            let rect = self.dag.rect(index)?;
            bounds.contains(rect.x + rect.width * 0.5, rect.center_y()).then_some(rect)
        }).cycle().take(3).nth(usize::from(ordinal));
        let Some(rect) = candidate else { return false };
        let abs = self.view.to_screen(self.rect, rect.x + rect.width * 0.5, rect.center_y());
        input::scripted_click(self, cx, abs);
        true
    }

    pub fn harness_scene(&self) -> Scene {
        let visible = self.harness_bounds(48.0);
        let labels = if self.view.scale < LABEL_MIN_SCALE { 0 } else {
            self.dag.order().iter().filter(|&&index| {
                self.graph.node(index).is_some()
                    && self.dag.rect(index).is_some_and(|rect| rect.intersects(visible))
            }).take(200).count()
        };
        Scene { nodes: self.node_count(), edges: self.edge_count(), labels }
    }

    pub fn harness_drawn(&self) -> bool {
        !self.needs_fit && self.rect.size.x >= 1.0 && self.rect.size.y >= 1.0
    }

    fn harness_bounds(&self, margin: f32) -> LayoutRect {
        let min = self.view.to_world(self.rect, self.rect.pos);
        let max = self.view.to_world(self.rect, dvec2(
            self.rect.pos.x + self.rect.size.x, self.rect.pos.y + self.rect.size.y,
        ));
        LayoutRect {
            x: min.0.min(max.0) - margin, y: min.1.min(max.1) - margin,
            width: (max.0 - min.0).abs() + margin * 2.0,
            height: (max.1 - min.1).abs() + margin * 2.0,
        }
    }
}

fn layout_center(rect: LayoutRect) -> (f64, f64) {
    (f64::from(rect.x + rect.width * 0.5), f64::from(rect.center_y()))
}
