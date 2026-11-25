# Track B steps: speed

> Status: Plan, 2026-11-15, revised for V53 (PostgreSQL is the only store).
> Step-by-step breakdown of the packages in [B_SPEED.md](../B_SPEED.md),
> reordered by V52: the baseline (B0) and the benchmark (B5) come first, and
> every speed step records a number before and after. Packages B6 to B9 are
> added by this plan for speed work the tree supports but B_SPEED.md does not
> list. B4 is the PostgreSQL write path; B0.1 and B4.2 were removed by V53. No step here
> claims a speedup; each states what is measured and what it is compared with.

Conventions: as in [STEPS_A.md](STEPS_A.md). Measurement names P1 to P17 and
the benchmark profiles (`thin`, `full`, `proxy`) are defined in
[BUILD_PLAN.md](../BUILD_PLAN.md). "Record" means: add a row to
`docs/performance-evidence/sweep-benchmark.md` (new, created by B5.3) with the
step ID, the commit, the machine, the build profile and the command.

After D1.2 the sweep runtime lives in `crates/sweep/src/` (package
`inventory-sweep`). Paths below use the new location; the same files are under
`src/sweep_runtime/` today.

## B0 Baseline

B0.1 was removed by V53 (see BUILD_PLAN). The first recorded number is B0.2,
on PostgreSQL, at T0. Every speed step compares against it or against a later
row.

### B0.2 Release baseline on PostgreSQL through the benchmark

- **Goal:** The first row of the benchmark table: the engine on PostgreSQL
  before any speed step, in a release build, on the `thin` profile.
- **Depends on:** D1.1 (the sweep builds without the 19 SDK crates, so a
  release build fits in this machine's memory), B5.2, B5.3, A4.5 (bulk seed),
  C6.10.
- **Files:** `docs/performance-evidence/sweep-benchmark.md`.
- **Change:** run `cmdb-bench sweep --profile thin --runs 3` on a release
  build and record P1 to P10 with the server settings (B5.2). At this point
  the store is C6.6's first form (`UNNEST`, one transaction per page,
  `resource_scopes` still rewritten), which is the "before" for C6.7 and B4.
  Record the row again when A2.2 widens the catalog.
- **Tests:** none.
- **Prove it:** `cargo build --release --locked --features sweep -j1` succeeds
  within the 6 GiB limit; the command prints the table; the row is in the
  file.
- **Done when:** the row is in the table with machine, build profile and
  server settings.
- **Size:** no Rust. One change.
- **Watch out:** `[profile.release]` uses `lto = "thin"` and `codegen-units =
  1`; the build is slow. Build once and pass the binary path to the benchmark.
  This machine has no working container runtime, so its rows come from a
  native server configured like the compose file (C6.2); say which in the row.

## B5 Sweep benchmark (built first)

### B5.1 The sweep reports what it did

- **Goal:** The sweep prints the counters the benchmark needs. Without them the
  benchmark could only see wall time.
- **Depends on:** D1.3.
- **Files:** `crates/sweep/src/metrics.rs` (new), `crates/sweep/src/lib.rs`
  (`SweepSummary`), `crates/sweep/src/call.rs`,
  `crates/sweep/src/execution/page.rs`, `crates/sweep/src/execution.rs`,
  `src/sweep_command/enabled.rs` (`print_summary`),
  `crates/aws-wire/src/transport/counters.rs`.
- **Change:**
  1. `pub struct SweepMetrics` (new) with `u64` fields: `bootstrap_ms`,
     `seed_ms`, `drain_ms`, `api_attempts`, `api_retries`, `api_throttles`,
     `connections_opened`, `requests_sent`, `requests_reused`, `bytes_read`,
     `pages`, `decode_ns`, `store_commit_ns`, `store_commits`, `claims`,
     `cpu_user_ms`, `cpu_system_ms`, `peak_rss_kb`. `SweepSummary` gains
     `metrics: SweepMetrics`.
  2. Sources: `ApiExecutor::stats()` (`attempts`, `retries`, `throttles`);
     `Client::statistics()` (`connections_opened`, `requests_sent`,
     `requests_reused`, `bytes_read`); two `AtomicU64` accumulators in
     `metrics.rs` updated around `aws_wire::response::decode` in
     `call.rs::decode_response` and around `commit_page` in `page.rs`, with
     `std::time::Instant`.
  3. CPU and peak memory: read `/proc/self/stat` (fields 14 and 15, in clock
     ticks of 100 per second) and the `VmHWM` line of `/proc/self/status` at
     the end of the run. On a system without `/proc` the three fields are 0 and
     the line prints `process_metrics=unavailable`.
  4. `Statistics` gains `http2_requests: usize`, counted in `Client::exchange`
     from `parts.version == Version::HTTP_2`.
  5. `print_summary` prints a second line, `metrics key=value ...`, after the
     existing summary line. The first line is unchanged; `tests/sweep_e2e`
     parses it.
- **Tests:** `/proc` parsers on fixed text, including a process name with
  spaces and parentheses in field 2; counters add up across concurrent pages;
  the first summary line is byte-identical for the same run.
- **Budget:** P1 and P6 on the `thin` profile must not rise by more than the
  noise band B5.3 states. Timing every decode costs two `Instant::now` calls
  per page.
- **Prove it:** `aws-inventory sweep ...` prints both lines; the e2e baseline
  passes.
- **Done when:** every field is non-zero in a real run except the three that
  depend on `/proc` on other systems.
- **Size:** about 280 added. One change.
- **Watch out:** `libc::getrusage` needs `unsafe`, which is forbidden; that is
  why `/proc` is read. No `as` casts: convert ticks with `checked_mul` and
  `u64::try_from`. `decode_ns` as a share of `cpu_user_ms` is P10, the number
  that could bring page hashing back (V33).

### B5.2 `cmdb-bench sweep` and `cmdb-bench open`

- **Goal:** One command that owns the emulator and a fresh database, seeds,
  runs a cold and a warm sweep with the shipped binary, and prints the table
  together with the settings of the server it ran on.
- **Depends on:** B5.1, A4.5, A7.1 (the shared stack code in the harness),
  C6.10.
- **Files:** `crates/cmdb-bench/Cargo.toml` (dependency on
  `inventory-harness`), `crates/cmdb-bench/src/main.rs` (288 lines; move
  `generate` and `measure` into `src/generate.rs` and `src/measure.rs`, new),
  `crates/cmdb-bench/src/sweep.rs` (new), `src/sweep/table.rs` (new),
  `src/sweep/server.rs` (new), `src/open.rs` (new).
- **Change:**
  1. `cmdb-bench sweep --collector <path> --profile thin|full [--runs <n>]
     [--proxy latency_ms=<n>,rps=<n>] [--out <file>]`. It starts the emulator
     and graph server through `inventory_harness::e2e::stack`, seeds with
     `crates/harness/seeds/bench` (`thin`) or all seed files (`full`), then
     for each run: a new `TestDatabase`, cold sweep, warm sweep; parse both
     summary lines; drop the database.
  2. Output, one row per measurement with min, median and max over the runs:
     P1 cold wall ms, P2 warm wall ms, P3 attempts, retries and throttles, P4
     connections opened and requests reused, P5 resources per second (rows
     divided by cold wall time), P6 CPU seconds of the collector, P7 its peak
     RSS, P8 bytes written and rows written on warm, P9 commits and writer
     rows per second (rows divided by `store_commit_ns`), P10 `decode_ns`
     divided by user CPU time.
  3. Bytes written: `pg_database_size(current_database())` after the cold
     sweep, and the difference of `pg_current_wal_lsn()` across each sweep
     (`pg_wal_lsn_diff`). Rows written on warm: the differences of
     `n_tup_ins`, `n_tup_upd` and `n_tup_del` summed over
     `pg_stat_user_tables`, split into logged and unlogged tables.
  4. `server.rs`: every table is headed by what the server reports, read with
     `SHOW` and `current_setting`: `server_version`, `shared_buffers`,
     `effective_cache_size`, `work_mem`, `max_wal_size`, `checkpoint_timeout`,
     `synchronous_commit`, `fsync`, `full_page_writes`, `wal_level`,
     `max_connections`, `max_locks_per_transaction`. A flag `--server
     container|native|managed` and `--volume named|bind|native-disk` are
     recorded as given (the benchmark cannot detect them). If `fsync` is off
     the run is marked "not publishable".
  5. `cmdb-bench open --catalog <file>`: P11. Time
     `RuntimeCatalog::from_bytes`, schema creation in a new `TestDatabase`,
     reopen (connect and load the registry), and preparing one insert per kind
     table; print table count and column count.
  6. Other header lines: machine (`/proc/cpuinfo` model, core count, memory),
     operating system, build profile of the collector (`--build
     release|debug`), commit, catalog SHA-256, emulator version, seed counts.
