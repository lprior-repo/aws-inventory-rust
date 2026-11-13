//! One `DrawVector` batch for edges and node boxes, then one `DrawText` pass.
//!
//! Everything is converted to window coordinates on the CPU (the camera is a
//! scale + translation), which keeps text at a fixed readable size and lets
//! the whole graph share a single vector draw call.

use makepad_widgets::*;

use super::DagView;
use super::paint_style::{ACCENT, clamp_chars, edge_box, edge_style, max_chars};
use crate::dag::layout::{LayoutEdge, LayoutRect};
use crate::dag::view::{LABEL_MIN_SCALE, f32_of};

const NODE_FILL: u32 = 0x1b1e27;
const NODE_FILL_HOVER: u32 = 0x222736;
const NODE_FILL_SELECTED: u32 = 0x2b3142;
const FALLBACK_ACCENT: u32 = 0x8f98a8;
/// Upper bound on text runs per frame; larger graphs rely on hover only.
const MAX_LABELS: usize = 200;

#[derive(Clone, Copy)]
struct NodeGeometry {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
}

fn as_f32(value: f64) -> f32 {
    f32_of(value)
}

fn label_color(selected: bool) -> Vec4f {
    if selected {
        vec4(1.0, 1.0, 1.0, 1.0)
    } else {
        vec4(0.84, 0.86, 0.91, 1.0)
    }
}

impl DagView {
    pub(super) fn paint(&mut self, cx: &mut Cx2d) {
        self.draw_vector.begin();
        self.paint_edges();
        self.paint_nodes();
        self.draw_vector.end(cx);
        self.paint_labels(cx);
    }

    fn paint_edges(&mut self) {
        let visible = self.visible_world();
        for slot in 0..self.dag.edges().len() {
            let Some(&edge) = self.dag.edges().get(slot) else {
                continue;
            };
            let (Some(from), Some(to)) = (self.dag.rect(edge.from), self.dag.rect(edge.to)) else {
                continue;
            };
            if edge_box(from, to).intersects(visible) {
                self.paint_edge(edge, from, to);
            }
        }
    }

    fn paint_edge(&mut self, edge: LayoutEdge, from: LayoutRect, to: LayoutRect) {
        let (color, width) = edge_style(edge, self.selection);
        let (start, end, forward, bend) = self.edge_anchors(from, to);
        self.stroke_edge(start, end, color, width, bend);
        let arrow = (6.0 * self.view.scale).clamp(6.0, 12.0);
        self.paint_arrow(end, forward, arrow, color);
    }

    fn edge_anchors(&self, from: LayoutRect, to: LayoutRect) -> (DVec2, DVec2, bool, f64) {
        let same_tier = (to.x - from.x).abs() < 1.0;
        let forward = !same_tier && to.x > from.x;
        let (sx, sy, tx, ty) = if same_tier {
            (from.right(), from.center_y(), to.right(), to.center_y())
        } else if forward {
            (from.right(), from.center_y(), to.x, to.center_y())
        } else {
            (from.x, from.center_y(), to.right(), to.center_y())
        };
        let start = self.view.to_screen(self.rect, sx, sy);
        let end = self.view.to_screen(self.rect, tx, ty);
        let bend = if same_tier {
            44.0 * f64::from(self.view.scale)
        } else {
            (end.x - start.x) * 0.5
        };
        (start, end, forward, bend)
    }

    fn stroke_edge(&mut self, start: DVec2, end: DVec2, color: u32, width: f32, bend: f64) {
        let same_tier = (end.x - start.x).abs() < 1.0;
        let second = if same_tier {
            end.x + bend
        } else {
            end.x - bend
        };
        self.draw_vector.set_color_hex(color, 1.0);
        self.draw_vector.move_to(as_f32(start.x), as_f32(start.y));
        self.draw_vector.bezier_to(
            as_f32(start.x + bend),
            as_f32(start.y),
            as_f32(second),
            as_f32(end.y),
            as_f32(end.x),
            as_f32(end.y),
        );
        self.draw_vector.stroke(width);
    }

    fn paint_arrow(&mut self, tip: DVec2, forward: bool, size: f32, color: u32) {
        let direction = if forward { -1.0_f64 } else { 1.0_f64 };
        let base_x = tip.x + f64::from(size) * direction;
        let half = f64::from(size) * 0.5;
        self.draw_vector.set_color_hex(color, 1.0);
        self.draw_vector.move_to(as_f32(tip.x), as_f32(tip.y));
        self.draw_vector
            .line_to(as_f32(base_x), as_f32(tip.y - half));
        self.draw_vector
            .line_to(as_f32(base_x), as_f32(tip.y + half));
        self.draw_vector.close();
        self.draw_vector.fill();
    }

