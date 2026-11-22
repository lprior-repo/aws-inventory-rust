# Track E steps: relationships and graph

> Status: Plan, 2026-11-15, revised for V53 (edges and projection reads are on
> PostgreSQL only). Step-by-step breakdown of the packages in
> [E_GRAPH.md](../E_GRAPH.md). Lighter than tracks A to C but concrete.
> Nothing here has been built.

Conventions: as in [STEPS_A.md](STEPS_A.md).

Facts this track relies on, read from the tree:

- `catalog/kinds.tsv` has 42 `relation` rows; 1 is live after A0.2.
- The 968 reviewed rules are JSON pointers against CloudFormation properties:
  945 under `resource_types[].relationships` in
  `catalog/cloudcontrol-resources.json` and 23 in
  `catalog/cloudcontrol-parent-resources.json` (counted with `jq`).
- The model compiler does not keep the reference traits: `RETAIN_PAYLOADS` in
  `crates/catalog-tool/src/smithy/model/traits/retained.rs` has neither
  `smithy.api#references` nor `aws.api#arnReference`.
- The sweep emits only reviewed-rule references
  (`rows.rs::reference` sets `EdgeSource::ReviewedRule`) and a relation's
  source path must lead to one scalar (`record::path_value`), so a reference
  inside a list is not followed.
- Cypher text exists in two crates: `crates/engine/src/inventory/projection/
  cypher.rs` and `crates/cmdb-view/src/graph/query.rs`, plus the schema 13
  path under `crates/engine/src/graph/`.

## E1 All three edge sources

### E1.1 Model-declared references

- **Goal:** Edges from the model's own reference traits, labelled as such.
- **Depends on:** A2.2, A0.2.
- **Files:** `crates/catalog-tool/src/smithy/model/traits/retained.rs`,
  `crates/catalog-tool/src/runtime_catalog/relations.rs` (from A0.2),
  `runtime_catalog/relations/declared.rs` (new), `runtime_catalog/encode.rs`
  (relation field 7), `crates/core/src/catalog/runtime/types.rs`
  (`RelationshipPlan`), `crates/core/src/catalog/runtime/decode/metadata.rs`,
  `crates/sweep/src/execution/page/rows.rs`.
- **Change:**
  1. Keep the payloads of `smithy.api#references` and `aws.api#arnReference`
     in the base catalog.
  2. `declared.rs`: for each callable kind's item shape, walk members. A
     member carrying `aws.api#arnReference` with a `resource` shape ID, or a
     structure carrying `smithy.api#references`, yields a relation when the
     referenced resource shape belongs to a callable kind in the same model
     (through the resource's `list` or `read` operation's kind). Relation
     label `REFERENCES`; target path is the target kind's identity member.
     A reference to a resource of another service resolves by ARN in E1.2.
  3. `RelationshipPlan` gains `source: EdgeSource`; catalog relation field 7
     (0 model, 1 reviewed rule). `rows.rs::reference` uses it.
  4. When a reviewed rule and a declared reference name the same source path
     and target kind, the reviewed rule wins and the declared one is dropped
     (one edge, labelled reviewed).
- **Tests:** a fixture model with each trait; a reference to a non-callable
  kind goes to the pending list of A0.2; rule beats declaration; the reader
  rejects an unknown source code.
- **Budget:** P16 catalog bytes (the retained payloads add to the base
  catalog); P1 on `thin` with the extra references.
- **Prove it:** `models compile --write` then `--check`; `catalog-tool` tests;
  the relations report shows a `declared` count.
- **Done when:** the report states how many declared relations are live for
  the thin slice (the models hold 534 reference traits and 200 ARN-reference
  traits in total, VISION section 3).
- **Size:** about 320 added. One change.
- **Watch out:** changing `RETAIN_PAYLOADS` changes the base catalog bytes for
  every service; record the size before and after.

### E1.2 Inferred edges by exact match

- **Goal:** An edge where a field value exactly equals one collected
  resource's ARN or unique ID; ambiguous matches counted, not drawn; late
  targets resolved.
