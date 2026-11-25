# D6.0: retained instances at Makepad `506ec05`

Recorded 2026-11-22. Example: `crates/cmdb-ui/examples/retained_spike.rs`.
Makepad revision: `506ec05105b9adb08d1d21f9879df181b5244ea7`.

## Result and limits

Real Vulkan runs accepted all four requested instance counts on an RTX 5090
and the Ryzen 9950X3D's integrated Radeon GPU. Each run reached its bounded
600-frame exit successfully. A separate RTX 3090 offscreen run accepted
250,000 instances. Both `headless` and `offscreen` started without DISPLAY or
WAYLAND_DISPLAY. The retained API is usable at this revision; no fallback was
needed.

These are spike measurements, **not a P14 presented-frame pass**. The only
connected display was 60 Hz, the compositor tiled the requested 1280x720
window, both NVIDIA cards already had inference workloads, and this pinned
Linux HAL produced no `MAKEPAD_TRACE=present` observations. Actual presented
frame counts, missed presents, and 120 Hz performance are **not measured**.
The callback rates below are not presented FPS. No tested GPU can be certified
as holding its display's refresh rate from these observations alone.

Physical upload bytes are also not exposed by the public diagnostics used
here. Upload verdicts and source inspection are distinguished below; a false
verdict is not claimed as a measurement of zero physical transfer bytes.

## Machine and build

- Linux `7.2.11-arch1-1`, x86_64; AMD Ryzen 9 9950X3D, 32 logical CPUs.
- `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-1`,
  `XDG_RUNTIME_DIR=/run/user/1000`; Hyprland, HDMI-A-1, VIZIO E420VO,
  1920x1080 at 60.000 Hz, scale 1, VRR disabled.
- NVIDIA driver `615.78.08`:
  - RTX 5090, UUID `GPU-b2092e2c-3f30-482d-182b-17f3f6cda12c`,
    32,607 MiB total; observation before runs: 28,019 MiB used, 100% GPU use,
    with `ninfer-serve` running.
  - RTX 3090, UUID `GPU-403c2dbf-8f1c-f0d6-94a8-bd1fa4f6629f`,
    24,576 MiB total; observation before runs: 23,402 MiB used, 100% GPU use,
    with `llama-server` running.
- Integrated GPU: PCI AMD Granite Ridge Radeon Graphics / Radeon 610M
  (`1002:13c0`), amdgpu. Makepad reported
  `AMD Ryzen 9 9950X3D 16-Core Processor (RADV RAPHAEL_MENDOCINO)`,
  vendor `amd`, `discrete=Some(false)`. Selected with the installed Radeon
  Vulkan ICD, not a software renderer.
- Rust `1.100.0`; workspace release profile: opt-level 3, thin LTO,
  one codegen unit, stripped symbols. Package name in this tree: `cmdb-ui`.
- Private target directory: `/home/lewis/.cache/cargo-target-d60`.
  Cargo parallelism: `-j2`. No inference services were stopped or reconfigured.

The requested window size is not a controlled benchmark surface size: the
first X11 input capture was 476x522 and another geometry observation was
476x523. Therefore these results do not establish a full-window fill-rate
budget at 1280x720. The shader still receives the requested N instances.

## What is implemented and read back

The worker is a scoped, joined OS thread. Before painting starts it builds an
immutable `Arc<[f32]>`, validates N in `1..=4_000_000`, and calls
`RetainedInstances::new(16, data)`. There are no detached tasks. The main grid
spans 1200x650 logical units; a second, orange 400x200 rectangle is another
retained publication. The widget keeps shallow retained handles, not a copy
of N records, when recording a draw.

`DrawSpike` derives Script/ScriptHook, dereferences `DrawQuad`, and adds a
`#[live] color: Vec4f`. Its pinned Script shader inherits `mod.draw.DrawQuad`,
declares `transform: uniform(vec4(...))`, assigns `self.vertex_pos` using
`clip_and_transform_vertex`, and returns `self.color` from `pixel`.

Before binding, `Cx::layout_of(shader, LayoutKind::Instance,
LayoutPacking::VertexFetch)` checks the following reflected word offsets.
Every measured backend accepted this layout; the observed layout ID was
`4304705377461903664` and stride was **16 f32 words / 64 bytes**.

