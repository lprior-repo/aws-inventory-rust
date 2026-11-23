# Track A steps: coverage

> Status: Plan, 2026-11-15, revised for V53 (PostgreSQL is the only store).
> Step-by-step breakdown of the packages in [A_COVERAGE.md](../A_COVERAGE.md).
> Order, waves and measurement names (P1 to P17) are in
> [BUILD_PLAN.md](../BUILD_PLAN.md). Nothing here has been built. Names marked
> "new" do not exist in the tree yet.

Conventions used in every step:

- "Thin slice" means the services `ec2`, `sts`, `iam`, `sns`, `sqs`, `dynamodb`,
  `kms`, `lambda`, `s3`, `cloudwatch` (all seven protocols). Which of them the
  emulator serves is not known until A5.2 runs. BUILD_PLAN lists the substitutes.
- The store is PostgreSQL (V51, V53).
- "Budget" appears on every step that sits on the sweep, write, read or render
  path (V52). It names the number that must not get worse and the command that
  measures it.
- No step runs or adds Python. No step adds Rust comments, `unsafe`, `unwrap`,
  `expect`, `panic`, unchecked indexing or `as` casts
  (`[workspace.lints]` in `Cargo.toml`, which also denies
  `arithmetic_side_effects`).
- `scripts/check-function-size.sh` checks `crates/catalog-tool/src` and
  `crates/harness/src`, so the 25-line limit is enforced on almost all of this
  track.

## A0 Catalog gate

### A0.1 Give the relation failure a cause

- **Goal:** `catalog-tool models compile --check` fails today with the bare text
  "runtime capability metadata is invalid or unsupported". Make the failure name
  the kind, the path and the reason.
- **Depends on:** nothing. First step of the whole plan.
- **Files:** `crates/catalog-tool/src/runtime_catalog.rs`,
  `crates/catalog-tool/src/runtime_catalog/model_path.rs` (302 lines today, over
  the limit), `crates/catalog-tool/src/runtime_catalog/model_path/validation.rs`
  (new).
- **Change:**
  1. Move `ValidationContext`, `validate_reviewed_paths`,
     `validate_display_paths`, `validate_relations`, `validate_bootstrap_paths`,
     `validate_bootstrap`, `validate_region_paths`, `validate_kind_path`,
     `item_target_for_selector`, `operation_for` and `operation_for_checked` out
     of `model_path.rs` into `model_path/validation.rs`. `model_path.rs` keeps
     `resolve`, `terminal_member`, `item_target`, `list_element`, `kind`,
     `string_target`, `shape_for`, `member`. This brings the file under 300.
  2. Add one variant to `RuntimeCatalogCompileError`:
     `KindContext { kind: String, path: String, reason: &'static str }` (new),
     text "runtime capability metadata is unsupported for kind {kind}, path
     {path}: {reason}". The existing `CapabilityContext { service, operation,
     path }` has no kind field, so it is used where a service and operation are
     known and `KindContext` where only a kind is known.
  3. In `validate_kind_path`: when no `KindMap::operations()` row has the kind,
     return `KindContext` with reason "no operation row produces this kind".
     When `operation_for_checked` finds no callable operation, return
     `CapabilityContext` with the row's service and operation and the path
     (reason is implied: not callable). When `resolve` fails, return
     `CapabilityContext` with the same service, operation and the offending
     path. When `string_target` fails, return `KindContext` with reason "path
     does not end in a string".
  4. `validate_relations` wraps each call so that the error also says which side
     failed: map the reason to "relation source ..." or "relation target ...".
- **Tests:** unit tests in `model_path/validation.rs` behind `#[cfg(test)]`,
  using the committed `catalog/aws-api.catalog` as `runtime_catalog.rs` tests
  already do: (a) a relation whose source kind has no operation row gives
  `KindContext` naming that kind; (b) a relation whose kind's operation is
  deferred gives `CapabilityContext` naming `ec2`, `DescribeSecurityGroups`,
  `VpcId`; (c) a relation with a misspelled path names the path; (d) the one
  live relation still validates.
- **Prove it:** `/home/claude/target/debug/catalog-tool models compile --check`
  rebuilt from this change still exits non-zero, and its message now contains
  `AWS::EC2::SecurityGroup` or `DescribeSecurityGroups` and `VpcId`.
- **Done when:** the failing gate names its cause. The gate still fails; A0.2
  fixes it.
- **Size:** about 120 added, 60 moved, 10 deleted. One change.
- **Watch out:** `RuntimeCatalogCompileError` is `#[non_exhaustive]`; adding a
  variant is not a breaking change inside the workspace. Every moved function
  must stay at or under 25 lines.
- **Review fix (2026-11-18):** this plan was written from an archive with no
  git history. Before changing anything, record the commit this step runs on
  (`git rev-parse HEAD`) in `docs/e2e/RUN_HERE.md`, and re-run the seven
  catalog checks and both size guards on it. If the tree has moved since
  2026-11-15, the facts in [BUILD_PLAN.md](../BUILD_PLAN.md) section 10 are
  re-checked before any other step starts.

### A0.2 Carry relations as pending until both ends are callable

- **Goal:** Make `models compile --check` pass with the 42 relation rows left in
  `catalog/kinds.tsv`, by compiling only the relations whose source and target
  kinds each have a callable operation.
- **Depends on:** A0.1.
- **Files:** `crates/catalog-tool/src/runtime_catalog.rs`,
  `runtime_catalog/model_path/validation.rs`, `runtime_catalog/encode.rs`,
  `runtime_catalog/schemas.rs`, `crates/catalog-tool/src/runtime_catalog/relations.rs`
  (new), `catalog/aws-relations.report` (new, generated), `catalog/aws-api.catalog`
  (regenerated), `moon.yml`.
- **Change:**
  1. In `relations.rs` add `pub(super) struct RelationSplit<'a> { live:
     Vec<&'a ReviewedRelation>, pending: Vec<(&'a ReviewedRelation, &'static
     str)> }` and `pub(super) fn split(kinds: &KindMap, checked:
     &CheckedOverlays) -> RelationSplit<'_>`. A relation is live when both
     `source_kind` and `target_kind` have a `RuntimeOperation` row whose
     `(service, operation)` is in `ReviewedService::callable()`. Otherwise it is
     pending with reason "source kind not callable", "target kind not callable"
     or "neither kind callable".
  2. `validate_relations` validates only `live`. A live relation whose path does
     not resolve is still a hard error with the context from A0.1.
  3. `encode::relations` and `schemas::text_columns` take the live slice instead
     of `kinds.relations()`.
  4. Write the pending list as text: one line `pending <source_kind>
     <source_path> <target_kind> <target_path> <relation> <reason>`, sorted, with
     a totals line `relations live=<n> pending=<m>`. Publish it through
     `crate::report::publish` under `catalog/aws-relations.report`, in the same
     `ReportMode` as the catalog.
  5. Run `models compile --write`. Record the SHA-256 of
     `catalog/aws-api.catalog` before and after in the change description.
  6. `moon.yml`: no new task. `binary-catalog` now also checks the report.
- **Tests:** (a) with the real `kinds.tsv` and overlays, `live` has 1 entry
  (`AWS::EC2::Subnet VpcId`) and `pending` 41; (b) a fixture where a second
  kind becomes callable moves its relations to live; (c) a live relation with a
  bad path fails with the kind and path; (d) regenerating the report twice
  gives identical bytes.
- **Prove it:** `catalog-tool models compile --check` exits 0. Then the baseline:
  `cargo test --locked --features aws --test sweep_e2e -j1 -- --ignored
  --test-threads 1` gives 2 passed.
- **Done when:** the seven catalog checks of `moon ci` pass on a clean checkout
  with the pinned models, and the end-to-end baseline still passes on the
  regenerated catalog.
- **Size:** about 180 added, 20 deleted. One change.
- **Watch out:** RUN_HERE tested one of the 41 rows, not all. If the regenerated
  catalog differs from the committed one in more than the expected places, stop
  and report the difference before committing it; the sweep reads this file.
  The runtime reader (`crates/core/src/catalog/runtime/decode/`) is closed: do
  not add fields to the extension here.

### A0.3 Context at the remaining bare `Capability` sites

- **Goal:** Replace the bare `RuntimeCatalogCompileError::Capability` with a
  contextual error wherever a service, operation, kind or path is in scope, so
  that the next failure after generation (A1) is readable.
- **Depends on:** A0.2.
- **Files:** `crates/catalog-tool/src/runtime_catalog/plans.rs`,
  `plans/compile.rs`, `plans/page.rs`, `plans/page/tokens.rs`,
  `plans/identity.rs`, `plans/service.rs`, `plan_validation.rs`,
  `schemas/selected.rs`, `encode.rs`.
- **Change:**
  1. `plans/compile.rs::contextual_plan` already converts a bare `Capability`
     from `plan()` into `CapabilityContext`, but with the items path as the
     path. Change the inner functions to return the specific path: binding
     paths in `binding_value` and `plan_validation::validate_bindings`, identity
     paths in `validate_identity`, token paths in `plans/page/tokens.rs`,
     exclusion paths in `exclusions`.
  2. `ServiceIndex::get` and `ServiceIndex::field_id` return `KindContext`-style
     context through two small helpers `unknown_service(name)` and
     `unknown_member(name)` (new variants are not needed: use
     `CapabilityContext` with `operation` and `path` set to "-" and the member
     name).
  3. `schemas/selected.rs::column`: when a kind has more than one item type,
     return `KindContext` with reason "operations of this kind disagree on the
     item type".
  4. `encode::relations`: an unknown `scope` label returns `KindContext` with
     the label as the path.
  5. Remove the two Rust comment lines in this crate: `plan_validation.rs`
     line 118 and `plans/identity.rs` line 4. The other 15 comment lines in
     the tree are removed by A0.4.