- **Depends on:** C1.4, C6.8.
- **Files:** `crates/engine/src/inventory/postgres/schema/shared.rs` (table
  `identities`), `inventory/postgres/infer.rs` (new),
  `inventory/postgres/commit.rs`, `inventory/store_api.rs` (`infer_edges`),
  `crates/catalog-tool/src/runtime_catalog/schemas/columns.rs` (flag),
  `crates/sweep/src/execution/draining.rs`.
- **Change:**
  1. Table `identities(value text, account text, region text, kind text, id
     text, PRIMARY KEY(value, account, region, kind, id))`: one row per
     resource for its `id`, and one for its ARN when the kind has an ARN
     column. Maintained in the page commit for new, changed and deleted
     resources.
  2. Catalog column flag `inferable` (column map field 4, boolean): text
     columns whose last member name ends in `Arn` or `Id`, are not part of the
     kind's own identity and are not already the source of a declared or
     reviewed relation.
  3. `infer_edges(sweep)` runs once when the queue is empty, set-based, one
     statement per source kind and inferable column: join the kind table to
     `identities` on the value, group by source row and value, insert an edge
     with relation `REFERENCES` and source `inferred` where exactly one
     identity matches, and add the others to a per-sweep counter
     `ambiguous_inferred`. Only rows changed in this sweep, or values equal to
     an identity added in this sweep, are considered, so the cost follows
     change.
  4. Edges of source `inferred` whose target no longer matches are removed in
     the same pass.
- **Tests:** unique match draws one edge; two resources with the same ID in
  different kinds draw none and count one; a target collected in a later
  sweep gets its edge although the source did not change; a deleted target
  loses its edge; a column that already has a reviewed rule is not inferred
  twice.
- **Budget:** P1 and P2 on `thin` and `full` with inference on and off; the
  number of statements in the pass (must not depend on row counts).
- **Prove it:** store conformance tests; e2e run with a seeded cross-service
  reference that no rule covers.
- **Done when:** the fixture cases pass and the cost rows are recorded.
- **Size:** about 380 added. Two changes: (1) `identities` and the flag,
  (2) the pass.
- **Watch out:** schema chain link 6: `identities` is a new table and lands in
  its own turn. Choosing columns by name suffix selects candidates for
  inference; it authorizes no call. A unique match is still a guess: the edge
  is labelled `inferred` everywhere it is shown.
- **Review fix (2026-11-18):** inferred edges need a namespace and a way
  back. Matching rules: an ARN matches globally; a bare ID matches only
  inside the same account and, for a regional kind, the same region; nothing
  matches across partitions. `identities` already carries account and
  region; the match query uses them. Retraction: when an identity is added,
  removed or changed, the sources whose values equal it are recomputed in
  the same pass; an edge whose match is no longer unique is removed and
  counted as ambiguous, and one that becomes unique is drawn. Tests: the
  same ID in two accounts draws no cross-account edge; a second resource
  with the same ID appears and the existing edge is retracted; it disappears
  and the edge returns; the target's identity changes.

### E1.3 Counts by source, and the fixture evidence

- **Goal:** The sweep summary prints edge counts by source, and the M3.7
  fixture tests exist for every source.
- **Depends on:** E1.1, E1.2.
- **Files:** `crates/engine/src/inventory/postgres/outbox.rs`
  (`sweep_summary`), `crates/sweep/src/lib.rs`
  (`SweepSummary`), `src/sweep_command/enabled.rs`,
  `crates/engine/tests/inventory_store/conformance.rs`.
- **Change:** `SweepSummary` gains `edges_model`, `edges_rule`,
  `edges_inferred`, `ambiguous`, `unresolved`; the metrics line prints them.
  Conformance cases: each source yields an edge with its label; an ambiguous
  value yields none and raises the count; an edge appears when its target is
  collected later; it disappears when the target is deleted.
- **Tests:** as listed.
- **Prove it:** the conformance suite; the summary line of an e2e run.
- **Done when:** the E1 exit evidence holds.
- **Size:** about 180 added. One change.
- **Watch out:** the first summary line is parsed by the e2e suite; add to
  the metrics line only.

## E2 Remap the reviewed rules

### E2.1 The remap tool and its report

- **Goal:** Turn the 968 CloudFormation-pointer rules into API field-path
  relations where the mapping is certain, and report the rest with reasons.
