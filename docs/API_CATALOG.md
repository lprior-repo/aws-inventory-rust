# Binary AWS API catalog (M1.2d)

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
There is no JSON in the runtime catalog. The v3 root record contains schema
version 3, the 40-character source Git commit, the shared field-name bytes and
fixed-width offsets, and a list of independently encoded service records.
Each service stores a fixed-width ShapeOffsets byte string so shape lookup is
constant-time. The reader rejects older catalogs; there is no lossy compatibility path.
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

The v3 root field-name registry stores each bare member name once in
`FieldNames`; `FieldNameOffsets` is a sequence of big-endian 32-bit start/end
pairs indexed by nonzero `FieldId`. Root validation checks contiguous bounds and
UTF-8 before any service view is returned. Member lookup is O(1), borrowed, and
does not allocate or copy names into member records.

Service, shape, member, pseudo-member, and operation records retain typed
protocol metadata only when modeled. The public views expose HTTP bindings,
timestamp formats, XML namespaces, names, sparse/streaming/sensitive and
idempotency flags, checksum requirements, error metadata, and merged paginator
data. Missing optional traits remain absent; malformed known trait payloads or
references fail compilation or catalog validation. Metadata is descriptive
only and does not make an operation callable.

The compiler recognizes traits by their complete Smithy IDs; unknown namespaces
and same-suffix traits do not create catalog metadata. Service version comes from
the service shape's `version` property, signing name from
`aws.auth#sigv4.name`, and host prefix from `smithy.api#endpoint.hostPrefix`.
Service pagination defaults use `smithy.api#paginated` and are merged with
operation values while compiling. Service `errors` and operation `errors` are
merged, deduplicated in stable order, and stored only on each operation.
Checksums use `smithy.api#httpChecksumRequired`. Present known traits with
malformed payloads fail compilation with service, owning shape, and full trait
ID context.

Independent source-model count oracles (run from the Smithy checkout):

```sh
find models -type f -name '*.json' -print0 | xargs -0 jq -s '[.[] | (.shapes // {} | length)] | add'
find models -type f -name '*.json' -print0 | xargs -0 jq -r '.shapes[]? | (.members // {}) | keys[]' | sort -u | wc -l
```

Independent trait-count oracle helpers expose source counts for compiler
comparisons without reading the generated catalog:

```sh
model_trait_count() {
  find models -type f -name '*.json' -print0 |
    xargs -0 jq -s --arg id "$1" '[.[] | .shapes[]? | select((.traits // {}) | has($id))] | length'
}
wire_member_trait_count() {
  find models -type f -name '*.json' -print0 |
    xargs -0 jq -s --arg id "$1" \
      '[.[] | .shapes[]? as $shape |
        [($shape.members // {} | .[]?), $shape.member?, $shape.key?, $shape.value?][] |
        select(. != null) | select((.traits // {}) | has($id))] | length'
}
for id in aws.api#service aws.auth#sigv4 aws.protocols#awsQueryCompatible \
  smithy.api#xmlNamespace smithy.api#http smithy.api#endpoint \
  smithy.api#paginated smithy.api#httpChecksumRequired \
  smithy.api#mediaType smithy.api#sensitive smithy.api#idempotencyToken \
  smithy.api#streaming smithy.api#requestCompression smithy.api#sparse \
  smithy.api#error smithy.api#httpError aws.protocols#awsQueryError \
  smithy.api#retryable; do
  printf '%s model=%s wire-member=%s\n' "$id" \
    "$(model_trait_count "$id")" "$(wire_member_trait_count "$id")"
done
```

### `CatalogKey`

