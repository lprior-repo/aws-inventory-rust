# Binary AWS API catalog (M1.2c)

`catalog/aws-api.catalog` is generated from the pinned Smithy checkout and
`catalog/field-ids.txt`. The product embeds it with `include_bytes!`; it never
opens a model file, registry, or catalog file at runtime. `aws-inventory catalog
--models` prints the M1.2a model report from those embedded bytes. The existing
`aws-inventory catalog` JSON output is unchanged.

The generated catalog is committed. If bootstrapping from a zero-byte placeholder,
generate it before acceptance tests: empty input returns `CatalogError::Empty`,
including the generation command. The embedded-catalog test deliberately fails
with that command instead of skipping the test.

## Encoding and identity

Every map and record is encoded through `inventory_core::record::Record::encode`.
There is no JSON in the catalog. The root record contains format version 1, the
40-character source Git commit, and a list of byte strings. Each byte string is
an independently encoded service record. Map keys are ascending `FieldId`s.

Before reading the models, compilation requires the canonical source directory
to equal Git's repository root and rejects tracked or untracked changes beneath
`models`, including ignored files hidden by `.git/info/exclude` or other ignore
rules. In addition to Git status, `git ls-files --others` is run without exclude
flags, matching the compiler's discovery of ignored model files.
`SourceNotRepositoryRoot` identifies mismatched roots; `SourceModified` retains
the first offending model path. Shape encoding failures retain both the
model-directory service and full owning shape ID around their original typed cause.

Service identity is the model directory name, matching the classifier and
endpoint artifacts (`ec2`, `storage-gateway`, and so on). The full service shape
ID is retained separately. Duplicate directory identities or duplicate full
service shape IDs are typed compile errors. Shapes and operations retain their
unqualified names. All shapes, including operations and resources, receive a
zero-based service-local index in sorted full-shape-ID order. References are
integers, so cycles require neither recursive encoding nor recursive reading.

The field registry is different: its identity is the **bare member name**, shared
across services and structure/union shapes. It is compiler input only. Registry
`id` and `name` now accept/return `inventory_core::record::FieldId`, without a
parallel public number type. Missing members name the required `models fields
--write` repair. Enum values are not structure/union fields.

### `CatalogKey`

| ID | Variant | Value and meaning |
|---:|---|---|
| 1 | Format | Root integer schema version, currently 1 |
| 2 | ModelsCommit | Root source Git commit text |
| 3 | Services | Root list of independently encoded service record bytes |
| 4 | Name | Service directory name, or unqualified shape/operation name |
| 5 | Protocols | Service list of full protocol trait IDs |
| 6 | Operations | Service list of operation maps |
| 7 | Shapes | Service list of shape maps, indexed by list position |
| 8 | Resources | Service list of unqualified resource names |
| 9 | Kind | Shape integer kind code below |
| 10 | Members | Shape list of `[field_id, target, required]` triples, ordered by field ID |
| 11 | Element | List shape element target |
| 12 | MapKey | Map shape key target |
| 13 | MapValue | Map shape value target |
| 14 | Arn | Shape ARN template text, or null |
| 15 | Flags | Operation bitset: readonly=1, paginated=2, List/Describe=4, input-free=8, read-named=16 |
| 16 | Input | Operation input target; absent Smithy input becomes Unit |
| 17 | Output | Operation output target; absent Smithy output becomes Unit |
| 18 | ServiceShape | Full service shape ID, for example `com.amazonaws.ec2#AmazonEC2` |

Kind codes, in order starting at zero:
`blob boolean string byte short integer long float double bigInteger bigDecimal
 timestamp document unit structure union list map enum intEnum service operation
 resource`.

Nonnegative targets are local shape indexes. Negative targets are `-(kind + 1)`
for the scalar/prelude kinds 0 through 13. Smithy `Primitive*` scalar spellings
normalize to the corresponding kind. Unsupported kinds, missing targets,
unresolved references, and malformed ARN templates are typed compile errors.
A missing operation input or an input structure with no required members is
input-free, matching the existing report rather than introducing a new rule.

## Reader API

The public entry points are:

```rust
Catalog::open(bytes: &'static [u8]) -> Result<Catalog<'static>, CatalogError>
Catalog::borrow(bytes: &'a [u8]) -> Result<Catalog<'a>, CatalogError>
Catalog::models_commit(&self) -> Result<&'a str, CatalogError>
Catalog::services(&self)
    -> Result<impl Iterator<Item = Result<Service<'a>, CatalogError>>, CatalogError>
Catalog::service(&self, name: &str) -> Result<Option<Service<'a>>, CatalogError>
Catalog::counts(&self) -> Result<Counts<'a>, CatalogError>
Counts::get(&self, name: &str) -> Option<usize>
Counts::render(&self) -> Result<String, CatalogError>
```

`Service` exposes `name`, `shape_id`, `protocols`, `resources`, `operations`,
`shape_count`, `shapes`, and indexed `shape`. `Shape` exposes `name`, `kind`,
`members`, `element`, `map_key`, `map_value`, and `arn`. `Operation` exposes
`name`, `flags`, `input`, and `output`. `Member` contains a `FieldId`, a `Target`,
and requiredness. `Target` distinguishes local indexes from prelude kinds.
All fallible view accessors return `CatalogError`.