- **Tests:** one failure test per changed function, asserting on the error's
  fields and not on its text.
- **Prove it:** `cargo test -p catalog-tool --locked runtime_catalog` passes;
  `grep -rn "Capability)" crates/catalog-tool/src/runtime_catalog | wc -l` is
  under 15, and each remaining site is in a function with no service, kind or
  path in scope.
- **Done when:** the count above is recorded and the tests pass.
- **Size:** about 200 added, 80 deleted. One change; split by file if review
  asks.
- **Watch out:** 62 lines mention the bare variant today. Do not thread context
  through `model_path::resolve` by adding parameters to every caller; convert at
  the call site with `map_err`.

### A0.4 Make the size guards pass

- **Goal:** Five Rust files are over 300 lines in this archive, so
  `source-size` fails before any new work lands. Bring them under the limit.
- **Depends on:** A0.1 (it already fixes `model_path.rs`).
- **Files:** `crates/core/src/catalog/runtime/decode/plans.rs` (305),
  `crates/cmdb-ui/src/app.rs` (304),
  `crates/harness/src/install/graph/installer_tests.rs` (305),
  `crates/cmdb-ui/src/worker/tests.rs` (327).
- **Change:** move one cohesive group of functions out of each file into a
  sibling module. No behavior change. For `decode/plans.rs` move the binding
  decoders into `decode/plans/bindings.rs` (new). Also remove the 15 Rust
  comment lines outside `catalog-tool`: `crates/cmdb-view/src/graph/query.rs`
  (lines 6 to 8, 27, 66), `crates/cmdb-ui/src/worker/queries.rs` (lines 3 to
  5, 31, 32) and `crates/cmdb-ui/src/worker/tests.rs` (lines 243, 244, 304,
  317, 319). What they say that is a contract moves to the crate's document.
- **Tests:** existing tests, unchanged.
- **Prove it:** in a git clone, `scripts/check-source-size.sh` prints "all at
  most 300 lines" and `scripts/check-function-size.sh` exits 0. In the archive
  (no git), `find . -path ./.local -prune -o -name '*.rs' -print | xargs wc -l |
  awk '$1>300 && $2!="total"'` prints nothing.
- **Done when:** both guards pass.
- **Size:** about 0 net lines; four files touched. One change.
- **Watch out:** `crates/cmdb-ui/src/app.rs` is also changed by D2 and D3; land
  this first. `check-source-size.sh` uses `git ls-files`, so it cannot run in
  the extracted archive.

## A1 Overlay generator

### A1.1 Generated overlays as a second, lower-priority source

- **Goal:** Let a service have a generated overlay file and an optional
  hand-written one, with the hand-written entry winning per operation.
- **Depends on:** A0.2.
- **Files:** `crates/catalog-tool/src/overlay/directory.rs`,
  `overlay/types.rs`, `overlay/parse.rs`, `overlay/check/orchestration.rs`,
  `overlay/check.rs`, `overlay/merge.rs` (new),
  `crates/catalog-tool/src/runtime_catalog/plans.rs` (`review_value`),
  `docs/API_CATALOG.md`.
- **Change:**
  1. Generated files live in `catalog/overlay/generated/<service>.toml` (new
     directory). Hand-written files stay in `catalog/overlay/<service>.toml`.
     `load_dir` keeps reading only the top level. Add
     `pub fn load_generated_dir(root)` with the same bounds (`MAX_FILES`,
     `MAX_DIRECTORY_ENTRIES`, `MAX_FILE_BYTES`).
  2. Add `pub enum Origin { Hand, Generated }` (new) and a field
     `origin: BTreeMap<OperationName, Origin>` on `ServiceOverlay`, with a
     getter `origin_of(&OperationName) -> Origin`.
  3. `merge.rs`: `pub(super) fn merge(hand: Vec<ServiceOverlay>, generated:
     Vec<ServiceOverlay>) -> Result<Vec<ServiceOverlay>, OverlayError>`. Per
     service, start from the generated operations and replace each one that
     the hand file also names. Service-level `reviewed` and `reviewer` come
     from the hand file when it exists.
  4. A generated file must carry `reviewer = "generated"`. `parse.rs` rejects a
     file under `generated/` with any other reviewer, and rejects
     `reviewer = "generated"` in a top-level file.
  5. `orchestration::check` calls `merge` in place of `load_dir`.
  6. `review_value` writes the reviewer per operation: "generated" for
     `Origin::Generated`, the file's reviewer otherwise. `CallableOperation`
     gains `origin: Origin` with a getter.
- **Tests:** (a) hand entry replaces generated entry of the same name; (b) an
  operation only in the generated file is kept; (c) a generated file with
  another reviewer is rejected; (d) a hand file claiming "generated" is
  rejected; (e) with no `generated/` directory the result equals today's
  (`overlay --check` passes with the committed report); (f) directory bounds
  apply to `generated/`.
- **Prove it:** `catalog-tool overlay --source .local/aws-models --root .
  --check` passes with no `generated/` directory present; `models compile
  --check` passes.
- **Done when:** both checks pass without a generated overlay directory, and
  the overlay and relation reports match A0.2 byte for byte. The compiled
  catalog must match A0.2 byte for byte except for the embedded-source
  SHA-256 (`embedded_sources_sha256`), which must match a fresh digest of
  the actual embedded sources. A1.1 edits the source tree covered by that
  receipt; do not reuse A0.2's digest or change what it covers.
- **Size:** about 260 added, 30 deleted. One change.
- **Watch out:** the deny decision is not touched here. `is_promoted` in
  `overlay/check/validation/evaluation.rs` still turns any
  enumerate or expand entry on a denied operation into the
  `OverlayPromotesDenied` violation, whatever its origin. Keep `ServiceOverlay`
  fields `pub(super)`; the read-only getters are the M1.4b contract.

### A1.2 Derive one enumeration entry from the model

- **Goal:** A pure function that, for one enumerate candidate, returns either a
  complete overlay entry or a deferral with its reason.
- **Depends on:** A1.1.
- **Files:** `crates/catalog-tool/src/overlay/generate.rs` (new),
  `overlay/generate/derive.rs` (new), `overlay/generate/identity.rs` (new),
  `overlay/generate/items.rs` (new), `overlay.rs` (module declaration),
  `catalog/overlay/defer-inputs.txt` (new, data).
- **Change:**
  1. `pub(crate) enum Derived { Callable(OperationOverlay), Deferred(String) }`.
     `pub(crate) fn enumerate(model: &plans::Model<'_>, operation: &str, scope:
     Scope, vocabulary: &plans::Vocabulary, deny: &DenyList<'_>) -> Derived`.
  2. Deny first: the caller passes only operations whose
     `classes::classify` result is `Class::Enumerate`. `enumerate` re-checks and
     returns `Deferred("not an enumerate candidate")` otherwise. It never
     returns an entry for `Class::Denied`.
  3. Scope: the caller supplies it from
     `endpoints::verified_endpoints(source)`: `ServiceClass::Global` gives
     `Scope::Account`, `ServiceClass::Regional` gives `Scope::Region`. A service
     with no resolved endpoint (the 5 "needs overlay" and 2 unsupported
     services in `catalog/aws-endpoints.report`) defers every operation with
     "endpoint: not resolved".
  4. Items (`items.rs`): if `plans::pagination` returns `Pagination::Modeled {
     items: Some(_), .. }`, leave `items` unset (the checker infers it).
     Otherwise take the output structure's members whose target is a list or
     map: exactly one gives `ItemSelector::Member`; none or several defers with
     "items: no single list member".
  5. Paginator: `Modeled` leaves it unset. `Unpaginated` writes `Paginator::None`.
     `NeedsOverlay(TokenBothSides)` writes `Paginator::Token` only when exactly
     one pair from `Vocabulary::pairs` is present on both sides; otherwise
     defer with "paginator: ambiguous token pair". The other three
     `PaginationReason` values defer with the reason's name.
  6. Page size: never written. `modeled_page_size` already supplies the model's
     maximum.
  7. Identity (`identity.rs`), on the selected item shape, first match wins:
     a string member named `Arn`; else exactly one string member whose name
     ends in `Arn` and starts with the resource noun of the operation name;
     else the identifiers of a modeled resource whose `list` operation is this
     one, each present as an item member (`Identity::Key`); else, when the item
     is a scalar string, `Identity::Key(["$"])`. Anything else defers with
     "identity: no ARN member and no modeled identifiers".
  8. Owner filters: if any input member name is listed in
     `catalog/overlay/defer-inputs.txt` (start with `Owners`, `OwnerIds`,
     `Owner`, `IncludePublic`, `IncludeShared`, `Scope`), defer with "input
     <name> needs a reviewed fixed value". This is a deny on a name, which the
     rules allow.
  9. Dry run: build the candidate entry and pass it through
     `overlay::check::validation::operation` (make `StateArgs` and `operation`
     `pub(in crate::overlay)`). If it returns violations, defer with the first
     violation's key and value as the reason. A generated entry therefore
     cannot fail the gate later.