| ID | Variant | Value and meaning |
|---:|---|---|
| 1 | Format | Root integer schema version, currently 3 |
| 2 | ModelsCommit | Root source Git commit text |
| 3 | Services | Root list of independently encoded service record bytes |
| 4 | Name | Service directory name, or unqualified shape/operation name |
| 5 | Protocols | Service list of full protocol trait IDs |
| 6 | Operations | Service list of operation maps |
| 7 | Shapes | Service list of shape maps, indexed by list position |
| 8 | Resources | Service list of unqualified resource names |
| 9 | Kind | Shape integer kind code below |
| 10 | Members | Shape list of member maps, ordered by field ID |
| 11 | Element | List shape element target |
| 12 | MapKey | Map shape key target |
| 13 | MapValue | Map shape value target |
| 14 | Arn | Shape ARN template text, omitted when absent |
| 15 | Flags | Operation bitset: readonly=1, paginated=2, List/Describe=4, input-free=8, read-named=16 |
| 16 | Input | Operation input target; absent Smithy input becomes Unit |
| 17 | Output | Operation output target; absent Smithy output becomes Unit |
| 18 | ServiceShape | Full service shape ID, for example `com.amazonaws.ec2#AmazonEC2` |
| 19 | SdkId | AWS service SDK ID, when present |
| 20 | EndpointPrefix | AWS endpoint prefix, when present |
| 21 | ArnNamespace | AWS ARN namespace, when present |
| 22 | Sigv4Name | `aws.auth#sigv4.name`, when present |
| 23 | ApiVersion | Smithy service shape `version` property |
| 24 | Reserved | Preserved key assignment; not emitted |
| 25 | Reserved | Preserved key assignment; not emitted |
| 26 | Reserved | Preserved key assignment; not emitted |
| 27 | XmlNamespace | Service XML namespace map (`uri`, optional `prefix`) |
| 28 | Reserved | Service paginator defaults are merged into operations, not stored |
| 29 | HttpMethod | Operation HTTP method |
| 30 | HttpUri | Operation HTTP URI template |
| 31 | HttpSuccess | Operation HTTP success status (default 200 in the reader) |
| 32 | HostPrefix | Operation endpoint host prefix |
| 33 | PaginatedInput | Effective paginated input-token member |
| 34 | PaginatedOutput | Effective paginated output-token member |
| 35 | PaginatedResult | Effective paginated `items` member |
| 36 | PaginatedLimit | Effective maximum page size from the range constraint |
| 37 | PaginatedPageSize | Effective paginated page-size member |
| 38 | Errors | Stable-deduplicated service and operation error targets |
| 39 | HttpChecksum | Operation HTTP checksum requirement |
| 40 | JsonName | Member JSON name |
| 41 | MemberXmlName | Member XML name |
| 42 | Ec2QueryName | Member EC2 Query name |
| 43 | HttpLabel | HTTP label binding marker |
| 44 | HttpQuery | HTTP query binding name |
| 45 | HttpQueryParams | HTTP query-params binding marker |
| 46 | HttpHeader | HTTP header binding name |
| 47 | HttpPrefixHeaders | HTTP prefix-header binding name |
| 48 | HttpPayload | HTTP payload binding marker |
| 49 | HttpResponseCode | HTTP response-code binding marker |
| 50 | MemberHostLabel | Host-label binding marker |
| 51 | XmlFlattened | XML flattened marker |
| 52 | XmlAttribute | XML attribute marker |
| 53 | Reserved | Preserved key assignment; not emitted |
| 54 | TimestampFormat | Member timestamp format |
| 55 | IdempotencyToken | Idempotency-token marker |
| 56 | MediaType | Smithy media type |
| 57 | Sensitive | Sensitive marker |
| 58 | ShapeXmlName | Shape XML name |
| 59 | ShapeXmlNamespace | Shape XML namespace URI |
| 60 | ShapeTimestampFormat | Shape timestamp format |
| 61 | Reserved | Preserved key assignment; not emitted |
| 62 | Reserved | Preserved key assignment; not emitted |
| 63 | Reserved | Preserved key assignment; not emitted |
| 64 | Sparse | Sparse collection marker |
| 65 | Streaming | Streaming marker |
| 66 | HttpError | Smithy HTTP error status |
| 67 | AwsQueryError | AWS Query error code |
| 68 | ErrorRetryable | Retryable error marker |
| 69 | MemberId | Member-name registry ID |
| 70 | MemberTarget | Member target reference |
| 71 | MemberRequired | Required-member marker |
| 72 | FieldNames | Concatenated UTF-8 member-name data |
| 73 | FieldNameOffsets | Fixed-width indexed offsets into `FieldNames` |
| 74 | XmlPrefix | Service XML namespace prefix |
| 75 | XmlUri | Service XML namespace URI |
| 76 | AwsQueryStatus | AWS Query error response status |
| 77 | ErrorThrottling | Retryable throttling marker |
| 78 | MemberXmlUri | Member XML namespace URI |
| 79 | MemberXmlPrefix | Member XML namespace prefix |
| 80 | ShapeXmlPrefix | Shape XML namespace prefix |
| 81 | Reserved | Preserved key assignment; version is stored at key 23 |
| 82 | NoErrorWrapping | Protocol IDs with `noErrorWrapping` enabled |
| 83 | AwsQueryCompatible | AWS Query-compatible marker |
| 84 | Reserved | Service errors are merged into each operation |
| 85 | Reserved | Maximum page size is stored at key 36 |
| 86 | ElementWire | Optional wire metadata for a list pseudo-member |
| 87 | MapKeyWire | Optional wire metadata for a map key pseudo-member |
| 88 | MapValueWire | Optional wire metadata for a map value pseudo-member |
| 89 | ErrorKind | Modeled client/server error kind |
| 90 | RequestCompression | Canonical, ordered request `encodings` list |
| 91 | DefaultValue | Optional target-normalized canonical member default; null differs from absence |
| 92 | ClientOptional | Whether client-side member defaults are suppressed |
| 93 | S3UnwrappedXmlOutput | Whether the S3 XML output wrapper is omitted |
| 94 | ShapeOffsets | Per-shape big-endian 32-bit offsets into the encoded Shapes list |