- **Depends on:** A2.1.
- **Files:** `crates/catalog-tool/src/relations.rs` (new),
  `relations/remap.rs` (new), `relations/pointer.rs` (new), `main.rs`,
  `catalog/relations.generated.tsv` (new, generated),
  `catalog/aws-relations-remap.report` (new, generated),
  `kind_map.rs` (load the generated relation rows), `moon.yml`.
- **Change:**
  1. `catalog-tool relations remap [--check | --write]` reads `relationships`
     from both JSON files. Each rule has `pointer`, `relation`, `target_kind`
     (a legacy name such as `aws_kms_key`), `target_format`.
  2. Source: the rule's CloudFormation type must map to a kind with a callable
     operation. Each pointer segment must name a member of the kind's item
     shape along the path, by exact name; `*` means "each element" and is
     dropped from the path (member paths traverse lists implicitly).
  3. Target: map the legacy target name to a CloudFormation type through
     `view_name` in the same JSON, then to a kind; the target path is that
     kind's ARN column for `target_format = "arn"`, its identity member
     otherwise. `aws_reference` targets (any ARN) are left to E1.2.
  4. Output rows in the `relation` format of `kinds.tsv`:
     `relation <kind> <path> <target kind> <target path> <label> <scope>`.
     Hand rows in `kinds.tsv` win over generated ones with the same source.
  5. Report per type: mapped and unmapped counts, and per unmapped rule its
     reason: `type not callable`, `pointer segment <x> not in item shape`,
     `target kind not callable`, `generic ARN target`.
- **Tests:** exact mapping; a pointer with `*`; a segment that differs in
  case only is unmapped with its reason (no guessing); a hand row wins;
  regenerating gives identical bytes; totals sum to 968.
