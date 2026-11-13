# Using your local AWS Emulator

The emulator and FalkorDB are installed and running. The demonstrated scan
completed with 150 resources, 340 configuration relationships, 131 completed
collection scopes, and zero failed scopes. Its synthetic account is
`000000000000`, with regional resources in `us-east-1`.

AWS Emulator runs a local AWS API server and has no graphical interface.
Your Rust inventory CLI collects its resources. [Official overview](https://github.com/aws/aws-emulator#aws-emulator-readme).

This walkthrough describes the current machine. For a new clone, follow
`docs/LOCAL_SETUP.md` first, then run the new-scan command below.

## 1. Open the project

```bash
cd /home/lewis/Documents/Codex/2026-11-12/let/aws-inventory-rs
scripts/local-stack.sh status
```

Expect `READY`. Status prints the current endpoints. At verification:

- AWS Emulator: `https://127.0.0.1:42257`
- FalkorDB: `redis://127.0.0.1:50459`

Ports change after restart. The wrapper automatically loads the current local
endpoint, generated emulator credentials, and trusted certificate.

## 2. Inspect the example already collected

```bash
scripts/local-stack.sh cli query \
  --db .local/learning-inventory.sqlite \
  'SELECT scan_id,status FROM scans'

scripts/local-stack.sh cli query \
  --db .local/learning-inventory.sqlite \
  'SELECT id,region FROM aws_ec2_instance LIMIT 10'

scripts/local-stack.sh cli query \
  --db .local/learning-inventory.sqlite \
  'SELECT relation,count(*) AS edges FROM edge_observations GROUP BY relation ORDER BY edges DESC LIMIT 5'
```

A compact report for the demonstrated scan, using the installed `jq`:

```bash
scripts/local-stack.sh cli report \
  --db .local/learning-inventory.sqlite \
  --scan-id 08033e81-87c9-47eb-9868-7c7256aead67 \
  | jq '{status,summary,failures}'
```

## 3. Run a new scan

```bash
scripts/local-stack.sh cli scan \
  --region us-east-1 --service all \
  --db .local/learning-inventory.sqlite \
  --output-dir .local/learning-exports \
  --falkor --graph aws_inventory_learning \
  --provenance all-observations
```

This reads the seeded resources, persists a new SQLite scan, writes inventory
and graph-plan JSONL, and delivers relationships to FalkorDB. Copy the returned
`scan_id` into report and traversal commands. Repeated scans retain their own
history and export attempt directories.

`--service all` runs this project's implemented collectors. Additional discovery
and Cloud Control options are described by `scripts/local-stack.sh cli scan --help`.

Files are under `.local/learning-exports/<scan-id>/<attempt-id>/`:

- `inventory.jsonl`: observations and provenance.
- `graph.jsonl`: replayable FalkorDB delivery plan.
- `delivery.json`: export and delivery receipt.

The shared delivery checkpoint is `.local/learning-exports/projection.sqlite`.

## 4. Explore relationships

```bash
scripts/local-stack.sh cli neighbors \
  --db .local/learning-inventory.sqlite \
  --scan-id 08033e81-87c9-47eb-9868-7c7256aead67 \
  --resource-id arn:aws:ec2:us-east-1:000000000000:instance/i-00000000000000001 \
  --max-depth 1
```

This example completed successfully with nine returned nodes. Relationships
represent observed configuration. The graph can include explicitly unresolved
references; it does not establish network reachability.

## 5. Manage the services

```bash
scripts/local-stack.sh start
scripts/local-stack.sh status
scripts/local-stack.sh logs
scripts/local-stack.sh stop
```

After a reboot, run `start` and wait for `READY`. Startup seeds and checks a fresh
emulator. SQLite and exports remain on disk when stopped. Emulator state and the
graph are disposable; run a new scan to populate a newly started graph. The
transient service also has a seven-day maximum runtime.

## 6. Run complete local verification

```bash
scripts/local-stack.sh verify
```

This uses separate disposable servers, runs the Rust workspace including guarded
live tests, checks SQLite/export/FalkorDB agreement and replay, and retains
reports under `.local/verification/`. Let it finish without rebuilding or editing
Rust source during the run.

Private repository: [aws-inventory-rust](https://github.com/lprior-repo/aws-inventory-rust).
