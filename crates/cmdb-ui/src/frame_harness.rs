mod output;
pub mod path;
pub mod report;
#[cfg(test)]
mod tests;

use makepad_widgets::makepad_platform::{frame_trace, gpu_frame_timer, perf_monitor};
use makepad_widgets::{Cx, Event, NextFrame};
use std::time::{Duration, Instant};

const MAX_SAMPLES: usize = 262_144;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameConfig {
    duration: Duration,
}

impl FrameConfig {
    pub fn parse(value: &str) -> Result<Self, String> {
        let seconds = value
            .parse::<f64>()
            .map_err(|_| "invalid --frame-harness seconds")?;
        if !seconds.is_finite() || !(1.0..=600.0).contains(&seconds) {
            return Err("--frame-harness seconds must be between 1 and 600".to_owned());
        }
        let duration = Duration::try_from_secs_f64(seconds).map_err(|error| error.to_string())?;
        Ok(Self { duration })
    }
}

pub struct FrameHarness {
    config: FrameConfig,
    started: Option<Instant>,
    stopped: Option<Duration>,
    last_draw: Option<Instant>,
    next: NextFrame,
    channel: perf_monitor::PerfChannel,
    monitor: Vec<perf_monitor::PerfMonitorFrame>,
    ui: Vec<report::Sample>,
    gpu: Vec<report::Sample>,
    cadence: Vec<report::Sample>,
    presented_start: u64,
    presented: u64,
    pub clicks: u8,
    pub ready: bool,
    pub successful_clicks: u8,
    pub resources: u64,
    pub source_edges: u64,
    scene: report::Scene,
    label_range: Option<(usize, usize)>,
    window: Option<(f64, f64, f64)>,
    error: Option<String>,
}

impl FrameHarness {
    pub fn new(cx: &mut Cx, config: FrameConfig) -> Result<Self, String> {
        let ui = reserve(MAX_SAMPLES)?;
        let gpu = reserve(MAX_SAMPLES)?;
        let cadence = reserve(MAX_SAMPLES)?;
        let monitor = reserve(240)?;
        let channel = cx
            .perf_monitor
            .channel("frame_harness_app_dispatch", 0x58b6ff);
        let mut harness = Self::empty(config, channel);
        harness.ui = ui;
        harness.gpu = gpu;
        harness.cadence = cadence;
        harness.monitor = monitor;
        Ok(harness)
    }

    fn empty(config: FrameConfig, channel: perf_monitor::PerfChannel) -> Self {
        Self {
            config,
            started: None,
            last_draw: None,
            next: NextFrame::default(),
            channel,
            monitor: Vec::new(),
            ui: Vec::new(),
            gpu: Vec::new(),
            cadence: Vec::new(),
            presented_start: 0,
            clicks: 0,
            ready: false,
            scene: report::Scene::default(),
            label_range: None,
            window: None,
            error: None,
            successful_clicks: 0,
            resources: 0,
            source_edges: 0,
            stopped: None,
            presented: 0,
        }
    }

    pub fn before_event(&mut self, cx: &mut Cx, event: &Event) -> Option<path::PathState> {
        if let Event::WindowGeomChange(change) = event {
            let geom = &change.new_geom;
            self.window = Some((geom.inner_size.x, geom.inner_size.y, geom.dpi_factor));
        }
        let Event::NextFrame(frame) = event else {
            return None;
        };
        if self.stopped.is_some() || !frame.set.contains(&self.next) {
            return None;
        }
        let elapsed = self.elapsed();
        self.next = cx.new_next_frame();
        Some(path::at(elapsed, self.config.duration))
    }

    pub fn after_event(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        dispatch: Duration,
        scene: report::Scene,
    ) {
        if self.stopped.is_some() || !self.ready {
            return;
        }
        let micros = match u64::try_from(dispatch.as_micros()) {
            Ok(value) => value.min(u64::from(u32::MAX)),
            Err(_) => u64::from(u32::MAX),
        };
        cx.perf_monitor.add(self.channel, micros);
        if matches!(event, Event::Draw(_)) {
            self.draw_complete(cx, scene);
        }
    }