Defaults are compiled once: blob trait strings become decoded bytes and
timestamp defaults become canonical `Time` values. `WireTraits::default_value`
borrows the value; `client_optional` is false when absent. Request encoders apply
non-null defaults to missing nested structure members, not operation top-level
inputs or client-optional members. Explicit values, including false, zero and
empty containers, take precedence. Smithy mixins are resolved in the shared
reader used by catalog compilation and Smithy stats. Raw classification readers
reject unresolved input mixins rather than omitting inherited requirements.

Request encoders resolve modeled host prefixes before building the URI. AWS JSON
uses member names rather than `jsonName`; REST JSON honors `jsonName`.
AWS Query-compatible services receive `x-amzn-query-mode` on body protocols.
Required HTTP checksums cover the transmitted body after compression. Native
Glacier requests use the service version and SHA-256/tree-hash headers rather
than a default MD5; Machine Learning `Predict` uses its `PredictEndpoint` input.

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

`Service` exposes `name`, `shape_id`, `version`, service identity/protocol metadata,
`wire` and protocol metadata. Service paginator defaults are merged into each
paginated operation at compile time; no service-default record is stored. `Shape`
exposes `name`, `kind`, `members`, collection targets, optional `arn`, modeled
error metadata, and wire traits. Missing `Members` is an empty iterator; missing
`Arn` is `None`, while malformed present values are errors. `Operation` exposes
HTTP method/URI/status, host prefix, checksum requirement, merged paginator data,
request compression algorithm, and common plus operation-local modeled errors.
`Operation::request_compression()` checks the ordered encoding list, returning
`Some(CompressionAlgorithm::Gzip)` when supported and `None` when absent.
Malformed or unsupported serialized values are typed errors.
`Member<'a>` exposes its `FieldId`, borrowed indexed name, target, requiredness,
and wire traits. Lists, maps, and structures retain pseudo-member wire metadata.
All views borrow the catalog bytes; metadata does not grant callability.

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
- Member encodings have strictly ascending, nonzero field IDs and validated
  required flags and optional wire metadata. Kinds and flags have explicit finite
  allowed ranges.
- Model files and registry input reuse the existing tool readers and their size
  limits; catalog snapshots use the existing bounded publication reader.
- Compilation reuses `smithy/fields/publish.rs`, including exclusive temporary
  file creation, stale-snapshot checking, atomic rename and owned-lock cleanup.
  A catalog uses `aws-api.catalog.tmp`; the field registry keeps its existing
  `field-ids.txt.tmp` name. `--check` and the default print mode never publish.

## Size estimate and decisions

The pinned models contain 142,767 shapes and 238,070 structure/union members.
The earlier format measured 6,951,532 bytes. The v2 catalog generated from the
same pinned source on 2026-11-14 is **10,995,359 bytes** (10.486 MiB). A fresh
`models compile --check` recompilation passed the byte comparison. The root
limit remains 16 MiB.

The coordinator's identity clarification replaces ambiguous short service names
with model directories and adds key 18 for the full service ID. The optional
member-name newtype in decision 9 was not introduced: `FieldId` is the only
public member-number type, and the existing string-name convention is retained.
The allocation-free catalog contract is preserved through fixed-capacity inline
validation storage. Generic record validation retains its dynamic stack and
deep-unknown-value compatibility. No other frozen decision is intentionally changed.

