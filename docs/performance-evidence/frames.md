# Frame harness: current painter (D7.1)

## Status

The harness implementation is supplied; release/GPU baselines are **not run**.
`docs/e2e/AGENT_PROMPT.md` delegates Cargo execution, integration checks and
measurements to the coordinator. No frame-time, throughput or headroom result
is claimed here. D7.1's measured-baseline completion gate remains unsatisfied
until the coordinator records the runs below.

The painter is unchanged. The harness reads the current SQLite reader database
through `cmdb-view`; PostgreSQL URL support belongs to the later reader cutover.
The generator currently writes schema 15 and the reader accepts schemas 15–17.
`docs/VISION.md`, requested by the kickoff prompt, is absent from this checkout.

## Captured quantities and limitations

The Makepad revision is `506ec05105b9adb08d1d21f9879df181b5244ea7`.

| Quantity | Meaning at this pin |
| --- | --- |
| Presented frames | Delta of `frame_trace::presented_frames()`: all-window compositor handoffs during capture, not timestamps of visible glass updates. |
| Per-frame presented timestamp | `unavailable`: `event/hal_intake.rs` discards `FrameDone.present_ns`; no public timestamp consumer exists. |
| Presented missed count/share | `unavailable`: cannot apply the 1.5-refresh-period rule without presented timestamps. Draw cadence is not substituted. |
| Present-topic 1 Hz aggregate | `unavailable`: `platform/src/present_trace.rs` has no active producers at this pin. |
| Frame-topic 2-second aggregate | `unavailable`: `FrameTrace` is declared but never constructed or called. Enabling a trace topic cannot repair missing producers. |
| GPU busy milliseconds | Completed samples from `gpu_frame_timer::take_frames()`, enabled only in harness mode. Samples have no timestamp or frame ID; their receipt time is recorded and they are not correlated with UI samples. Final in-flight GPU work is excluded. An empty series prints an explicit unavailable reason. |
| UI app-dispatch milliseconds | An app `perf_monitor` channel accumulates measured `App::handle_event` dispatch time between completed draws. The pinned platform never calls `perf_monitor.frame_boundary`, so the harness supplies boundaries. This excludes backend encoding, framework work outside the App handler, and capture bookkeeping after dispatch timing is taken; it is **not total UI-thread CPU time**. |
| Draw cadence milliseconds | Wall-clock intervals between completed App draw dispatches; neither CPU cost nor presented-frame intervals. A separately named count/share can compare these intervals with an operator-stated refresh period. |
| Visible labels | Last/min/max rendered label count, using the current painter's scale threshold, 48-world-unit visibility margin and 200-label budget. |

GPU, UI and cadence samples print as
`frame_sample,<series>,<elapsed_ms>,<milliseconds>` before their summary.
GPU elapsed time means **receipt time**, not GPU start or presentation time.
Percentiles use the nearest-rank definition; empty summaries print
`unavailable`. The missed-interval helper uses a strict `> 1.5` periods and
normalizes by the number of observed intervals, not by an inferred refresh count.

These limitations were approved by the coordinator for this pin. D7.2 cannot
claim its presented-frame missed-share or total-UI-thread budgets using the
currently exposed data. That requires backend instrumentation, not an estimate.

## Workload and bounds

`--database PATH --frame-harness SECONDS` accepts a finite duration from 1 to
600 seconds. After the loaded DAG's first complete draw, elapsed wall time drives
whole-view → one-node zoom → whole-view, then a zoomed pan from the first to the
last layout node. Three scripted clicks use the real hit-test/selection path at
20%, 60% and 90%; skipped updates retain all due clicks. Attempted and successful
click counts are both reported. Selection does not start inventory expansion
queries while the fixed harness graph is active.

The whole-view transform can go below the normal interactive minimum scale so
that the complete bounded DAG actually fits. The subsequent pan ramps to at
least normal 1× scale. Neither the painter nor interactive zoom limits change.

The worker loads a consistent read-only snapshot of at most 2,000 resources,
paged with the existing reader page bound, and up to 4,000 induced edges between
those resources. Traversals and database startup retain explicit bounds and the
existing 10-second startup timeout. Source resource/edge totals and visual caps
are printed, so input truncation is visible. Edges beyond the per-node bounded
edge-page traversal or between resources outside the loaded subset are not
rendered. The generator's synthetic fixture has zero edges; no invented edges
or grouping nodes are added to inflate a baseline.

