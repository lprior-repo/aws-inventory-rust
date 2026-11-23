use makepad_widgets::*;
use makepad_widgets::makepad_platform::{draw_shader_layout::{LayoutKind, LayoutPacking}, gpu_frame_timer, retained_instances::RetainedInstances, shared_instances::PublishHints};
use std::{sync::Arc, thread, time::Instant};

app_main!(App, configure: configure);

script_mod! {
    use mod.prelude.widgets.*
    set_type_default() do #(DrawSpike::script_shader(vm)) {
        ..mod.draw.DrawQuad
        color: #x30b8e8
        transform: uniform(vec4(20.0, 40.0, 1.0, 1.0))
        vertex: fn() {
            self.vertex_pos = self.clip_and_transform_vertex(
                self.rect_pos * self.transform.zw + self.transform.xy,
                self.rect_size * self.transform.zw)
        }
        pixel: fn() { return self.color }
    }
    let SpikeBase = #(Spike::register_widget(vm))
    let Spike = set_type_default() do SpikeBase { width: Fill height: Fill }
    startup() do #(App::script_component(vm)) {
        ui: Root {
            main_window := Window {
                window.inner_size: vec2(1280, 720)
                pass.clear_color: #x03070d
                body +: { spike := Spike {} }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App { #[live] ui: WidgetRef }

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        self::script_mod(vm)
    }
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

#[derive(Clone)]
struct Blocks { main: RetainedInstances, lod: RetainedInstances }

fn configure(cx: &mut Cx) {
    let arg = std::env::args().nth(1).map_or_else(String::new, |value| value);
    let count = if arg.is_empty() { Ok(250_000) } else { arg.trim_start_matches("--").parse::<usize>() };
    let result = count.map_err(|e| e.to_string()).and_then(|n| {
        thread::scope(|s| s.spawn(move || build_blocks(n)).join())
            .map_err(|_| "instance worker failed".to_owned())?
    });
    match result {
        Ok(blocks) => cx.set_global(blocks),
        Err(error) => { eprintln!("spike startup: {error}"); std::process::exit(2); }
    }
    gpu_frame_timer::set_enabled(true);
    if std::env::var_os("SPIKE_ENVELOPE_PROBE").is_some() {
        cx.set_publication_envelope(1);
        let result = cx.publish_instances(16, Arc::from([0.0; 16]), PublishHints::default());
        println!("envelope_probe={:?} accounting={:?}", result.err(), cx.publication_accounting());
    }
}

fn build_blocks(count: usize) -> Result<Blocks, String> {
    if !(1..=4_000_000).contains(&count) { return Err("N must be in 1..=4000000".to_owned()); }
    let start = Instant::now();
    let side = (1u16..=2000).find(|n| usize::from(*n).checked_mul(usize::from(*n)).is_some_and(|v| v >= count))
        .ok_or("grid limit")?;
    let words = count.checked_mul(16).ok_or("word count overflow")?;
    let mut data = Vec::new();
    data.try_reserve_exact(words).map_err(|e| e.to_string())?;
    let sx = 1200.0 / f32::from(side);
    let sy = 650.0 / f32::from(side);
    for i in 0..count {
        let x = f32::from(u16::try_from(i % usize::from(side)).map_err(|e| e.to_string())?);
        let y = f32::from(u16::try_from(i / usize::from(side)).map_err(|e| e.to_string())?);
        data.extend_from_slice(&record(x * sx, y * sy, sx * 0.7, sy * 0.7, [0.18, 0.72, 0.91, 1.0]));
    }
    let main = RetainedInstances::new(16, Arc::from(data)).map_err(str::to_owned)?;
    let lod = RetainedInstances::new(16, Arc::from(record(400.0, 220.0, 400.0, 200.0, [1.0, 0.35, 0.1, 1.0])))
        .map_err(str::to_owned)?;
    println!("worker n={} slots={} bytes={} build_ms={:.3}", main.count(), main.slots(), main.byte_len(), start.elapsed().as_secs_f64() * 1000.0);
    Ok(Blocks { main, lod })
}

fn record(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> [f32; 16] {
    let [r, g, b, a] = color;
    [x, y, w, h, -1e6, -1e6, 1e6, 1e6, 1.0, 0.0, 0.0, 0.0, r, g, b, a]
}

#[derive(Script, ScriptHook)]
#[repr(C)]
pub struct DrawSpike {
    #[deref] draw_super: DrawQuad,
    #[live] color: Vec4f,
}

#[derive(Script, ScriptHook, Widget)]
pub struct Spike {
    #[uid] uid: WidgetUid,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[redraw] #[live] draw: DrawSpike,
    #[rust] next: NextFrame,
    #[rust] area: Area,
    #[rust] items: [Area; 2],
    #[rust] frame: u16,
    #[rust] previous_time: f64,
    #[rust] draws: u16,
    #[rust] lod: bool,
    #[rust] validated: bool,
    #[rust] drag: Option<Vec2d>,
    #[rust] pan: Vec2f,
    #[rust(1.0)] zoom: f32,
}

impl Widget for Spike {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::Startup => self.next = cx.new_next_frame(),
            Event::NextFrame(e) if e.set.contains(&self.next) => self.tick(cx, e.time),
            Event::MouseDown(e) => self.drag = Some(e.abs),
            Event::MouseUp(_) => self.drag = None,
            Event::MouseMove(e) => self.move_pointer(cx, e.abs),
            Event::Scroll(e) => {
                self.zoom = (self.zoom * if e.scroll.y > 0.0 { 0.98 } else { 1.02 }).clamp(0.1, 8.0);
                self.transform(cx);
            }
            Event::KeyDown(e) if e.key_code == KeyCode::KeyL && !e.is_repeat => self.toggle(cx),
            Event::KeyDown(e) if e.key_code == KeyCode::Escape => cx.quit(),
            _ => {}
        }
    }
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        cx.begin_turtle(walk, self.layout);
        if !self.validated { self.validated = validate(cx, &self.draw.draw_vars); }
        if self.validated {
            if let Some(blocks) = cx.get_global_ref::<Blocks>().cloned() {
                self.items = [cx.add_retained_instances(&self.draw.draw_vars, &blocks.main),
                    cx.add_retained_instances_count(&self.draw.draw_vars, &blocks.lod, usize::from(self.lod))];
                self.draws = self.draws.saturating_add(1);
                println!("record frame={} draws={} lod={} areas={:?}", self.frame, self.draws, self.lod, self.items);
                self.residency(cx);
                self.transform(cx);
            }
        } else { cx.quit(); }
        cx.end_turtle_with_area(&mut self.area);
        DrawStep::done()
    }
}