## Streaming record construction

Runtime decoders can use `inventory_core::record::RecordBuilder` instead of
constructing a `Value` tree. `new(max_bytes, max_nodes, max_depth)` requires
positive budgets, bounded by 16 MiB, 65,536 nodes and 32 container levels.
`begin_map`, `begin_list`, `scalar` and `end` form one root map; `finish` returns
the validated `Record`. Root and list events have no key; map children require
`KeyRef::Field` or `KeyRef::Name`. Container values cannot be passed to `scalar`.

Maps are sorted by the existing record key order, duplicate keys are rejected,
and lists preserve event order. Scalars retain the existing CBOR representation,
including canonical floating-point values and tagged seconds/nanoseconds times.
Open containers retain encoded fragments, not a decoded object tree. Aggregate
fragment bytes are bounded by twice the configured record-byte budget while
containers are assembled, and allocations use fallible reservation. Node and
depth limits apply during event processing. Any event error is terminal:
subsequent events and `finish` cannot recover a partial record.


## Coordinator commands

Run from the repository root. Regenerate before testing the embedded product.

```sh
SOURCE=.local/aws-models
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --write
cp catalog/aws-api.catalog /tmp/aws-api.catalog.first
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --write
cmp /tmp/aws-api.catalog.first catalog/aws-api.catalog
cargo run -p catalog-tool -- models compile --source "$SOURCE" --root . --check
wc -c catalog/aws-api.catalog
cargo fmt --all
cargo clippy -p inventory-core -p catalog-tool -p aws-inventory --all-targets -- -D warnings
cargo test -p inventory-core -p catalog-tool -p aws-inventory
scripts/check-source-size.sh
scripts/check-function-size.sh
```

`moon ci` also checks that the catalog is current. The standalone test
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

## Reviewed overlay syntax (M1.4b)

`catalog/overlay/<service>.toml` records a review, not authorization by itself.
The filename stem matches `[a-z0-9][a-z0-9-]*`. Each file is at most 1 MiB
and contains at most 4,096 operations. Directory loading visits at most 65,536
entries and accepts at most 4,096 TOML files, in filename order. Non-TOML
entries, including `must-deny.txt`, count toward traversal but are ignored.
Distinct accepted filenames cannot share a service stem.

Required root keys are integer `format = 1`, a quoted Gregorian `reviewed`
date in `YYYY-MM-DD` form (years 0001–9999), and a nonempty `reviewer` string
of at most 64 UTF-8 bytes. `operation` is an optional table keyed by exact
`[A-Za-z][A-Za-z0-9]*` operation names.

Unknown keys and wrong types are errors at every structured boundary.
Member paths contain `[A-Za-z][A-Za-z0-9_]*` segments separated by dots,
at most eight segments and 256 bytes. Lists and maps are traversed implicitly,
not indexed. Arrays reject duplicates and contain at most 64 entries unless
a tighter limit is listed below. The file budget bounds `fixed` and `bind`.

| Operation key | Accepted value | Absence |
|---|---|---|
| `class` | `enumerate`, `expand`, `deny`, `defer` | Candidate class |
| `deny_reason` | `content`, `secret`, `paid`, `mutating`, `reference-data` | Required exactly for `deny`; forbidden otherwise |
| `scope` | `region`, `account`, `home-region` | Modeled scope |
| `items` | Output list/map member path, or `$` for the output structure itself | Modeled list, otherwise unresolved |
| `page_size` | Integer 1–100000 | Largest modeled allowed size |
| `exclude_from_hash` | Item member paths | No extra exclusions |
| `skip` | Operation names, excluding self | No extra scheduling suppression |
| `quota_group` | `[a-z][a-z0-9-]*`, at most 32 bytes | Service-wide group |
| `queryable` | Item member paths | Modeled SQL columns |
| `tags` | Item member path | Modeled tags, otherwise unresolved |
| `fixed` | Input paths to string, i64, boolean, or string array | No extra fixed inputs |
| `paginator` | Reviewed pagination table | Modeled pagination evidence |
| `identity` | Reviewed identity table | Modeled identity evidence |
| `bind` | Input paths to binding tables | Modeled binding evidence |
| `batch` | Reviewed batch table | No extra batch replacement |