| Field | Shader type | Word offset |
| --- | --- | ---: |
| rect_pos | Vec2 | 0 |
| rect_size | Vec2 | 2 |
| draw_clip | Vec4 | 4 |
| depth_clip | F32 | 8 |
| draw_depth | F32 | 9 |
| pad1 | F32 | 10 |
| pad2 | F32 | 11 |
| color | Vec4 | 12 |

`add_retained_instances` and `add_retained_instances_count` return
`Area::Instance(InstanceArea)`. The example obtains `draw_list_id` and
`draw_item_id` by matching those returned areas, never by assuming an item
index. An observed binding was list `DrawListId(21, 0)`, item 0 for the grid
and item 1 for LOD; those numbers are observations, not hard-coded inputs.

Drag, wheel and automatic bounded animation call
`set_draw_item_uniform(list, item, id!(transform), &values)` on both items.
They do not request a widget redraw. L toggles the second binding's count
between 0 and 1 and redraws the widget area. Automatic toggles at frames 120,
240 and 360 make the same path observable without injected input. Escape
quits; otherwise the example exits after 600 NextFrame callbacks. Pan is
clamped to +/-8192 and zoom to 0.1..8.

Pinned implementation sources read for these APIs include
`draw/src/draw_list_2d.rs`, `draw/src/shader/draw_quad.rs`,
`platform/src/{area,cx_api,draw_shader_layout,retained_instances}.rs`,
`platform/src/gpu_frame_timer.rs`, the pinned shader/frame-probe examples,
and `platform/src/frame_build.rs`. The instance layout is checked before
calling APIs whose pinned implementation asserts on mismatched stride.

## Release measurements

The final measured executable was built with the final LOD area-redraw fix.
All rows below exited 0 with `complete frames=600`. Samples delivered at
callbacks 60 through 600 were retained: **541 GPU samples and 541 callback
intervals per GPU row**. Percentiles use nearest ranks on sorted samples.
GPU numbers are `gpu_frame_timer::take_frames().busy_ms`, not CPU time or
present latency. The `ui_us` diagnostic covers only the callback's uniform
update and cadence logging, not a complete UI frame. Diagnostic printing is
part of this spike and can perturb its timings.

| GPU | N | Payload bytes | Worker build ms | GPU mean ms | GPU p50 ms | GPU p95 ms | GPU p99 ms | GPU max ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RTX 5090, windowed | 50,000 | 3,200,000 | 0.845 | 0.135449 | 0.133514 | 0.144958 | 0.162125 | 0.198841 |
| RTX 5090, windowed | 250,000 | 16,000,000 | 2.781 | 0.610451 | 0.607967 | 0.621557 | 0.631094 | 0.640154 |
| RTX 5090, windowed | 1,000,000 | 64,000,000 | 28.036 | 4.681968 | 4.672050 | 4.805088 | 4.827023 | 4.838705 |
| RTX 5090, windowed | 4,000,000 | 256,000,000 | 113.919 | 11.770965 | 11.783600 | 11.925697 | 12.061119 | 12.326241 |
| Integrated Radeon, windowed | 50,000 | 3,200,000 | 1.024 | 0.436816 | 0.436200 | 0.453800 | 0.478240 | 0.490760 |
| Integrated Radeon, windowed | 250,000 | 16,000,000 | 5.671 | 1.152092 | 1.141720 | 1.510600 | 1.547900 | 1.554070 |
| Integrated Radeon, windowed | 1,000,000 | 64,000,000 | 13.998 | 2.666182 | 2.684560 | 2.911320 | 3.344690 | 3.471940 |
| Integrated Radeon, windowed | 4,000,000 | 256,000,000 | 35.350 | 8.216433 | 8.212590 | 8.561990 | 8.739320 | 11.120030 |

| GPU | N | Mean callback ms | Callback Hz | Max callback ms | Intervals >25 ms | Missed presents |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| RTX 5090 | 50,000 | 17.476327 | 57.220 | 17.738516 | 0 | Not measured |
| RTX 5090 | 250,000 | 17.337417 | 57.679 | 17.740455 | 0 | Not measured |
| RTX 5090 | 1,000,000 | 17.284174 | 57.856 | 17.738936 | 0 | Not measured |
| RTX 5090 | 4,000,000 | 16.958823 | 58.966 | 17.942565 | 0 | Not measured |
| Integrated Radeon | 50,000 | 17.482101 | 57.201 | 17.749666 | 0 | Not measured |
| Integrated Radeon | 250,000 | 17.433452 | 57.361 | 17.736906 | 0 | Not measured |
| Integrated Radeon | 1,000,000 | 17.244586 | 57.989 | 17.740497 | 0 | Not measured |
| Integrated Radeon | 4,000,000 | 17.308185 | 57.776 | 20.157667 | 0 | Not measured |

