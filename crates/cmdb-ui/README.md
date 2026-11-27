# cmdb-ui

A read-only Makepad desktop viewer for a PostgreSQL inventory database written by `aws-inventory sweep`. Inventory reads and FalkorDB graph queries run on a crate-owned background worker; the UI thread only applies owned snapshots.

## Walkthrough: sweep to DAG

This walkthrough stores a sweep in PostgreSQL and views it. Only the sweep calls AWS, through the standard credential chain; the viewer makes no AWS calls and does not modify the database. The graph (FalkorDB) is optional. Run commands from the repository root, because the sweep reads `catalog/aws-api.catalog` relative to it.

**Step 1 — Build**

```bash
cargo build --locked --release --features aws -p aws-inventory
cargo build --locked --release -p cmdb-ui
```

Expected result: `target/release/aws-inventory` (the sweep requires the `aws` feature) and `target/release/cmdb-ui` are built.

**Step 2 — Set the database URLs**

Use two roles on one database: a writer for the sweep and a read-only role for the
viewer. Neither URL contains a password. SQLx reads the password from `PGPASSFILE`, or
from `~/.pgpass` when `PGPASSFILE` is unset, while it parses the URL. Put one line per
role in that file, in the form `host:port:database:user:password`, and set `chmod 600`:

```
db.example.com:5432:inventory:inventory_writer:<writer password>
db.example.com:5432:inventory:inventory_viewer:<viewer password>
```

Set the URLs in a private shell profile:

```bash
# Sweep writer (write role). The sweep reads this variable (clap `env`), not a flag.
export INVENTORY_SWEEP_DATABASE='postgresql://inventory_writer@db.example.com:5432/inventory?sslmode=require'

# Viewer (read-only role). Shell variable you define; the viewer receives it only via --database.
export INVENTORY_VIEWER_URL='postgresql://inventory_viewer@db.example.com:5432/inventory?sslmode=require'
```

`INVENTORY_SWEEP_DATABASE` keeps the writer URL out of the sweep's argument list.

Bootstrap privileges: the sweep creates the `inventory` schema itself on first use, so
the writer role needs `CONNECT` and `CREATE` on the database for that first run. Do not
pre-create an empty `inventory` schema; the sweep must create it. The store uses
temporary tables, so if `TEMP` has been revoked from the writer role, grant it back
before the sweep.

**Step 3 — Sweep the account into PostgreSQL**

```bash
./target/release/aws-inventory sweep --profile audit --region us-east-1
```

Expected result: a sweep summary is printed and the resources are stored in PostgreSQL. Exit codes: `0` the sweep completed and any requested graph delivery drained; `2` a scope was partial, failed, inaccessible, unsupported or unobserved, or a resource lacked attribution; `3` (only with `--drain`) graph delivery is still pending; `4` work remains, so rerun with `--resume`; `5` (only without `--drain`) a projection was attempted and is still pending. When several apply, the precedence is 4, then 3, then 2, then 5.

**Step 3a (optional) — Project the graph with the sweep**

The graph needs a FalkorDB endpoint on its own port, not 6379: 6379 is Redis's default
and another server may already own it. This walkthrough assumes an operator-owned
FalkorDB server is already running and reachable at `127.0.0.1:$PORT`. To provision and
start one, follow the local-stack instructions in
[scripts/local-stack.sh](../../scripts/local-stack.sh). Its default port is `16379`
(`FALKORDB_PORT`). This walkthrough does not start, stop or signal any process.

```bash
STATE_DIR="${AWS_INVENTORY_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/aws-inventory}"
RUNTIME="$STATE_DIR/runtime/graph"
PORT="${FALKORDB_PORT:-16379}"
"$RUNTIME/redis-8.10.3/src/redis-cli" -p "$PORT" ping
```

Expected result: `PONG`. Then run the sweep with the graph flags on the same port.
`--drain` waits until the projection outbox is delivered:

```bash
./target/release/aws-inventory sweep --profile audit --region us-east-1 \
  --graph-url "redis://127.0.0.1:$PORT" --graph-name aws_inventory --drain
```

