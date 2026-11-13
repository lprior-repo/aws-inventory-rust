# cmdb-api

Read-only HTTP query API over one `aws-inventory export --format json` snapshot.
The export is read once, parsed into `cmdb-view`, fingerprinted, and then served
from memory; no database, network collector or write path is involved.

## Run

```sh
cargo run -p cmdb-api -- \
  --export path/to/inventory.json \
  --bind 127.0.0.1:8787
```

`--bind` defaults to `127.0.0.1:8787`. The process prints one JSON `ready` line
with the effective bind address and snapshot fingerprint. A parse failure (or any
other startup failure) prints one JSON line to stderr:

```json
{"event":"startup_failed","kind":"parse_export","message":"export parse failed: ..."}
```

## Local-only / no auth

There is **no authentication or authorization**. The listener therefore refuses
any non-loopback bind (`0.0.0.0`, `::`, LAN addresses) at startup; expose it only
through loopback or a separately authenticated proxy. CORS allows only GET (plus
its preflight), and every route is read-only.

## Endpoints

All responses are JSON and carry the snapshot `fingerprint`.

| Route | Query | Notes |
| --- | --- | --- |
| `GET /health` | - | `{"status":"ok","fingerprint":...}` |
| `GET /api/v1/meta` | - | scan identity, counts, digests |
| `GET /api/v1/resources` | `account`, `region`, `kind`, `contains`, `limit`, `cursor` | exact filters AND-combined; `contains` is ASCII case-insensitive; `limit` defaults to 100, max 10,000; opaque `cursor` resumes after a page |
| `GET /api/v1/resources/{id}` | - | one resource; unknown id is 404 |
| `GET /api/v1/graph/{id}` | `depth` (1-2, default 1), `max_nodes` (default 1,000), `max_edges` (default 2,000) | caps bounded by `cmdb-view` (10,000 nodes, 20,000 edges); dangling targets are returned with `present:false`; unknown root is 404 |
| `GET /api/v1/lod` | `top` (default 25, max 1,000), `buckets` (default 256, max 1,024) | `buckets` caps the kind and account axes |
| `GET /api/v1/coverage` | - | collection scopes and failures |

Errors are typed:

```json
{"error":{"code":"invalid_page_limit","message":"page limit must be in 1..=10000"}}
```

Codes include `malformed_query`, `malformed_id`, `query_too_long`,
`payload_too_large`, `invalid_page_limit`, `invalid_cursor`,
`invalid_graph_depth`, `invalid_graph_limit`, `invalid_lod_limit`,
`resource_not_found`, `not_found`, and `method_not_allowed`.

Request bounds: query strings above 4 KiB and declared bodies above 1 KiB are
refused; GET handlers never read a body.