- **Tests:** table arithmetic (median of 1, 2 and 3 runs); summary-line parser
  rejects a line with a missing key; `--runs 0` refused; `--proxy` parser;
  settings reader against a `TestDatabase`.
- **Prove it:** `cmdb-bench sweep --collector target/release/aws-inventory
  --profile thin --runs 3` prints the table with the server block.
- **Done when:** the command runs unattended and exits non-zero if any sweep
  did.
- **Size:** about 520 added, 200 moved. Two changes: (1) split `main.rs`,
  (2) the commands.
- **Watch out:** `crates/cmdb-bench/src` is under the under-60-line function gate.
  Published numbers come from Linux: Docker on macOS runs in a virtual machine
  and is slower on disk, more so with a bind mount, so a macOS row is marked
  as such and never compared with a Linux row. Container defaults are small
  (`shared_buffers` 128MB); that is why the settings are printed with every
  table and set explicitly in `deploy/local/compose.yaml`. The collector's
  CPU and memory are measured, not the server's; the server shares this
  machine's 2 CPUs with the emulator, so say so.

### B5.3 The recorded table and the regression rule

- **Goal:** A committed table with one row per step, and a command that says
  whether a number regressed.
- **Depends on:** B5.2.
- **Files:** `docs/performance-evidence/sweep-benchmark.md` (new),
  `crates/cmdb-bench/src/compare.rs` (new), `docs/performance-evidence/runs/`
  (new directory; raw `--out` files).
- **Change:**
  1. `cmdb-bench sweep --out <file>` writes the table as tab-separated text
     with the header lines.
  2. `cmdb-bench compare <before> <after> [--band <percent>]` prints each
     measurement with its change, and exits 1 if any median got worse by more
     than the band. Default band: 5%, or twice the spread (max minus min,
     relative to the median) of the "before" runs if that is larger. "Worse"
     is higher for P1, P2, P3, P6, P7, P8, and lower for P5 and P9.
  3. The table file has columns: step, date, commit, server, profile, P1 to P10
     medians, raw file, note. A regression row needs a note with the reason.
  4. Wave rule (BUILD_PLAN): a wave's checkpoint includes `cmdb-bench compare`
     of the wave's first and last raw files.
- **Tests:** band arithmetic; direction per measurement; two files with a
  different profile or a different server settings block are refused unless
  `--allow-different-server` is given, and the output then says so; a missing
  measurement is an error.
- **Prove it:** `cmdb-bench compare` of two runs of the same binary exits 0.
- **Done when:** B0.2's row is the first row of the table.
- **Size:** about 220 added. One change.
- **Watch out:** this machine has 2 CPUs and shares them with the emulator and
  PostgreSQL, so run-to-run spread may be large. If the spread exceeds 20%,
  raise `--runs` and say so; do not narrow the band by hand.

### B5.4 The proxy profile: latency and a rate limit

- **Goal:** Measure the parts the emulator cannot show: pacing against a quota
  and connection reuse under latency.
- **Depends on:** B5.2.
- **Files:** `crates/cmdb-bench/src/sweep.rs`, `src/sweep/proxy.rs` (new);
  uses `inventory_harness::proxy::{Config, RateLimit, ScopeFilter}`.
- **Change:** `--proxy latency_ms=<n>,rps=<n>` starts the harness fault proxy
  in front of the emulator with a fixed added latency and one `RateLimit` per
  scope filter, and points the collector's `--endpoint` at the proxy. The
  stated default profile `proxy` is latency 50 ms and 20 requests per second
  per service and region. Extra rows: steady throughput in requests per second
  over the last half of the run, throttles as a share of responses after the
  first 30 seconds, and time to reach 90% of the limit.
- **Tests:** throughput and share arithmetic on a recorded proxy log; the
  proxy's seed is fixed so the fault sequence repeats.
- **Prove it:** `cmdb-bench sweep --profile thin --proxy latency_ms=50,rps=20
  ...` prints the extra rows.
- **Done when:** a baseline row for the `proxy` profile is recorded before
  B1.1 lands.
- **Size:** about 200 added. One change.
- **Watch out:** the proxy's rate limit is per account, region and service
  (M4.3). The emulator presents one region; a second region for B1's proof
  comes from the proxy's `ScopeFilter`, not from the emulator.

### B5.5 Scale run and the final table (X9)

- **Goal:** The benchmark at the largest size the emulator allows, and the
  completed table with before and after for every speed package.
- **Depends on:** A7.2 (full coverage), B1 to B4, B6 to B8.
- **Files:** `docs/performance-evidence/sweep-benchmark.md`.
- **Change:** run `--profile full` with the bulk seed raised until either
  1,000,000 API calls are made or the emulator refuses more; state the limit
  reached. Record peak memory and the decode share. Check that every step B1.1
  to B8.2 has a before row and an after row.
- **Tests:** none.
- **Prove it:** the command and the table.
- **Done when:** X9: the table exists, from a release build, and no number
  regressed without a written reason.
- **Size:** no Rust. One change.
- **Watch out:** the write settings are settled by the rows of B4.1 and B4.4,
  on the server named in each table. Do not restate them here without the
  rows.

## B1 Buckets per account, region and quota group

### B1.1 The bucket key

- **Goal:** Pace each account, region and quota group independently. Today the
  key is the service name (`call.rs::Call::run`), so all regions share one
  budget.
- **Depends on:** D1.2, A3.4 (extension version 2 already in place).
- **Files:** `crates/sweep/src/call.rs`, `crates/sweep/src/bucket.rs` (new),
  `crates/catalog-tool/src/runtime_catalog/plans/compile.rs`,
  `crates/core/src/catalog/runtime/types/plan.rs`,
  `crates/core/src/catalog/runtime/decode/plans.rs`.
- **Change:**
  1. Runtime plan field 15, optional text `quota_group`, written from
     `OperationOverlay::quota_group` when set. `CallablePlan::quota_group(&self)
     -> &str` returns it or the service name.
  2. `bucket.rs`: `pub(crate) fn key(account: &AccountId, region: &Region,
     group: &str) -> String` giving `<account>/<region>/<group>`, with `global`
     for `Region::Global`. The key is at most 12 + 1 + 63 + 1 + 32 bytes, well
     under `MAX_BUCKET_KEY_BYTES` (512).
  3. `Call` gains `bucket: &str`; `page.rs::process_lease` computes it from the
     lease's scope. The two bootstrap calls use the home region and the
     literal account `bootstrap` (the account is not known before
     `GetCallerIdentity` answers).
- **Tests:** key format; global scope; two regions give two keys; overlay
  `quota_group` reaches the plan; reader rejects a non-text field 15. Engine
  test with `tokio::time::pause`: two keys for one service are each admitted at
  their own rate, and a throttle on one does not slow the other.
- **Budget:** P1 on `thin` must not rise. On the `proxy` profile with two
  region filters, record steady throughput before and after: before, both
  regions share one bucket; after, each has its own.
- **Prove it:** `cargo test -p inventory-engine --locked --test limiter`;
  `cargo test -p inventory-sweep --locked`; the two benchmark rows.
- **Done when:** the rows are recorded.
- **Size:** about 200 added. One change.
- **Watch out:** `lease_budget` in `execution.rs` derives the lease time from
  the global concurrency and the rate. More buckets do not change it, but B2.1
  does; keep the formula in one place.

