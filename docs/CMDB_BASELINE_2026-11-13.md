# Preliminary release baseline: local storage and scheduler

Measured 2026-11-13 on source commit
`57ee7de687be3f445a39bf2e2eb81807e5d68959`. Three sequential runs of the existing
`inventory-engine` benchmark passed. No production source changed for this run.
Raw results and provenance are in `performance-evidence/2026-11-13/`.

## Results

Each storage case writes 1,000 synthetic resources and 1,000 relationships to a
new SQLite database. Every run verified the exact persisted counts. SQLite uses
WAL and `synchronous=FULL`. Timing includes fixture construction, page ingestion,
and scan finalization; database open/schema initialization and scan start are
outside that stage timer, but included in external process wall time.

| Existing storage configuration | Minimum resources/s | Median resources/s | Maximum resources/s |
| --- | ---: | ---: | ---: |
| 1 resource + 1 edge per transaction | 765 | 821 | 1,646 |
| 100 resources + 100 edges per transaction | 17,280 | 25,035 | 34,904 |

This shows the existing batch mode has substantially lower transaction overhead
on this fixture. It is not a before/after optimization or an AWS collection result.
Large storage variation requires longer, controlled campaigns before regression
thresholds or capacity estimates are justified.

The scheduler case executes 512 mock operations, each sleeping for a requested
2 ms, with one independent quota bucket per operation and a configured 100,000
requests/s ceiling. This avoids shared quota contention and exercises bounded
concurrency rather than realistic AWS quota behavior.

| Global concurrency | Minimum mock operations/s | Median mock operations/s | Maximum mock operations/s |
| --- | ---: | ---: | ---: |
| 1 | 326 | 326 | 327 |
| 8 | 2,604 | 2,607 | 2,614 |
| 64 | 20,395 | 20,675 | 20,709 |

All scheduler cases reported 512 attempts and successes, zero failures/retries/
throttles/timeouts, and zero requests in flight at completion. External wall time
for each whole process was 2.50–3.29 seconds; peak process RSS was
15,104–16,732 KiB. These memory figures cover this small example, not the full
SDK catalog, serving application, database page cache, or a distributed service.

## Provenance and reproduction

- CPU: AMD Ryzen 9 9950X3D, 16 cores / 32 logical CPUs.
- RAM: 129,392,836 KiB total; approximately 64,060,664 KiB available near the run.
- Filesystem: `/home`, Btrfs, SSD option, Zstd compression level 3. Benchmark
  databases used `artifacts/` on that filesystem, not `/tmp` tmpfs.
- Shared host, not isolated: sampled load averages were 2.91 / 4.45 / 6.80.
  No other workloads were stopped and no host performance settings changed.
- Rust: `1.102.0-nightly (9c5648833 2026-11-10)`, LLVM 23.1.5,
  `x86_64-unknown-linux-gnu`.
- Release settings: opt-level 3, thin LTO, one codegen unit, stripped symbols;
  four Tokio runtime worker threads. No custom optimization flags were added.
- Cargo.lock SHA-256:
  `a02e00faf32f61ed82be6e85c5f8be24004fad835a7be7cb0a675332acf6e1a6`.
- Executed benchmark binary SHA-256:
  `5100963acb496ed1adbd4b40dc4c05eb0d2ae06b4d9a972ccdbd04badff92ee6`.

Build from the repository root:

```sh
env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
  cargo build --release -p inventory-engine --example benchmark --locked -j1
```

The actual build ran in a user systemd scope capped at 16 GiB memory, with
`MemoryHigh=12G` and swap disabled for that scope; it completed successfully in
1 minute 39 seconds. The measurements then ran directly, sequentially:

```sh
LC_ALL=C /usr/bin/time \
  -f '{"wall_seconds":%e,"cpu_user_seconds":%U,"cpu_system_seconds":%S,"peak_rss_kib":%M}' \
  -o run-1-process.json \
  target/release/examples/benchmark 1000 > run-1.json 2> run-1.stderr
```

Repeat with separate filenames for runs 2 and 3. Keep each exit status and raw
JSON. See `summary.json` for unrounded min/median/max and `validation.json` for
the independently checked invariants.

## What remains unmeasured

No real AWS calls, fleet collection, shared quota contention, provider change
delivery, historical database growth, realistic large payload distributions,
query latency, browser rendering, crash/recovery durability, allocation profile,
energy/cost, competitor runs, or soak test was measured. Case order was fixed,
there was no dedicated warm-up or hardware isolation, and only three short
samples were collected. No p95/p99, market ranking, or fleet capacity claim is
supported. The associated performance contract is a draft proposal.
