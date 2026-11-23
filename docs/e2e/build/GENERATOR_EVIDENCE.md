# All-service overlay generator evidence

Pinned models: `46135f8468cd5cb60d983b97290c9f6661e359c6` (436 services).
Baseline: wave1 `d8909a1c`; measurements are from `/tmp/w4-gen` only.
This is generator/checker evidence, **not an emulator coverage claim**.

## Behavior and precedence

- Default selection is `*`; generation publishes a deterministic TOML for each
  pinned service, including services with no safely derivable callable reads.
- Children first use a modeled resource/list relationship and the existing
  checker’s implicit identifier bindings. Otherwise all required non-context
  inputs must match identity fields of exactly one same-service enumeration.
  Name equality and compatible model shapes are both required. Self-binding,
  ambiguous sources and context-only parent guesses are rejected.
- Required string `AccountId` and `Region` can bind collector context. Fallback
  bindings are explicit TOML and round-trip through the strict parser.
- Root/wrapped singleton outputs and modeled read identities are supported.
  Multiple possible item collections, unsupported identities and unresolved
  endpoints remain deferred rather than guessing selectors or IDs.
- A bounded, cycle-safe traversal inspects the complete reachable output graph,
  including list/map values and sensitive traits. Credential fields, prefixed
  variants such as `MasterUserPassword`, environment variables and HTTP headers
  fail closed. Pagination is still validated by the existing checker.
- Explicit denials are omitted. Specific reviewed defers are emitted as defers,
  not misleading callable proposals; they cannot become parent sources.
  Reviewed callables keep precedence. Only an explicitly migrated, source-typed
  `GenerationBlocker::RequiredBindings` permits a validated generated child to
  replace a generic missing-binding defer. TOML cannot set this marker.
- Thirty-five operations across fourteen services received that typed marker;
  thirty-two now derive valid children. Markers do not authorize a call and do
  not bypass classification, ownership, output, identity or binding checks.

## Counts

The generator formerly selected eight services and proposed 28 enumerations.
It now evaluates all 436 services and proposes **767 enumerate + 86 expand =
853 callable entries**. Of 2,025 enumeration candidates, 1,258 remain deferred;
of 3,773 expansion candidates, 3,687 remain deferred.

The effective merged catalog changes from **1,565 enumerate + 632 expand =
2,197** to **1,565 enumerate + 664 expand = 2,229**, adding 32 children without
changing any previous callable. All 19,636 operations remain accounted for:
2,229 callable, 7,980 denied and 9,427 deferred. Of those denials, 7,350 are
explicit overlay denials and 630 come from classification. Generated proposals are not
additional calls when a reviewed policy already owns the operation.

### By endpoint service class

Each cell is `enumerate / expand`. Endpoint class comes from
`catalog/aws-endpoints.report`; it is not a new service classification.

| Class | Generated before | Generated after | Merged before | Merged after |
|---|---:|---:|---:|---:|
| regional | 20 / 0 | 740 / 84 | 1471 / 577 | 1471 / 609 |
| global | 8 / 0 | 27 / 2 | 90 / 50 | 90 / 50 |
| needs-overlay | 0 / 0 | 0 / 0 | 4 / 5 | 4 / 5 |
| unsupported | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 |

### By selected wire protocol

Selection follows `runtime_catalog/plans.rs`: RPCv2 CBOR, AWS JSON 1.1,
AWS JSON 1.0, AWS Query, EC2 Query, REST JSON, then REST XML. An
`awsQueryCompatible` trait is not counted as a separate wire protocol.

| Protocol | Generated before | Generated after | Merged before | Merged after |
|---|---:|---:|---:|---:|
| awsJson1_0 | 6 / 0 | 63 / 10 | 98 / 60 | 98 / 60 |
| awsJson1_1 | 1 / 0 | 185 / 17 | 375 / 158 | 375 / 162 |
| awsQuery | 12 / 0 | 60 / 1 | 106 / 18 | 106 / 18 |
| ec2Query | 0 / 0 | 27 / 0 | 145 / 20 | 145 / 20 |
| restJson1 | 5 / 0 | 350 / 46 | 636 / 314 | 636 / 334 |
| restXml | 1 / 0 | 7 / 0 | 40 / 17 | 40 / 17 |
| rpcv2Cbor | 3 / 0 | 75 / 12 | 165 / 45 | 165 / 53 |

Machine-readable evidence:

- [`generator-callable-counts.tsv`](generator-callable-counts.tsv): all four
  stages grouped by service class and selected protocol.
