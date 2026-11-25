# Head-to-head collector harness

B9.1 measures sequential collector invocations against **one seeded emulator**.
It installs no CloudQuery/Steampipe binaries, images, or plugins. Tool definitions
live in `tools/`, outside product code. Definitions execute trusted local commands;
review them before running. Do not put real AWS credentials in these files.

## Self-comparison

From the repository root, with the release collector and harness already built:

```sh
export INVENTORY_POSTGRES_URL='postgres://USER:PASSWORD@127.0.0.1:55432/postgres'
target/release/inventory-harness head-to-head \
  --work /tmp/inventory-head-to-head \
  --ours target/release/aws-inventory \
  --tool ours2=docs/performance-evidence/head-to-head/tools/ours2.toml \
  --runs 3
```

`--work` must be empty or absent. The PostgreSQL role needs CREATEDB. Install the
emulator and FalkorDB using the existing harness install commands first. Existing
`INVENTORY_EMULATOR_BINARY`, `INVENTORY_REDIS_SERVER`, and
`INVENTORY_FALKOR_MODULE` overrides are honored by the shared `e2e::stack::Stack`.

Defaults are `catalog/aws-api.catalog`, `crates/harness/seeds`, and the EC2 and
S3 recipes (six VPCs, six subnets, two buckets). Override with `--catalog`,
`--seeds`, and repeated `--service <recipe-name>`. Every external definition must
declare its collection scope with `services = ["ec2", "s3"]`; it must match the
requested scope before timing is comparable. Seed recipes are copied into `work/dataset/`
with SHA-256 hashes before seeding. `oracle.tsv` records identities and
`seed-matrix.tsv` records accepted/rejected/skipped mutations. Seed counts are
actual accepted resources, including oracle rows tagged `service-created`, not
promises inferred from recipe repeat counts.

`ours2.toml` points at our own collector; its relative `collector` path resolves
against the definition file's directory, not the invoking working directory.
Absolute paths and `..` are allowed after canonicalisation, but the resolved
collector must be a regular file. Empty, missing and directory collectors are
rejected while loading, with the tool name and definition/binary paths.
Both names use the shared collector command builder, fresh disposable PostgreSQL
databases, and a cleared local graph before every invocation. Check that
`comparison_ready` is true before using their comparison timings.
Our normal sweep exit 2 is **not** accepted here: every nonzero process exit
fails the sample and has no timing. Collection or version failures still produce
a report, so inspect `comparison_ready` and each tool's run status rather than
interpreting harness exit 0 as a successful comparison.

## External TOML definitions

Each command is a nonempty array of program and arguments; no shell is implied.
Use an explicitly named shell or a wrapper script when SQL/output normalization
is needed. Programs must be absolute paths or resolvable through PATH. Example
schema (replace commands with locally installed, version-appropriate wrappers):

```toml
services = ["ec2", "s3"]
version = ["/opt/local/bin/tool-adapter", "version"]
reset = ["/opt/local/bin/tool-adapter", "reset"]
run = ["/opt/local/bin/tool-adapter", "collect"]
counts = ["/opt/local/bin/tool-adapter", "counts"]
api_calls = ["/opt/local/bin/tool-adapter", "api-calls"] # optional
compatibility = "Record the tested plugin version, endpoint/CA flags, license and findings here."

[environment]
AWS_ENDPOINT_URL = "${H2H_ENDPOINT}"
AWS_CA_BUNDLE = "${H2H_CA_BUNDLE}"
AWS_REGION = "${H2H_REGION}"
TOOL_OUTPUT = "${H2H_WORK}/output.db"
```

The harness clears inherited environment for external and version commands,
retains PATH, and supplies the emulator's environment plus issued default-profile
credentials/config files. `AWS_SHARED_CREDENTIALS_FILE`, `AWS_CONFIG_FILE`,
`AWS_PROFILE`, `AWS_REGION`, `AWS_DEFAULT_REGION`, `AWS_ENDPOINT_URL`,
`AWS_CA_BUNDLE`, and metadata disabling come from Stack. Tools must actually use
these endpoint/CA settings; TLS verification must not be disabled. Add explicit
local HOME/plugin/cache paths to `[environment]` if the tool requires them.
Values override the supplied environment. Never point a definition at production.
Collector entries reject `[environment]` (including an empty table) and `services`;
they always use the shared native adapter's environment and requested scope.