- **Tests:** table tests over small inline Smithy JSON models: modeled paginator
  with ARN; unpaginated with single list; two list members (deferred); token
  both sides with one known pair; truncation flag (deferred); scalar item list;
  input named `Owners` (deferred); an operation named in `must-deny.txt`
  (never callable); an operation with a required input (not a candidate).
- **Prove it:** `cargo test -p catalog-tool --locked overlay::generate`.
- **Done when:** every branch above has a passing and a failing case.
- **Size:** about 420 added. One change.
- **Watch out:** reuse `plans::pagination`, `plans::expansion` and the checker's
  code; do not parse `catalog/aws-plans.report`. `defer_reason` is limited to
  1,024 bytes. `plans::model` is `pub(crate)`; do not widen it to `pub`.

### A1.3 Write generated files deterministically

- **Goal:** Turn the derived entries of one service into the bytes of
  `catalog/overlay/generated/<service>.toml`, with an entry for every candidate.
- **Depends on:** A1.2.
- **Files:** `crates/catalog-tool/src/overlay/generate/render.rs` (new),
  `overlay/generate/service.rs` (new).
- **Change:**
  1. `service.rs`: for each operation of the model, in name order:
     `Class::Enumerate` goes through `derive::enumerate`; `Class::Expand` is
     written as `class = "defer"` with `defer_reason = "expand: bindings not
     generated yet"` (A3 replaces this); `Class::Unclassified` is written as
     `class = "defer"`, `defer_reason = "unclassified read: not reviewed"`;
     `Class::Denied` gets no entry. The checker requires an entry for every
     enumerate, expand and unclassified candidate of a reviewed service
     (`IncompleteReview`), so none may be skipped.
  2. `render.rs` writes TOML by hand, not through `toml::to_string`, so the
     bytes are stable. File layout:

     ```toml
     # Generated by `catalog-tool overlays generate`. Do not edit; put
     # corrections in catalog/overlay/<service>.toml.
     format = 1
     reviewed = "2026-11-15"
     reviewer = "generated"

     [operation.ListThings]
     class = "enumerate"
     scope = "region"
     items = "Things"
     [operation.ListThings.identity]
     arn = "ThingArn"
     [operation.ListThings.paginator]
     kind = "none"

     [operation.GetThingPolicy]
     class = "defer"
     defer_reason = "unclassified read: not reviewed"
     ```

     Key order inside an entry is fixed: `class`, `defer_reason`, `scope`,
     `items`, `fixed`, `identity`, `paginator`, `bind`. Keys the checker infers
     are omitted. `reviewed` is the constant `GENERATED_RULES_DATE` (new) in
     `generate.rs`, changed only when the derivation rules change, so that
     regenerating does not change bytes.
  3. Strings are written with TOML basic-string escaping; reject control
     characters instead of escaping them.
- **Tests:** (a) render then `ServiceOverlay::parse` round-trips to the same
  operations; (b) rendering twice gives identical bytes; (c) a service with
  4,097 operations is refused (`MAX_OPERATIONS`); (d) output over
  `MAX_FILE_BYTES` is refused before writing; (e) a reason containing a quote
  or backslash round-trips.
- **Prove it:** `cargo test -p catalog-tool --locked overlay::generate::render`.
- **Done when:** tests pass.
- **Size:** about 300 added. One change.
- **Watch out:** the largest service, `ec2`, has 171 enumerate and 25 expand
  candidates and its hand file is 51 KB; `MAX_FILE_BYTES` is 1 MiB, so one file
  per service fits. Grow output with `try_reserve` as `names` does.

### A1.4 The `overlays generate` command, its report and its gate

- **Goal:** `catalog-tool overlays generate [--service <name>]... [--check |
  --write]`, a report with counts by reason, and a gate task.
- **Depends on:** A1.3.
- **Files:** `crates/catalog-tool/src/main.rs` (207 lines; move the argument
  parsers for `overlay` and `overlays` into `src/cli_overlay.rs`, new),
  `overlay/generate.rs`, `overlay/generate/publish.rs` (new),
  `catalog/overlay/generated.services` (new, data),
  `catalog/aws-overlay-generated.report` (new, generated), `moon.yml`,
  `docs/API_CATALOG.md`.
- **Change:**
  1. `catalog/overlay/generated.services` lists the services to generate for,
     one per line, sorted, or the single line `*` for all. `--service` on the
     command line overrides it for one run and is refused together with
     `--check`. This file is how the thin slice is generated before all 436
     services (A1.6, A1.7).
  2. `--write`: for each listed service write its file through
     `smithy::fields::publish::publish_bytes` (exclusive temporary file and
     atomic rename), and delete any file under `generated/` whose service is
     not listed. `--check`: fail if any file differs, is missing or is extra.
     No flag: print the report only.
  3. Report rows: `service <name> callable=<n> deferred=<m>` and
     `reason <text> <count>`, plus `totals enumerate_candidates=<n>
     callable=<n> deferred=<n>` computed over the listed services. The sum of
     callable and deferred must equal the enumerate candidates of those
     services; a mismatch is an error, as in `plans/report.rs::validate_sums`.
  4. `moon.yml`: add task `overlay-generated` running `catalog-tool overlays
     generate --check`, between `collection-plans` and `overlay-check`.
- **Tests:** CLI tests in `crates/catalog-tool/tests/` on a temporary root with
  two tiny models: write then check passes; editing a generated file makes
  check fail; removing a service from `generated.services` and writing deletes
  its file; `--service` with `--check` is refused.
- **Prove it:** with an empty `generated.services`, `catalog-tool overlays
  generate --check` exits 0 and `overlay --check` still passes.
- **Done when:** the gate task exists and passes with nothing generated.
- **Size:** about 330 added, 40 moved. One change.
- **Watch out:** `main.rs` functions are near the 25-line limit already.
  `publish_bytes` compares with the bytes read at the start; pass the original.

### A1.5 Prove generation cannot open a denied operation

- **Goal:** Tests over all 436 pinned models showing that no generated entry is
  callable for a denied operation, and that today's four callable operations
  are unchanged.
- **Depends on:** A1.4.
- **Files:** `crates/catalog-tool/tests/overlay_generation_safety.rs` (new).
- **Change:** tests only.
- **Tests:**
  1. For every service: generate in memory; assert no operation whose
     `classes::classify` result is `Class::Denied(_)` has an entry with class
     enumerate or expand. This covers all 85 `must-deny.txt` entries and the
     11,275 no-read-verb operations.
  2. Take one generated file, change a denied operation's entry to
     `class = "enumerate"` by hand, run `overlay::check`: it fails with
     `ViolationRule::OverlayPromotesDenied`.
  3. Add a new line to a copy of `must-deny.txt` naming an operation that was
     generated callable; regenerate: the entry is gone and `check` passes.
  4. With every service generated in a temporary root that also holds the
     committed `ec2.toml` and `sts.toml`, the `CallableOperation` values for
     `DescribeVpcs`, `DescribeSubnets`, `DescribeRegions` and
     `GetCallerIdentity` equal today's, and the three STS operations denied as
     `secret` are still denied.
  5. Generating twice gives identical bytes for all files.
- **Prove it:** `cargo test -p catalog-tool --locked --test
  overlay_generation_safety -- --ignored` (ignored because it reads 188 MB of
  models; the gate runs it in the `test` task with `--include-ignored` for this
  file only, or as its own moon task `overlay-safety`).
- **Done when:** all five pass with the pinned models.
- **Size:** about 250 added (tests). One change.
- **Watch out:** reading all models takes time and memory; reuse one
  `read_model_values` result across the tests in the file. Do not weaken test 1
  to a sample.

### A1.6 Generate the thin slice

- **Goal:** Commit generated overlays for the thin-slice services only and
  record what became callable.
- **Depends on:** A1.5.
- **Files:** `catalog/overlay/generated.services`,
  `catalog/overlay/generated/{iam,sns,sqs,dynamodb,kms,lambda,s3,cloudwatch}.toml`
  (new, generated), `catalog/aws-overlay-generated.report`,
  `catalog/aws-overlay.report` (regenerated).
- **Change:** list the eight services, run `overlays generate --write`, then
  `overlay --write`. `ec2` and `sts` keep their hand-written files. Do not
  compile the catalog yet: every callable operation needs a kind row first
  (A2.2).
- **Tests:** none new; A1.5 covers the rules.
- **Prove it:** `catalog-tool overlays generate --check` and `overlay --check`
  pass. The report shows, per service, callable and deferred counts summing to
  the enumerate candidates in `catalog/aws-plans.report` (iam 17, sns 6, sqs 1,
  dynamodb 9, kms 4, lambda 5, s3 2, cloudwatch 9).
- **Done when:** the counts are in the report and in the change description,
  with the three most common deferral reasons.
- **Size:** data only. One change.
- **Watch out:** `kms/DescribeKey` is in `must-deny.txt` and must not appear as
  callable. `s3` needs bucket addressing for anything below `ListBuckets`; those
  stay deferred. If a service generates zero callable operations, say so and
  pick a substitute from BUILD_PLAN's list instead of loosening a rule.

### A1.7 Generate for all services

- **Goal:** Generated overlays for all 436 services, with totals that sum to
  the 2,197 enumerate candidates.
- **Depends on:** A1.6, A2.3, A3.2, C1.2 (column policy), A3.3 (catalog
  indexes), and the T2 checkpoint (wave 3).
