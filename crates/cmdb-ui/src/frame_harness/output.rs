use super::{FrameHarness, report};

impl FrameHarness {
    pub(super) fn print_totals(&self) {
        let presented = self.presented;
        println!(
            "duration_requested_s={} duration_observed_s={}",
            self.config.duration.as_secs_f64(),
            self.elapsed().as_secs_f64()
        );
        println!(
            "frames_presented_handoffs={presented} (all-window compositor handoffs, not glass timestamps)"
        );
        println!(
            "nodes={} edges={} visible_labels_last={} visible_labels_min_max={:?} scripted_clicks={}",
            self.scene.nodes, self.scene.edges, self.scene.labels, self.label_range, self.clicks
        );
        println!("capture_status={:?}", self.capture_status());
    }

    fn capture_status(&self) -> &str {
        match self.error.as_deref() {
            Some(error) => error,
            None if self.elapsed() >= self.config.duration => "complete",
            None => "interrupted before scripted duration",
        }
    }

    pub(super) fn print_cadence(&self) {
        println!(
            "input_resources={} input_edges={} resource_cap=2000 edge_cap=4000 successful_clicks={}",
            self.resources, self.source_edges, self.successful_clicks
        );
        let refresh = report::stated("CMDB_UI_FRAME_REFRESH_HZ").parse::<f64>();
        let count = refresh
            .ok()
            .and_then(|hz| report::missed(&self.cadence, hz));
        println!(
            "draw_gap_over_1_5_stated_period_count_share={count:?} (not presented missed frames)"
        );
    }

    pub(super) fn print_series(&mut self) {
        if self.gpu.is_empty() {
            println!(
                "gpu_busy_unavailable_reason=backend supplied no completed GPU timings during capture"
            );
        }
        report::series("gpu_busy", &mut self.gpu);
        report::series("ui_app_dispatch", &mut self.ui);
        report::series("draw_cadence", &mut self.cadence);
    }
}
