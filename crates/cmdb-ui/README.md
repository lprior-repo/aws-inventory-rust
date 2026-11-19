# cmdb-ui

A read-only Makepad desktop viewer for inventory stored in the schema16 SQLite database. SQL inventory access and FalkorDB graph queries run on a crate-owned background worker; the UI thread only applies owned snapshots.

## Run

From the workspace root:

```bash
cargo run -p cmdb-ui -- --database PATH
cargo run -p cmdb-ui -- --database PATH --graph-url FALKORDB_URL --graph-name GRAPH_NAME
```

`--database` is required. Graph configuration is optional; provide both `--graph-url` and `--graph-name` to enable graph expansion. Without graph configuration or when FalkorDB is unreachable, inventory remains usable and the graph panel reports its offline/error state. The viewer does not connect to AWS or modify the database.

## Viewer behavior

- Loads SQL account, region, service, and resource totals and a bounded page of 500 matching resources.
- Account, region, and resource-kind chips are derived from loaded database rows. Search and filters query the read-only database in the worker; the table status reports matching versus loaded rows.
- Selecting a resource asks FalkorDB for its bounded neighborhood and shows actual stored relationships; if the query reaches a traversal limit, the graph status marks the result truncated. Selecting a graph resource updates the detail panel and expands that resource.
- Empty databases, database failures, graph-offline states, and no-neighbor results are displayed separately.
- The first visible resource is selected after each page update. All detail fields are derived from stored resource identity and record metadata.

Makepad's Linux window backend can be selected with `--linux-backend=x11` or `--linux-backend=wayland` before the application arguments.