- [`generator-service-counts.tsv`](generator-service-counts.tsv): all 436
  services, including zero-call services, at all four stages.
- [`generator-promotions.tsv`](generator-promotions.tsv): each of the 32
  effective changes with its advisory verdict. All are `EXPAND-OK`.

## Advisory audit cross-check and investigation

The `/tmp/svc-audit/audit-<service>.md` files are advisory, not authorization.
Operation names were checked against each pinned model, and generated callables
were validated against those models by the operation checker. Audit tables,
verdict-group sections, named later corrections and aggregate existing-callable
sets yielded 8,569 operation verdicts across all 436 services. This extraction
is not equivalent to summing the audits’ `VERDICT` lines: those summaries include
unnamed grouped operations and sometimes disagree with their own detailed rows.
No absent verdict is treated as approval.

[`generator-audit-diff.tsv`](generator-audit-diff.tsv) lists every extracted
mismatch, with generated/effective class, the precise retained/derivation reason,
audit source line, source evidence and parser authority (3 explicit table,
2 explicit narrative, 1 verdict-group context, 0 aggregate baseline membership).
There are **932 ENUMERATE-OK + 1,526 EXPAND-OK** rows not callable in generated
output. Some are already callable through their reviewed policies; generated
coverage must not be confused with effective catalog coverage.

Investigation outcomes, applied across the mismatch categories:

1. **Ownership/reference catalogs, optional discriminators and semantic scope.**
   Smithy shapes alone do not establish account ownership, complete default
   status coverage, per-item region/kind dispatch or whether a list is a public
   AWS catalog. Existing specific reviewed defers therefore remain authoritative
   in generated output as well as the merged catalog. No reason-string matching
   or operation-name allowlist was introduced.
2. **Missing identities or collection ambiguity.** The model must provide an
   ARN or modeled identifiers on the selected item. Parent-only IDs, config
   singletons without identity and flattened/nested collections are not invented.
   For example, account-access `GetApplication` has no output identity and stays
   deferred despite its generic request-binding marker.
3. **Bindings and parent availability.** Modeled relationships are preferred;
   fallback requires an exact identity-member name, compatible shape and one
   complete source. Aliases such as `analyzerName` versus `name`, multiple required
   parents, required enum/object fan-out and unavailable/denied parent sources
   are recorded as blockers, not guessed. Hand-written bindings remain valid.
4. **Sensitive output.** Reachable sensitive traits and credential names are
   rejected before publication. The cross-check exposed the prefixed
   `MasterUserPassword` case, which now has a regression alongside nested
   sensitive fields, recursive shapes, environment variables and headers.
5. **Read classification, pagination and endpoints.** Unclassified reads without
   sufficient read-only evidence, unresolved token pairs and unresolved endpoint
   inputs retain their concrete reasons. A suggestive `Get` name is not approval.
6. **Duplicate detail, telemetry, relation-only and content semantics.** Valid
   read-only shape/binding derivation does not override a reviewed scope veto.
   The first cross-check exposed 158 callable proposals against negative audits;
   preserving reviewed defers at publication removed 152 of them.

Six generated proposals still disagree with negative advisory verdicts, all
already callable in the baseline: appconfig `ListExtensions`, arc-region-switch
`ListPlansInRegion`, cleanroomsml `ListAudienceGenerationJobs`, iot
`ListJobTemplates`, route53resolver `ListFirewallDomainLists`, and
verifiedpermissions `GetPolicyStoreAlias`. Their reviewed callables were not
changed by this generator work. They remain visible in the TSV rather than
silently deleted. **No negative-audit operation was newly made callable.**

[`generator-audit-conflicts.tsv`](generator-audit-conflicts.tsv) retains seven
conflicting extracted audit rows. The iot/Glue cases arise from grouped rows or
references to parent operations; the specific first operation classification
was retained. Macie `GetMember` has contradictory dedicated verdicts, so its
later `KEEP-DEFER` correction was retained conservatively. These advisory
conflicts do not authorize generated promotions.

## Executed checks

All commands ran in `/tmp/w4-gen`; Cargo used
`CARGO_TARGET_DIR=/tmp/tgt-gen` and `-j3`.