The 25 ms column is only an explicitly chosen callback-stall proxy, not a
count of missed 16.667 ms display deadlines. `MAKEPAD_TRACE=present` was set
for the runs but emitted **zero present trace records**. Source inspection
found no caller of `platform/src/present_trace.rs::begin` in this pinned
platform tree. Consequently the largest *accepted tested* N is **4,000,000**
on the 5090 and Radeon, but the largest N proven to hold refresh is
**undetermined**, not four million. Larger counts were not attempted because
the example's deliberately bounded input limit is four million.

Additional backend runs, each at N=250,000 and exit 0:

| Backend / GPU | GPU mean / p95 / max ms | Mean callback ms | Draw recordings | Observation |
| --- | --- | ---: | ---: | --- |
| headless | No GPU timer samples | 16.666640 | 6 | 600 callbacks, backend `Headless`, no display variables |
| offscreen / RTX 5090 | 0.605881 / 0.613689 / 0.628471 | 16.666690 | 38 | Real Vulkan, no display variables |
| offscreen / RTX 3090 | 4.085778 / 4.196167 / 4.435301 | 16.666677 | 33 | Renderer explicitly reported RTX 3090 |

The offscreen callback cadence is a backend timer, not a monitor. Only
250,000 was tested on the 3090; the four-count discrete matrix used the 5090.
The renderer UUID override used for the 3090 applies to the pinned Vulkan
offscreen surface path; it is not claimed to select the visible-window GPU.

## Behavior evidence

### Given retained data, when only the transform changes

Then the 250,000-instance 5090 run finished with only **6 draw recordings**
for **600 callbacks**. Following startup, recordings at frames 120, 240 and
360 were the LOD transitions; uniform updates at frames 121, 241 and 361
reported `changed=true` for both items, without another recording in between.
This demonstrates the uniform-update API path rather than rebuilding N
records for pan/zoom.

### Given a count-zero LOD block, when it is enabled and disabled

Then the final 5090 run recorded item 1 at counts **1, 0, 1** at frames
**120, 240, 360**, respectively. The public upload verdict was
`upload_needed=false` for both items at all three transitions.

There is an important qualification: the pinned HAL's `fb_draw` consumes a
zero-count item without allocating an instance buffer and updates its
retained identity. Thus the initially zero-count LOD's first activation must
create/upload its **64-byte** buffer despite that public verdict. In
`frame_build.rs::fb_instances`, the retained buffer is cached by list/item,
list generation, publication ID and byte count; a zero count does not destroy
that cache. **[INFERENCE]** A warmed 1 -> 0 -> 1 transition with those keys
unchanged reuses the buffer without another instance upload. Actual physical
transfer bytes were **not measured**: the HAL's `FrameStats.bytes_uploaded`
is private and has no public reporting path in this pinned tree. Do not use
the false verdict alone as a zero-upload proof or describe the never-visible
LOD as already physically resident.

Real X11 mouse and key events were also sent using `xdotool`: a 50x20 drag,
a wheel-up click and L. L was logged at frame 22, followed by item 1 count 1
at frame 23; the process completed 600 callbacks and 10 recordings, exit 0.
The inspected capture visibly showed the cyan grid translated from its
original origin. The LOD rectangle was outside that narrow tiled capture.

### Given insufficient admission space, when a publication is requested

With `SPIKE_ENVELOPE_PROBE=1`, a **64-byte** publication requested under an
explicit **1-byte publication envelope** returned:

```text
Some(NoRoom { requested: 64, charged: 0, pending_retirement: 0, envelope: 1 })
PublicationAccounting { charged: 0, pending_retirement: 0, envelope: 1, live: 0 }
```

The separate legacy 50,000-instance retained drawing still completed 600
callbacks, exit 0. This publication-admission envelope is **not** a hard
physical GPU cap and is not the legacy retained API's admission contract.

A second bounded probe set `MAKEPAD_GPU_WORKING_SET=1048576` while drawing
**3,200,000 bytes** of legacy retained instances (N=50,000), offscreen on the
5090. It also completed 600 callbacks, 7 recordings, exit 0. That setting did
not cause this legacy example to reject its block; it must not be presented
as an enforced allocation cap. Physical device-memory exhaustion was **not
run**: both discrete GPUs had live inference workloads near their available
memory and exhausting shared device memory would risk unrelated work.
Behavior beyond the real hardware envelope, including physical OOM recovery,
remains unmeasured. The bounded admission rejection above is the measured
safe pressure result, not a substitute claim about driver OOM.