### B1.2 Seventy-four thousand buckets

- **Goal:** 10 accounts, 17 regions and 436 services make about 74,000
  buckets; `MAX_BUCKETS` is 65,536 and a bucket is never removed.
- **Depends on:** B1.1.
- **Files:** `crates/engine/src/limiter.rs`, `limiter/registry.rs`,
  `limiter/bucket.rs`, `limiter/stats.rs`.
- **Change:**
  1. `Bucket` gains `last_used: AtomicU64` (milliseconds on the Tokio clock),
     updated in `try_start`.
  2. When the registry is full, `ApiExecutor::bucket` removes buckets that
     have no in-flight permit taken, no waiter, no cooldown and no rejection,
     oldest `last_used` first, up to 1,024 per sweep of the map, then inserts.
     If none can be removed it still returns `ApiError::Exhausted`.
  3. An evicted bucket's learned rate is kept in a side map `learned:
     HashMap<String, u32>` bounded at 262,144 entries, so a bucket recreated
     later starts from it (B2 reads the same map).
  4. `ExecutorStats` gains `evicted_buckets`.
- **Tests:** 74,000 distinct keys admitted in sequence without error; a bucket
  with an in-flight call is never evicted; an evicted bucket comes back with
  its rate; the side map's bound holds.
- **Budget:** P7 (peak memory) with 74,000 buckets created is recorded: bytes
  per bucket. Command: an engine bench test that creates them and reads
  `/proc/self/status`.
- **Prove it:** `cargo test -p inventory-engine --locked --test limiter`.
- **Done when:** the 74,000-bucket test passes and the memory number is
  recorded.
- **Size:** about 220 added. One change.
- **Watch out:** the registry is one `Mutex<HashMap>`; eviction runs under it.
  Bound the work per call (1,024) so admission never stalls on a long scan.
- **Scale (V76):** build this for 74 million keys, not 74,000. A bucket is
  created on first use and dropped when idle, so memory follows the accounts
  in flight. One lock over one map will not carry massive scale; shard the
  registry by key hash now, while it is being rewritten. S1.2 then only adds
  the moving window of accounts.

## B2 Adaptive rate with no low ceiling, persisted

### B2.1 A rate that can rise

- **Goal:** `AdaptiveRate::PerSecond { current, maximum }` starts with both at
  5, so recovery can never pass 5. Add a policy that starts at a configured
  rate, rises until throttled, cuts on a throttle and holds just under.
- **Depends on:** B1.1.
- **Files:** `crates/engine/src/limiter/policy.rs` (193 lines),
  `limiter/pacing.rs` (165 lines), `limiter/adaptive.rs` (new),
  `crates/sweep/src/execution.rs` (`executor_config`).
- **Change:**
  1. New `BucketPolicy::Adaptive(AdaptiveLimits)` with `pub struct
     AdaptiveLimits { pub start_rps: u32, pub floor_rps: u32, pub ceiling_rps:
     u32, pub concurrency: usize }`, validated `1 <= floor <= start <= ceiling
     <= MAX_REQUESTS_PER_SECOND`.
  2. New `AdaptiveRate::Seeking { current_mrps: u64, floor_mrps: u64,
     ceiling_mrps: u64, last_throttle_mrps: Option<u64> }` in milli-requests
     per second. `recover`: add 1,000 divided by the current whole rate (so one
     more request per second after about one second of successes), and when
     `last_throttle_mrps` is set, do not pass 90% of it until 60 seconds of
     successes have gone by, then clear it. `throttle`: set `last_throttle_mrps
     = current`, multiply `current` by 7 and divide by 10, not below the
     floor.
  3. `PerSecond` and `PerPeriod` keep their behavior. The old `scan` path and
     the test `adaptive_recovery_stops_at_the_bucket_policy_ceiling` keep
     passing unchanged.
  4. The sweep's `executor_config` builds the executor with
     `with_rate_policies` and a default policy of `Adaptive { start_rps: 10,
     floor_rps: 1, ceiling_rps: cap }`, where `cap` is `--max-rps` (B2.3) or
     `MAX_REQUESTS_PER_SECOND`. `ApiExecutor` needs a default policy for
     unknown keys: add `default_policy: BucketPolicy` to the constructor used
     by the sweep (`with_default_policy`, new).
  5. `ApiExecutor::bucket_rates(&self) -> Vec<(String, u32, u64)>` (new):
     key, whole current rate, sample count, for B2.2.
- **Tests:** with paused time: rate rises from start with successes; one
  throttle cuts to 70%; the 90% hold lasts 60 seconds; floor and ceiling hold;
  arithmetic never overflows at the bounds (property test over the whole
  `u32` range of start values); old policies unchanged.
- **Budget:** `proxy` profile: steady throughput against the 20 rps limit and
  throttle share, before and after. Before, the engine is capped at 5 per
  bucket. P3 retries must not grow without bound: record them.
- **Prove it:** `cargo test -p inventory-engine --locked --test limiter`; the
  two `proxy` rows.
- **Done when:** the rows are recorded. The 10% and 1% targets are B2.3's.
- **Size:** about 320 added. One change.
- **Watch out:** `ApiExecutor` stays the only retry owner; this changes
  pacing, not retries. `lease_budget` uses `requests_per_second` to bound how
  long a lease can wait for admission; with a rising rate use the floor there,
  or leases will expire while waiting. `per_bucket_concurrency` is 8 today
  (`concurrency.clamp(1, 8)`); a rate above about 8 divided by the request
  time needs more. Make it a field of the policy and record what was used.

### B2.2 Learned rates in the store

- **Goal:** The next sweep starts each bucket at the rate the last one
  learned.
- **Depends on:** B2.1, C6.12.
- **Files:** `crates/engine/src/inventory/postgres/schema/shared.rs`,
  `crates/engine/src/inventory/postgres/rates.rs` (new),
  `crates/engine/src/inventory/store_api.rs` (trait),
  `crates/sweep/src/execution.rs`.
- **Change:**
  1. Replace the unused table `learned_rates(service, operation, rate,
     sample_count, updated_at)` with `learned_rates(bucket text PRIMARY KEY,
     rate bigint NOT NULL, sample_count bigint NOT NULL, updated_at bigint NOT
     NULL)`, where `bucket` is the B1.1 key.
  2. Trait methods `load_rates(limit: u32) -> Vec<LearnedRate>` and
     `save_rates(&[LearnedRate])` (one `UNNEST` upsert per batch of 500).
  3. At sweep start, load up to 262,144 rates and pass them to the executor as
     the starting rate of each key (90% of the stored value). At the end, and
     every 60 seconds during the sweep, save `bucket_rates()` for buckets with
     at least 20 samples.
- **Tests:** round trip; a stored rate above the cap is clamped on load; a
  store with no rates starts from the configured start; saving during a sweep
  does not block page commits (it uses its own pooled connection).
- **Budget:** `proxy` profile: time to reach 90% of the limit on the first and
  on the second sweep. P8: the periodic save must not add more than one
  statement per 60 seconds.
- **Prove it:** store conformance tests; the two rows.
- **Done when:** the second-sweep convergence time is recorded.
- **Size:** about 200 added, 10 deleted. One change.
- **Watch out:** schema chain link 4. The rate is per account, so two profiles
  in one database do not share it. Never persist a rate learned from fewer
  than 20 samples; a quiet bucket would lock in its start value.

### B2.3 The cap flag and the quota proof

- **Goal:** `--max-rps` for accounts that share quota with production, and the
  B2 exit evidence.
- **Depends on:** B2.2, B5.4.
- **Files:** `src/cli/sweep.rs`, `src/sweep_command/enabled.rs`,
  `crates/sweep/src/lib.rs` (`SweepConfig`), `crates/sweep/src/config.rs`,
  `tests/sweep_e2e/` (new test file `quota.rs`).
- **Change:** `SweepConfig.max_rps: Option<u32>` (1 to 100,000), CLI
  `--max-rps`. New ignored e2e test through the fault proxy with a fixed limit
  L: (a) after convergence, throughput over 60 seconds is within 10% of L and
  under 1% of responses are throttles; (b) a second sweep is inside that
  margin in its first minute; (c) with `--max-rps` set to L divided by 2,
  throughput never exceeds it in any one-second window.
- **Tests:** as above, plus config validation for 0 and 100,001.
- **Budget:** the three numbers are the evidence; record them.
- **Prove it:** `cargo test --locked --features sweep --test sweep_e2e -j1 --
  --ignored --test-threads 1 quota`.
- **Done when:** all three hold, or the measured values and the gap are
  reported to the coordinator.
- **Size:** about 240 added. One change.
- **Watch out:** the emulator needs enough seeded pages to keep a bucket busy
  for minutes; use the bulk seed with `--max-page-size` (B6.1) set small. The
  10% and 1% margins are the proposed ones from ROADMAP M4.1.

## B3 Continuous claim

### B3.1 Claim several leases in one statement

- **Goal:** `drain_work` calls `claim` once per lease, each its own
  transaction. Claim a batch at once.
- **Depends on:** C5.1, C6.12.
- **Files:** `crates/engine/src/inventory/postgres/work.rs`,
  `inventory/store_api.rs`.
- **Change:** trait method `claim_batch(sweep, now_ms, lease_ms, limit: u16)
  -> Vec<WorkLease>`, one statement: `UPDATE work_queue SET state=1,
  generation=generation+1, lease_until=$2 WHERE (sweep_id, scope_key) IN
  (SELECT sweep_id, scope_key FROM work_queue WHERE sweep_id=$1 AND (state=0
  OR (state=1 AND lease_until<=$3)) ORDER BY last_complete, scope_key LIMIT $4 FOR UPDATE
  SKIP LOCKED) RETURNING scope_key, token, bindings, generation`. `claim`
  becomes `claim_batch(.., 1)`.
- **Tests:** conformance: batch of N returns N distinct leases; an expired
  lease is claimable again with a higher generation; limit 0 refused;
  generation overflow reported; two connections claiming at once never get
  the same scope.
- **Budget:** P9 `claims` and `store_commits` per page on `thin`, before and
  after (after B3.2 uses it).
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store -- --ignored claim`.
- **Done when:** tests pass.
- **Size:** about 120 added. One change.
- **Watch out:** this step first said `ORDER BY scope_key`. The claim that
  landed before it already orders by `last_complete, scope_key` in both stores,
  and `tests/inventory_store/behavior/stalest_first.rs` holds that order, so
  the batch keeps it. The order is still fixed by stored columns alone, which
  is what the kill matrix needs. `RETURNING` does not promise the subquery's
  order; sort the result.