impl Spike {
    fn residency(&self, cx: &Cx) {
        for area in self.items {
            if let Area::Instance(item) = area {
                let verdict = cx.instance_upload_verdict(item.draw_list_id, item.draw_list_id, item.draw_item_id);
                println!("residency frame={} item={} upload_needed={}", self.frame, item.draw_item_id, verdict.upload_needed);
            }
        }
    }
    fn tick(&mut self, cx: &mut Cx, time: f64) {
        let start = Instant::now();
        self.frame = self.frame.saturating_add(1);
        if self.frame > 1 { println!("cadence frame={} delta_ms={:.6}", self.frame, (time - self.previous_time) * 1000.0); }
        self.previous_time = time;
        if matches!(self.frame, 120 | 240 | 360) { self.toggle(cx); }
        self.transform(cx);
        let ui_us = start.elapsed().as_secs_f64() * 1e6;
        for timing in gpu_frame_timer::take_frames() {
            println!("timing frame={} busy_ms={:.6} span_ms={:.6} ui_us={ui_us:.3} draws={}",
                self.frame, timing.busy_ms, timing.span_ms, self.draws);
        }
        if self.frame >= 600 {
            println!("complete frames={} draws={}", self.frame, self.draws);
            cx.quit();
        } else { self.next = cx.new_next_frame(); }
    }
    fn toggle(&mut self, cx: &mut Cx) {
        self.lod = !self.lod;
        println!("lod frame={} enabled={}", self.frame, self.lod);
        self.area.redraw(cx);
    }
    fn move_pointer(&mut self, cx: &mut Cx, abs: Vec2d) {
        if let Some(previous) = self.drag {
            let delta = Vec2f::from(abs - previous);
            if delta.x.is_finite() && delta.y.is_finite() {
                self.pan.x = (self.pan.x + delta.x).clamp(-8192.0, 8192.0);
                self.pan.y = (self.pan.y + delta.y).clamp(-8192.0, 8192.0);
                self.transform(cx);
            }
            self.drag = Some(abs);
        }
    }
    fn transform(&self, cx: &mut Cx) {
        let phase = f32::from(self.frame) * 0.025;
        let values = [20.0 + self.pan.x + phase.sin() * 10.0, 40.0 + self.pan.y, self.zoom, self.zoom];
        for area in self.items {
            if let Area::Instance(item) = area {
                let changed = cx.set_draw_item_uniform(item.draw_list_id, item.draw_item_id, id!(transform), &values);
                if matches!(self.frame, 1 | 121 | 241 | 361) {
                    println!("uniform frame={} item={} changed={changed}", self.frame, item.draw_item_id);
                }
            }
        }
    }
}

fn validate(cx: &Cx, vars: &DrawVars) -> bool {
    println!("gpu backend={:?} renderer={} vendor={} discrete={:?}", cx.gpu_backend(), cx.gpu_info().renderer, cx.gpu_info().vendor, cx.gpu_info().discrete);
    let Some(shader) = vars.draw_shader_id else { eprintln!("shader not initialized"); return false; };
    let layout = cx.layout_of(shader, LayoutKind::Instance, LayoutPacking::VertexFetch);
    let Ok(layout) = layout else { eprintln!("layout error: {layout:?}"); return false; };
    println!("instance_layout={layout:?}");
    let fields = [(id!(rect_pos), 0), (id!(rect_size), 2), (id!(draw_clip), 4),
        (id!(depth_clip), 8), (id!(draw_depth), 9), (id!(pad1), 10), (id!(pad2), 11), (id!(color), 12)];
    let valid = layout.stride_words == 16 && fields.iter()
        .all(|(name, offset)| layout.field(*name).is_some_and(|field| field.offset_words == *offset));
    if !valid { eprintln!("refusing retained bind: reflected layout differs from the worker's 16-slot records"); }
    valid
}