With `--drain`, a pending graph delivery exits `3` (exit `4` still takes precedence when
work remains). Without `--drain`, a pending projection exits `5`. If you skip this step,
the viewer still works and reports the graph as offline.

**Step 4 — Open the viewer** (requires a graphical session: a running Wayland or X11 desktop with `WAYLAND_DISPLAY` or `DISPLAY` set)

```bash
./target/release/cmdb-ui --database "$INVENTORY_VIEWER_URL"
```

Expected result: The Makepad application window opens. The left panel lists the account, region and resource-kind chips derived from the database; the header shows account, resource, region and service totals. The Resources list (Table or Graph tab) shows a bounded page of resources. Click an account, region, or resource kind to filter. Click a resource row to select it; the graph tab shows its neighbors and relationships.

The viewer takes its URL only as a flag, so the URL is visible to other local users in the
process list (`ps`). The URL carries no password, so no secret appears there. Give the
viewer a role with read access and nothing more; the viewer never writes.

**Step 5 (optional) — Show the graph in the viewer**

Requires Step 3a, so the `aws_inventory` graph is populated:

```bash
./target/release/cmdb-ui --database "$INVENTORY_VIEWER_URL" \
  --graph-url "redis://127.0.0.1:$PORT" --graph-name aws_inventory
```

Expected result: graph queries run against FalkorDB and the graph tab shows stored relationships for the selected resource. A resource with no recorded relationships shows `No graph neighbors were recorded`, which is the online, no-neighbor state rather than the offline state. Without these flags, the inventory is fully usable and the graph panel reports its offline state.

Check the graph count. After a drained projection it is nonzero, and the viewer header shows the database's resource total:

```bash
$RUNTIME/redis-8.10.3/src/redis-cli -p "$PORT" GRAPH.QUERY aws_inventory "MATCH (n:Resource) RETURN count(n)"
```

This walkthrough does not start, stop or signal the FalkorDB server; the local-stack
instructions cover its lifecycle.

## Build

```bash
cargo build -p cmdb-ui --release
```

Makepad is a git dependency pinned in the workspace Cargo.toml; the first build downloads and compiles the full Makepad toolkit.

## Arguments

`cmdb-ui --help` (or `-h`) prints the usage and exits 0 without opening the window. An unknown or incomplete argument prints the error and usage to stderr and exits nonzero, also without opening the window.

| Flag | Required | Description |
|------|----------|-------------|
| `--database URL` | Yes | PostgreSQL URL (`postgres://` or `postgresql://`). Any other value, including a file path, is rejected without echoing the value |
| `--graph-url URL` | No | FalkorDB endpoint (use with `--graph-name`) |
| `--graph-name NAME` | No | FalkorDB graph name (use with `--graph-url`); must match the sweep's `--graph-name` (default `aws_inventory`) |
| `--linux-backend=BACKEND` | No | Force the Linux display backend, e.g. `x11` or `wayland` (must use `=`; a space-separated value is rejected as an unknown argument) |
| `--hot` | No | Makepad hot reload (development) |
| `--stdin-loop` | No | Makepad hosted-child mode (started by a Makepad host) |
| `-h`, `--help` | No | Print usage and exit |

## Viewer behavior

- Loads SQL account, region, service, and resource totals and a bounded page of 500 matching resources.
- Account, region, and resource-kind chips are derived from loaded database rows. Search and filters query the read-only database in the worker.
- Selecting a resource asks FalkorDB for its bounded neighborhood and shows actual stored relationships. Selecting a graph resource updates the detail panel and expands that resource.
- Empty databases, database failures, graph-offline states, and no-neighbor results are displayed separately.
- Each page refresh and each graph-detail enrichment reads in its own short-lived, pinned repeatable-read, read-only snapshot, opened and closed by the worker. One page is consistent within itself; two consecutive reads may reflect different committed states, so a live sweep can change the totals between refreshes.
- The viewer does not connect to AWS or modify the database.
