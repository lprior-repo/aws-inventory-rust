# Local AWS inventory stack

This checkout has an authenticated AWS Emulator and a real FalkorDB server.
Both bind to dynamically allocated loopback ports. Collection uses generated
emulator credentials, account `000000000000`, and verified TLS. Telemetry is off.
The emulator's fixtures are synthetic. No real AWS account is required.

Run these commands from the repository root:

```bash
scripts/local-stack.sh start
scripts/local-stack.sh status
scripts/local-stack.sh logs
```

Startup seeds all five fixture families, checks the collector, and runs the
five snapshot integration tests twice before reporting `READY`. This can take
several minutes on the first run. The supervised service runs independently
of the terminal. Its name includes a hash of the checkout path, so separate
checkouts do not stop each other's services.

Use the collector with the active local environment:

```bash
scripts/local-stack.sh cli catalog
scripts/local-stack.sh cli scan --region us-east-1 --service ec2 \
  --db .local/my-inventory.sqlite --output-dir .local/exports --falkor \
  --provenance all-observations
scripts/local-stack.sh cli query --db .local/my-inventory.sqlite \
  'SELECT scan_id,status FROM scans'
```

`cli` clears inherited AWS variables and loads only the verified local session.
The status check validates the emulator's TLS health endpoint and FalkorDB PING.
Do not point the fixture seeding scripts at an AWS account. Known unsupported
emulator operations remain explicit partial scan outcomes; `--allow-partial`
is needed when exporting those snapshots.

Run complete fresh local integration verification:

```bash
scripts/local-stack.sh verify
```

This creates separate disposable servers, executes the Rust workspace tests
including all ten guarded integration tests, compares SQLite with JSON/JSONL
and real FalkorDB contents, checks checkpointed replay, and stops its servers.
The command preserves its reports under `.local/verification/` and does not
replace the interactive stack's inventory.
Do not rebuild or edit Rust source during verification: the harness checks
source and binary hashes and rejects evidence if either changes during a run.

Stop the interactive stack with:

```bash
scripts/local-stack.sh stop
```

The services stop on reboot or after seven days. Run `start` again to seed a
fresh session. SQLite snapshots and exports survive stopping the servers.
Emulator fixtures and the interactive graph are disposable. Service processes
are bounded by a 32 GiB cgroup; builds use one Cargo job. Startup does not enable
a system service or require sudo.

## Reproduce the installation

Install Rustup, a C compiler, make, git, curl, GnuPG, OpenSSL, Python, and either
`dpkg-deb` or `bsdtar`. `uv` is optional. On a new clone:

```bash
scripts/local-stack.sh install
scripts/local-stack.sh start
```

The checked-in toolchain is used. The installer verifies the emulator package
SHA-256 and AWS signature before extracting it, and the Redis and FalkorDB
downloads have pinned checksums. Arch Linux uses `bsdtar` when `dpkg-deb` is
absent. The wrapper installs no system packages. Cargo uses Rustup's pinned
user toolchain; Rustup may download it if it is not already installed.

Runtime downloads, Python packages, logs and session environment files live
under the ignored `.local/` tree. On the initial machine, its `runtime` and
`checks` entries link to this chat's `../work/` installation. Generated AWS
credential and CA files use the emulator's own `~/.aws/emulator/<port>/` path;
the wrapper saves paths, not secret key values. Never commit that directory.

## Verification and review scope

The original ZIP's manifest describes its original payload, including ignored
historical artifacts; it is not a manifest of subsequent repository edits.
The archive remains unchanged in Downloads. The repository retains the upstream
source, pinned catalogs, fixtures and documentation; historical artifacts and
generated credentials are excluded by `.gitignore`.

Use `scripts/verify.sh` for the upstream canonical checks. In an agent shell
with compiler wrappers, explicitly clear them for the mutation helper tests:

```bash
env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER \
  INVENTORY_PYTHON="$(realpath .local/checks)/bin/python" bash scripts/verify.sh
```

Operational local integration and strict Rust assurance have separate results.
See `LOCAL_REVIEW.md` for the black-hat findings and verifier applicability.
The original project's universal AWS coverage and whole-project mutation score
remain open; successful local tests do not establish either claim.
