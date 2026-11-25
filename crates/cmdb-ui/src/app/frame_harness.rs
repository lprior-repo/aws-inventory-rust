use super::App;
use crate::dag::graph::DagGraph;
use crate::dag::widget::DagView;
use crate::frame_harness::{FrameHarness, report::Scene};
use makepad_widgets::*;
use std::time::Duration;

impl App {
    pub(super) fn apply_worker_graph_status(&mut self, cx: &mut Cx, error: Option<String>) {
        self.graph_access_error = error.clone();
        self.apply_graph_event(cx, crate::data::Graph::default(), error);
    }

    pub(super) fn select_graph_resource(&mut self, cx: &mut Cx, id: &str) {
        let row = self
            .table(cx)
            .borrow::<crate::table::ResourceTable>()
            .and_then(|table| table.row_for_id(id));
        if let Some(row) = row {
            self.request_selection(cx, row);
        }
    }

    pub(super) fn start_frame_harness(&mut self, cx: &mut Cx) {
        let config = self
            .worker_config
            .as_ref()
            .map_or_else(crate::cli::startup_config, Clone::clone);
        let Some(config) = config.frame_harness else {
            return;
        };
        match FrameHarness::new(cx, config) {
            Ok(harness) => self.frame_harness = Some(harness),
            Err(error) => {
                eprintln!("frame_harness unavailable: {error}");
                cx.quit();
            }
        }
    }

    pub(super) fn apply_harness_graph(
        &mut self,
        cx: &mut Cx,
        graph: DagGraph,
        resources: u64,
        edges: u64,
    ) {
        let dag_ref = self.ui.widget(cx, ids!(dag));
        if let Some(mut dag) = dag_ref.borrow_mut::<DagView>() {
            dag.set_dag(cx, graph);
            drop(dag);
            self.mark_harness_ready(cx, resources, edges);
        } else if let Some(harness) = self.frame_harness.as_mut() {
            harness.abort(cx, "DagView is unavailable".to_owned());
        }
    }

    fn mark_harness_ready(&mut self, cx: &mut Cx, resources: u64, edges: u64) {
        if let Some(harness) = self.frame_harness.as_mut() {
            harness.resources = resources;
            harness.source_edges = edges;
            harness.ready = true;
            self.ui.label(cx, ids!(graph_counts)).set_text(
                cx,
                &format!("Frame harness · {resources} input resources · current painter cap 2000"),
            );
        }
    }

    pub(super) fn step_frame_harness(&mut self, cx: &mut Cx, event: &Event) {
        let Some(harness) = self.frame_harness.as_mut() else {
            return;
        };
        let Some(state) = harness.before_event(cx, event) else {
            return;
        };
        let dag_ref = self.ui.widget(cx, ids!(dag));
        if let Some(mut dag) = dag_ref.borrow_mut::<DagView>() {
            dag.harness_path(cx, state);
            for click in harness.clicks..state.clicks {
                harness.successful_clicks = harness
                    .successful_clicks
                    .saturating_add(u8::from(dag.harness_click(cx, click)));
            }
            harness.clicks = state.clicks;
        }
    }

    pub(super) fn finish_harness_event(&mut self, cx: &mut Cx, event: &Event, dispatch: Duration) {
        let Some(harness) = self.frame_harness.as_mut() else {
            return;
        };
        let dag_ref = self.ui.widget(cx, ids!(dag));
        let Some(dag) = dag_ref.borrow::<DagView>() else {
            return;
        };
        if dag.harness_drawn() {
            let scene = if matches!(event, Event::Draw(_)) {
                dag.harness_scene()
            } else {
                Scene::default()
            };
            harness.after_event(cx, event, dispatch, scene);
        }
    }

    pub(super) fn shutdown_frame_harness(&mut self, cx: &mut Cx) {
        if let Some(harness) = self.frame_harness.as_mut() {
            harness.finish(cx);
        }
    }

    pub(super) fn apply_worker_snapshot(
        &mut self,
        cx: &mut Cx,
        counts: crate::data::Counts,
        rows: Vec<crate::data::Row>,
        total: u64,
        error: Option<String>,
    ) {
        if let (Some(harness), Some(error)) = (self.frame_harness.as_mut(), error.as_ref()) {
            harness.abort(cx, error.clone());
            return;
        }
        self.apply_snapshot(cx, counts, rows, total, error);
    }
}