```sh
cargo fmt -p catalog-tool
bash scripts/check-function-size.sh crates/catalog-tool/src/overlay/generate \
  crates/catalog-tool/src/overlay/parse.rs crates/catalog-tool/src/overlay/merge.rs \
  crates/catalog-tool/src/overlay/builtin.rs \
  crates/catalog-tool/src/overlay/builtin/generation_blockers.rs
cargo clippy --locked -p catalog-tool -j3 --lib -- -D warnings
cargo test --locked -p catalog-tool -j3 --lib overlay::
cargo run --locked -p catalog-tool -j3 -- overlays generate --write
cargo run --locked -p catalog-tool -j3 -- overlay --source .local/aws-models --root . --write
cargo clippy --locked -p catalog-tool -j3 --all-targets -- -D warnings
cargo test --locked -p catalog-tool -j3 --test overlay_generation_safety -- --ignored
cargo run --locked -p catalog-tool -j3 -- overlays generate --check
```

Results: formatting/function limits and both Clippy modes passed; 126 overlay
unit tests passed; generator write, merged overlay write and deterministic check
passed. All five pinned-model safety scenarios passed over all 436 services,
including deny mutation, deterministic regeneration, and preservation of all
previous callable plans. Test temporary files for the final safety gate used
`TMPDIR=/tmp/w4-gen/.local/checks/gen`.

The final focused cargo-mutants run tested **35 mutations: 33 caught, 2
unviable, zero missed**. It covers modeled/fallback bindings, exact name/shape
checks, context bindings, self-source rejection, item ambiguity, identities,
secret traversal and credential variants, reviewed-defer publication and typed
merge priority. Both unviable mutations replace `&&` with `||` in an `if let`
chain; rustc rejects them because `||` is not supported in let-chain conditions.
Mutation copies and their fixture inputs were confined to this worktree;
`--cargo-arg=--lib` kept both mutation build and test phases library-scoped.
Initial attempts failed on recursive scratch copying and disk quota/ignored
fixture handling, not on mutant outcomes; the fixture-complete isolated tree
resolved those infrastructure failures.

## Compatibility and remaining coverage boundaries

The TOML parser now accepts strict `expand` entries with `bind` values containing
`from`/`path`, `account = true`, or `region = true`. Unknown fields remain errors.
`OperationOverlay` gains a public optional source metadata field. The struct was
already `#[non_exhaustive]`, so external exhaustive construction remains forbidden.
The catalog binary schema and runtime collector API are unchanged.

Reviewed ownership/content/scope blockers are deliberately not erased. An
all-service generator is not proof that every AWS resource is collectable.
No emulator, database or graph end-to-end run was performed in this lane, and
`catalog/emulator-probe.report` was not refreshed. The coordinator must regenerate
catalog binaries/manifests and probe the resulting current callable denominator
through pinned emulator 1.0.348; the older 2,134-row probe is not evidence for the
new 2,229-operation effective catalog.

## Scoped mutation command

The fixture-complete mutation input tree was made inside `.local`, excluding
the original `.local` directory, Git metadata, targets, catalog binary and
generated catalog reports. Ignored compile-time fixtures were included.

```sh
TMPDIR=/tmp/w4-gen/.local/mutation-tmp CARGO_TARGET_DIR=/tmp/tgt-gen \
RUSTC_WRAPPER= CARGO_INCREMENTAL=0 cargo mutants --no-config \
  --dir .local/gen-mutation-tree --gitignore false -p catalog-tool \
  -f 'crates/catalog-tool/src/overlay/generate/bind_by_name.rs' \
  -f 'crates/catalog-tool/src/overlay/generate/expand.rs' \
  -f 'crates/catalog-tool/src/overlay/generate/safety.rs' \
  -f 'crates/catalog-tool/src/overlay/generate/items.rs' \
  -f 'crates/catalog-tool/src/overlay/generate/identity.rs' \
  -f 'crates/catalog-tool/src/overlay/generate/service.rs' \
  -f 'crates/catalog-tool/src/overlay/merge.rs' \
  --re 'generated_wins -> bool|bind_by_name.rs:.*(replace derive|replace != with == in derive|replace == with != in source|delete match arm|replace != with == in context)|expand.rs:.*(modeled_parent ->|eligible -> bool with false)|identity.rs:.*(arn_name -> bool with false|modeled_identifiers ->.*with None)|items.rs:.*(replace > with == in expand|delete match arm)|safety.rs:.*(replace output|replace \|\| with &&|delete ! in enqueue|credential -> bool)|service.rs:.*retain_reviewed' \
  --baseline skip --copy-vcs false \
  --output /tmp/w4-gen/.local/gen-mutation-final --jobs 1 \
  --timeout 120 --build-timeout 120 --cargo-arg=-j3 \
  --cargo-arg=--locked --cargo-arg=--lib -- overlay::
```