### B3.2 Keep the configured number of leases in flight

- **Goal:** Replace the claim-a-batch-then-wait-for-all loop with a scheduler
  that refills as each page finishes.
- **Depends on:** B3.1, D1.3, C5.3.
- **Files:** `crates/sweep/src/execution/draining.rs` (200 lines),
  `crates/sweep/src/execution/scheduler.rs` (new).
- **Change:**
  1. `scheduler.rs`: a loop over a `futures_util::stream::FuturesUnordered` of
     `page::process` futures. When the number in flight is below
     `config.concurrency` and the queue is not known to be empty, call
     `claim_batch` for the difference (at most 256) and push the futures. When
     a future completes, add its summary and loop. When `claim_batch` returns
     nothing and nothing is in flight, finish. When it returns nothing but
     pages are in flight, wait for the next completion before claiming again
     (a finishing page may add child work).
  2. `drain_work` becomes: run the scheduler, then `apply_persisted_summary`.
  3. Cancellation: the loop selects on the `CancellationToken` from D1.3 and
     stops claiming; in-flight pages finish or are cancelled by the executor.
  4. Progress events (D1.3) are sent from the completion branch.
- **Tests:** with a fake store and fake page future: in-flight count stays at
  the configured value until the queue empties; one stalled page does not stop
  the others from completing; child work added by a late page is picked up;
  cancellation stops new claims; an error from one page is returned after the
  in-flight set drains.
- **Budget:** `thin` with one page stalled by the proxy for 10 seconds: P1
  before and after. P1 and P6 on plain `thin` must not rise.
- **Prove it:** `cargo test -p inventory-sweep --locked scheduler`; e2e
  baseline; the rows.
- **Done when:** the stall rows are recorded.
- **Size:** about 260 added, 40 deleted. One change.
- **Watch out:** no detached tasks: the futures live in the scheduler and are
  dropped with it. `FuturesUnordered` polls on one task; that is intended here
  (the work is I/O), but it means decoding runs on one core. B8.2 measures
  whether that binds.

### B3.3 Lift the bounds, and stay on a scope across pages

- **Goal:** Concurrency is capped at 128 in the sweep and 512 connections in
  the transport, while the limiter allows 4,096. Also, each page of a
  multi-page scope goes back to the queue and is claimed again.
- **Depends on:** B3.2.
- **Files:** `crates/sweep/src/config.rs` (`MAX_CONCURRENCY`),
  `crates/sweep/src/execution.rs` (`transport_config`, `executor_config`),
  `crates/aws-wire/src/transport/config.rs` (`validate`),
  `crates/sweep/src/execution/page.rs`, `src/cli/sweep.rs`.
- **Change:**
  1. `MAX_CONCURRENCY` becomes `inventory_engine::limiter::MAX_CONCURRENCY`
     (4,096). The default stays 16 until B5 rows justify another value; record
     P1 for 16, 64, 256 and 1,024 on `thin` and `proxy` and set the default
     from the rows.
  2. Transport: `max_connections` bound 512 becomes 4,096, and the sweep sets
     it to `min(concurrency, soft file-descriptor limit minus 256)`, read from
     `/proc/self/limits` (fallback 256).
  3. `process_lease` loops: after a page commits with a next token and
     `CommitResult::next_generation` is `Some`, continue with the next page
     under the same lease instead of returning. The commit already renews the
     lease's validity through `validate_lease`; extend `lease_until` in the
     same transaction (new store behavior: the queue update of C6.6 step 9 sets
     `state=1, lease_until=now+lease_ms` when the caller passes `keep: true`).
     At most 64 pages per claim, then return to the queue so other scopes get a
     turn.
- **Tests:** config bounds; descriptor-limit parser; a 3-page scope makes 1
  claim and 3 commits; after 64 pages the scope is released and resumed; a
  kill between pages resumes from the last committed token (store test).
- **Budget:** P9 `claims` per page and P1 on `thin` with a small
  `--max-page-size`, before and after. P4 `connections_opened` for each
  concurrency value.
- **Prove it:** unit and store tests; the rows; e2e baseline.
- **Done when:** the concurrency table is recorded and the default is chosen
  from it in the change description.
- **Size:** about 280 added. Two changes: (1) bounds, (2) page loop.
- **Watch out:** `PageCommit` and the trait change shape here; B4.1 changes
  them again. Land B3.3 first. The kill matrix (B4.3) must include a kill
  between two pages of one lease.

## B4 The PostgreSQL write path

Every choice on the write path is measured: how many pages share a
transaction, `synchronous_commit`, `UNNEST` against COPY (C6.7), pool size, one
writer or several, and staging tables. B4.2 was removed by V53 (see BUILD_PLAN).
The requirement that does not change: a page's records, edges, projection keys
and queue update commit together or not at all, and the kill matrix proves it.

### B4.1 Group pages into one transaction

- **Goal:** C6.6 commits one transaction per page. Let a writer task commit
  several pages in one transaction, and measure whether that helps.
- **Depends on:** B3.3, C5.3, C6.12.
- **Files:** `crates/engine/src/inventory/postgres/writer.rs` (new),
  `inventory/postgres/commit.rs`, `inventory/postgres.rs`,
  `inventory/postgres/config.rs` (new; `PgWriteConfig`).
