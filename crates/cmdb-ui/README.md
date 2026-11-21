# cmdb-ui

A read-only Makepad desktop viewer for inventory stored in the schema17 SQLite database. SQL inventory access and FalkorDB graph queries run on a crate-owned background worker; the UI thread only applies owned snapshots.

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
cargo build -p cmdb-ui -p cmdb-bench -p aws-inventory --release
```

Expected result: `target/release/cmdb-ui`, `target/release/cmdb-bench` and `target/release/aws-inventory` binaries are built.

**Step 3 — Generate a synthetic inventory database**

```bash
./target/release/cmdb-bench pipeline ~/aws-inventory-demo/inventory.db catalog/aws-api.catalog 1 50 2
```

Expected result: a `pipeline_state=initial resources=100` line, then three `pipeline_phase=` lines (`initial`, `tag_update`, `duplicate_replay`), each reporting `resources=100`.

The command creates a schema-16 SQLite database with 100 synthetic AWS resources across 2 accounts, plus the projection outbox that Step 5a needs. It makes no AWS calls. The database path must not already exist; an existing file is rejected with a usage error.

**Step 4 — Open the viewer** (requires a graphical session: a running Wayland or X11 desktop with `WAYLAND_DISPLAY` or `DISPLAY` set)

```bash
./target/release/cmdb-ui --database ~/aws-inventory-demo/inventory.db
```

Expected result: The Makepad application window opens. The left panel lists the account, region and resource-kind chips derived from the database; the header shows account, resource, region and service totals. The Resources list (Table or Graph tab) shows a bounded page of resources. Click an account, region, or resource kind to filter. Click a resource row to select it; the graph tab shows its neighbors and relationships.

**Step 5 (optional) — Enable the resource graph**

Prerequisites: a running FalkorDB server with the `inventory` graph populated from this database (Step 5a). With an unpopulated graph, the viewer reports the graph as offline with a query error (for example, `Invalid graph operation on empty key`).

```bash
./target/release/cmdb-ui --database ~/aws-inventory-demo/inventory.db \
  --graph-url redis://localhost:6379 --graph-name inventory
```

Expected result: Graph queries execute against FalkorDB and the graph tab shows stored relationships for the selected resource. The walkthrough database stores no AWS relationship edges (its `edges` table is empty; the graph holds only Resource-to-Tag links), so for a selected resource the graph tab reports `No graph neighbors were recorded`. That is the online, no-neighbor state, not the offline state. Without these flags, inventory remains fully usable and the graph panel reports its offline state.

**Step 5a — Start FalkorDB and populate the graph**

This step runs the locally built FalkorDB and rebuilds the `inventory` graph from the SQLite database. It makes no AWS calls. Run it from the repository root.

Check that port 6379 is free, then start the server from the runtime installed by `scripts/local-stack.sh install`:

```bash
ss -ltnp | grep ':6379 ' || echo "port 6379 is free"
RUNTIME=$PWD/.local/runtime/graph
$RUNTIME/redis-8.10.3/src/redis-server --loadmodule $RUNTIME/falkordb.so \
  --port 6379 --save "" --appendonly no --daemonize yes
$RUNTIME/redis-8.10.3/src/redis-cli -p 6379 ping
```

Expected result: `PONG`.

Rebuild the graph from the database. This replaces the contents of the named graph:

```bash
./target/release/aws-inventory rebuild-graph --database ~/aws-inventory-demo/inventory.db \
  --graph-url redis://localhost:6379 --graph-name inventory
```

Expected result:
```
{
  "graph": "inventory",
  "pending": 0,
  "status": "rebuilt"
}
```

If the rebuild instead fails with `projection delivery ownership was superseded`, the server still holds a delivery fence from an earlier database (the key `aws-inventory:projection-owner:inventory`). Only on a server dedicated to this walkthrough, delete that key with `$RUNTIME/redis-8.10.3/src/redis-cli -p 6379 DEL aws-inventory:projection-owner:inventory` and rerun the rebuild. Starting from a freshly started server avoids this.

Verify that the graph matches the database. The first result value of each graph query must equal the SQL count. For this walkthrough database they are 100 resources and 148 tags:

```bash
$RUNTIME/redis-8.10.3/src/redis-cli -p 6379 GRAPH.QUERY inventory "MATCH (n:Resource) RETURN count(n)"
$RUNTIME/redis-8.10.3/src/redis-cli -p 6379 GRAPH.QUERY inventory "MATCH ()-[r:HAS_TAG]->() RETURN count(r)"
sqlite3 ~/aws-inventory-demo/inventory.db "SELECT count(*) FROM resources"
sqlite3 ~/aws-inventory-demo/inventory.db "SELECT count(*) FROM tags"
```

When finished, stop the server:

```bash
$RUNTIME/redis-8.10.3/src/redis-cli -p 6379 shutdown nosave
```

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