    fn draw_complete(&mut self, cx: &mut Cx, scene: report::Scene) {
        if self.started.is_none() {
            self.start(cx);
            return;
        }
        let now = Instant::now();
        let elapsed = self.elapsed().as_secs_f64();
        if let Some(last) = self.last_draw.replace(now) {
            let ms = now.saturating_duration_since(last).as_secs_f64() * 1000.0;
            self.push_cadence(report::Sample {
                elapsed_ms: elapsed * 1000.0,
                ms,
            });
        }
        self.record_scene(scene);
        self.collect(cx, elapsed);
        if self.elapsed() >= self.config.duration || self.error.is_some() {
            self.finish(cx);
        }
    }

    fn record_scene(&mut self, scene: report::Scene) {
        self.scene = scene;
        self.label_range = Some(
            self.label_range
                .map_or((scene.labels, scene.labels), |(min, max)| {
                    (min.min(scene.labels), max.max(scene.labels))
                }),
        );
    }

    fn start(&mut self, cx: &mut Cx) {
        gpu_frame_timer::set_enabled(false);
        gpu_frame_timer::set_enabled(true);
        cx.perf_monitor.set_enabled(false);
        cx.perf_monitor.set_enabled(true);
        cx.perf_monitor.frame_boundary(0.0);
        self.started = Some(Instant::now());
        self.last_draw = self.started;
        self.presented_start = frame_trace::presented_frames();
        self.next = cx.new_next_frame();
    }

    fn collect(&mut self, cx: &mut Cx, elapsed: f64) {
        cx.perf_monitor.frame_boundary(elapsed);
        cx.perf_monitor.read(&mut self.monitor);
        if let Some(frame) = self.monitor.last()
            && let Some(&micros) = frame.channel_us.get(self.channel.0)
        {
            let sample = report::Sample {
                elapsed_ms: elapsed * 1000.0,
                ms: f64::from(micros) / 1000.0,
            };
            if !push(&mut self.ui, sample) {
                self.capacity_error();
            }
        }
        for frame in gpu_frame_timer::take_frames() {
            let sample = report::Sample {
                elapsed_ms: elapsed * 1000.0,
                ms: f64::from(frame.busy_ms),
            };
            if !push(&mut self.gpu, sample) {
                self.capacity_error();
            }
        }
    }

    fn push_cadence(&mut self, sample: report::Sample) {
        if !push(&mut self.cadence, sample) {
            self.capacity_error();
        }
    }

    fn capacity_error(&mut self) {
        if self.error.is_none() {
            self.error =
                Some("sample budget reached or invalid timing; capture stopped".to_owned());
        }
    }

    fn elapsed(&self) -> Duration {
        match self.stopped {
            Some(elapsed) => elapsed,
            None => self
                .started
                .map_or(Duration::ZERO, |started| started.elapsed()),
        }
    }

    pub fn abort(&mut self, cx: &mut Cx, error: String) {
        self.error = Some(error);
        self.finish(cx);
    }

    pub fn finish(&mut self, cx: &mut Cx) {
        if self.stopped.is_some() {
            return;
        }
        self.stopped = Some(self.elapsed());
        self.presented = if self.started.is_some() {
            frame_trace::presented_frames().saturating_sub(self.presented_start)
        } else {
            0
        };
        report::metadata(cx, self.window);
        report::limitations();
        self.print_totals();
        self.print_cadence();
        self.print_series();
        gpu_frame_timer::set_enabled(false);
        cx.perf_monitor.set_enabled(false);
        cx.quit();
    }
}

fn reserve<T>(capacity: usize) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|error| error.to_string())?;
    Ok(values)
}

fn push(samples: &mut Vec<report::Sample>, sample: report::Sample) -> bool {
    if samples.len() >= MAX_SAMPLES || !sample.ms.is_finite() || sample.ms < 0.0 {
        return false;
    }
    samples.push(sample);
    true
}