- **Files:** `catalog/overlay/generated.services` (`*`),
  `catalog/overlay/generated/*.toml`, both overlay reports,
  `catalog/kinds.generated.tsv`, `catalog/aws-api.catalog`.
- **Change:** set `generated.services` to `*`; run `overlays generate --write`,
  `kinds generate --write`, `overlay --write`, `models compile --write`.
- **Tests:** A1.5 runs over the full set. Add one assertion to it: totals
  `callable + deferred == 2197` for enumerate and `== 2940` for expand.
- **Budget:** P16 (catalog compile time and catalog bytes) and P11 (catalog
  load time at sweep start). Command: `time catalog-tool models compile
  --check`; `cmdb-bench open` (B5.2). The catalog is 11,976,647 bytes today and
  three places bound it at 16,777,216 (`MAX_RECORD_BYTES`,
  `inventory.rs::MAX_CATALOG_BYTES`, `read_catalog` in the sweep command).
- **Prove it:** all eight catalog gate tasks pass; `wc -c
  catalog/aws-api.catalog` is under 16,777,216; the two numbers above are
  recorded before and after.
- **Done when:** the gate passes and the size and times are recorded.
- **Size:** data only, large diff. One change per run; rerun after each rule
  fix.
- **Watch out:** this is the step most likely to hit the 16 MiB bound: every
  operation of a reviewed service gets a decision row with its reason text
  (field 6), about 8,000 rows. If the catalog passes 14 MiB, stop and take the
  fallback in BUILD_PLAN's risk register (reason table with indices, or one
  named bound raised in all three places in one change). `MAX_PLANS` is 65,536
  and `MAX_ITEMS` 65,536; neither is close.

## A2 A kind for every callable operation

`catalog/kinds.tsv` is written by hand today: nothing in the tree generates it
(`smithy/compile/runtime_inputs.rs` only reads it). It already holds 313
`operation` rows for 17 first-wave services, 168 `cfn` rows and 113 `api` rows.
It has no row for `sqs`.

### A2.1 Kind generator

- **Goal:** `catalog-tool kinds generate [--check | --write]` writes
  `catalog/kinds.generated.tsv` with a kind row for every callable operation
  that `kinds.tsv` does not already cover.
- **Depends on:** A1.4.
- **Files:** `crates/catalog-tool/src/kind_map.rs`, `kind_map/parse.rs`,
  `crates/catalog-tool/src/kind_map/generate.rs` (new),
  `kind_map/generate/match_cfn.rs` (new), `smithy/compile/runtime_inputs.rs`,
  `main.rs`, `catalog/kinds.generated.tsv` (new, generated), `moon.yml`.
- **Change:**
  1. `kind_map::load_with_cfn` takes a second byte slice, the generated file,
     and parses its rows after the hand rows with the same `parse_row`. A
     generated row whose key (`cfn` type, `api` kind, or `operation` service and
     name) already exists in the hand file is skipped, not an error. Generated
     files may contain only `cfn`, `api` and `operation` rows; `meta`,
     `relation`, `bootstrap` and `default-unmapped` are rejected there.
  2. For each `CallableOperation` with no hand `operation` row, choose a kind:
     - CloudFormation service name: the `cloudFormationName` of the model's
       `aws.api#service` trait (retained in `RETAIN_PAYLOADS`).
     - Resource name: the item shape's name with a trailing `Summary`,
       `Description`, `Info`, `Entry`, `Item` or `Detail` removed.
     - If `AWS::<service>::<resource>` is one of the 1,881 types in
       `catalog/cloudcontrol-universal.json`, compared exactly, and no other
       operation already maps to it: emit `cfn <type> map <type> <display>`.
     - Otherwise emit `api AWS::ApiOnly::<SdkId><Resource> kind generated: no
       exact CloudFormation type` where `<SdkId>` is the trait's `sdkId` with
       spaces removed.
     - Emit `operation <service> <Operation> <kind>`.
  3. Display name (the fifth `cfn` cell): the item's string member named `Name`,
     else `<Resource>Name`, else `-`. `api` rows have no display cell today; add
     an optional fifth cell to `parse_api` and a field `display_name_path` to
     `ApiOnlyKind`, and pass it in `schemas::compile` where `None` is passed
     now.
  4. Two callable operations of one service must not share a kind unless their
     item shapes are the same shape ID. If they differ, the second (in name
     order) is not given a kind and the generator prints it; A1's generator
     reads that list and defers the operation with "kind: another enumeration
     already produces this resource". (`schemas/selected.rs` and
     `validate_kind_path` both assume one item type per kind.)
  5. `runtime_inputs::augment` reads `catalog/kinds.generated.tsv` when it
     exists. `moon.yml`: task `kinds-generated` after `overlay-check`.
- **Tests:** exact CloudFormation match; near miss falls to `ApiOnly`; hand row
  wins; two operations with different item shapes; scalar item (kind from the
  list member name); SQL-name collision is reported by the existing
  `validate_collisions`; regenerating gives identical bytes.
- **Prove it:** `cargo test -p catalog-tool --locked kind_map`; with nothing
  generated, `kinds generate --check` and `models compile --check` pass.
- **Done when:** the command and gate task exist and pass on the current tree.
- **Size:** about 380 added, 30 changed. One change.
- **Watch out:** `MAX_ROWS` in `kind_map.rs` is 10,000 per file and the full
  set will have about 2,200 operation rows plus kind rows: within bounds, but
  check. A published table name must never change; the gate compares the
  committed file. Matching on a CloudFormation name is a naming choice, not an
  authorization, so the "never allow on a name" rule is not involved.

### A2.2 Kinds for the thin slice, and the first wider catalog

- **Goal:** Compile a catalog in which the thin-slice operations are callable.
- **Depends on:** A1.6, A2.1.
- **Files:** `catalog/kinds.generated.tsv`, `catalog/aws-api.catalog`,
  `catalog/aws-relations.report`, `catalog/aws-overlay.report`.
- **Change:** run `kinds generate --write`, then `models compile --write`.
  Review the generated kind names for the eight services by eye once and move
  any wrong mapping into `catalog/kinds.tsv` as a hand row.
- **Tests:** add to `crates/catalog-tool/tests/cli_models_compile.rs`: every
  plan in the compiled catalog has a kind with a schema; all table names are
  distinct; the four original plans are byte-identical to before.
- **Budget:** P16 and P11, recorded before and after (first growth of the
  catalog).
- **Prove it:** all catalog gate tasks pass; `aws-inventory catalog --models`
  still prints the model counts; the baseline `sweep_e2e` suite still passes (it
  now also sweeps the new operations against the emulator, with nothing seeded
  for them).
- **Done when:** the gate passes, and the suite result is recorded including
  how many scopes ended unsupported or failed. Those outcomes are expected
  input for A5, not a failure of this step, but the suite's exit code matters:
  see "Watch out".
- **Size:** data only. One change.
- **Watch out:** `tests/sweep_e2e` asserts no failed scopes
  (`scenario/arguments.rs::assert_no_failures`). Once more operations are
  callable, an operation the emulator does not serve makes the sweep exit 2. If
  that happens, land A5.2 (the probe) first and defer the failing operations with
  the probe's reason; do not relax the assertion. Existing databases cannot be
  reopened with a new catalog (`StoreError::CatalogMismatch`); tests create
  fresh ones.

### A2.3 Table names that fit PostgreSQL

- **Goal:** Every table name is distinct and is a valid PostgreSQL identifier
  without quoting.
- **Depends on:** A2.1.
- **Files:** `crates/catalog-tool/src/names.rs`, `names/identifier.rs`,
  `crates/core/src/inventory/identifiers.rs` (`SqlName`),
  `catalog/aws-names.report` (new, generated).
- **Change:**
  1. PostgreSQL truncates identifiers to 63 bytes without an error.
     `MAX_SQL_NAME_BYTES` allows 128, and ROADMAP M1.5a records names up to 69
     bytes. Add a check: a table name over 63 bytes is replaced by its first 54
     bytes, then `_`, then the first 8 hex digits of the BLAKE3 hash of the
     full name. Index names built from a table name (C1.5) use the same rule.
  2. Add PostgreSQL's reserved words to the keyword check (the words that
     cannot be a table name unquoted). The keyword list and the reserved
     prefix rule already in `names/identifier.rs` stay until K2.1, because the
     schema 13 views still use these names.
  3. Report: `long <full> <shortened>` rows and a `names total=<n> long=<m>
     distinct=<n>` line. Fail if distinct is less than total.
- **Tests:** a 64-byte and a 128-byte name; two long names that share the first
  54 bytes; a name equal to a PostgreSQL reserved word; the 1,881 CloudFormation
  names stay distinct after the rule (extend `names/tests/catalog.rs`).
- **Prove it:** `cargo test -p catalog-tool --locked names`; `kinds generate
  --check`.
- **Done when:** the report exists and shows how many names were shortened (the
  number is not verified today; the plan only knows it is at least one).
- **Size:** about 150 added. One change.
- **Watch out:** shortening renames a published table. Do it before any
  database outside tests exists (V24 allows it now). `SqlName::parse` in core
  must accept the shortened form; it has its own validation.

## A3 Child operations

### A3.1 Bindings from modeled identifiers

- **Goal:** Make the 337 expansions with modeled identifiers callable through
  generation.
- **Depends on:** A1.4, A2.1.
- **Files:** `crates/catalog-tool/src/overlay/generate/service.rs`,
  `overlay/generate/expand.rs` (new).
