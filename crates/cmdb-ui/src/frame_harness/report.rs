use makepad_widgets::Cx;

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub elapsed_ms: f64,
    pub ms: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Scene {
    pub nodes: usize,
    pub edges: usize,
    pub labels: usize,
}

pub fn percentile<T: Copy>(sorted: &[T], percent: usize) -> Option<T> {
    if !(1..=100).contains(&percent) {
        return None;
    }
    let rank = sorted.len().checked_mul(percent)?.checked_add(99)?.checked_div(100)?;
    sorted.get(rank.checked_sub(1)?).copied()
}

pub fn missed(intervals: &[Sample], refresh_hz: f64) -> Option<(usize, f64)> {
    if !refresh_hz.is_finite() || refresh_hz <= 0.0 || intervals.is_empty() {
        return None;
    }
    let count = intervals.iter().filter(|sample| sample.ms > 1500.0 / refresh_hz).count();
    let total = f64::from(u32::try_from(intervals.len()).ok()?);
    Some((count, f64::from(u32::try_from(count).ok()?) / total))
}

pub fn series(name: &str, samples: &mut [Sample]) {
    for sample in samples.iter() {
        println!("frame_sample,{name},{:.6},{:.6}", sample.elapsed_ms, sample.ms);
    }
    samples.sort_unstable_by(|left, right| left.ms.total_cmp(&right.ms));
    let [p50, p95, p99, maximum] = [50, 95, 99, 100].map(|percent| {
        metric(percentile(samples, percent).map(|sample| sample.ms))
    });
    println!("{name}_ms: n={} p50={p50} p95={p95} p99={p99} max={maximum}", samples.len());
}

fn metric(value: Option<f64>) -> String {
    match value {
        Some(value) => format!("{value:.6}"),
        None => "unavailable".to_owned(),
    }
}

pub fn metadata(cx: &Cx, window: Option<(f64, f64, f64)>) {
    println!("frame_harness: makepad=506ec05105b9adb08d1d21f9879df181b5244ea7");
    println!("gpu={:?} vendor={:?} backend={}",
        cx.gpu_info().renderer, cx.gpu_info().vendor, cx.gpu_backend().name());
    println!("window_width_height_scale={window:?}");
    for (field, variable) in [
        ("display_refresh_hz", "CMDB_UI_FRAME_REFRESH_HZ"),
        ("display_resolution", "CMDB_UI_FRAME_RESOLUTION"),
        ("driver", "CMDB_UI_FRAME_DRIVER"),
        ("vsync", "CMDB_UI_FRAME_VSYNC"),
        ("machine", "CMDB_UI_FRAME_MACHINE"),
        ("power", "CMDB_UI_FRAME_POWER"),
        ("thermal", "CMDB_UI_FRAME_THERMAL"),
    ] {
        println!("{field}={} (operator-stated; unavailable if unset)", stated(variable));
    }
}

pub fn stated(variable: &str) -> String {
    match std::env::var_os(variable) {
        Some(value) if value.len() <= 256 => value.to_string_lossy().replace(['\n', '\r'], " "),
        Some(_) => "unavailable: value exceeds 256 bytes".to_owned(),
        None => "unavailable".to_owned(),
    }
}

pub fn limitations() {
    println!("presented_timestamp_ms=unavailable: pinned HAL discards FrameDone.present_ns");
    println!("presented_missed_count=unavailable presented_missed_share=unavailable: no per-frame present timestamps");
    println!("present_trace_1hz=unavailable: pinned present_trace has no producers");
    println!("present_gap_aggregate=unavailable: pinned FrameTrace histogram has no active callers");
    println!("ui_ms_scope=app event dispatch through perf_monitor app channel; excludes backend encoding and framework work outside App::handle_event");
    println!("draw_cadence_ms_scope=wall clock between completed draws, not CPU cost or presentation timing");
    println!("gpu_ms_scope=completed GPU busy samples; independent series, no timestamp/correlation API; final in-flight GPU work excluded");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_sorted_samples_when_percentiles_requested_then_nearest_rank_is_used() {
        let values: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(percentile(&values, 50), Some(50.0));
        assert_eq!(percentile(&values, 95), Some(95.0));
        assert_eq!(percentile(&values, 99), Some(99.0));
        assert_eq!(percentile(&values, 100), Some(100.0));
        assert_eq!(percentile(&[7.0], 99), Some(7.0));
        assert_eq!(percentile(&[1.0, 2.0, 3.0], 50), Some(2.0));
        assert_eq!(percentile(&[1.0, 2.0, 3.0], 95), Some(3.0));
        assert_eq!(percentile::<f64>(&[], 50), None);
        assert_eq!(percentile(&values, 0), None);
        assert_eq!(percentile(&values, 101), None);
    }

    #[test]
    fn given_refresh_period_when_counting_then_threshold_is_strict_and_share_is_intervals() {
        let samples = [16.0, 25.0, 25.001, 50.0].map(|ms| Sample { elapsed_ms: 0.0, ms });
        assert_eq!(missed(&samples, 60.0), Some((2, 0.5)));
        assert_eq!(missed(&samples, 120.0), Some((4, 1.0)));
        assert_eq!(missed(&[], 60.0), None);
        assert_eq!(missed(&samples, 0.0), None);
        assert_eq!(missed(&samples, f64::NAN), None);
    }
}