- **Change:**
  1. `writer.rs`: a task owned by `PgStore` (started in `create` and `open`,
     joined in a new `close(self)`), fed by a bounded channel of capacity 256.
     A job is `{ page: PageCommit, reply: oneshot::Sender<Result<CommitResult,
     StoreError>> }`.
  2. The task takes one job, then drains up to `group_pages - 1` more that are
     already queued without waiting (`PgWriteConfig.group_pages`, default 1,
     so behavior is unchanged until the rows say otherwise). It opens one
     transaction on one pooled connection and runs each page's steps (C6.6)
     inside a `SAVEPOINT`. A page that fails rolls back to its savepoint and
     gets its error; the others continue. Then one `COMMIT`, then every reply
     is sent.
  3. A page is therefore still atomic: it is one savepoint inside one
     transaction, and no reply is sent before the commit. A crash before
     `COMMIT` loses the whole group, and no lease in it was acknowledged.
  4. `commit_page` sends a job and awaits the reply. `claim_batch`,
     `finish_scope` and reads do not go through the writer; they use the pool.
  5. Measure `group_pages` 1, 8 and 64 on `thin` and `full`.
- **Tests:** a group of 10 pages commits once (count through the counting
  wrapper); a failing page in a group does not affect the others; no reply
  arrives before the commit (a test hook drops the task between the last
  savepoint and the commit, and every reply is an error); the channel bound
  applies back-pressure; `close` drains.
- **Budget:** P9 commits per page and writer rows per second, P8 WAL bytes and
  P1, for the three group sizes.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_store -- --ignored writer`; the rows.
- **Done when:** the three rows are recorded and the default follows them.
- **Size:** about 340 added, 40 changed. One change.
- **Watch out:** two pages in one group may touch the same resource (a global
  resource seen from two scopes); they run in order inside one transaction, so
  the second sees the first. With several writers (B4.4) the same pair can
  deadlock unless rows are locked in key order. A larger group raises the cost
  of one failure and the latency of the first page in it; record P1, not only
  throughput.

### B4.3 The kill matrix

- **Goal:** Killing the collector at 100 seeded random points and restarting
  gives the same final database as an uninterrupted run.
- **Depends on:** B3.3, C6.12.
- **Files:** `crates/engine/tests/inventory_kill_matrix.rs` (new),
  `crates/engine/tests/inventory_kill_matrix/driver.rs` (new),
  `.../digest.rs` (new), `crates/engine/examples/kill_matrix_worker.rs` (new).
- **Change:**
  1. `kill_matrix_worker` is a small process: open the store, run a
     deterministic page source (a fixed list of scopes, pages, records, edges,
     children and one deletion, generated from a seed) through `claim_batch`,
     `commit_page` and `finish_scope`, and exit. It reads `KILL_AFTER_OPS=<n>`
     and calls `std::process::abort()` after the n-th store call returns, and
     `KILL_BEFORE_COMMIT=<n>` to abort inside the transaction after the last
     statement and before `COMMIT` (a test hook behind a cargo feature
     `kill-hooks`, off in product builds).
  2. The driver creates a `TestDatabase`, runs the worker to completion once
     for the reference digest, then for 100 seeds: a new database, run with a
     random kill point, restart with resume until it completes, compute the
     digest, compare.
  3. Digest: SHA-256 over sorted rows of `resources`, every kind table,
     `edges`, `unresolved_refs`, `resource_parents`, `resource_scopes`,
     `scope_outcomes`, `work_queue` and `projection_outbox`, leaving out
     `first_seen`, `last_changed`, `last_seen`, `observed_at`, `lease_until`,
     `generation` and the outbox revision.
  4. After each restart run a consistency query: no `edges` row without both
     resources, no kind row without its `resources` row, no `work_queue` row
     in state 1 with a lease in the past after resume.
  5. A second mode makes one statement fail (`SET statement_timeout = '1ms'`
     on the writer's session for one page, standing in for a full disk): the
     run stops with the error, nothing of that page is stored, and a restart
     completes.
  6. The matrix runs again whenever the write path changes: after B4.1 with
     `group_pages` above 1, after B4.4 for each setting kept, after C6.7 with
     COPY forced, and after B4.5.
- **Tests:** this step is the test.
- **Budget:** none on the product path; the hook is compiled out.
- **Prove it:** `cargo test -p inventory-engine --locked --features kill-hooks
  --test inventory_kill_matrix -- --ignored`.
- **Done when:** 100 of 100 points match, with the seeds listed.
- **Size:** about 420 added (tests and the worker). Two changes: (1) worker
  and digest, (2) driver and failure mode.
- **Watch out:** `abort()` is not `panic!` and is allowed by the lints, but
  keep it out of library code: it lives in the example binary and behind the
  feature. What is killed is the client; the server stays up, which is the
  case a sweep can meet at any time. A server crash is a different event: it
  is covered by PostgreSQL's own recovery when `synchronous_commit` is on, and
  B4.4 states what is lost when it is off.
- **Review fix (2026-11-18):** this matrix kills the collector, not the
  database. It proves nothing about a database crash with the collector
  still running. B4.6 covers that, and B4.4 and B4.5 may not relax
  durability until B4.6 passes.

### B4.4 Write settings chosen from measurements

- **Goal:** Choose the PostgreSQL write settings from rows in the table, not
  from assumptions.
- **Depends on:** C6.7, B4.1, B4.3, B5.2.
- **Files:** `crates/engine/src/inventory/postgres/config.rs`,
  `inventory/postgres/commit.rs`, `inventory/postgres/writer.rs`,
  `src/cli/sweep.rs`, `docs/performance-evidence/sweep-benchmark.md`.
- **Change:** `pub struct PgWriteConfig { pub group_pages: u32, pub
  synchronous_commit: bool, pub copy_min_rows: u32, pub pool_size: u32, pub
  writers: u32, pub unlogged_staging: bool }`. Measure on `thin` (and `full`
  when available), one variable at a time against the default:
  1. `synchronous_commit` on (server default) versus `SET LOCAL
     synchronous_commit = off` on the writer's transactions.
  2. `UNNEST` versus COPY, and the row threshold between them (C6.7's rows,
     repeated here with grouping on).
  3. Pool size 4, 16, 64 (the compose file sets `max_connections=200`).
  4. One writer task for the whole sweep versus one per account and region
     (pages of different scopes rarely touch the same rows, but they share
     `resources` indexes and the outbox).
  5. Staging tables `UNLOGGED` and shared versus `TEMPORARY ... ON COMMIT
     DELETE ROWS` per connection.
  Record P1, P2, P8 (WAL bytes) and P9 for each. Set each default to the best
  measured value and state it. `--durable` forces `synchronous_commit` on.
- **Tests:** every combination passes the store conformance tests; the kill
  matrix passes for every setting that becomes a default.
- **Budget:** the rows are the evidence.
- **Prove it:** `cmdb-bench sweep --profile thin` once per setting, with the
  flags; the conformance and kill-matrix commands.
- **Done when:** five small tables are recorded, with the server settings
  block of B5.2 on each, and the defaults are set from them.
- **Size:** about 220 added. One change, plus data.
- **Watch out:** several writers must keep each page atomic and must not
  deadlock on `resources`: lock in key order, or keep one writer if the rows
  show no gain. With `synchronous_commit = off` a killed client loses nothing
  the server acknowledged, but a server crash can lose the latest commits;
  here that means those pages are fetched again, because a page's data and its
  queue position commit together. That argument needs the kill matrix and is
  why `--durable` exists. A laptop disk or a container volume does not predict
  a managed PostgreSQL; every table says which server it ran on (H1.1 repeats
  it on a managed one). O4 no longer applies (V53).
- **Review fix (2026-11-18):** the defaults stay durable:
  `synchronous_commit` on, every table logged. A relaxed value may be
  measured here, but it becomes selectable only after B4.6 passes with it,
  and never the default. With `synchronous_commit = off` a database crash
  can lose commits the collector was told had succeeded while the collector
  keeps running, and the graph may already hold what SQL lost (E4.3).

### B4.5 An unchanged resource writes nothing

- **Goal:** The page commit updates `resource_scopes` for every observation on
  every sweep and rewrites the parent link (C6.6 carries this over from
  `commit.rs::persist_observation` and `persist_parent`). A warm sweep of an
  unchanged account therefore writes at least one row per resource. Make it
  write none.
- **Depends on:** B4.1, C6.8, B4.3.
- **Files:** `crates/engine/src/inventory/postgres/commit.rs`,
  `inventory/postgres/completion.rs`, `inventory/postgres/schema/shared.rs`,
  `inventory/postgres/seen.rs` (new).
- **Change:**
  1. Stop recording "seen in this sweep" per resource. `resource_scopes` keeps
     `(account, region, kind, id, scope_key, missing_complete)` and is written
     only when a resource is first seen in a scope, and when
     `missing_complete` changes.
  2. New table `scope_seen(sweep_id text, scope_key bytea, id text)`,
     logged by default (review, 2026-11-18): it is deletion evidence. An
     `UNLOGGED` variant is a measured option only, behind item 5's guard and
     B4.6. A page writes its ids there only when the scope has more than one
     page (the page is not both first and last). At completion,
     `complete_scope` computes the missing set as: resources of this scope in
     `resource_scopes`, minus the ids of the completing page, minus
     `scope_seen` for this sweep and scope. Then it deletes the scope's
     `scope_seen` rows.
  3. A single-page scope (most enumerations) therefore writes nothing for
     unchanged resources: the page's own id list is the seen set.
  4. Parent links are reconciled, not only inserted (review, 2026-11-18:
     insert-when-absent keeps a stale parent after a resource moves). In the
     page's transaction, compare the parent set observed for each child with
     the stored set: insert links that are new, delete links that are no
     longer observed, and write nothing when the two are equal. The stored
     set is read with the page's other lookups, so an unchanged resource
     still costs no write.
  5. After a client kill `scope_seen` is intact. A database crash is
     detected by the running collector, not only at its next start: the
     sweep row stores `pg_postmaster_start_time()` when the scope's first
     page commits, and `complete_scope` compares it inside its transaction.
     If the server restarted in between, the scope is not completed; it is
     replayed from its first page. This guard is required for the `UNLOGGED`
     variant, which a crash empties, and is kept for the logged one.
- **Tests:** unchanged single-page scope: zero rows written (through
  `pg_stat_xact_user_tables`); unchanged three-page scope: only `scope_seen`
  rows, removed at completion; a resource missing from the second complete
  sweep is still deleted after two complete enumerations and not after one;
  partial sweep deletes nothing; the kill matrix passes with a kill between
  pages of a multi-page scope; a child moved to a new parent has exactly the
  new link afterwards, and deleting the old parent then leaves the child in
  place; a database restart between two pages of a scope, with the collector
  still running, completes nothing and deletes nothing until the replay.
- **Budget:** P8 rows written on the warm sweep (target 0 in logged tables)
  and P2, before and after.
- **Prove it:** store conformance tests; kill matrix; `cmdb-bench sweep`
  rows.
- **Done when:** the warm-sweep row count is recorded as 0 for single-page
  scopes.
- **Size:** about 300 added, 70 deleted. Two changes: (1) seen set and
  completion, (2) parent links.
- **Watch out:** this changes how deletion is decided, the part of the store
  with the least room for error. Schema chain link 7. Land it only after the
  kill matrix and the deletion conformance cases pass with it.

### B4.6 Kill the database, not the collector

- **Goal:** Prove the store's promises when PostgreSQL itself crashes while
  the collector keeps running (review finding 1).
- **Depends on:** B4.3, C6.2.
- **Files:** `crates/engine/tests/inventory_db_crash.rs` (new),
  `crates/engine/tests/inventory_db_crash/driver.rs` (new),
  `crates/harness/src/postgres/runtime.rs` (a `crash` and `start` control:
  `pg_ctl stop -m immediate` for a native server, `kill` for the container),
  `crates/sweep/src/execution.rs` (reconnect policy).
- **Change:**
  1. A stated reconnect policy for the store: a bounded number of attempts
     with bounded backoff, then the sweep ends as resumable with a distinct
     error. Work in flight when the connection drops is not assumed
     committed: its leases are left to expire or are released on reconnect,
     and its pages are fetched again.
  2. The test crashes the server at 50 seeded random points of a sweep over
     the thin slice, restarts it, lets the same collector process continue,
     and compares the final database digest with an uninterrupted run.
  3. Runs once per write setting B4.4 measures: durable defaults,
     `synchronous_commit = off`, unlogged staging, and the `UNLOGGED`
     `scope_seen` variant.
- **Tests:** as above, plus: no resource that still exists is ever marked
  missing or deleted; a scope interrupted by the crash is replayed from its
  first page; a crash during `complete_scope` leaves the scope incomplete; a
  server that does not come back ends the sweep resumable, and `--resume`
  completes it.
- **Budget:** recovery time from server start to the sweep making progress,
  recorded.
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_db_crash -- --ignored`, with a native server and with the
  container.