Each of the three sample series reserves 262,144 entries before capture. Reaching
the budget or receiving invalid timing stops capture and marks the report.
The pinned GPU API itself allocates its returned batch; the monitor requires
copying its bounded 240-frame history to read the newest app channel sample.
No threads, detached tasks, render loop or timing estimates are introduced.
Normal framework shutdown stops and joins the existing data worker. A valid
harness invocation reports and exits normally with status 0, including a reported
database-startup failure; command-line syntax errors retain the existing status 2.

## Observed machine/display metadata (not a performance run)

On 2026-11-22, these commands returned successfully:

```sh
hostname
nvidia-smi --query-gpu=name,driver_version,display_active --format=csv,noheader
xrandr --current
```

- Host: `omarchy`; `DISPLAY=:0`, `WAYLAND_DISPLAY=wayland-1`.
- RTX 5090: NVIDIA driver `615.78.08`, display active.
- RTX 3090: NVIDIA driver `615.78.08`, display inactive.
- Xwayland reports `HDMI-A-1`, `1920x1080`, active mode `59.96 Hz`.
  `xrandr` warns that it is querying Xwayland; this is operator-stated metadata,
  not a verified physical monitor refresh measurement.
- No 120 Hz mode is listed by that query. A 120 Hz run is **not run** for that
  reason. Actual selected GPU/backend, window size and effective scale must come
  from a real harness report; they are not inferred from the installed cards.
- Vsync, power and thermal state have not been measured.

## Coordinator measurement commands

Run from the D7.1 worktree. Use the already-built `cmdb-bench` generator executable;
all Cargo commands remain coordinator-owned. Record actual metadata at run time.

```sh
export HOME=/home/lewis
export CARGO_TARGET_DIR="$HOME/.cache/cargo-target-d71"
cargo build --release -j2 -p cmdb-ui
mkdir -p "$CARGO_TARGET_DIR/frame-baselines"
for count in 500 2000 2001; do
  cmdb-bench generate "$CARGO_TARGET_DIR/frame-baselines/$count.sqlite" 1 "$count"
done
export CMDB_UI_FRAME_REFRESH_HZ=59.96
export CMDB_UI_FRAME_RESOLUTION=1920x1080
export CMDB_UI_FRAME_DRIVER=615.78.08
export CMDB_UI_FRAME_MACHINE=omarchy
for count in 500 2000 2001; do
  "$CARGO_TARGET_DIR/release/cmdb-ui" \
    --database "$CARGO_TARGET_DIR/frame-baselines/$count.sqlite" \
    --frame-harness 20 \
    > "$CARGO_TARGET_DIR/frame-baselines/$count.log" 2>&1
done
```

`CMDB_UI_FRAME_VSYNC`, `CMDB_UI_FRAME_POWER` and `CMDB_UI_FRAME_THERMAL` are optional
operator statements, each limited to 256 bytes; unset values print `unavailable`.
Refresh, resolution, driver and machine use the same stated/unavailable labeling.
GPU/vendor/backend are read from Makepad, and window geometry/scale from its
window-geometry event. Preserve the complete logs, exit codes and all report
fields. Do not compare a 60 Hz draw cadence with a 120 Hz throughput claim.

### Baseline execution ledger

| Input resources | Intended visual nodes | Result |
| ---: | ---: | --- |
| 500 | 500 | Not run: coordinator-only release build/run. No timing fields measured. |
| 2,000 | 2,000 (current painter cap) | Not run: coordinator-only release build/run. No timing fields measured. |
| 2,001 | At most 2,000; overflow probe | Not run: largest accepted/rendered size is not experimentally established. |

This ledger is explicitly **not measured baseline evidence**. The cap is known
from `dag/graph.rs`, not from a performance run.

## Checks to execute after integration

Not run by this worker: build, formatting, lints, tests, GPU/display runs.
The new scenarios cover CLI-to-worker harness configuration, invalid duration and
sample bounds, nearest-rank percentiles, missed-interval arithmetic, deterministic
elapsed-time path and retained clicks, current-event delivery, and a real
schema-backed SQLite inventory-to-DAG startup with actual resource labels/edges.

```sh
cargo test -j2 -p cmdb-ui frame_harness -- --test-threads=1
```

The release capture commands above remain the end-to-end draw/shutdown evidence;
unit scenarios do not stand in for a real GPU run.