- **Prove it:** `catalog-tool relations remap --check`; `models compile
  --check` (the remapped relations go through A0.2's live and pending split).
- **Done when:** the report exists and every mapped rule's target resolves to
  a kind in the kind map.
- **Size:** about 400 added. One change.
- **Watch out:** CloudFormation property names and API member names often
  agree but not always (`KmsKeyId` against `KMSKeyId`); an unmapped rule keeps
  its reason and waits for a hand row. `kind_map::MAX_ROWS` is 10,000.

### E2.2 References inside lists, and the seeded proof

- **Goal:** Follow a relation whose source path goes through a list, and show
  a remapped rule yielding a seeded edge.
- **Depends on:** E2.1, A4.3.
- **Files:** `crates/sweep/src/record.rs` (`path_value`),
  `crates/sweep/src/record/multi.rs` (new),
  `crates/sweep/src/execution/page/rows.rs` (`reference`),
  `crates/catalog-tool/src/runtime_catalog/model_path/validation.rs`.
- **Change:** `path_values(root, path) -> Vec<ValueRef>` walks a path and
  fans out at each list, bounded at 1,024 values per record and row
  (`MAX_PAGE_REFERENCES` still bounds the page). `reference` emits one
  unresolved reference per value. The catalog validation accepts a relation
  path that crosses lists when the terminal is a string. Seed one pair the
  emulator accepts whose rule crosses a list (for example an instance's block
  device KMS key, or a Lambda function's VPC subnets) and assert the edge.
- **Tests:** fan-out through one and two lists; the bound; an empty list
  yields nothing; e2e assertion on the seeded edge.
- **Budget:** P6 on `thin` (the fan-out runs per row).
- **Prove it:** `cargo test -p inventory-sweep --locked multi`; e2e run.
- **Done when:** the E2 exit evidence holds for at least one seeded pair.
- **Size:** about 220 added. One change.
- **Watch out:** which pairs the emulator can seed is not known until A4.3.

## E3 Graph interface

### E3.1 A graph crate with one trait and one Cypher module

- **Goal:** Projection and traversal go through one trait; Cypher text lives
  in one module; FalkorDB is the only implementation.
- **Depends on:** C5.1 (to avoid colliding edits in
  `crates/engine/src/inventory.rs`).
- **Files:** `crates/graph-store/Cargo.toml` (new; package `inventory-graph`),
  `crates/graph-store/src/lib.rs` (new), `src/cypher.rs` (new; from
  `crates/engine/src/inventory/projection/cypher.rs`, 187 lines, and the
  query text of `crates/cmdb-view/src/graph/query.rs`), `src/falkor.rs` (new;
  from `projection/client.rs`), `Cargo.toml` (workspace member).
- **Change:**
  1. `pub trait GraphBackend: Send + Sync` with methods returning `impl Future
     + Send`: `upsert_node(&NodeRow)`, `delete_node(&NodeKey)`,
     `replace_edges(&NodeKey, &[EdgeRow])`, `clear(&self)`,
     `neighbors(&NodeKey, GraphLimits)`, `path(&NodeKey, &NodeKey,
     GraphLimits)`, `blast_radius(&NodeKey, GraphLimits)`, `node_keys(after,
     limit)`, `edge_keys(after, limit)`.
  2. `cypher.rs` holds every query builder. It stays inside the openCypher
     subset FalkorDB and Neptune share: `MATCH`, `MERGE`, `SET`, `DETACH
     DELETE`, parameters, variable-length paths with explicit bounds; no
     procedures and no FalkorDB-only functions. A doc file
     `docs/GRAPH_SUBSET.md` (new) lists what is used.
  3. `falkor.rs` implements the trait over `redis` (`GRAPH.QUERY`).
  4. The crate depends on `inventory-core` only (plus `redis`, `thiserror`,
     `tokio`).
- **Tests:** the moved Cypher builder tests; the trait methods against a live
  FalkorDB (ignored).
- **Prove it:** `cargo test -p inventory-graph --locked`; with a graph URL,
  `-- --ignored`.
- **Done when:** the crate exists with the moved code and nothing uses it
  yet.
- **Size:** about 300 added, 300 moved. One change.
- **Watch out:** the collector depends on this crate, so it must not depend
  on the app. Everything is MIT (V63), so this is layering only. Keep the
  trait free of anything FalkorDB-specific: E3.4 puts other engines behind
  it.
  Say so in its `Cargo.toml` description.

### E3.2 Projection through the trait

- **Goal:** `run_projection` uses `GraphBackend`.
- **Depends on:** E3.1, C5.3.
- **Files:** `crates/engine/src/inventory/projection/runner.rs` (273 lines),
  `projection/client.rs` and `projection/cypher.rs` (deleted),
  `crates/engine/Cargo.toml`, `src/sweep_command/enabled.rs`.
- **Change:** `run_projection<S: InventoryStore, G: GraphBackend>(store: &S,
  graph: &G, batch: u16)`. The CLI builds the FalkorDB backend from
  `--graph-url` and `--graph-name`.
- **Tests:** existing projection tests through the trait, plus a fake backend
  that records calls: a drained key leads to one node upsert and one
  `replace_edges`; a deleted key to one `delete_node`; an offline backend
  leaves the outbox intact.
- **Budget:** P15 before and after (the trait is generic, not dynamic).
- **Prove it:** `cargo test -p inventory-engine --locked --test
  inventory_projection_live -- --ignored`; e2e suite.
- **Done when:** the existing FalkorDB tests pass through the trait.
- **Size:** about 120 added, 300 deleted. One change.
- **Watch out:** the schema 13 projection under `crates/engine/src/graph/` is
  not touched; K2 deletes it.

### E3.3 Traversal through the trait, and the single-module check

- **Goal:** `cmdb-view` traversal uses `GraphBackend`, and no Cypher string
  exists outside the one module.
- **Depends on:** E3.1.
- **Files:** `crates/cmdb-view/src/graph.rs`, `graph/query.rs` (151 lines),
  `graph/path.rs`, `graph/response.rs`, `crates/cmdb-view/Cargo.toml`,
  `scripts/check-cypher-module.sh` (new), `moon.yml`.
- **Change:** `GraphConnection` wraps a `GraphBackend`; `neighbors`,
  `blast_radius` and `path` delegate. The script fails if `MATCH (`, `MERGE
  (` or `GRAPH.QUERY` appears in any `.rs` file outside
  `crates/graph-store/src/` and `crates/engine/src/graph/` (the schema 13
  path, removed by K2).
- **Tests:** existing `cmdb-view` graph tests; the script on the tree.
- **Budget:** P12: neighbourhood query latency through `cmdb-bench measure`
  with a graph URL, before and after.
- **Prove it:** `scripts/check-cypher-module.sh`; `cargo test -p cmdb-view
  --locked`.
- **Done when:** the E3 exit evidence holds.
- **Size:** about 100 added, 150 deleted. One change.
- **Watch out:** traversals stay bounded (`GraphLimits`: depth 2, 200 nodes,
  500 edges in the UI today).

### E3.4 Choose the graph engine by measurement

- **Goal:** The graph engine that ships in version 1 (V60) is chosen on
  measured speed, resource use and licence, and the choice is recorded.
- **Depends on:** E3.2, E3.3, E4.1.
- **Files:** `crates/graph-store/src/<engine>.rs` (new; one implementation
  of `GraphBackend` per candidate, each behind a cargo feature),
  `crates/cmdb-bench/src/traverse.rs` (new; times `neighbors`, `path` and
  `blast_radius` through the trait, which D11.2 then reuses for P17),
  `docs/performance-evidence/graph-engines.md` (new).
- **Change:**
  1. Candidates, as listed on 2026-11-22 and to be re-checked on the day:
     FalkorDB (the engine in the tree; a server; SSPLv1), Grafeo (Rust;
     embedded or server; Apache-2.0; pre-1.0), LadybugDB (the continuation
     of Kuzu; C++, embedded, with a Rust binding; MIT), and an index of our
     own built from the `edges` table (no query language; the floor).
     Recursive SQL on PostgreSQL is timed as a fifth row, with no engine at
     all.
  2. Implement `GraphBackend` for each candidate. Nothing outside
     `crates/graph-store` changes; that is what E3.1 to E3.3 were for.
  3. On the bench database at the size of O14, three runs each on the same
     machine: load and full rebuild (P15), the three traversals (P12, P17),
     peak memory, size on disk, and time to first query after start.
  4. Run E5.1 (graph equals SQL) against each candidate.
  5. Record for each: licence; whether it is linked into our binaries or is
     a separate server to install, start and secure; whether it builds and
     runs on macOS arm64 and Linux x86-64 (V58); its version and how often
     it releases; and what it does when two processes (the CLI and the app)
     open the same data.
  6. The owner chooses. The choice goes into
     [DECISIONS.md](../DECISIONS.md). The implementations that lose are
     deleted (V21).
- **Tests:** the existing graph tests run once per implementation; E5.1.
- **Budget:** P12, P15, P17, peak memory. The winner must not be slower than
  FalkorDB on any of them without a written reason.
- **Prove it:** `cmdb-bench projection --graph <engine>` and `cmdb-bench
  traverse --graph <engine>` for every candidate, and the evidence file.
- **Done when:** one engine is chosen, with its numbers and its licence
  written down, before L1.1 and L5.1 need the answer.
- **Size:** about 150 added per candidate, most of it deleted again.
- **Watch out:** a vendor's own benchmark is not evidence (V19). Not
  verified: the Rust API of any candidate; that Grafeo or LadybugDB can
  express the path and blast-radius queries; that either is stable enough
  to ship. V46 requires a graph database, so choosing our own index would
  need the owner to change V46. An embedded engine removes a server from
  the install (L1.1) and from the connection rules (L3.1); a server does
  not.

## E4 Projection at scale

### E4.1 Measure the drain and a full rebuild

- **Goal:** Throughput of the pending-projection drain and the time of a full
  rebuild at 250,000 nodes and 1,000,000 edges.
- **Depends on:** E3.2, D4.1.
- **Files:** `crates/cmdb-bench/src/projection.rs` (new),
  `docs/performance-evidence/sweep-benchmark.md`.
- **Change:** `cmdb-bench projection <target> --graph-url <url> [--rebuild]`
  drains the outbox of a generated database into an empty graph and prints
  keys per second, total time, and Redis memory; with `--rebuild` it clears
  the graph and rebuilds from SQL.
- **Tests:** arithmetic only.
- **Budget:** this step defines P15's first rows at scale.
- **Prove it:** the command against the harness graph server.
- **Done when:** the rows are recorded.
- **Size:** about 160 added. One change.
- **Watch out:** the projection sends one query per node and per node's edge
  set today (`runner.rs`); the row will show whether that binds.

### E4.2 Batch the projection if the row says so, and prove rebuild equals incremental

- **Goal:** If E4.1 shows the per-key round trip dominating, send batches;
  either way, a rebuilt graph equals the incrementally updated one.
- **Depends on:** E4.1, E5.1.
- **Files:** `crates/graph-store/src/cypher.rs`, `src/falkor.rs`,
  `crates/engine/src/inventory/projection/runner.rs`.
- **Change:** trait methods `upsert_nodes(&[NodeRow])` and
  `replace_edges_many(&[(NodeKey, Vec<EdgeRow>)])` using `UNWIND $rows AS row
  MERGE ...`, with a batch of at most 1,000 rows and a query-size bound. The
  runner acks a batch only after the backend acknowledged all of it. Test:
  build a graph incrementally over several sweeps with changes and deletions,
  rebuild a second graph from SQL, and compare both with E5.1's comparer.
- **Tests:** batch of 1, 1,000 and 1,001; a failed batch acks nothing; the
  equality test.
- **Budget:** P15 before and after.
- **Prove it:** `cmdb-bench projection` rows; the equality test with a live
  graph server.
- **Done when:** the M6.1 evidence holds and the rows are recorded.
- **Size:** about 260 added. One change.
- **Watch out:** `UNWIND` with a list-of-maps parameter is in the shared
  openCypher subset; how FalkorDB's parameter text encodes a list of maps is
  not verified. If it cannot, keep single-key queries pipelined on one
  connection and record that.

### E4.3 The graph after the store lost commits

- **Goal:** If the store recovers to an earlier state than the graph has
  seen, the graph is brought back to equal SQL (review finding 1).
- **Depends on:** E4.2, E5.1, B4.6.
- **Files:** `crates/engine/src/inventory/postgres/projection.rs`,
  `crates/cmdb-view/src/graph/reconcile.rs` (new),
  `crates/engine/tests/inventory_db_crash.rs`.
- **Change:** the projection records, in the graph, the store's identity and
  the last projected sequence. When the projection starts or reconnects and
  the store's sequence is behind the graph's, or the store reports a restart
  since the last projection, it runs E5.1's comparison and repairs the
  difference (or rebuilds, whichever the recorded times favor) before
  projecting anything new.
- **Tests:** project, crash the database with `synchronous_commit = off` so
  that acknowledged commits are lost, restart: the graph equals SQL after
  the repair, and edges for the lost rows are gone.
- **Budget:** repair time at 250,000 nodes, recorded.
- **Prove it:** the `inventory_db_crash` suite's graph cases.
- **Done when:** E5.1 reports zero difference after every crash case.
- **Size:** about 220 added. One change.
- **Watch out:** with the durable defaults this path should never run; it
  exists so that relaxing durability later cannot leave a wrong graph.

## E5 Graph equals SQL

### E5.1 The comparer (X5)

- **Goal:** A command that compares node and edge keys between SQL and the
  graph database and prints the difference.
- **Depends on:** E3.2, C6.9.
- **Files:** `src/cli.rs` (subcommand `GraphVerify`),
  `src/graph_verify.rs` (new), `crates/harness/src/e2e/verify.rs` (stage
  `verify-graph`).
- **Change:** `aws-inventory graph-verify --database <target> --graph-url
  <url> --graph-name <name>`: stream node keys from SQL in key order
  (`projection_resources`) and from the graph (`GraphBackend::node_keys`),
  merge-compare, and do the same for edges (`from`, `to`, `relation`,
  `source`). Print counts and up to 100 keys missing on either side. Exit 0
  only when both differences are empty and the outbox is empty; exit 3 when
  the outbox still has rows (the graph is behind, not wrong).
- **Tests:** equal; node missing in graph; extra edge in graph; pending
  outbox; a million keys compared in bounded memory.
- **Budget:** comparison time at 250,000 nodes and 1,000,000 edges, recorded.
- **Prove it:** `inventory-harness e2e run` passes its `verify-graph` stage
  after the sweep and again after a re-sweep with one seeded change.
- **Done when:** X5 holds.
- **Size:** about 260 added. One change.
- **Watch out:** the graph stores a digest of the full key as `key`
  (`cypher.rs::node` uses a scoped digest); compare on the same digest, not on
  display fields.
