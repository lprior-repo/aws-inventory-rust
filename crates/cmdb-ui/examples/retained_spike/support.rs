use makepad_widgets::makepad_platform::{
    draw_shader_layout::{LayoutKind, LayoutPacking},
    gpu_frame_timer,
    retained_instances::RetainedInstances,
    shared_instances::PublishHints,
};
use makepad_widgets::*;
use std::{sync::Arc, thread, time::Instant};

const FIELDS: [(LiveId, u32); 8] = [
    (id!(rect_pos), 0),
    (id!(rect_size), 2),
    (id!(draw_clip), 4),
    (id!(depth_clip), 8),
    (id!(draw_depth), 9),
    (id!(pad1), 10),
    (id!(pad2), 11),
    (id!(color), 12),
];

#[derive(Clone)]
pub(super) struct Blocks {
    pub(super) main: RetainedInstances,
    pub(super) lod: RetainedInstances,
}

pub(super) fn configure(cx: &mut Cx) {
    let count = match std::env::args().nth(1) {
        Some(arg) if !arg.is_empty() => arg.trim_start_matches("--").parse::<usize>(),
        _ => Ok(250_000),
    };
    let result = count.map_err(|e| e.to_string()).and_then(|n| {
        thread::scope(|s| s.spawn(move || build_blocks(n)).join())
            .map_err(|_| "instance worker failed".to_owned())?
    });
    match result {
        Ok(blocks) => cx.set_global(blocks),
        Err(error) => {
            eprintln!("spike startup: {error}");
            std::process::exit(2);
        }
    }
    gpu_frame_timer::set_enabled(true);
    probe_envelope(cx);
}

fn probe_envelope(cx: &mut Cx) {
    if std::env::var_os("SPIKE_ENVELOPE_PROBE").is_some() {
        cx.set_publication_envelope(1);
        let result = cx.publish_instances(16, Arc::from([0.0; 16]), PublishHints::default());
        println!(
            "envelope_probe={:?} accounting={:?}",
            result.err(),
            cx.publication_accounting()
        );
    }
}

fn build_blocks(count: usize) -> Result<Blocks, String> {
    let start = Instant::now();
    let side = grid_side(count)?;
    let main =
        RetainedInstances::new(16, Arc::from(grid_data(count, side)?)).map_err(str::to_owned)?;
    let lod = RetainedInstances::new(
        16,
        Arc::from(record(400.0, 220.0, 400.0, 200.0, [1.0, 0.35, 0.1, 1.0])),
    )
    .map_err(str::to_owned)?;
    println!(
        "worker n={} slots={} bytes={} build_ms={:.3}",
        main.count(),
        main.slots(),
        main.byte_len(),
        start.elapsed().as_secs_f64() * 1000.0
    );
    Ok(Blocks { main, lod })
}

fn grid_side(count: usize) -> Result<u16, String> {
    if !(1..=4_000_000).contains(&count) {
        return Err("N must be in 1..=4000000".to_owned());
    }
    (1u16..=2000)
        .find(|n| {
            usize::from(*n)
                .checked_mul(usize::from(*n))
                .is_some_and(|v| v >= count)
        })
        .ok_or_else(|| "grid limit".to_owned())
}

fn grid_data(count: usize, side: u16) -> Result<Vec<f32>, String> {
    let words = count.checked_mul(16).ok_or("word count overflow")?;
    let mut data = Vec::new();
    data.try_reserve_exact(words).map_err(|e| e.to_string())?;
    let sx = 1200.0 / f32::from(side);
    let sy = 650.0 / f32::from(side);
    for i in 0..count {
        let x = coordinate(i.checked_rem(usize::from(side)))?;
        let y = coordinate(i.checked_div(usize::from(side)))?;
        data.extend_from_slice(&record(
            x * sx,
            y * sy,
            sx * 0.7,
            sy * 0.7,
            [0.18, 0.72, 0.91, 1.0],
        ));
    }
    Ok(data)
}

fn coordinate(value: Option<usize>) -> Result<f32, String> {
    let value = value.ok_or("grid divisor is zero")?;
    u16::try_from(value)
        .map(f32::from)
        .map_err(|e| e.to_string())
}

fn record(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> [f32; 16] {
    let [r, g, b, a] = color;
    [
        x, y, w, h, -1e6, -1e6, 1e6, 1e6, 1.0, 0.0, 0.0, 0.0, r, g, b, a,
    ]
}

pub(super) fn validate(cx: &Cx, vars: &DrawVars) -> bool {
    report_gpu(cx);
    let Some(shader) = vars.draw_shader_id else {
        eprintln!("shader not initialized");
        return false;
    };
    let layout = cx.layout_of(shader, LayoutKind::Instance, LayoutPacking::VertexFetch);
    let Ok(layout) = layout else {
        eprintln!("layout error: {layout:?}");
        return false;
    };
    println!("instance_layout={layout:?}");
    let valid = layout.stride_words == 16
        && FIELDS.iter().all(|(name, offset)| {
            layout
                .field(*name)
                .is_some_and(|field| field.offset_words == *offset)
        });
    if !valid {
        eprintln!(
            "refusing retained bind: reflected layout differs from the worker's 16-slot records"
        );
    }
    valid
}

fn report_gpu(cx: &Cx) {
    println!(
        "gpu backend={:?} renderer={} vendor={} discrete={:?}",
        cx.gpu_backend(),
        cx.gpu_info().renderer,
        cx.gpu_info().vendor,
        cx.gpu_info().discrete
    );
}