### Given no display variables, when starting a backend

Then both explicit backend runs reached frame 600 and exited 0. `headless`
reported no GPU and returned no GPU timing samples; `offscreen` reported
Vulkan/RTX 5090 and returned GPU timing samples. Headless startup therefore
does not imply GPU rendering, whereas offscreen startup did exercise Vulkan.

## Exact build and run commands

All commands below ran in `/tmp/w-d6-0`. The first cargo run, before the
wrapper overrides, failed with exit 101 because inherited sccache tried to
create a temporary file under a stale, nonexistent worktree temp directory.
This was an environment failure, not a Rust compiler diagnostic.

```bash
export HOME=/home/lewis
export CARGO_TARGET_DIR=$HOME/.cache/cargo-target-d60
cargo run --release -j2 -p cmdb-ui --example retained_spike -- 250000
```

The corrected environment was used thereafter:

```bash
export HOME=/home/lewis CARGO_TARGET_DIR=$HOME/.cache/cargo-target-d60
export TMPDIR=/tmp RUSTC_WRAPPER= MAKEPAD_TRACE=present
set -o pipefail
timeout 900 cargo run --release -j2 -p cmdb-ui --example retained_spike -- 250000 \
  2>&1 | tee /tmp/d60-release-250000-fixed-env.log
```

That first successful release build finished in 2m57s and the program exited
0. Its then-missing LOD area-redraw request was corrected, so its timing data
is **not** in the final tables. The final release command passed (incremental
compile 44.68s, then 600 callbacks):

```bash
timeout 300 cargo run --release -j2 -p cmdb-ui --example retained_spike -- 250000 \
  2>&1 | tee /tmp/d60-final-nvidia-250000.log
b="$CARGO_TARGET_DIR/release/examples/retained_spike"
for n in 50000 1000000 4000000; do
  timeout 45 "$b" "$n" >"/tmp/d60-final-nvidia-$n.log" 2>&1
done
for n in 50000 250000 1000000 4000000; do
  VK_DRIVER_FILES=/usr/share/vulkan/icd.d/radeon_icd.json \
    timeout 45 "$b" "$n" >"/tmp/d60-final-radeon-$n.log" 2>&1
done
for backend in headless offscreen; do
  env -u DISPLAY -u WAYLAND_DISPLAY MAKEPAD_HAL_BACKEND="$backend" \
    timeout 45 "$b" 250000 >"/tmp/d60-final-$backend.log" 2>&1
done
MAKEPAD_HAL_BACKEND=offscreen SPIKE_ENVELOPE_PROBE=1 \
  timeout 45 "$b" 50000 >/tmp/d60-final-envelope.log 2>&1
MAKEPAD_HAL_BACKEND=offscreen \
  MAKEPAD_VULKAN_DEVICE_UUID=403c2dbf8f1cf0d694a8bd1fa4f6629f \
  timeout 45 "$b" 250000 >/tmp/d60-final-3090.log 2>&1
MAKEPAD_HAL_BACKEND=offscreen MAKEPAD_GPU_WORKING_SET=1048576 \
  timeout 45 "$b" 50000 >/tmp/d60-final-working-set.log 2>&1
```

Every process in that matrix reported exit 0. Timing summaries were computed
with awk and numeric sort from the `timing` and `cadence` records, selecting
`frame>=60`; the summary is `/tmp/d60-statistics.txt`. These raw logs and
input screenshots are local ephemeral evidence, not tracked repository files.
The diagnostic `grep` for present records returned 1 because there were no
matches; this is the reported instrumentation limitation, not a claimed pass.

The first successful real-input command sequence was:

```bash
MAKEPAD=x11 "$b" 250000 >/tmp/d60-input-x11.log 2>&1 &
child=$!
trap 'kill "$child" 2>/dev/null || true' EXIT
window=$(timeout 8 xdotool search --sync --pid "$child" | head -n 1)
xdotool mousemove --window "$window" 300 300 mousedown --window "$window" 1 \
  mousemove --window "$window" 350 320 mouseup --window "$window" 1 \
  click --window "$window" 4 key --window "$window" l
import -window "$window" /tmp/d60-input-x11.png
wait "$child"
trap - EXIT
```