- **Change:**
  1. For a `Class::Expand` operation, call `plans::expansion`. For
     `Expansion::Modeled { list_source: true, ambiguous_resource: false, .. }`
     whose list operation is callable in the same generation pass, emit
     `class = "expand"`, `scope` (as A1.2), and `identity`, `items` and
     `paginator` by the A1.2 rules applied to the expansion's output. Bindings
     are not written: `overlay/check/validation/derived.rs::bindings` already
     derives them.
  2. When the output is a single structure and not a list (a `Get` or
     `Describe` of one resource), write `items = "<member>"` for the one
     structure member of the output, or `items = "$"` when the output itself is
     the resource.
  3. Order matters: generate enumerations first, then expansions, because the
     dry run needs the parent's entry in the `overlays` map.
  4. Everything else stays `defer`, now with the specific reason:
     "expand: required member is not a modeled identifier", "expand: operation
     is not part of a modeled resource", "expand: parent enumeration is
     deferred".
- **Tests:** modeled expansion with callable parent; parent deferred; output is
  a list; output is one structure; binding cycle is rejected by the existing
  check (`ViolationRule::BindingCycle`).
- **Prove it:** `cargo test -p catalog-tool --locked overlay::generate`; for the
  thin slice, `overlays generate --write` then `overlay --check` passes and the
  report shows `callable_expand` above 0 (lambda has 7 modeled expansions).
- **Done when:** `catalog/aws-overlay.report` shows `callable_expand>0` for the
  first time.
- **Size:** about 280 added. One change.
- **Watch out:** a child that describes its parent returns the same resource
  again with more fields. Until A3.4 decides how that is stored, give such a
  child the parent's kind only when the item shapes are identical; otherwise it
  gets its own kind (A2.1 rule 4 applies).

### A3.2 Bindings by name and shape

- **Goal:** Bind the remaining expansions where the model leaves no doubt, and
  account for all 2,940.
- **Depends on:** A3.1.
- **Files:** `overlay/generate/expand.rs`, `overlay/generate/bind_by_name.rs`
  (new), `overlay/check/validation/pagination/compatibility.rs` (visibility of
  `compatible`).
- **Change:**
  1. For each required input path of the expansion
     (`Model::required_paths`), look at every callable enumeration of the same
     service. A candidate source is an item member with exactly the same name
     as the input member and a compatible shape (`compatible`). Also accept the
     enumeration's identity member when the input member is named
     `<Resource>Id`, `<Resource>Name` or `<Resource>Arn` and the identity member
     has that exact name.
  2. Bind only when exactly one enumeration offers a source for every required
     input and it is the same enumeration for all of them. Write
     `[operation.<Op>.bind]` with `<Input> = { from = "<Enumeration>", path =
     "<Member>" }` for each.
  3. Inputs named `AccountId` or `Region` bind with `{ account = true }` or
     `{ region = true }` when their shape is a string.
  4. Otherwise defer with "expand: no unique source for <Input>" or "expand:
     <n> possible sources for <Input>".
  5. Report totals: `expand_candidates`, `bound_modeled`, `bound_by_name`,
     `deferred` and a row per reason. They must sum to 2,940 when all services
     are listed.
