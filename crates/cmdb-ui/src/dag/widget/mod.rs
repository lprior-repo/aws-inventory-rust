mod frame_harness;
mod input;
mod paint;
mod paint_style;

use makepad_widgets::*;

use crate::data::Graph as DataGraph;

use super::builder;
use super::graph::{DagGraph, DagNode};
use super::layout::DagLayout;
use super::view::ViewTransform;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.DagViewBase = #(DagView::register_widget(vm))
    mod.widgets.DagView = set_type_default() do mod.widgets.DagViewBase{
        width: Fill
        height: Fill
        draw_bg +: { color: theme.color_bg_app }
        draw_vector +: { draw_depth: 2.0 }
        draw_text +: { color: theme.color_text, draw_depth: 3.0 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DagSelectAction {
    pub id: Option<String>,
}

#[derive(Script, ScriptHook, Widget)]
pub struct DagView {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_vector: DrawVector,
    #[live]
    draw_text: DrawText,
    #[rust]
    graph: DagGraph,
    #[rust]
    dag: DagLayout,
    #[rust]
    view: ViewTransform,
    #[rust]
    selection: Option<usize>,
    #[rust]
    hover: Option<usize>,
    #[rust]
    drag: Option<DragState>,
    #[rust]
    rect: Rect,
    #[rust]
    needs_fit: bool,
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    start_abs: DVec2,
    start_offset: DVec2,
    moved: f64,
}

impl DagView {
    pub fn set_graph(&mut self, cx: &mut Cx, source: &DataGraph) {
        let graph = builder::from_data_graph(source);
        self.set_dag(cx, graph);
    }

    pub fn set_dag(&mut self, cx: &mut Cx, graph: DagGraph) {
        if self.graph == graph {
            return;
        }
        self.dag = DagLayout::compute(&graph);
        self.graph = graph;
        self.selection = None;
        self.hover = None;
        self.drag = None;
        self.needs_fit = true;
        self.redraw(cx);
    }

    pub fn selected_node(&self) -> Option<&DagNode> {
        self.selection.and_then(|index| self.graph.node(index))
    }

    pub fn selected_id(&self) -> Option<&str> {
        self.selected_node().map(DagNode::id)
    }

    pub fn select_id(&mut self, cx: &mut Cx, id: &str) -> bool {
        let node = self.graph.index_of(id);
        let Some(node) = node else {
            return false;
        };
        if self.selection == Some(node) {
            return false;
        }
        self.selection = Some(node);
        self.redraw(cx);
        true
    }

    pub fn clear_selection(&mut self, cx: &mut Cx) {
        if self.selection.take().is_some() {
            self.redraw(cx);
        }
    }

    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edges().len()
    }

    pub fn back_edge_count(&self) -> usize {
        self.dag.back_edges()
    }

    pub fn dropped_nodes(&self) -> usize {
        self.graph
            .dropped_nodes()
            .saturating_add(self.dag.dropped_nodes())
    }

    pub fn dropped_edges(&self) -> usize {
        self.graph
            .dropped_edges()
            .saturating_add(self.dag.dropped_edges())
    }
}

impl DagView {
    fn fit_once(&mut self) {
        if self.needs_fit {
            self.view = ViewTransform::fit(self.rect, self.dag.width(), self.dag.height());
            self.needs_fit = false;
        }
    }

    fn begin_canvas(&self, cx: &mut Cx2d) {
        cx.begin_turtle(
            Walk {
                abs_pos: Some(self.rect.pos),
                width: Size::Fixed(self.rect.size.x),
                height: Size::Fixed(self.rect.size.y),
                ..Default::default()
            },
            Layout {
                clip_x: true,
                clip_y: true,
                ..Layout::default()
            },
        );
    }
}

impl Widget for DagView {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.rect = cx.walk_turtle(walk);
        if self.rect.size.x < 1.0 || self.rect.size.y < 1.0 {
            return DrawStep::done();
        }
        self.fit_once();
        self.draw_bg.draw_abs(cx, self.rect);
        self.begin_canvas(cx);
        self.paint(cx);
        cx.end_turtle();
        DrawStep::done()
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        input::handle_event(self, cx, event);
    }
}