- **Done when:** 50 of 50 match for the durable defaults. A relaxed setting
  that does not reach 50 of 50 is removed from B4.4's options.
- **Size:** about 320 added. Two changes: (1) reconnect policy, (2) the test.
- **Watch out:** `ApiExecutor` stays the only retry owner for AWS calls; the
  store's reconnect is separate, bounded, and must not re-issue an AWS call
  whose page it cannot prove was stored: it re-fetches. This machine can run
  it against the native server.

## B6 Fewer calls

### B6.1 The largest page size

- **Goal:** Every enumeration asks for the most items the model allows. The
  two real enumerations today ask for 5 (`catalog/overlay/ec2.toml`,
  `page_size = 5` on `DescribeVpcs` and `DescribeSubnets`), which exists so
  the baseline suite sees several pages.
- **Depends on:** D1.2, A4.5.
- **Files:** `catalog/overlay/ec2.toml`, `crates/sweep/src/lib.rs`
  (`SweepConfig`), `crates/sweep/src/execution/page.rs` (`prepare`),
  `src/cli/sweep.rs`, `tests/sweep_e2e/scenario/arguments.rs`.
- **Change:**
  1. Remove both `page_size = 5` lines; the checker then infers the model's
     maximum (`modeled_page_size`).
  2. `SweepConfig.max_page_size: Option<u32>`, CLI `--max-page-size`, applied
     in `prepare` as `min(plan maximum, flag)`. The baseline suite passes
     `--max-page-size 5` to keep its multi-page proof.
  3. The generator never writes `page_size` (A1.2).
- **Tests:** flag bounds; the request carries the model maximum by default and
  the smaller value with the flag; plans with no page-size member are
  unaffected.
- **Budget:** P3 attempts and P1 on `thin`, before and after.
- **Prove it:** `models compile --write` then `--check`; e2e baseline with the
  flag; the rows.
- **Done when:** the rows are recorded.
- **Size:** about 70 added. One change.
- **Watch out:** 605 paginated operations have an unbounded page size and 167
  have a paginator with no page-size member (`catalog/aws-plans.report`); for
  those nothing is sent and the service's default applies. The response body
  limit is 8 MiB (`transport/config.rs`); a largest page that exceeds it fails
  the scope. Record any such operation and give it a hand `page_size`.

### B6.2 Skip a describe when the list already returns everything

- **Goal:** Do not make one child call per resource when the parent's item has
  the same members.
- **Depends on:** A3.2, A3.4.
- **Files:** `crates/catalog-tool/src/overlay/generate/expand.rs`,
  `overlay/generate/skip.rs` (new),
  `runtime_catalog/plans/compile.rs` (plan field 16),
  `crates/core/src/catalog/runtime/types/plan.rs`, `decode/plans.rs`,
  `crates/core/src/catalog/runtime/index.rs`.
- **Change:**
  1. The overlay key `skip` exists in the syntax and is validated
     (`check_skipped_references`) but is not compiled into the runtime plan.
     Add plan field 16, a list of operation names, and
     `CallablePlan::skips()`.
  2. `children_of` (A3.3) leaves out children named in the parent's `skip`.
  3. Generator rule: for a parent enumeration P and a bound child C that
     returns one structure, if every scalar and nested member path of C's
     item exists in P's item with a compatible shape, write `skip = ["C"]` on
     P and list the pair in the report. If C has members P lacks, do nothing.