Arguments and environment values expand `${H2H_ENDPOINT}`, `${H2H_CA_BUNDLE}`,
`${H2H_REGION}`, `${H2H_SERVICES}` (comma-separated), `${H2H_WORK}`, and `${H2H_CATALOG}`. Collector definitions also
have `${H2H_COLLECTOR}` for their canonical binary path. Each version and each
sample command runs in a separate work subdirectory. Use absolute paths to any
shared output store. `reset` **must empty every output store** before collection;
a failed reset fails the sample without running collection. Reset, version,
counting, and API-call extraction are excluded from wall time. Each child has a
300-second deadline. Every exit kills remaining group members before reaping the
leader, preventing PID/PGID reuse. A successful command that leaves live group
members stays timed, with `left_processes_running: true` and the per-run
`process_cleanup` message "left processes running after exit; killed". Clean exits
report false and a null message. Timeout fails the sample. SIGINT (Ctrl-C),
SIGTERM, SIGHUP and SIGQUIT cancel the active process group and stop the emulator
and graph before returning an error. Handlers are registered before snapshotting;
a signal during the snapshot drains that copy without starting the stack.
Interrupted startup is awaited and its owned children cleaned up before returning.
A first signal during normal shutdown makes the run fail after shutdown finishes.
Once interrupted, a second signal during drain or shutdown does not escalate.
An already-created sample database has a 65-second cleanup deadline. If CREATE is
in flight, its independently owned worker settles creation before attempting DROP:
allow up to 175 seconds for connection/setup, CREATE, settlement, close and DROP.
Unsettled creation or failed deletion names the database in `cleanup_failures`,
disables comparison timing and makes the harness exit nonzero.
Startup drain retains the existing readiness budgets: 30 seconds for the emulator
and 300 graph attempts (up to 1,830 seconds of network/poll waits at three 2-second
I/O budgets plus a 100-ms pause per attempt). Version checks, filesystem access
and operating-system process reaping have no hard wall-clock bound.
SIGKILL bypasses cleanup and can leave children and databases behind. After such
an exit, inspect surviving process groups and kill only the groups from this run:

```sh
ps -eo pid,ppid,pgid,args
PGID=12345 # replace with a verified surviving command group, not your shell's group
kill -KILL -- "-${PGID}"
inventory-harness postgres prune
```

Repeat the group kill for surviving emulator/graph groups as needed. `postgres
prune` removes harness-named databases older than 3,600 seconds, so run it after
that age threshold; it does not immediately remove a fresh leaked database.
Cleanup only covers a command's original process group; descendants that create
a new session or leave that group (for example `setsid`) are outside this guarantee.
Native PostgreSQL count extraction has a separate 60-second deadline covering
connection and query. Each command has separate stdout/stderr logs.
Tool definitions have at most 256 arguments per command, 31 competitors,
100 runs, and 10,000 counted kinds. Read-back text is capped at 16 MiB.

`counts` prints exactly `kind<TAB>unsigned-count` per nonempty line, for example
`AWS::EC2::VPC<TAB>6`. Map tool-specific table names to canonical AWS kind names;
include zero rows for kinds collected successfully with no resources. Duplicate
kinds, negative counts, headers and malformed rows fail the sample.
Observed kinds outside the common-kind intersection are listed in the sorted
`not_compared_kinds` diagnostic. An explicit zero and an absent kind are
different observations, even for unseeded kinds. An absent seeded kind or a
seeded count outside its documented policy fails the sample without timing.
A SQL-based adapter can invoke its database client (for example psql with `-At -F '\t'` and
`SELECT ... GROUP BY ...`) through this command interface. Bind any query values
in that adapter; do not interpolate untrusted values into SQL. `api_calls`, when
provided, prints one unsigned integer. Missing API metrics remain null, never a
fabricated zero. Our collector's `api_attempts` includes retries.

## Evidence and interpretation

`work/report.json` contains tool version output and definition paths,
compatibility notes, machine CPU/memory/kernel data, dataset snapshots/hashes and
accepted counts, the ordered samples, common kinds, sorted `not_compared_kinds`,
common-kind resource counts, API calls where reported, and median/min/max/range of wall seconds. The initial
order is ours followed by repeated `--tool` order; each round rotates its first
tool by one. Default three runs means every tool runs three times, sequentially.