`deny` and `defer` entries carry no plan keys, even explicitly empty arrays
or tables. The typed deny variant contains its reason.

`paginator.kind = "token"` requires `input` and `output` paths and forbids
other keys. `"tokens"` requires one to eight `{input, output}` pairs with
distinct inputs and optionally a `truncated` output path. `"none"` forbids
every other key. Model-aware validation checks the token and flag types.

`identity` has exactly one of `arn` (item path), `template` (string), or
`key` (one to eight item paths). A template has balanced, nonempty
placeholders naming `partition`, `region`, `account`, or an item path;
at least one placeholder is required.

Each binding is exactly `{from = "Operation", path = "Item.Path"}`,
`{account = true}`, or `{region = true}`. False context flags, self-bindings
and explicit binding cycles are rejected. `batch` requires `member` (input
list path), `max` (1–10000), and `replaces` (a different operation).
Nested required keys have no defaults.

The complete syntax example is
[`complete.toml`](../crates/catalog-tool/tests/fixtures/overlay/complete.toml).
Parsed `ServiceOverlay` values expose read-only accessors; callers cannot
mutate their validated format, review metadata or operation map.

The syntax parser does not establish callability. Parsing a review never
promotes a denied operation; the model-aware gate below decides whether a
complete, safe callable plan exists.

## Model-aware overlay gate (M1.4c)

`catalog-tool overlay --source .local/aws-models --root . --check` validates
reviews against the pinned models and compares `catalog/aws-overlay.report`.
`--write` publishes the deterministic report only after validation succeeds.
Moon runs the freshness check between collection plans and the binary catalog.

The checker resolves operation names and paths in their modeled input, output
or selected-item context, including built-in Smithy types. Ordinary `items`
paths select a list or map; explicit `items = "$"` selects the output structure.
Root selection is never inferred, and `$` is not a general member path.

A `CallableOperation` has private fields and is constructed only by the checker.
It represents an operation of a reviewed service with a permitted effective
enumerate/expand class and complete pagination, item selection, identity and
required-input bindings. Fixed values, token types and page sizes must match the
models; member page-size ranges take precedence over target ranges. Activating
a nested input also requires its required siblings. Dependencies must themselves
be callable, and effective binding cycles are rejected.

Every enumerate, expand or unclassified candidate of a reviewed service must be
callable, explicitly denied or explicitly deferred. An override cannot promote
an operation denied by the global list, service policy or no-read-verb rule.
Validation reports all sorted diagnostics with their offending paths and does
not publish a partial report.

Deferred operations carry a mandatory `defer_reason` (at most 1,024 bytes);
denied overrides carry a typed `deny_reason` (`content`, `secret`, `paid`,
`mutating`, `reference-data`). Reviewed services are currently `ec2` and `sts`:
four callable operations (EC2 `DescribeVpcs`, `DescribeSubnets`,
`DescribeRegions`; STS `GetCallerIdentity`), three STS credential-minting
operations denied as `secret`, and every other candidate deferred. All other
services remain unreviewed and uncallable.

## Runtime catalog extension

`catalog-tool models compile --write` appends root field 60000 after the
overlay gate passes, then loads the result through
`inventory_core::catalog::runtime::RuntimeCatalog::from_bytes` and refuses to
publish a catalog the runtime reader rejects. The runtime never links
catalog-tool.

| Field | Content |
| ---: | --- |
| 1 | Extension version, currently 1 |
| 2 | Callable plans: service, operation, kind, scope, items, identity, bindings, page plan, relations, verified endpoint, review, purpose, protocol (fields 1–13) |
| 3 | Kind schemas: kind, SQL table, typed text columns (display name path and every reviewed relation target path), optional name path |
| 4 | Reviewed relations: source kind/path, target kind/path, relation, scope (0 account-region, 1 account, 2 home region) |
| 5 | Kind coverage for every CloudFormation type: mapped, API-only or unmapped with reason |
| 6 | Review decisions: service, operation, disposition (0 callable, 1 denied, 2 deferred, 3 unclassified), reason |

Every map is closed: the reader rejects unknown fields, out-of-range enums and
absent optional fields encoded as anything other than omission. Callable
decisions and plans form an exact bijection by `(service, operation)`, and each
referenced operation must exist once in its service. Bootstrap roles
(account identity, region discovery) are explicit `kinds.tsv` rows matched by
exact `(service, operation)` and never authorize a call on their own.