    fn paint_nodes(&mut self) {
        let visible = self.visible_world();
        for slot in 0..self.dag.order().len() {
            let Some(&index) = self.dag.order().get(slot) else {
                continue;
            };
            let Some(rect) = self.dag.rect(index) else {
                continue;
            };
            if rect.intersects(visible) {
                self.paint_node(index, rect);
            }
        }
    }

    fn paint_node(&mut self, index: usize, rect: LayoutRect) {
        let geometry = self.node_geometry(rect);
        let (fill, stroke, width, alpha) = self.node_style(index);
        self.fill_node(geometry, fill);
        self.stroke_node(geometry, stroke, width, alpha);
    }

    fn node_geometry(&self, rect: LayoutRect) -> NodeGeometry {
        let top_left = self.view.to_screen(self.rect, rect.x, rect.y);
        NodeGeometry {
            x: as_f32(top_left.x),
            y: as_f32(top_left.y),
            width: as_f32(f64::from(rect.width) * f64::from(self.view.scale)),
            height: as_f32(f64::from(rect.height) * f64::from(self.view.scale)),
            radius: (6.0 * self.view.scale).clamp(2.0, 10.0),
        }
    }

    fn node_style(&self, index: usize) -> (u32, u32, f32, f32) {
        let selected = self.selection == Some(index);
        let hovered = self.hover == Some(index);
        let fill = if selected {
            NODE_FILL_SELECTED
        } else if hovered {
            NODE_FILL_HOVER
        } else {
            NODE_FILL
        };
        let accent = self
            .graph
            .node(index)
            .map_or(FALLBACK_ACCENT, |node| node.kind().accent());
        let stroke = if selected { ACCENT } else { accent };
        let width = if selected {
            2.4
        } else if hovered {
            1.8
        } else {
            1.0
        };
        (fill, stroke, width, if selected { 1.0 } else { 0.75 })
    }

    fn fill_node(&mut self, geometry: NodeGeometry, color: u32) {
        self.draw_vector.set_color_hex(color, 1.0);
        self.draw_vector.rounded_rect(
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            geometry.radius,
        );
        self.draw_vector.fill();
    }

    fn stroke_node(&mut self, geometry: NodeGeometry, color: u32, width: f32, alpha: f32) {
        self.draw_vector.set_color_hex(color, alpha);
        self.draw_vector.rounded_rect(
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            geometry.radius,
        );
        self.draw_vector.stroke(width);
    }

    fn paint_labels(&mut self, cx: &mut Cx2d) {
        if self.view.scale < LABEL_MIN_SCALE {
            return;
        }
        let font = (10.0 * self.view.scale).clamp(6.5, 10.0);
        self.draw_text.text_style.font_size = font;
        let visible = self.visible_world();
        let mut drawn = 0;
        for slot in 0..self.dag.order().len() {
            if drawn >= MAX_LABELS {
                break;
            }
            let Some(&index) = self.dag.order().get(slot) else {
                continue;
            };
            let Some(rect) = self.dag.rect(index) else {
                continue;
            };
            if rect.intersects(visible) && self.paint_label(cx, index, rect, font) {
                drawn = drawn.saturating_add(1);
            }
        }
    }

    fn paint_label(&mut self, cx: &mut Cx2d, index: usize, rect: LayoutRect, font: f32) -> bool {
        let Some(node) = self.graph.node(index) else {
            return false;
        };
        let (top_left, width, height) = self.label_metrics(rect);
        let label = clamp_chars(node.label(), max_chars(width, font));
        self.draw_text.color = label_color(self.selection == Some(index));
        self.draw_text.draw_abs(
            cx,
            dvec2(
                top_left.x + 7.0,
                top_left.y + height * 0.5 - f64::from(font) * 0.5,
            ),
            label,
        );
        true
    }

    fn label_metrics(&self, rect: LayoutRect) -> (DVec2, f64, f64) {
        (
            self.view.to_screen(self.rect, rect.x, rect.y),
            f64::from(rect.width) * f64::from(self.view.scale),
            f64::from(rect.height) * f64::from(self.view.scale),
        )
    }

    fn visible_world(&self) -> LayoutRect {
        let min = self.view.to_world(self.rect, self.rect.pos);
        let max = self.view.to_world(
            self.rect,
            dvec2(
                self.rect.pos.x + self.rect.size.x,
                self.rect.pos.y + self.rect.size.y,
            ),
        );
        let margin = 48.0;
        LayoutRect {
            x: min.0.min(max.0) - margin,
            y: min.1.min(max.1) - margin,
            width: (max.0 - min.0).abs() + margin * 2.0,
            height: (max.1 - min.1).abs() + margin * 2.0,
        }
    }
}