Opening and service access allocate nothing: structural validation uses a
caller-owned inline array of 32 optional frames, never a heap-backed stack.
It borrows the root `MapRef` and service `ListRef`, validates the root schema and
byte-string envelope, and does not open service payloads. First service access
validates that service record and its references using the same inline storage.
Tags do not spend container frames. Views borrow input bytes and contain no
owning collections. `counts`/`render` are cold aggregation paths: they allocate
ordered maps, a set of distinct field IDs, and the rendered string.
No throughput or latency benchmark claim is made.

### Bounds and failures

- Root and each service obey the existing record `MAX_RECORD_BYTES` (16 MiB).
  The compiler checks the service-byte sum while accumulating; the record writer
  checks the complete root including its envelope. Neither limit is raised.
- Catalog validation uses `MapRef::from_bytes_bounded`: at most 32 container
  frames, including containers inside unknown keys, tags and indefinite values.
  Exceeding that bound returns `RecordError::DepthLimit`. Generic record readers
  retain their depth-unbounded unknown-value compatibility. Catalog references
  are numeric and never recursively followed; scans have finite record lengths.
- Shape targets are checked against the current service's validated shape count;
  scalar targets against the 14 scalar/prelude codes. Field IDs use `FieldId::new`.
- Member triples must have exactly three typed entries and strictly ascending,
  nonzero field IDs. Kinds and flags have explicit finite allowed ranges.
- Model files and registry input reuse the existing tool readers and their size
  limits; catalog snapshots use the existing bounded publication reader.
- Compilation reuses `smithy/fields/publish.rs`, including exclusive temporary
  file creation, stale-snapshot checking, atomic rename and owned-lock cleanup.
  A catalog uses `aws-api.catalog.tmp`; the field registry keeps its existing
  `field-ids.txt.tmp` name. `--check` and the default print mode never publish.

## Size estimate and decisions

The pinned models contain 142,767 shapes and 238,070 structure/union members.
The design estimate is approximately **7–9 MiB**, not a generated measurement:
unqualified shape names contribute about 3 MiB instead of roughly 6.7 MiB for
full IDs, member triples about 2.1–2.4 MiB, shape maps/kinds/empty lists about
1.5–2 MiB, and operation references/protocols/resources less than 1 MiB. Small
CBOR integer indexes avoid repeating target strings. The coordinator measured
**6,951,532 bytes**, compiled twice byte-identically. That observed size is below
the estimate and the unchanged 16 MiB root limit. Review repairs do not change
the encoding, so regeneration from the same inputs should preserve those bytes.

The coordinator's identity clarification replaces ambiguous short service names
with model directories and adds key 18 for the full service ID. The optional
member-name newtype in decision 9 was not introduced: `FieldId` is the only
public member-number type, and the existing string-name convention is retained.
The allocation-free catalog contract is preserved through fixed-capacity inline
validation storage. Generic record validation retains its dynamic stack and
deep-unknown-value compatibility. No other frozen decision is intentionally changed.

## Coordinator commands

Run from `/home/lewis/src/aws-inventory-worktrees/m1.2c`. The worker does not run
Cargo or a formatter. Generate before testing the embedded product.

```sh
SOURCE=/home/lewis/src/aws-inventory-rust/.local/aws-models
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --write
cp catalog/aws-api.catalog /tmp/aws-api.catalog.m1.2c.first
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --write
cmp /tmp/aws-api.catalog.m1.2c.first catalog/aws-api.catalog
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --check
wc -c catalog/aws-api.catalog
cargo fmt --all
cargo clippy -p inventory-core -p catalog-tool -p aws-inventory --all-targets -- -D warnings
cargo test -p inventory-core -p catalog-tool -p aws-inventory
scripts/check-source-size.sh
scripts/check-function-size.sh
```

`scripts/verify.sh` also checks that the catalog is current. The standalone test
commands above include the core corruption/property tests, compiler fixtures,
real CLI publication checks, migrated registry tests, and embedded product golden
report test. Independent code and test review remain coordinator acceptance gates.

## Corruption and provenance regression coverage

The small catalog keeps the exhaustive every-byte XOR-mask sweep (all 255
nonzero masks) and every proper truncation. A richer fixture has three services,
a recursive structure, list, map, union, resource ARN, operation input/output,
and prelude references. It sweeps every byte using masks `1` and `128`, plus every
proper truncation. Both fixtures and the fixed-seed arbitrary-byte property use
the same complete public-accessor traversal. Every returned local target is
independently asserted below its service's shape count, and operation flags are
asserted within 31. The byte-vector property disables failure persistence.

Exact-error tests cover exclusive target bounds, invalid negative prelude codes,
unknown flags, and depth 32 versus 33 under an unknown key. Compiler tests assert
the ordered shape name/kind sequence, all modeled scalar/enum kinds, operation
references and Unit defaults. Real CLI fixtures initialize and commit their own
Git repository with explicit identity and no global Git configuration; they
exercise clean provenance, nested-directory rejection, modified/untracked models
(including ignored untracked inputs), same-length byte drift and existing empty
catalog files. Inline-stack tests cover caller-storage identity, the inclusive
32-frame bound, LIFO ordering, mutable top/pop reuse and generic dynamic storage.