- **Tests:** unique source; two sources (deferred); name matches but shape does
  not (deferred); account and region context bindings; a required nested input
  whose required siblings are missing (the checker's rule "activating a nested
  input also requires its required siblings").
- **Prove it:** `cargo test -p catalog-tool --locked overlay::generate`;
  regenerate the thin slice and check.
- **Done when:** the report sums and the thin-slice numbers are recorded.
- **Size:** about 330 added. One change.
- **Watch out:** binding by name can produce a call fan-out the account does
  not need (one child call per parent row). The emitted report must list, per
  bound expansion, its parent, so B6.2 and B6.3 can remove calls with evidence.
  A child of a child is allowed; cycles are rejected by the check.

### A3.3 Child work without scanning every plan

- **Goal:** `child::work` loops over every row and, inside that, over every
  plan in the catalog. With about 2,000 plans that is millions of iterations per
  page. Index children by parent before the catalog grows.
- **Depends on:** D1.2 (the code moves to the sweep crate first).
- **Files:** `crates/core/src/catalog/runtime.rs`,
  `crates/core/src/catalog/runtime/index.rs` (new),
  `crates/sweep/src/execution/page/child.rs` (after D1.2; today
  `src/sweep_runtime/execution/page/child.rs`),
  `crates/sweep/src/execution/page.rs` (`lookup_plan`),
  `crates/sweep/src/execution/page/rows.rs` (`resource_name`, `references`).
- **Change:**
  1. `RuntimeCatalog::from_bytes` builds, once: `by_operation:
     BTreeMap<(Box<str>, Box<str>), usize>` for `lookup`; `children:
     BTreeMap<usize, Box<[usize]>>` from a parent plan index to the plans that
     have a `BindingSource::Operation` naming it; `name_path_by_kind` and
     `relations_by_source_kind` keyed by `Kind`.
  2. Add `pub fn children_of(&self, plan: &CallablePlan<'_>) ->
     impl Iterator<Item = &CallablePlan<'a>>`, `pub fn name_path(&self, kind:
     &Kind) -> Option<&[FieldId]>`, `pub fn relationships_from(&self, kind:
     &Kind) -> &[RelationshipPlan]`. `lookup` uses the map.
  3. `child::work`, `resource_name` and `references` use them.
- **Tests:** core: indexes agree with a linear scan on the committed catalog and
  on a fixture with a two-level parent chain. Sweep crate: the existing
  `child.rs` tests pass unchanged.
- **Budget:** P6 (CPU seconds) and P1 on the thin benchmark must not rise;
  record both before and after. Command: `cmdb-bench sweep --profile thin`.
- **Prove it:** `cargo test -p inventory-core --no-default-features --locked`;
  `cargo test -p inventory-sweep --locked`; the benchmark rows.
- **Done when:** both numbers are recorded.
- **Size:** about 220 added, 40 deleted. One change.
- **Watch out:** `BindingSource::ParentIdentity` has no named parent; keep the
  old behavior for it (tried against every parent) but note that the catalog
  compiler never emits it today (`binding_value` writes sources 0, 1, 2 and 4
  only).

### A3.4 Children against the emulator, and what a describe stores

- **Goal:** The first real run of `child.rs`: a parent page creates child work
  and the children are stored. Decide and implement how a list of names plus a
  describe becomes one kind.
- **Depends on:** A3.1, A3.3, A4.3, C6.10 (PostgreSQL through the CLI).
- **Files:** `crates/catalog-tool/src/overlay/types.rs`, `overlay/operation.rs`,
  `overlay/raw.rs` (new key), `runtime_catalog/plans/compile.rs` (new plan
  field), `crates/core/src/catalog/runtime/types/plan.rs`,
  `crates/core/src/catalog/runtime/decode/plans.rs`,
  `crates/sweep/src/execution/page.rs`, `tests/sweep_e2e/scenario.rs`,
  `docs/API_CATALOG.md`.
- **Change:**
  1. New overlay key `emit`, values `"resources"` (default) and `"work"`. An
     enumeration with `emit = "work"` creates child work from its rows and
     stores no resource rows. The generator writes it when the item is a scalar
     or has only identifier members, and exactly one callable expansion bound to
     it returns one structure.
  2. Runtime plan field 14 `emit` (0 resources, 1 work). The extension version
     in field 1 becomes 2; the reader accepts 2 only. Reader and compiler change
     in this one change because every map is closed.
  3. `prepare_response` skips `rows::observations` for a work-only plan and
     builds child work from the items alone (identity still computed for the
     child scope's parent key).
  4. End-to-end test in `tests/sweep_e2e`: seed IAM roles with attached
     policies, or SNS topics with subscriptions, sweep, and assert that child
     rows exist, that `resource_parents` links them, and that deleting the
     parent in the emulator removes the children after two complete sweeps.
- **Tests:** unit: work-only page yields children and zero observations;
  catalog reader rejects version 1 and unknown `emit` codes. End to end as
  above.
- **Budget:** P3 (API calls): record calls per seeded resource for the slice
  before and after children are on. This is the baseline B6 works against.
- **Prove it:** `cargo test --locked --features sweep --test sweep_e2e -j1 --
  --ignored --test-threads 1` with the new test.
- **Done when:** children are stored for at least one parent kind of at least
  two services, and the parent-deletion case passes.
- **Size:** about 350 added, 30 changed. Two changes if needed: (1) key and
  catalog field, (2) runtime and test.
- **Watch out:** the parent of a work-only scope is not a stored resource, but
  `ScopeKey::new(.., Some(parent))` and the `resource_parents` foreign key
  expect one. For work-only parents pass `None` as the scope parent and carry
  the parent identity in the bindings only. `MAX_PAGE_CHILDREN` is 10,000.
- **Review fix (2026-11-18):** a work-only list must still leave durable
  evidence of what it listed, or an ID that later disappears can never be
  removed. Add a logged table `scope_members(account, region, scope_key,
  member_id, missing_complete)`, written in the same transaction as the page
  that creates the describe work, and reconciled at scope completion by the
  same two-complete-enumerations rule as `resource_scopes`. A member that is
  no longer listed has its described resource removed even if its describe
  was never called again. Tests: an ID disappears from a later list; the
  describe is denied (the member stays, the resource is marked inaccessible,
  nothing is deleted); pagination is interrupted and resumed; the same
  members arrive in a different page order; a member listed twice.

## A4 Seed files

### A4.1 Seed scenarios as data

- **Status:** A4.1 harness implementation complete. `cargo fmt -p
  inventory-harness -- --check`, `cargo clippy -p inventory-harness
  --all-targets --locked -j2 -- -D warnings` and `cargo test -p
  inventory-harness --locked -j2 inventory_seed` pass. The unchanged
  `aws-inventory` `sweep_e2e` proof remains the integration coordinator's lane;
  it is not a test target in the package-limited harness run.
- **Format details:** expansion is repeat-index-major, retaining the old
  VPC/subnet interleaving. Symbolic IDs stay separate from numeric `u16` IDs;
  duplicates and forward/missing bindings are errors. A singleton reference
  broadcasts; otherwise its matching repeat instance must exist. Nested
  structures resolve modeled field IDs; maps retain bounded named keys.
- **Additional bounds:** 256 nodes per input, 1,024 bytes per expanded text,
  128 bytes per field name, 4,096 relationships and a conservative 64 MiB
  expanded-string budget. Counts, depth and byte estimates are checked before
  repeat materialization. TOML
  field order is preserved so the embedded default equals the legacy
  descriptors, including input and response-check order.
- **Blob literals:** a TOML string targeting a modeled blob (including nested
  structure, list or map members) is canonical standard-alphabet padded base64.
  Preflight decodes it once into bytes; the existing 1,024-byte encoded-text
  bound limits decoded content to 768 bytes. Invalid alphabet, padding or
  trailing bits fail before network calls. Ordinary modeled strings stay text.

- **Goal:** Read a seed scenario from a TOML file into the existing
  `SeedScenario`, with the input types real create calls need.
- **Depends on:** nothing.
- **Files:** `crates/harness/src/inventory_seed/scenario.rs` (269 lines),
  `inventory_seed/scenario/validation.rs`, `inventory_seed/file.rs` (new),
  `inventory_seed/file/value.rs` (new), `inventory_seed/model.rs` (291 lines),
  `inventory_seed/wire.rs` (`insert_path`), `inventory_seed.rs`,
  `crates/harness/seeds/ec2.toml` (new).
- **Change:**
  1. File format, one file per service:

     ```toml
     format = 1
     service = "ec2"

     [[step]]
     id = "vpc"
     operation = "CreateVpc"
     kind = "AWS::EC2::VPC"
     identity = "Vpc.VpcId"
     repeat = 6
     [step.input]
     CidrBlock = "10.88.0.0/16"
     [step.check]
     "Vpc.CidrBlock" = "CidrBlock"

     [[step]]
     id = "subnet"
     operation = "CreateSubnet"
     kind = "AWS::EC2::Subnet"
     identity = "Subnet.SubnetId"
     repeat = 6
     [step.input]
     VpcId = { from = "vpc", path = "Vpc.VpcId" }
     CidrBlock = "10.88.{n}.0/24"

     [[relationship]]
     source = "subnet"
     target = "vpc"
     relation = "IN_VPC"
     observed = "VpcId"
     ```

     `kind = ""` marks a step that creates nothing to count (for example
     attaching a policy). `{n}` is the 1-based repeat index. A `from` reference
     to a repeated step takes the same index.
  2. Extend `SeedInputValue` with `List(Vec<SeedInputValue>)` and
     `Structure(Vec<(String, SeedInputValue)>)`; TOML arrays and inline tables
     map to them. `PreparedInputValue` and `wire::insert_path` gain the same
     two cases, building `Value::List` and `Value::Map`.
  3. Step IDs become strings in the file; `file.rs` assigns `StepId` numbers.
     `StepId(pub u8)` becomes `StepId(pub u16)`.
  4. Bounds: `MAX_STEPS` 12 becomes 4,096 after repeat expansion,
     `MAX_INPUTS_PER_STEP` 8 becomes 64, `MAX_TOTAL_INPUTS` 24 becomes 65,536,
     nesting depth 8, file size 1 MiB, unknown keys rejected at every level.
  5. `default_scenario()` is replaced by loading `crates/harness/seeds/ec2.toml`
     (embedded with `include_str!`), and must produce the same 12 steps and 6
     relationships. Delete `default_pair_steps`, `default_vpc_step`,
     `default_subnet_step`, `default_relationship`.
- **Tests:** the embedded file equals the old scenario step for step; unknown
  key; reference to a later or missing step; repeat of 0 and over the bound;
  list and structure inputs round-trip into the request record; `{n}` expands.
- **Prove it:** `cargo test -p inventory-harness --locked inventory_seed`; the
  baseline `sweep_e2e` suite passes unchanged (it calls `default_scenario`).
- **Done when:** no scenario is written in Rust.
- **Size:** about 420 added, 110 deleted. One change.
- **Watch out:** `TypedSeedOracle` has VPC-specific fields (`vpc_id`,
  `subnet_id`, `pairs`) used by `tests/sweep_e2e/state.rs`; keep them filled
  for the EC2 file and use `resources` and `edges` everywhere else. `model.rs`
  is 9 lines from the limit: put new prepare functions in
  `inventory_seed/model/values.rs` (new).

### A4.2 `seed run` and the oracle file

- **Goal:** `inventory-harness seed run` seeds an emulator from a directory of
  seed files and writes what it created.
- **Depends on:** A4.1.
- **Files:** `crates/harness/src/cli.rs`, `crates/harness/src/main.rs`,
  `inventory_seed/runner.rs`, `inventory_seed/runner/oracle.rs`,
  `inventory_seed/report.rs` (new).
- **Change:**
  1. `inventory-harness seed run --seeds <dir> --catalog <file> --endpoint
     <https loopback URL> --ca-bundle <file> --region <name> [--service
     <name>]... --oracle <file> --matrix <file>`. `SeedConfig::new` already
     enforces a loopback HTTPS endpoint and a CA file.
  2. Run files in name order and steps in file order. A rejected step does not
     stop the run: it is recorded, and every later step that references it is
     recorded as "skipped: depends on <id>".
  3. Oracle file, tab separated, sorted: `resource <kind> <account> <region>
     <id>` and `edge <from kind> <from id> <relation> <to kind> <to id>`.
  4. Matrix file, sorted: `<service> <operation> <step id> accepted` or
     `... rejected <error code> <first 200 bytes of message>`.
  5. Exit 0 when every file ran; the matrix carries the rejections.
- **Tests:** against a local test server (the pattern in
  `crates/harness/src/proxy` tests): accepted step, rejected step, dependent
  step skipped, oracle sorted and stable across two runs with the same
  responses.
- **Prove it:** start the pinned emulator as `tests/sweep_e2e/stack.rs` does,
  run `seed run` with `crates/harness/seeds`, and run it a second time against
  a fresh emulator: the two oracle files differ only in generated identifiers
  and have the same per-kind counts.
- **Done when:** seeding works with no Python on `PATH` and the matrix exists.
- **Size:** about 330 added. One change.
- **Watch out:** the write capability stays inside the harness crate:
  `MutationPermit` is `pub(super)` in `inventory_seed/config.rs` and
  `wire::call_mutation` requires it. Do not make either public.

### A4.3 Thin-slice seed files

- **Goal:** Hand-written seed files for the thin-slice services, kept to what
  the emulator accepts.
- **Depends on:** A4.2.
- **Files:** `crates/harness/seeds/{iam,sns,sqs,dynamodb,kms,lambda,s3,cloudwatch}.toml`
  (new), `catalog/emulator-seed.report` (new: the matrix from A4.2, committed).
- **Change:** for each service write two to four create steps with required
  members only, including at least one parent with a child (IAM role and
  attached policy, SNS topic and subscription, DynamoDB table) and at least one
  cross-service reference (SNS subscription to an SQS queue ARN, Lambda function
  to an IAM role ARN) with a `[[relationship]]`. Run, read the matrix, remove or
  fix rejected steps, and commit the matrix.
- **Tests:** a harness test loads every file in `crates/harness/seeds` and
  validates it against the committed catalog (operation exists, paths resolve),
  without an emulator.
- **Prove it:** `inventory-harness seed run ...` gives a matrix with at least
  one accepted create step for each service that the emulator serves; services
  it does not serve are listed as rejected with the emulator's error.
- **Done when:** the matrix is committed. A service with every step rejected is
  reported to the coordinator, who swaps it for a substitute.
- **Size:** data, about 300 lines of TOML. One change, or one per service.
- **Watch out:** which services the emulator serves is unknown until this
  runs. Lambda create needs code bytes; the seed may send a tiny zip as a blob
  input only if `SeedInputValue` gets a `Blob` case (add it here if needed, hex
  in the file). Seeding KMS keys creates keys in the emulator only; the
  loopback check in `SeedConfig` is what keeps this away from real AWS.

### A4.4 Draft seed files from the model

- **Goal:** `inventory-harness seed draft --service <name>` writes a first seed
  file from the model's create operations, to speed up A4.6.
- **Depends on:** A4.2.
- **Files:** `crates/harness/src/inventory_seed/draft.rs` (new),
  `inventory_seed/draft/values.rs` (new), `cli.rs`, `main.rs`.
- **Change:**
  1. Candidates: operations named `Create*`, `Put*` or `Register*` whose
     output or input carries an identifier of a kind that a callable
     enumeration returns.
  2. For each required input member, synthesize a value by shape: string
     `inv-<step>-{n}` (respecting `length` and, when the pattern is a simple
     character class, the pattern); integer the `range` minimum or 1; boolean
     false; enum its first value; list one element; structure its required
     members, recursively to depth 4. A member that needs another resource's
     identifier becomes a `from` reference when exactly one earlier step
     produces a member of that name; otherwise the step is written commented
     out with the reason.
  3. Output goes to stdout or `--output`; it is never written into
     `crates/harness/seeds` directly.
- **Tests:** string with length bounds; enum; nested required structure;
  unresolvable reference produces a commented step; output parses with A4.1's
  reader once comments are removed.
- **Prove it:** `inventory-harness seed draft --service sns` output, after
  `seed run`, has at least one accepted step.
- **Done when:** the command exists and is used for one service outside the
  thin slice.
- **Size:** about 380 added. One change.
- **Watch out:** this reads models through the compiled catalog
  (`RuntimeCatalog::catalog()`), not the JSON files; the harness already depends
  on `catalog-tool` but the runtime path should not. Never synthesize a value
  for a member the model marks `sensitive`; leave the step commented.

### A4.5 Bulk seeding for the benchmark, and the write-capability proof

- **Goal:** Seed thousands of resources quickly for B5, and prove the product
  binary cannot write.
- **Depends on:** A4.2.
- **Files:** `crates/harness/src/inventory_seed/runner.rs`,
  `crates/harness/seeds/bench/*.toml` (new), `tests/product_is_read_only.rs`
  (new), `crates/catalog-tool/tests/cli_models_compile.rs`.
- **Change:**
  1. `seed run --concurrency <n>` (default 1, maximum 64): repeated steps with
     no reference to their own earlier repeats run concurrently, bounded by a
     `tokio::sync::Semaphore`. Order of oracle rows is still sorted.
  2. Bench seed files with `repeat` in the thousands for cheap resources (SQS
     queues, SNS topics, IAM roles, DynamoDB tables, EC2 security groups).
  3. Read-only proof, two tests: (a) for every plan in the committed catalog,
     the operation's class is enumerate or expand and it is not in
     `must-deny.txt` (catalog-tool test); (b) `cargo metadata` shows that
     neither `aws-inventory` nor the sweep library crate has a normal
     dependency on `inventory-harness`, and the only `http::Method` values the
     sweep library can send come from catalog plans (assert on
     `request::encode`'s inputs in a unit test).
- **Tests:** as above, plus bulk run determinism: same file, same counts.
- **Budget:** seeding time for the `thin` bench profile is recorded (it bounds
  how often the benchmark can run). Command: `time inventory-harness seed run
  --seeds crates/harness/seeds/bench ...`.
- **Prove it:** both proof tests pass; bulk seed of 10,000 resources completes
  and its time is recorded.
- **Done when:** the numbers and tests exist.
- **Size:** about 260 added. One change.
- **Watch out:** the emulator may cap resources per service; record the cap as
  the benchmark's stated limit (ROADMAP M4.4 asks for that).

### A4.6 Seed files for everything the emulator serves

- **Goal:** Seed coverage for every service the probe shows the emulator
  serves.
- **Depends on:** A4.4, A5.3.
- **Files:** `crates/harness/seeds/*.toml`, `catalog/emulator-seed.report`.
- **Change:** take the services with "answered" outcomes in
  `catalog/emulator-probe.report`, most resource kinds first. For each: draft,
  run, trim to accepted steps, commit with the updated matrix. One change per
  batch of about ten services.
- **Tests:** the seed-file validation test from A4.3 covers every new file.
- **Prove it:** after each batch, `inventory-harness e2e run` (A7.1) passes its
  per-kind count check for the batch's services.
- **Done when:** every service with a probe outcome of "answered with items" or
  "answered empty" has either accepted seed steps or a matrix row saying why
  not.
- **Size:** data. Many changes.
- **Watch out:** this is the long pole for verified coverage. An operation is
  emulator-verified only when it returned at least one seeded resource, so an
  empty answer leaves its kind "implemented, not verified" (X3).

## A5 Emulator probe

### A5.1 Keep the reason when a scope fails

- **Goal:** `page.rs::finish_error` turns every error into an outcome and
  drops the message. Store the reason, as VISION section 7 requires for decode
  errors.
- **Depends on:** D1.2, C6.12 (so the trait method changes once, for one
  store).
- **Files:** `crates/engine/src/inventory/postgres/schema/shared.rs`,
  `inventory/postgres/work.rs` (`finish_scope`),
  `crates/engine/src/inventory/store_api.rs` (trait method signature),
  `crates/sweep/src/execution/page.rs`, `crates/sweep/src/error_detail.rs`
  (new).
- **Change:**
  1. `scope_outcomes` gains `detail text` (nullable, at most 1,024 bytes).
  2. `finish_scope(lease, outcome, observed_at_ms, detail: Option<&str>)`.
  3. `error_detail.rs`: `fn detail(error: &SweepError) -> String` giving
     `<class>: <message>` where class is one of `decode`, `protocol`,
     `transport`, `denied`, `unsupported`, `throttled`, `store`, `other`,
     truncated on a character boundary. For `SweepError::Decode` include the
     operation's output shape ID.
  4. `finish_error` passes it.
- **Tests:** each class maps as stated; a 5,000-byte message is truncated; the
  detail is readable back through SQL.
- **Budget:** P8 (bytes written) on the thin benchmark must not change for a
  run with no failures. Command: `cmdb-bench sweep --profile thin`.
- **Prove it:** store conformance tests; a sweep through the fault proxy with
  a malformed-body rule leaves a `decode:` detail.
- **Done when:** a failed scope can be explained from SQL alone.
- **Size:** about 160 added. One change.
- **Watch out:** this changes the shared schema; it is link 2 of the schema
  chain in BUILD_PLAN. Never store a response body or header in the detail.

### A5.2 The probe command and the thin-slice probe

- **Goal:** `inventory-harness probe` calls every callable enumeration once
  against the seeded emulator and writes `catalog/emulator-probe.report`.
- **Depends on:** D1.3 (sweep library API), A4.3, A2.2.
- **Files:** `crates/harness/src/probe.rs` (new), `probe/report.rs` (new),
  `cli.rs`, `main.rs`, `crates/harness/Cargo.toml` (dependency on the sweep
  library), `crates/sweep/src/lib.rs` (a public single-call entry),
  `catalog/emulator-probe.report` (new, generated).
- **Change:**
  1. The sweep library exposes `pub async fn call_once(catalog, credentials,
     client, executor, plan, region, endpoint_override) ->
     Result<ProbeAnswer, SweepError>` (new), where `ProbeAnswer { items: u64,
     http_version: http::Version }`. It builds the request exactly as a sweep
     does, for the first page only, and stores nothing.
  2. `inventory-harness probe --catalog <file> --endpoint <url> --ca-bundle
     <file> --region <name> --report <file>`: for each plan with
     `PlanPurpose::Enumeration` and no parent binding, call once. Bounded
     concurrency 8.
  3. Report rows, sorted: `<service> <operation> <protocol> items <n>` |
     `... empty` | `... unsupported <detail>` | `... failed <class> <detail>`,
     and a totals line per outcome and per protocol.
  4. The probe makes nothing callable and changes no overlay.
  5. Run it for the thin slice and commit the report.
- **Tests:** against a local test server: each of the four outcomes; a plan
  with a parent binding is listed as `skipped child`; the report is stable.
- **Prove it:** seeded emulator, then `inventory-harness probe ...`; the
  report has a row for every thin-slice enumeration.
- **Done when:** the report is committed, and every `failed decode` row has a
  follow-up: an overlay correction in `catalog/overlay/<service>.toml` or a
  decoder defect recorded with operation and shape.
- **Size:** about 320 added. One change.
- **Watch out:** this is the first evidence of which thin-slice services the
  emulator serves. If fewer than five protocols end up with an "items" row,
  take substitutes from BUILD_PLAN before Wave 2 closes. The harness gains a
  dependency on the sweep library; the reverse must never exist.

### A5.3 Full probe

- **Goal:** The probe over every callable enumeration of all services.
- **Depends on:** A1.7, A5.2.
- **Files:** `catalog/emulator-probe.report`, `catalog/overlay/*.toml` (hand
  corrections), `catalog/overlay/defer-inputs.txt`.
- **Change:** run the probe after seeding with what exists. Triage in this
  order: (1) `failed decode` and `failed protocol` are overlay or decoder
  defects, fix the data first; (2) operations that return AWS-owned data
  (hundreds of items with nothing seeded) get a hand `defer` or a `fixed`
  owner filter; (3) `unsupported` stays callable and is reported under X3.
- **Tests:** a gate test asserts that every callable enumeration has a row in
  the committed probe report (so the report cannot go stale silently).
- **Prove it:** `inventory-harness probe ...` twice gives the same outcome per
  row.
- **Done when:** no `failed decode` row is left without a recorded cause.
- **Size:** data and small overlay edits. Several changes.
- **Watch out:** an enumeration that answers with items when nothing was seeded
  is the signal for a missing owner filter. Check those before A7.

## A6 Cloud Control cross-check

### A6.1 `verify cloud-control`

- **Goal:** Compare the identities Cloud Control lists with the identities the
  sweep stored. Read-only on the inventory.
- **Depends on:** C6.10, A7.1.
- **Files:** `crates/harness/src/assertions/cloud_control.rs` and its
  directory (existing, schema 13 today), `crates/harness/src/verify.rs` (new),
  `verify/cloud_control.rs` (new), `cli.rs`, `main.rs`,
  `catalog/cloudcontrol-differences.tsv` (new, data).
- **Change:**
  1. `inventory-harness verify cloud-control --database <URL> --endpoint <url>
     --ca-bundle <file> --region <name> --report <file>`.
  2. For every kind whose `KindCoverage` has a CloudFormation type and whose
     type is list-capable in `catalog/cloudcontrol-universal.json`, call
     `ListResources` through the harness's existing `aws-sdk-cloudcontrol`
     dependency, bounded pages.
  3. Read stored identities with `SELECT id FROM resources WHERE kind = $1`
     through a read-only connection.
  4. Identity mapping: compare the Cloud Control primary identifier with the
     stored `id`; when the stored id is an ARN and the identifier is not,
     compare with the ARN's resource part.
  5. Report rows: `<type> matched <n> missing_from_sweep <n> extra_in_sweep
     <n>` and one row per differing identity. A difference is "explained" when
     `catalog/cloudcontrol-differences.tsv` has a row `<type> <rule> <reason>`
     that covers it.
  6. Exit 0 when no unexplained difference exists for a seeded kind.
- **Tests:** matched; missing; extra; explained by rule; a type Cloud Control
  cannot list is reported as `not listable` and not as a difference; the
  command issues no write statement (run it on a read-only role).
- **Prove it:** after `inventory-harness e2e run` on the thin slice,
  `inventory-harness verify cloud-control ...` prints the report.
- **Done when:** the report exists for the thin slice.
- **Size:** about 380 added. One change.
- **Watch out:** V39: this is a checker. It must not be linked into
  `aws-inventory` and must never write to the inventory. The emulator's Cloud
  Control support differs per type
  (`crates/harness/fixtures/universal-unsupported-aws-emulator-1.0.348.json`).

### A6.2 Zero unexplained differences for seeded kinds (X6)

- **Goal:** Exit-test line X6.
- **Depends on:** A6.1, A7.2.
- **Files:** `catalog/cloudcontrol-differences.tsv`, overlay corrections.
- **Change:** run the cross-check after the full sweep. Each difference is
  either a defect (fix the overlay identity or the seed) or explained with a
  written reason.
- **Tests:** the e2e command (A7.1) runs the cross-check as its last stage.
- **Prove it:** `inventory-harness e2e run --work <empty dir>` exits 0 with the
  cross-check stage reporting zero unexplained differences.
- **Done when:** X6 holds.
- **Size:** data. One or more changes.
- **Watch out:** an explanation must name a rule, not a single resource id, so
  it survives reseeding.

## A7 Full sweep proof

### A7.1 One command for the whole run

- **Goal:** `inventory-harness e2e run` takes an empty directory to a verified
  inventory: start the emulator and the graph server, create a fresh
  PostgreSQL database, seed, run the shipped `aws-inventory sweep`, project,
  verify. This is the CLI half of X1 and the checkpoint command of every wave
  from wave 1 on.
- **Depends on:** A4.2, C6.2, C6.10.
- **Files:** `crates/harness/src/e2e.rs` (new), `e2e/stack.rs` (new),
  `e2e/verify.rs` (new), `e2e/graph.rs` (new; moved from
  `tests/sweep_e2e/stack/graph.rs` so both use it), `cli.rs`, `main.rs`,
  `tests/sweep_e2e/stack.rs` (use the moved code), `scripts/e2e-baseline.sh`.
- **Change:**
  1. `inventory-harness e2e run --work <empty dir> [--postgres-url <admin
     URL>] [--seeds <dir>] [--service <name>]... [--collector <path to
     aws-inventory>] [--keep]`.
  2. Stages, each timed and printed as `stage <name> <ms> <result>`:
     `start` (emulator through `Emulator::start`, Redis with FalkorDB through
     the moved `GraphServer`, and a `TestDatabase` from C6.2 on the given
     server), `seed` (A4.2), `sweep` (spawn the collector binary with the
     arguments `tests/sweep_e2e/scenario/arguments.rs` builds today and the
     database URL), `verify-counts` (per-kind counts from SQL equal the
     oracle), `resweep` (second run, `resource_writes=0`), `verify-graph`
     (E5.1 when it exists), `cross-check` (A6.1 when it exists), `stop`.
  3. Refuse a non-empty `--work`. The emulator and graph server are owned
     child processes stopped on every exit path, as `Emulator` and
     `GraphServer` do in `Drop`. The database is dropped at `stop`. With
     `--keep` the command prints the database URL and the emulator endpoint,
     waits for Ctrl-C before stopping the children, and leaves the database in
     place. The PostgreSQL
     server itself is not owned by this command: it is the container of
     `inventory-harness postgres up`, a CI service, or a native server.
  4. With no URL available, exit 2 with the message of C6.2 (the variable and
     the two ways to get a server).
  5. Exit 0 only when every stage that exists passed.
- **Tests:** stage ordering and cleanup with stub binaries; non-empty work
  directory refused; a failing stage stops later stages, stops the children
  and drops the database; missing URL gives exit 2.
- **Budget:** the `sweep` and `resweep` stage times are printed on every run;
  they are a smoke signal only. Recorded numbers come from `cmdb-bench sweep`.
- **Prove it:** with a server up, `inventory-harness e2e run --work "$(mktemp
  -d)" --service ec2 --service sts` exits 0 on this tree's four operations.
- **Done when:** the command passes for the four current operations.
- **Size:** about 420 added, 100 moved. Two changes: (1) move `GraphServer`
  and add the stack, (2) stages and verification.
- **Watch out:** the product binary still has no write capability; the harness
  seeds, then spawns the collector as a separate process. The database URL
  carries a password: pass it to the collector through the environment or an
  argument that is never echoed, and never print it except with `--keep`.

### A7.2 Per-kind counts and the not-verified list (X2, X3)

- **Goal:** Per-kind counts equal the seed for everything the emulator serves,
  and every other callable kind is listed by name as implemented, not verified.
- **Depends on:** A7.1, A5.3, A4.6.
- **Files:** `crates/harness/src/e2e/verify.rs`,
  `crates/harness/src/coverage_status.rs` (new),
  `catalog/coverage-status.report` (new, generated).
- **Change:**
  1. `verify-counts` compares, per kind, the oracle's resource rows with
     `SELECT kind, count(*) FROM resources GROUP BY kind`, and the oracle's
     identities with the stored ids. Any difference fails with the kind and the
     first five differing ids.
  2. `coverage-status.report`: one row per callable kind:
     `<kind> emulator-verified <operation>` when an operation returned at least
     one seeded resource; `<kind> implemented <reason>` otherwise, where the
     reason is the probe outcome (`unsupported`, `empty: nothing seeded`,
     `seed rejected: <code>`). Totals line. No kind may be missing: the report
     is built from the catalog's plans, then joined with the evidence.
  3. `e2e run` fails if the report has a kind with no row.
- **Tests:** verified; implemented with each reason; a callable kind with no
  evidence at all still gets a row; counts mismatch names the kind.
- **Prove it:** `inventory-harness e2e run --work "$(mktemp -d)"` exits 0 and
  writes the report.
- **Done when:** X2 and X3 hold on the full set.
- **Size:** about 260 added. One change.
- **Watch out:** resources the emulator creates by itself (default VPC, default
  security group, service-linked roles) are in SQL but not in the oracle. Take
  a pre-seed sweep as the baseline and compare `after - before` with the
  oracle, instead of listing exceptions by hand.

### A7.3 Zero writes on the second sweep, at full scale (X4)

- **Goal:** Exit-test line X4 on the full set, on PostgreSQL.
- **Depends on:** A7.2, B4.5.
- **Files:** `crates/harness/src/e2e/verify.rs`, overlay `exclude_from_hash`
  corrections.
- **Change:** the `resweep` stage asserts `resource_writes=0` from the
  collector's summary line and, independently, that no row in `resources` has
  `last_changed` later than the first sweep's end. For each kind that fails,
  find the member that changes between reads and add it to that operation's
  `exclude_from_hash` in a hand overlay entry.
- **Tests:** a fixture kind with a volatile member fails the stage with the
  kind named; after exclusion it passes.
- **Budget:** P2 (warm re-sweep time) and P8 (rows written on the warm sweep,
  which must be 0 in logged tables after B4.5). Command: `cmdb-bench sweep
  --profile full`.
- **Prove it:** `inventory-harness e2e run --work "$(mktemp -d)"` exits 0.
- **Done when:** X4 holds and the exclusions added are listed.
- **Size:** data and about 60 lines. One change.
- **Watch out:** `exclude_from_hash` on a generated operation means writing a
  hand entry that repeats the generated keys; A1.1's merge replaces the whole
  operation entry. If that becomes common, add a merge rule "hand entry with
  only `exclude_from_hash` extends the generated one" as its own small step.
- **Review fix (2026-11-18):** do not make the warm sweep pass by hiding
  fields. Before any exclusion: (1) check whether the difference is order
  only, and normalize the order of that list in the decoder instead; (2)
  read the model's documentation for the member. An `exclude_from_hash`
  entry needs a written `reason` in the overlay from a closed list
  (request-scoped token, server clock reading, pagination artifact), and is
  refused for any member that is an identity, a name, a relation source, a
  tag, or a state or configuration value. Tests: a real change to a
  non-excluded member still updates SQL and, where it is a relation source,
  the graph; an exclusion without a reason fails the overlay check. A member
  that changes on every read and is state is reported as a limitation, not
  excluded.