Two later attempts to control window geometry using old Hyprland dispatcher
strings and then `hl.dispatch('name', 'arguments')` failed with the installed
Lua dispatcher API. Their process exits were 0 but they are **not** geometry
or input successes; one screenshot was black before mapping. No controlled
full-window measurements are claimed from these attempts.

A further streaming-input capture helper ran the example using a Bash
coprocess and attempted `xdotool search --onlyvisible --pid "$child"` at
callback 60, then `import -window "$window"` at callback 90. Its window lookup
did not yield a usable window: X11 reported `BadWindow`, resource `0x0`, and
the helper timed out after 45 seconds. Its log stopped at callback 90 and is
`/tmp/d60-input-visible.log`. This is a failed capture-helper run, not a
successful 600-callback run; it is excluded from all measurements above.

## Acceptance disposition and compatibility

- Shader slots, stride, readback and returned list/item handles: recorded
  above from the running final example.
- Integrated/discrete four-count runs and real GPU timings: recorded above.
- Largest accepted tested N: 4,000,000 on each matrix GPU, 256,000,000 payload
  bytes. Largest N holding refresh and missed presents: **not measured** for
  the presentation-instrumentation and display reasons above.
- Zero-count restoration: binding counts and public verdicts observed;
  warmed physical zero-reupload is source-backed inference, not measured.
- Past-envelope behavior: publication refusal and the bounded working-set
  probe recorded; real physical OOM **not run** to protect shared workloads.
- Headless and offscreen startup: both passed with no display variables.
- D6.1 may use retained blocks at this pin, but must not assume first
  zero-count activation is upload-free. If retained drawing becomes unusable,
  the planned fallback remains `begin_many_instances` plus visible-group
  culling, with a changed D6.1 budget; this spike did not need that fallback.
- No production app code, dependencies, catalog output, JSON/JSONL or CI
  files changed. The docs' statement that this machine has no display was
  contradicted by the observed active display and GPUs. The prompt-linked
  `docs/VISION.md` was absent; available tree instructions were used.
- During the initial evidence run, tests, linters, formatters and broad builds
  were **not run**, per delegated verification ownership. D6.0 specifies no
  tests, only evidence. The later, explicitly authorized repair checks below
  supersede that statement for formatting and the narrow example lint/build.

## Follow-up lint and formatting repair

The coordinator reported four example clippy errors. Argument parsing now
matches the optional argument directly (no unwrap-family calls), grid
coordinates use `checked_rem` and `checked_div` with propagated errors, and
pointer subtraction uses the pinned vector's scalar f64 components before
conversion. Non-finite converted deltas are explicitly diagnosed and ignored.
The pinned `libs/math/src/math_f64.rs` conversion and subtraction definitions
were read before this change.

With coordinator approval, the single example now has a private module at
`crates/cmdb-ui/examples/retained_spike/support.rs` for block construction,
admission probing and layout validation. Its reflected offsets use the
layout API's `u32` type. The main example is 247 Rust lines, support is 160
Rust lines, and an awk brace-span count found every Rust function at most
25 lines. The support module is not a second Cargo example.

The first repair fmt check failed on the original compact formatting.
Clippy then passed the four arithmetic/argument repairs. After splitting,
one intermediate check caught an offset-constant `usize`/`u32` mismatch and
two formatting differences; all were corrected without lint suppression.
The following final commands all passed, with the same HOME, private target,
TMPDIR and wrapper overrides documented above:

```bash
cargo fmt -p cmdb-ui --check
cargo clippy -p cmdb-ui --examples --locked -j2 -- -D warnings
cargo build --release -p cmdb-ui --example retained_spike --locked -j2
env -u DISPLAY -u WAYLAND_DISPLAY MAKEPAD_HAL_BACKEND=offscreen \
  MAKEPAD_TRACE=present timeout 45 \
  "$CARGO_TARGET_DIR/release/examples/retained_spike" 250000 \
  >/tmp/d60-repair-smoke.log 2>&1
```

Final clippy finished in 0.19s; release build finished in 50.15s. The single
smoke run reported Vulkan/RTX 5090, 250,000 instances, 16,000,000 payload
bytes, LOD states true/false/true at frames 120/240/360, and
`complete frames=600 draws=7`, exit 0. Gate output is
`/tmp/d60-repair-checks.log`. This smoke is not a new performance matrix:
the earlier timing tables remain the initial measured release observations.
No unit tests, broad builds or additional performance sweeps were run for
this repair.
