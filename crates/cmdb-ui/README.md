# cmdb-ui

A read-only Makepad desktop viewer for inventory stored in the schema16 SQLite database. SQL inventory access and FalkorDB graph queries run on a crate-owned background worker; the UI thread only applies owned snapshots.

## Walkthrough: from empty directory to the DAG

This walkthrough shows the full path from a blank folder to viewing an AWS resource graph. No AWS credentials are required; `cmdb-bench` generates synthetic data.

**Step 1 — Create an output directory**

```bash
mkdir -p ~/aws-inventory-demo
```

Expected result: a new directory is created.

**Step 2 — Build the viewer and data generator**

From the repository root:

```bash
cargo build -p cmdb-ui -p cmdb-bench --release
```

Expected result: `target/release/cmdb-ui` and `target/release/cmdb-bench` binaries are built.

**Step 3 — Generate a synthetic inventory database**

```bash
./target/release/cmdb-bench generate ~/aws-inventory-demo/inventory.db 2 50
```

Expected result:
```
created_synthetic_reader_fixture schema_version=15 resources=100 database=/home/USER/aws-inventory-demo/inventory.db
```
The command creates a SQLite database with 100 synthetic AWS resources across 2 accounts.

**Step 4 — Open the viewer** (requires a graphical session: a running Wayland or X11 desktop with `WAYLAND_DISPLAY` or `DISPLAY` set)

```bash
./target/release/cmdb-ui --database ~/aws-inventory-demo/inventory.db
```

Expected result: The Makepad application window opens. The left panel lists the account, region and resource-kind chips derived from the database; the header shows account, resource, region and service totals. The Resources list (Table or Graph tab) shows a bounded page of resources. Click an account, region, or resource kind to filter. Click a resource row to select it; the graph tab shows its neighbors and relationships.

**Step 5 (optional) — Enable the resource graph**

Prerequisites: a running FalkorDB server, and an `inventory` graph already populated from the inventory. This walkthrough does not cover starting FalkorDB or populating the graph. With an unpopulated graph, the viewer reports the graph as offline with a query error (for example, `Invalid graph operation on empty key`).

```bash
./target/release/cmdb-ui --database ~/aws-inventory-demo/inventory.db \
  --graph-url redis://localhost:6379 --graph-name inventory
```

Expected result: Graph queries execute against FalkorDB and the graph tab shows stored relationships for the selected resource. Without these flags, inventory remains fully usable and the graph panel reports its offline state.

## Build

```bash
cargo build -p cmdb-ui --release
```

Makepad is a git dependency pinned in the workspace Cargo.toml; the first build downloads and compiles the full Makepad toolkit.

## Arguments

`cmdb-ui --help` (or `-h`) prints the usage and exits 0 without opening the window. An unknown or incomplete argument prints the error and usage to stderr and exits nonzero, also without opening the window.

| Flag | Required | Description |
|------|----------|-------------|
| `--database PATH` | Yes | Path to the SQLite inventory database |
| `--graph-url URL` | No | FalkorDB endpoint (use with `--graph-name`) |
| `--graph-name NAME` | No | FalkorDB graph name (use with `--graph-url`) |
| `--linux-backend=BACKEND` | No | Force the Linux display backend, e.g. `x11` or `wayland` (must use `=`; a space-separated value is rejected as an unknown argument) |
| `--hot` | No | Makepad hot reload (development) |
| `--stdin-loop` | No | Makepad hosted-child mode (started by a Makepad host) |
| `-h`, `--help` | No | Print usage and exit |

## Viewer behavior

- Loads SQL account, region, service, and resource totals and a bounded page of 500 matching resources.
- Account, region, and resource-kind chips are derived from loaded database rows. Search and filters query the read-only database in the worker.
- Selecting a resource asks FalkorDB for its bounded neighborhood and shows actual stored relationships. Selecting a graph resource updates the detail panel and expands that resource.
- Empty databases, database failures, graph-offline states, and no-neighbor results are displayed separately.
- The viewer does not connect to AWS or modify the database.