- **Tests:** identical shapes skip; child with one extra member does not;
  hand `skip` is honored; a skipped child gets no work at run time; a skipped
  child's kind still has a table if another operation produces it.
- **Budget:** P3 attempts per seeded resource on `thin`, before and after, and
  per-kind counts must be unchanged (A7.2's check).
- **Prove it:** `overlays generate --check`; e2e run; the rows.
- **Done when:** the number of skipped pairs and the call counts are recorded.
- **Size:** about 240 added. One change.
- **Watch out:** equal shapes do not prove equal content: some services return
  a member as null in the list and filled in the describe. Verify each skipped
  pair that the emulator serves by comparing the list item with the describe
  result in the probe, and remove the skip where they differ.
- **Review fix (2026-11-18):** similar shapes are not proof that a list
  returns everything a describe does. An operation is skipped only when the
  overlay names the equivalence and the probe has compared, member by member
  on seeded resources, the list's values with the describe's and found no
  member missing or different. The comparison's result is stored with the
  overlay entry. A describe that returns one member more is never skipped.

### B6.3 Batch reads

- **Goal:** Use one batch call in place of many single reads where the model
  has one.
- **Depends on:** B6.2.
- **Files:** `crates/catalog-tool/src/runtime_catalog/plans/compile.rs` (plan
  field 17), core plan types and decoder,
  `crates/sweep/src/execution/page/child.rs`,
  `crates/sweep/src/execution/page/batch.rs` (new),
  `overlay/generate/batch.rs` (new).
- **Change:**
  1. Compile the existing overlay `batch` table (`member`, `max`, `replaces`)
     into plan field 17.
  2. At run time, when a parent page produces child work for the replaced
     operation and a batch plan exists, group the children's identifiers into
     chunks of `max` and create one work item per chunk for the batch
     operation, with the list bound to `member`.
  3. Generator: `catalog/aws-plans.report` counts 124 batch reads, 57 with a
     single counterpart. Write `batch` only for those 57, when the batch
     input list's element shape is compatible with the single read's
     identifier.
- **Tests:** chunking at `max` and at the page bound; a batch response with
  fewer items than requested stores what came back and leaves the rest to the
  missing-resource logic; the replaced operation gets no work.
- **Budget:** P3 attempts per seeded resource, before and after; counts
  unchanged.
- **Prove it:** unit tests; e2e run on a service the emulator serves with a
  batch read (which one is not known until A5.3); the rows.
- **Done when:** at least one batch pair runs against the emulator, or the
  step reports that the emulator serves none and stays implemented, not
  verified.
- **Size:** about 320 added. One change.
- **Watch out:** a batch call's scope key has no single parent; use the
  parent enumeration's scope with a chunk index in the bindings digest. The
  deletion logic in B4.5 keys on scope: a chunk boundary that moves between
  sweeps must not look like a deletion. Keep batch scopes out of the "two
  complete enumerations" rule and let the parent enumeration decide deletion.
- **Review fix (2026-11-18):** a batch response is not proof of absence.
  Every requested ID ends in exactly one of: success, authoritative
  not-found (the response names the ID as not found), unprocessed (retried
  under `ApiExecutor`, the single retry owner, within its bounds), denied,
  or unknown (asked for, not in the response, not named anywhere). Only
  authoritative not-found counts toward deletion. A batch with any
  unprocessed or unknown ID leaves its scope partial. Tests cover each of
  the five, a batch that is all unprocessed, and reordered results.

### B6.4 Remember inaccessible and unsupported scopes

- **Goal:** Do not ask again, on every sweep, for a service that is not in a
  region or that the role cannot read. This pulls the first half of F5
  forward because it removes calls.
- **Depends on:** A5.1, B2.2.
- **Files:** `crates/engine/src/inventory/postgres/schema/shared.rs`,
  `inventory/postgres/memory.rs` (new), `inventory/store_api.rs`,
  `crates/sweep/src/execution/draining.rs` (`seed_enumerations`),
  `src/cli/sweep.rs`.
- **Change:**
  1. Table `scope_memory(scope_key bytea PRIMARY KEY, outcome text NOT NULL,
     expires_at bigint NOT NULL, detail text)`.
  2. `finish_scope` with outcome `Inaccessible` or `Unsupported` upserts a row
     with expiry now plus 24 hours (unsupported) or 1 hour (inaccessible);
     flags `--remember-unsupported-hours` and `--remember-inaccessible-hours`,
     0 turns it off.
  3. `seed_enumerations` loads unexpired rows and does not seed those scopes;
     it writes their outcome into `scope_outcomes` for this sweep as
     `unobserved` with the remembered detail, so nothing is silent.
  4. A failure that hits every endpoint (no network) must not be remembered:
     only store when the executor classified the response as `Denied` or
     `UnsupportedType`, never on a transport error.
- **Tests:** remembered scope is not called on the next sweep and is reported;
  after expiry it is called again; a transport failure is not remembered; a
  remembered scope deletes nothing (it is not a complete enumeration).
- **Budget:** P3 attempts on the second sweep of an account where half the
  regions are unsupported (proxy rule returning the not-supported error),
  before and after.
- **Prove it:** store conformance tests; proxy e2e test; the rows.
- **Done when:** the rows are recorded.
- **Size:** about 260 added. One change.
- **Watch out:** schema chain link 5. A remembered denial hides a permission
  that was fixed in the meantime for up to an hour; print the count of
  remembered scopes in the summary line.

## B7 Transport

### B7.1 See what the transport does

- **Goal:** Know, per run, how many connections were opened, how many requests
  reused one, and how many went over HTTP/2.
- **Depends on:** B5.1.
- **Files:** `crates/aws-wire/src/transport/counters.rs`,
  `crates/aws-wire/src/transport.rs`, `crates/cmdb-bench/src/sweep/table.rs`.
- **Change:** B5.1 added `http2_requests`. Add `handshake_ns` (time inside
  `Bounded` connect, from `transport/bounded.rs`) and `hosts` (distinct
  authorities seen, bounded set of 16,384). Print connections per host and
  requests per connection in the benchmark table. No behavior change.
- **Tests:** counters on a local TLS test server (the pattern in
  `transport/trust_tests.rs`): HTTP/1.1 keep-alive counts reuse; an HTTP/2
  server counts `http2_requests`.
- **Budget:** P1 must not rise.
- **Prove it:** `cargo test -p aws-wire --locked transport`; a `thin` and a
  `proxy` row showing the new columns.
- **Done when:** the baseline for B7.2 is recorded: connections opened per
  request on `proxy`, and whether the emulator negotiates HTTP/2 (not verified
  today).
- **Size:** about 120 added. One change.
- **Watch out:** HTTP/2 is already enabled (`enable_http2()` in `build_pool`,
  ALPN decides). This step measures; it does not turn anything on.

### B7.2 Keep connections alive across a paced bucket

- **Goal:** The pool's idle timeout is 1 second. The transport's own default
  keeps 2 idle connections per host; the sweep overrides that to
  `min(max_connections, 8)`. (`concurrency.clamp(1, 8)` in the same file is
  the per-bucket concurrency, not a pool setting.) A bucket paced slower than
  one request per second therefore opens a new TLS connection for every
  request. Tune both from measurements.
- **Depends on:** B7.1, B3.3.
- **Files:** `crates/aws-wire/src/transport/config.rs` (`Default`,
  `validate`), `crates/sweep/src/execution.rs` (`transport_config`).
- **Change:**
  1. The sweep sets `idle_timeout` explicitly. Measure 1, 15 and 90 seconds on
     the `proxy` profile at 0.5, 5 and 50 requests per second per bucket.
     `validate` requires `idle_timeout <= request_deadline - idle_timeout`, so
     with the 30-second deadline the largest valid value is 15 seconds; to
     test 90, separate the two in `validate` (the reason for the coupling is
     not written down; find it in the M2.3 review notes in ROADMAP before
     changing it).
  2. `pool_idle_per_host`: measure 2, 8 and 32 with HTTP/1.1; with HTTP/2 one
     connection carries many streams, so record separately.
  3. Set the defaults from the rows. Idle connections hold a socket permit
     until eviction (an accepted limit recorded in ROADMAP M2.3), so cap idle
     sockets in total at half of `max_connections`.
- **Tests:** config validation at the new bounds; with a test server and
  paused time, a request 5 seconds after the previous one reuses the
  connection when the timeout is 15 and opens a new one when it is 1.
- **Budget:** P4 connections opened and P1 on `proxy`, before and after, for
  each setting.
- **Prove it:** `cargo test -p aws-wire --locked transport`; the rows.
- **Done when:** the table of settings is recorded and the defaults follow it.
- **Size:** about 120 added. One change.
- **Watch out:** the emulator is one endpoint, so emulator runs show almost no
  connection cost whatever the setting. The real effect (about 7,400 hosts for
  436 services and 17 regions) can only be estimated here from handshakes per
  request through the proxy; say that it is an estimate until J2.
- **Scale (V76):** key the pool by endpoint and never by account. The host
  for a service in a region is the same for every account, so the number of
  connections must not grow with the number of accounts (S1.4).
- **Status (2026-11-24):** steps 1 and 2 are settable but not measured:
  `--idle-timeout-ms` (1 to 10,000) and `--pool-idle-per-host` exist, and the
  defaults are unchanged (one second; the sweep keeps up to
  `min(max_connections, 8)` idle sockets per endpoint, the bare transport 2).
  No table of settings is recorded, so this step is not done. Step 3's total
  cap on idle sockets is not implemented: `hyper-util` 0.1.21 exposes only
  `pool_max_idle_per_host`, and its pool admission is private.

## B8 CPU on the hot path

### B8.1 Stop copying rows

- **Goal:** Remove per-row allocations that the code shows: `rows::observation`
  clones every record (`row.clone()`), `lookup_plan` and the bootstrap rebuild
  endpoint strings per page, and `Call::send_attempt` encodes the request
  again on every retry.
- **Depends on:** A3.3, B5.1.
- **Files:** `crates/sweep/src/execution/page/rows.rs`,
  `crates/sweep/src/execution/page.rs`, `crates/sweep/src/record.rs`
  (`item_records`), `crates/core/src/inventory/observation.rs`.
- **Change:**
  1. `record::item_records` returns owned `Record`s; `observations` takes them
     by value and moves each into its `Observation` instead of cloning.
     References and child work are computed before the move, from borrowed
     rows.
  2. Cache `ResolvedEndpoint` per `(plan index, region)` in a bounded map
     (65,536 entries) owned by the run context.
  3. Encode the request once per page in `Call::run` and clone the
     `http::Request<Bytes>` per attempt (`Bytes` is reference counted); sign
     per attempt as today, because the signature carries the time.
- **Tests:** observation bytes and hashes are identical to before for a
  fixture page; a retried call sends byte-identical bodies; the cache bound
  holds.
- **Budget:** P6 CPU seconds and P7 peak memory on `thin` and `full`, before
  and after.
- **Prove it:** `cargo test -p inventory-sweep --locked`; the rows.
- **Done when:** the rows are recorded.
- **Size:** about 160 added, 60 deleted. One change.
- **Watch out:** do not guess further hot spots. If a profiler is available
  (`perf` is not verified on this machine), attach its top 20 symbols to the
  row; otherwise the `decode_ns` and `store_commit_ns` counters are the only
  attribution.

### B8.2 Decode share, and the binary protocol

- **Goal:** Measure decoding as a share of CPU time, and compare the same
  service over JSON and over rpcv2Cbor.
- **Depends on:** B8.1, A5.2.
- **Files:** `crates/sweep/src/lib.rs` (`SweepConfig`),
  `crates/catalog-tool/src/runtime_catalog/plans.rs` (`protocol`),
  `crates/cmdb-bench/src/sweep.rs`, `evidence/decoder-bench/` (existing).
- **Change:**
  1. The compiler already prefers rpcv2Cbor when a service offers it
     (`plans.rs::protocol` tries it first). CloudWatch is the thin-slice
     service that offers both. Add a compile-time switch for measurement only:
     `catalog-tool models compile --prefer-json --write` to a scratch root,
     producing a catalog in which such services use their JSON protocol.
  2. Run `thin` restricted to `cloudwatch` with both catalogs and record P1,
     P6, P10 and bytes read.
  3. Record P10 for the whole `thin` and `full` profiles. V33 says page
     hashing may return only if decoding is the bottleneck; state the share.
- **Tests:** the scratch catalog differs only in field 13 of the affected
  plans; records decoded from JSON and from CBOR for the same seeded resources
  have identical bytes and hashes (the cross-format invariant ROADMAP M2.4c
  leaves unproven).
- **Budget:** the rows are the evidence.
- **Prove it:** the two benchmark runs; the invariant test against the
  emulator.
- **Done when:** the comparison and the decode share are recorded, or the step
  reports that the emulator does not serve CloudWatch over rpcv2Cbor (not
  verified).
- **Size:** about 150 added. One change.
- **Watch out:** the scratch catalog is never committed and never shipped. If
  the hashes differ between the two protocols, that is a correctness defect in
  a decoder and takes priority over the measurement.

## B9 Head-to-head, early

### B9.1 A harness that runs the other tools

- **Goal:** Measure this collector against CloudQuery and Steampipe on the
  same emulator dataset, right after the thin slice, on the services all
  sides collect.
- **Depends on:** A7.1, B5.2.
- **Files:** `crates/harness/src/head_to_head.rs` (new),
  `head_to_head/tool.rs` (new), `head_to_head/report.rs` (new), `cli.rs`,
  `main.rs`, `docs/performance-evidence/head-to-head/README.md` (new).
- **Change:**
  1. `inventory-harness head-to-head --work <dir> --ours <path> --tool
     <name>=<path to a tool definition file> ... --runs 3`.
  2. A tool definition is a TOML file kept outside this repository's product
     code, under `docs/performance-evidence/head-to-head/tools/` (new): the
     command line, the environment (endpoint URL, CA bundle, credentials
     files, region), how to count resources per kind (a SQL statement against
     the tool's output database or a command), and the tool's version command.
     No competitor binary or image is added to the repository (O7).
  3. Procedure: one seeded emulator; each tool in turn, order rotated per
     run, three runs each; before each run the tool's output store is emptied.
     Measure wall time and, where the tool reports it, API calls; count
     resources per kind.
  4. Report: tool versions, dataset (seed files and counts), machine, the set
     of kinds every tool collected, resources each returned for those kinds,
     median and spread of wall time. A result counts only the common kinds
     (VISION section 6).
- **Tests:** rotation order; median; a tool that exits non-zero is reported as
  failed and not timed; common-kind intersection.
- **Prove it:** a dry run with `--tool ours2=<definition pointing at our own
  binary>` produces the report.
- **Done when:** the harness runs our collector twice under two names and
  reports equal counts.
- **Size:** about 400 added. One change.
- **Watch out:** whether CloudQuery's and Steampipe's AWS plugins accept a
  custom endpoint with a private CA, and whether their current releases can be
  run freely, is not verified. The step must record what it finds. If a tool
  cannot be pointed at the emulator, the report says so by name and the
  comparison waits for J3.

### B9.2 The first comparison

- **Goal:** One recorded head-to-head on the thin slice.
- **Depends on:** B9.1, and the thin-slice checkpoint (Wave 2).
- **Files:** `docs/performance-evidence/head-to-head/<date>-thin.md` (new),
  tool definition files.
- **Change:** install the two tools from their published releases outside the
  repository, write their definition files, run the harness, commit the
  report. Repeat after Wave 4 with the speed work in.
- **Tests:** none.
- **Prove it:** the command line and the report.
- **Done when:** the report is committed with versions, dataset, machine and
  the common kinds, whatever the result.
- **Size:** no Rust. One change per run.
- **Watch out:** the emulator has no quotas and no latency, so this compares
  engines (CPU, decoding, writing, call count), not quota handling. "Fastest"
  is not claimed from it (V19 also needs a real account). Cartography joins in
  J1.
