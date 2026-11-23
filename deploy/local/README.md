# Local PostgreSQL (C6.2)

The harness uses an existing server through `INVENTORY_POSTGRES_URL`. The URL
must connect to an administrative database as a role with `CREATEDB` privilege
and permission to drop its own test databases. Each `TestDatabase` creates a
separate database; tests never share one. PostgreSQL 13 or newer is required
for forced cleanup. No PostgreSQL process is started by the test library.

## Podman or Docker

With `inventory-harness` on PATH:

```sh
inventory-harness postgres up
export INVENTORY_POSTGRES_URL="$(inventory-harness postgres url)"
cargo test -p inventory-harness --locked -j2 postgres -- --ignored
inventory-harness postgres down
```

`up` prefers a working Podman, then Docker; select explicitly with
`up --runtime docker` or `up --runtime podman`. `down` also accepts `--runtime`.
Without a working runtime it exits 2 and points here; use the native path below.
The compose path is resolved from the harness build's workspace, not the current
directory. The command has a 120-second deadline; runtime probes have 10 seconds.
A failed start may leave a container: use `down` to stop it.

`compose.yaml` pins the official multi-platform `16-bookworm` image by digest,
resolved from Docker Hub. C6.2 explicitly specifies version 16;
the newer architecture target of 19 is not claimed as verified by this setup.
The named volume is `inventory-local_inventory-pgdata`, never a bind mount.
`down` keeps it; `down --volumes` irreversibly deletes it. The port is loopback
only, `127.0.0.1:54329`. The local-only default password is
`inventory-local-only`; never reuse it. Set `INVENTORY_POSTGRES_PASSWORD` before
initializing a new volume to override it. Changing that variable does not
change an existing volume's database password. `postgres url` prints the URL
(including its secret) intentionally, for shell assignment; do not log it.
If `INVENTORY_POSTGRES_URL` is already set, `url` returns that validated URL.
Unset it before asking for the compose default.

## Native server (unprivileged user)

Run this as your ordinary non-root login user. A root shell must delegate
`initdb`, `pg_ctl`, and directory ownership to an unprivileged user (for
example `runuser -u postgres --`). Do not use the system PostgreSQL data directory.
The owner-only temporary directory and socket make `--auth=trust` appropriate
only for this throwaway local server. There is no TCP listener.

```sh
pgdir=$(mktemp -d /tmp/inventory-pg.XXXXXXXX)
trap 'pg_ctl -D "$pgdir/data" -m immediate -w stop; rm -rf "$pgdir"' EXIT
initdb -D "$pgdir/data" --auth=trust -U inv
pg_ctl -D "$pgdir/data" -l "$pgdir/server.log" \
  -o "-c listen_addresses='' -c unix_socket_directories=$pgdir" -w start
export INVENTORY_POSTGRES_URL="postgres://inv@localhost/postgres?host=$pgdir"
psql "$INVENTORY_POSTGRES_URL" -Atc 'SHOW server_version'
cargo test -p inventory-harness --locked -j2 postgres -- --ignored
pg_ctl -D "$pgdir/data" -m fast -w stop
rm -rf "$pgdir"
trap - EXIT
```

For benchmark parity, append these `-c` options to `pg_ctl -o` (rather than
assuming container/native defaults match):

```text
-c shared_buffers=1GB -c effective_cache_size=3GB -c work_mem=16MB
-c maintenance_work_mem=256MB -c max_wal_size=4GB -c checkpoint_timeout=15min
-c max_connections=200 -c max_locks_per_transaction=256
-c synchronous_commit=on -c fsync=on -c full_page_writes=on
```

These are starting settings, not tuned performance claims. The compose shared
memory budget is 2 GiB. B4.4 measures tuning and B5.2 records effective server
settings. The native acceptance run uses the smaller native memory defaults;
it proves lifecycle behavior, not benchmark parity.

## Cleanup and boundedness

Call `TestDatabase::finish().await?` for observable cleanup errors. Dropping the
guard also drops its database with `WITH (FORCE)`, on a joined helper thread
with its own runtime; it does not spawn detached tasks. This synchronous Drop
can block for up to the database deadline, so explicit async cleanup is preferred.
On failure Drop reports the error without printing credentials. Process kills
cannot run Drop: `inventory-harness postgres prune` drops only strictly valid
`inv_t_<unix time hex>_<pid hex>_<random hex>` names older than 3,600 seconds.
Never use that prefix for persistent databases. `prune` refuses more than 256
candidates rather than loading or dropping an unbounded collection. Database
operations have a 15-second deadline and DDL has a 10-second server statement
timeout. URLs are bounded to 8,192 bytes. No implicit database retries exist.

No CI configuration or `.moon/` task-definition file exists in this checkout;
the coordinator excluded adding a Moon task and CI service in this change.
A container runtime is not available on the implementation machine, so the
container path still requires a real Podman/Docker validation run. Stub tests
cover detection and compose arguments, not container availability.

## Acceptance evidence

Native PostgreSQL reported `18.6`. The throwaway server used the socket-only
recipe above and was stopped by its exit trap after the checks. With
`HOME=/home/lewis` and
`CARGO_TARGET_DIR=/home/lewis/.cache/cargo-target-c62`:

```text
cargo test -p inventory-harness --locked -j2 postgres -- --ignored
test result: ok. 5 passed; 0 failed; 0 ignored
cargo test -p inventory-harness --locked -j2 postgres
test result: ok. 8 passed; 0 failed; 5 ignored
```

These are filtered harness checks, not a workspace or container validation.
The initial acceptance compilation found SQLx 0.9's owned SQL-string boundary;
DDL now moves its validated identifier statement through `AssertSqlSafe`.

| Given | When | Then | Scenario |
| --- | --- | --- | --- |
| An administrative URL | A database is created, connected and finished with a connection still open | Forced cleanup removes it | `postgres_create_connect_finish_removes_database` |
| One server | Two database guards are created concurrently | Names differ and both databases exist independently | `postgres_simultaneous_databases_are_isolated` |
| A created guard | It leaves scope without `finish` | Its database no longer exists | `postgres_drop_cleans_without_finish` |
| Old and fresh harness databases | Pruning runs | Only the old database is removed | `postgres_prune_removes_old_leaves_fresh` |
| The administrative `postgres` database | Pruning runs | The unrelated database remains | `postgres_unrelated_databases_survive_prune` |

Runtime contract tests use executable stubs to prove Podman preference, Docker
fallback, explicit selection, compose arguments, failure propagation, and
native-server instructions. They do not prove a real container starts.