The common-kind intersection spans **all tools and all runs**. A failed tool or
missing sample empties it and defers comparison to J3. `equal_common_counts`
compares only that intersection and is diagnostic. Per-tool observed counts
and raw successful command times also remain diagnostic, not comparable results.
`comparison_ready` requires nonempty common kinds, identical full observed count
maps across all tools and runs, and equal declared/requested service scopes.
`services` and per-tool `services` expose those scopes; `equal_services` records
their agreement. Otherwise `comparison_timing` is null, including on any failure.
The emulator also creates default resources. Each seeded kind must be present in
every successful sample and equal its oracle count **exactly**. Only
`AWS::EC2::VPC` and `AWS::EC2::Subnet` allow counts **at least** the oracle,
because the emulator creates default resources of those kinds. The CLI and native
self-comparison helper use the same count policy. Thus even identical over-counts
of `AWS::S3::Bucket` fail both tools' samples without timing. Full observed maps
must also match **exactly** between tools and runs, including kinds and resources
not created by seed recipes.
Wall time covers the whole collection command, not time attributable to a subset
of kinds. No per-kind timing or throughput claim is inferred. Comparisons must
use equivalent scopes; unmatched extra work can affect these whole-command times.
There is no benchmark speed target or improvement claim in B9.1.

CloudQuery: custom emulator endpoint/private CA behavior and current AWS plugin
free-use availability are **unverified**. Steampipe: the same questions are
**unverified**. No competitor-specific executable definition is supplied because
neither plugin interface was verified in this assignment. A failing supplied
`--tool cloudquery=...` or `--tool steampipe=...` remains named in the report with
its version/collection failure and compatibility note; comparison waits for J3.
Record tested release/licensing and endpoint findings in its definition and
retain the generated logs before making a comparison claim.

## Verification commands

```sh
cargo test -p inventory-harness head_to_head
cargo test -p inventory-harness --test head_to_head_native -- --ignored
cargo test -p inventory-harness --test head_to_head_signals
cargo test -p inventory-harness --test head_to_head_signals -- --ignored
cargo test -p inventory-harness --lib head_to_head::tool::native_tests::given_10001_postgres_kinds_when_counting_then_truncation_is_rejected -- --ignored --exact
```

`INVENTORY_H2H_TEST_WORK` selects the parent directory for native test scratch files.
Native scenarios need the PostgreSQL URL, installed native runtimes, and an
already built collector (`INVENTORY_COLLECTOR_BINARY` overrides
`target/debug/aws-inventory`). Separate EC2+S3 and S3-only scenarios invoke the CLI
under two names for three runs. The successful S3 done-condition scenario asserts
exact observed counts between tools and runs, exact S3 bucket counts from the
run's `oracle.tsv`, a nonempty compared-kind set, and the six-sample rotation.
The EC2+S3 scenario asserts both names fail without timing because
`DescribeTrunkInterfaceAssociations` returns `Fatal (OperationNotPermitted)` and
the collector exits 2. The report includes the collector's nonempty stderr so it
names the failed scope. If that operation succeeds, this test intentionally fails.
This is a known-failure assertion, not equal-count done-condition evidence.
The S3-only self-comparison is the only successful equal-count evidence; it is
not the CloudQuery/Steampipe comparison required by B9.2. See
[`2026-11-24-thin.md`](2026-11-24-thin.md) for that status.
Unsupported-operation policy is an owner decision; no operation is filtered and
exit 2 is never accepted as success.
Other native scenarios cover the actual Stack-derived command environment and
collection/reset/version failures.
Unit scenarios cover scope/count gating, both sides of CLI/argument/kind bounds,
median/spread, intersection and omitted-kind reporting, malformed definitions,
subprocess environment isolation/expansion, successful and timed-out process-group
cleanup, exact graph reset bytes, and a PostgreSQL connection deadline.
Native runs require the prerequisites above; competitor compatibility remains unverified.
Default unit tests cover pure H2H/AWS environment assembly, collector binary
insertion, version rejection without polling collection, stop-path finalization,
API-call extraction, both PostgreSQL row-cap boundaries, cancelled wait cleanup,
and unrelated-session exclusion. Real CLI signal tests interrupt SIGINT, SIGTERM,
SIGHUP and SIGQUIT during incomplete startup, assert exit 1 and the entire error,
and never release the startup stub. Non-ignored sample cancellation scenarios use
a fake collector and database-action seam while exercising the real signal,
process-group, database-owner and outcome-reporting paths. A snapshot-reader seam
checks that interruption drains the real dataset copy without launching the stack.
An owned non-listening TCP socket tests that a real failed DROP appears in the
report and makes the run fail. Ignored native signal tests interrupt an active
PostgreSQL-counted sample and verify deletion on the real server.
`Stack::configure()` itself remains covered only by the native test.
