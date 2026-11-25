# Changes

## Unreleased — Direction: endpoint wire engine

- PostgreSQL COPY can be enabled at a positive row threshold; the opt-in direct
  mode copies new-resource pages into target tables without staging and rolls
  back to a page savepoint before staging fallback on a unique violation. The
  direct mode requires `INVENTORY_POSTGRES_COPY_MIN_ROWS` and
  `INVENTORY_POSTGRES_COPY_DIRECT=1`; the measurement is not yet recorded.
- Sweeps no longer store a resource's own identifier as a reference to itself.
  The next sweep removes those edges; deliberate self-relationships, such as an
  IAM role that trusts itself, are unchanged.
- CloudWatch sweeps collect metric, composite, and log alarms by ARN. They omit
  evaluated state explanations and log alarm scheduled query configuration from
  stored records.
- Organizations sweeps no longer store account email addresses: the management
  account email on `DescribeOrganization`, each account email from
  `DescribeAccount`, `ListAccounts` and `ListDelegatedAdministrators`, and both
  participant emails on responsibility transfers are omitted. Databases written
  by earlier versions keep the addresses in their stored records until the
  resource is swept again, which replaces the record without them. Records for
  resources that are no longer swept are not rewritten.
- Sweeps of ACM, ACM Private CA, AWS B2B Data Interchange, Amazon Chime SDK
  Voice, Amazon Connect, Amazon Connect Outbound Campaigns (v1 and v2),
  Detective, Macie, AWS User Notifications Contacts, Oracle Database@AWS, AWS
  End User Messaging SMS (v2), Security Hub, SES (v1 and v2), Systems Manager
  Incident Manager Contacts, and WorkMail omit these contact fields from stored
  records: certificate validation emails and RFC 822 names, member,
  invitation, user, group and resource emails, contact and customer-contact
  addresses, sender addresses of custom verification templates and
  deliverability reports, and the phone number fields of phone number,
  partnership, profile and campaign records. Other fields that can hold a
  contact value are still stored: SNS origination numbers and SES certificate
  sender addresses (they identify the resource), Chime SIP rule trigger values
  (a phone number or a hostname), Chime phone number IDs, and the Google Cloud
  service account email on Config connectors. Databases written
  by earlier versions keep the omitted values in their stored records until the
  resource is swept again, which replaces the record without them. Records for
  resources that are no longer swept are not rewritten.
- EC2 records now carry the tags returned on each item by the operations that
  list them, including VPCs, subnets, network interfaces, internet and NAT
  gateways, security groups, volumes, snapshots, images, and transit gateway
  resources. Four operations are deliberately not covered because their `Tags`
  member holds another resource's tags, such as `DescribeVpcClassicLink`, which
  returns the VPC's tags. Operations that are deferred or denied, such as
  `DescribeInstances`, store no records, so they carry no tags.
- Seed bindings resolve outputs by step ID after rejected or skipped steps. A
  binding to an unavailable step names that step, and the dependent request is
  not sent.
- Sweeps treat confirmed missing singletons, inactive features, and ineligible
  account scopes as empty without hiding IAM denials. Absence requires an HTTP
  client error other than 429, and any account or Region named by a message must
  match the request. Conflicting values in the selected error header are rejected.
  A conflicting readable body code also prevents a scope from counting as empty.
  The emulator probe now runs modeled endpoint discovery before dependent calls.
  `AmbiguousService`, observed only from the emulator, fails immediately.
- The harness graph installer builds `redis-server` and `redis-cli` one after
  the other under one shared deadline. Building both in one parallel `make`
  could fail intermittently.
- `sweep` and `serve` accept `--idle-timeout-ms` and `--pool-idle-per-host`
  (or `INVENTORY_SWEEP_IDLE_TIMEOUT_MS` and `INVENTORY_SWEEP_POOL_IDLE_PER_HOST`).
  Idle timeout must be 1–10,000 ms; the per-endpoint cap must be between 1 and
  the effective connection limit (bounded by concurrency and open files).
  Pool defaults remain one second and up to eight idle sockets per endpoint.
  Benchmark output includes distinct hosts and summed connect-attempt time
  (including permit waits and failed attempts) relative to cold sweep wall time.
- PostgreSQL sweeps can opt into binary COPY through temporary staging tables
  with `INVENTORY_POSTGRES_COPY_MIN_ROWS`. The default still uses `UNNEST`.
- When the mutation harness cannot launch its child program, it writes a final
  `supervisor_error` receipt with exit code 124 and the log digest. The receipt
  used to stay at `running`.
- Native seed and tag graph tests identify the missing emulator binary or
  FalkorDB endpoint. The native rebuild boundary test uses the harness's own
  inventory fixture, with SQLite coverage for the retained resource IDs.
- Native graph tests use checked frame lengths and explicit row shapes, and
  share a single tag commit fixture.
- The sweep cancellation test pauses the second response until cancellation
  settles and verifies that resume requests only the committed continuation.
- Inventory requests use compatible page limits for EC2 Image Builder lists,
  MediaPackage origin endpoints, and Chime SDK Voice phone numbers. Pagination
  still follows each operation's continuation token.
- Seed recipes for 23 services create more resources before the emulator
  probe. The regenerated reports record 390 accepted seed steps with none
  rejected and 335 list operations returning items, out of 2,237 probed.
  `securityagent ListSecurityRequirementPacks` now reads empty, and seeded ECS
  task definitions join the kinds with no Cloud Control target.
- `cmdb-bench` measures cold and warm sweeps in owned stacks, compares benchmark
  results, and times catalog opens. The proxy profile adds latency and request
  pacing; reports retain run provenance and distinguish incomplete runs.
- Catalog compilation retains modeled references and materializes resolvable
  `REFERENCES` edges with model provenance. Reviewed rules take precedence.
  Regenerate catalogs before use; relation provenance is now required.
- The SQLite and PostgreSQL store conformance suites now share batch claim
  ordering, repeated observation, deletion and projection outbox scenarios,
  plus manifest handling, scope reporting, reference retraction and outcome
  detail. PostgreSQL lists unobserved scopes.
- `inventory-harness verify cloud-control` compares stored identities with
  Cloud Control listings without writing to the inventory. The command reports
  missing, extra, explained and non-listable identities; ambiguous normalized
  identities fail instead of choosing a resource.
- Cloud Control listings with no identities are reported as not comparable when
  stored resources exist. Seeded kinds fail the gate; an empty listing with no
  stored identities remains a match.
- PostgreSQL test database creation and removal use a 60-second server deadline
  and a 65-second transport deadline after connecting. Prune bounds the candidate
  query separately from each drop.
- Local PostgreSQL image upgraded from 16-bookworm to 18-trixie (18.7),
  pinned by digest. The volume name changed to `inventory-pgdata-18` and
  its mount path to `/var/lib/postgresql`; the old `inventory-pgdata`
  volume is left in place and data is not migrated automatically.
- Sweep moves selected row buffers into observations after deriving references,
  tags and child work, shares up to 65,536 modeled endpoints per run, and encodes
  each page request once while still signing every retry. Observations share
  immutable hash-exclusion paths instead of copying them per row; endpoint-cache
  hits borrow region keys. Record bytes, exclusion values and content hashes
  remain unchanged.
- `inventory-harness seed draft --service <name> [--catalog <file>] [--output <file>]`
  drafts a seed file from the create operations in the compiled catalog. A step
  that cannot be drafted (sensitive required input, unresolved or ambiguous
  reference, unsupported pattern, unrepresentable integer range, home-region
  scope, traversal limit) is written commented out under `# Disabled: <reason>`
  and does not stop the other steps. Output cannot target
  `crates/harness/seeds`, including through a symlink.
- Catalog version 4 retains modeled integer bounds for seed drafting.
  Regenerate catalogs before use; version 3 catalogs are rejected.
- SQL refuses SQL-string executor entry points, timeout overrides, notifications
  and logical messages. Its transport watchdog terminates the timed-out reader
  backend, and scalar table cells are charged before heap formatting.
  Sweep integration checks repeated reads against a real uncommitted page
  through a latency-injecting proxy.
- SQL rendering charges escaped text before allocating it; harmless
  advisory-prefixed columns remain queryable, and reader fixtures clean up
  after assertion failures.
- `aws-inventory sql --database <reader-URL> "<statement>"` runs bounded,
  typed PostgreSQL queries in a dedicated read-only transaction. Reader
  privileges and standard-conforming strings are verified on connection.
  Exact quoted and unquoted advisory/backend-control function names and
  Unicode-escaped quoted identifiers are refused. Cleanup is time-bounded,
  failed queries retain their original errors, and results use aligned tables.
  `--limit` lowers the row budget. The SQLite `query` command is unchanged and
  shares the statement lexer.
- The whole-account hierarchy layout accepts borrowed interned arrays and emits
  deterministic resource positions and contiguous account/region/service/kind
  child ranges, independently of Makepad and database I/O.
- Catalog overlay operations gain an `emit` key with values `resources`
  (default) and `work`. An `emit = "work"` enumeration creates child work
  instead of resource rows. The compiler infers `work` for enumerations with
  no explicit setting, exactly one callable expansion returning a structure,
  and scalar or identifier-only items. The runtime plan extension version
  moves from 2 to 3 (plan field 25 carries the flag); catalogs compiled
  before this change are rejected and must be regenerated. The catalog
  version stays 4.
- Work-only pages record listed members in the same transaction that enqueues
  describe work. SQLite and PostgreSQL retire a describe scope after two
  complete enumerations omit its member. A still-listed member from another
  enumeration keeps the describe scope alive.
- Concurrent sweeps into SQLite no longer fail with a connection pool timeout.
  The store has one writer connection; writers now queue for it in arrival
  order before the five-second checkout deadline starts, and the scheduler
  keeps polling in-flight pages while it claims more work, so a page that
  holds the writer can finish. Any `--concurrency` in the allowed range is
  safe, including the default of 16.
- EKS clusters are stored under their full ARN instead of their name. Existing
  databases are not rekeyed in place; collect into a fresh database with the
  regenerated catalog.
- `catalog-tool relations remap --write` generates exact API-member relation
  rows and a per-type mapped/unmapped report from the reviewed CloudFormation
  rules; live relations rise from 468 to 528. Reviewed typed relation sources
  take precedence over generated rows. List-crossing mappings are written as
  `relation-held` rows that the kind loader excludes for now. Generic
  references are inferred only when the terminal API member name matches
  exactly one enumerated target kind's identifier; missing and ambiguous
  candidates are reported with a reason.
- The native Organizations seed test reads the emulator's existing
  organization instead of trying to create one.
- `catalog-tool iam-policy` derives one read-only IAM action per callable
  runtime plan and writes the policy, the cross-account role template and a
  gap report. Must-deny findings and broad aliases are reported; write-named
  exceptions that are not confirmed are refused. Sweep keeps a receipt for
  every attempt, including rejected and retried calls.
- Graph traversal retains breadth-first paths under edge budgets and
  distinguishes a depth cutoff from a budget-truncated path.
- FalkorDB projection rejects foreign edges before emission and writes at
  most 1,000 rows per constant-depth `UNWIND` statement, preserving relation
  labels. Connection loss uses the graph adapter deadline instead of Redis's
  500 ms response timeout. Cypher ownership is checked by a standalone script.
- PostgreSQL pages batch payload reads by kind and cache table names per snapshot;
  tag pairs retain their key ordering, malformed freshness rows are typed errors,
  and URL-free diagnostics distinguish authentication, database and schema failures.
- Graph expansion reports terminal failures and revision-scoped transport status.
  Offline diagnostics survive following SQL snapshots and clear after a successful
  graph query; resource-query failures remain scoped to their selected resource.
- `cmdb-view` and `cmdb-ui --database` now read PostgreSQL inventory URLs as
  well as SQLite paths. PostgreSQL reads use four read-only connections,
  pinned repeatable-read snapshots, full-identity cursors, escaped literal
  searches with ASCII-only case folding and bounded table-row projections.
  Configuration diagnostics redact PostgreSQL URLs; `Database::open` now accepts `&str`.
- PostgreSQL URL dispatch and diagnostic redaction share case-insensitive scheme
  handling, including URLs embedded in malformed CLI arguments. Both readers
  retain byte-budget continuations and reject oversized first rows or missing
  stored records consistently; SQLite record lengths use complete resource identities.
- Marketplace Agreement discovery now sends the pinned model's documented-required
  `AgreementType=PurchaseAgreement` filter on initial and subsequent search pages.
  Existing catalog/database pairs must be regenerated together for the new plan.
- EMR Containers seeds use distinct EKS role, cluster and idempotency-token
  identities, so replaying all service recipes in one account does not reject
  their prerequisites as duplicates of the standalone EKS recipe.
- Sweep preserves account-global Lightsail domain ARNs while retaining strict
  account and regional-resource identity checks.
- Timestream InfluxDB parameter groups now use their required, service-generated
  `id` as the canonical resource ID, replacing ARN IDs. This includes service
  defaults whose ARN is empty without discarding customer groups on the same page.
  Existing ARN-keyed parameter-group rows are replaced after complete observation.
- Required child requests with absent or empty source text are skipped per row
  and counted in the invocation's `child_requests_skipped` metric. Nonempty
  values are preserved verbatim; validation and transport failures remain errors.
- `ProgressEvent::Finished.summary` is now boxed to keep bounded progress-channel
  event slots compact as summary metrics grow. No listener means no event allocation.

- Reviewed per-operation response-member omissions now drop sensitive modeled
  values during decoding, before records, projections or persistence. This
  enables IAM instance-profile discovery/detail/role expansion, Lambda function
  discovery/configuration/aliases, and SNS platform-application metadata. These
  plans omit embedded profile-role trust documents, function environment/commands
  and platform credentials. Pre-existing IAM `ListRoles`/`GetRole` trust-summary
  projections are unchanged and are not covered by that omission guarantee.
  Validation rejects ambiguous terminal names and union variants.
  See `docs/e2e/SANITIZED_RESPONSE_MEMBERS.md` for bounds and compatibility.
- E2E count verification now rejects an entirely catalog-deferred seed oracle.
  Every explicit E2E service selector must have a seed recipe; bootstrap calls
  remain implicit, so the baseline script no longer selects recipe-less STS.
- Sweep accepts `INVENTORY_SWEEP_DATABASE` as an alternative to `--database`;
  the harness uses it to keep database passwords off collector argv. Generated
  profile/config files are exclusively created with mode 0600, issued credential
  reads are capped at 64 KiB, and signal-listener failures still finalize owned
  children and the database.
- Selection/parser regression scenarios cover bootstrap retention, sibling
  removal, empty-selection identity, duplicate plan names, invalid arguments,
  and unknown selectors rejected before database creation. E2E SQL snapshots
  self-validate seeded relationships and resource pagination bounds; crash and
  cancellation subscenarios use independent graph namespaces.

- Seed directories admit up to 512 entries while retaining bounded reads, sorted
  TOML selection and empty-directory rejection. Offline recipe checks report all
  invalid files together rather than stopping at the first model/path error.
- Seed TOML strings targeting modeled blobs decode strict canonical padded base64
  during preflight, within a 1,024-byte encoded blob bound; ordinary strings
  remain text and malformed blob literals fail before network calls.
- Fresh-emulator seed replays use catalog-enumerated Kinesis streams and Lightsail
  domains, omit unimplemented Fraud Detector prerequisites, and remove the
  unverified legacy SMS/Voice recipe.
- Native first-page probes accept repeatable `--service` filters for isolated
  service seed verification; omitted filters retain the full-catalog report.
- Native first-page probes account for every callable purpose, including bootstrap
  and tag plans, with exactly one row per service/operation. STS identity is reused
  for account-bound root inputs. Known absent features and singleton configurations
  produce empty-with-reason evidence and complete empty runtime scopes; permission
  and generic missing-resource errors remain visible failures.
- AWS error decoding honors explicit JSON error envelopes and error headers even
  for RPC/CBOR requests, preserving emulator protocol gaps instead of generic HTTP
  errors. Explicit unknown-operation responses are unsupported, not decoder failures.
- Batch service jobs, CloudFormation type versions, Kinesis Video mapped resources
  and Redshift reserved-node exchange status now use their actual parent identifiers.
  CloudFormation registry discovery explicitly selects account-private extensions.
  IAM login-profile `NoSuchEntity` is narrowly treated as absent; permission and
  service failures remain visible. Native failure triage reconciles all 2,197 keys.
- Overlay generation now covers all 436 pinned services, including validated
  child reads derived from modeled identifiers or a unique compatible
  same-service identity source. Reachable credential outputs, owner-sensitive
  inputs and ambiguous collections fail closed. Specific reviewed defers and
  denials remain authoritative; only source-typed request-binding blockers may
  be replaced by validated generated children. Per-class/protocol counts,
  advisory audit differences and mutation evidence are documented in
  `docs/e2e/build/GENERATOR_EVIDENCE.md`.
- Generator integration preserves reviewed safety decisions and shared
  MediaConnect gateway-instance kinds. Singleton detail identity uses a required,
  matching modeled ARN rather than treating related agent/security-group lists
  as the inventoried resource.

- Sweep accepts repeatable `--service service[:Operation]` selectors. Bootstrap
  calls and required ancestors remain enabled; descendants require an explicit
  selector (a bare service selects all its plans). Omitted selectors keep the
  full catalog, and unknown selectors exit with usage status 2.
- `inventory-harness e2e run` owns an isolated emulator, FalkorDB and fresh
  PostgreSQL database, seeds recipes, runs the shipped collector twice and
  checks seeded identities/counts and zero repeat writes. Timed stages and
  safe numeric collector outcomes and failed operation names remain visible.
  Completed product exits 0/2 require no partial scopes, pending work or graph
  backlog; unsupported/failed emulator outcomes do not bypass count/repeat gates.
  `--keep` retains the database until explicitly pruned and waits for Ctrl-C
  before stopping children. The baseline script now uses this command.
- The shared collector runner accepts a caller-supplied deadline for long-running
  benchmarks while ordinary E2E collection retains its 300-second deadline.
  Both paths use the same outcome classifier and kill/reap timed-out children.
- E2E count verification captures a pre-seed inventory and compares full
  account/region/kind/id additions, not default-resource exceptions or count
  subtraction. Oracle kinds with no compiled callable resource emission are
  explicitly catalog-deferred with reviewed reasons; callable kinds still
  require exact identities, with up to five differing identities on failure.
- E2E collection includes resource-producing cross-service seed prerequisites,
  derived from selected recipes, so Lambda signing profiles are not seeded
  without collecting Signer. Existing qualified service scopes remain explicit.


- The VPC/subnet baseline explicitly selects its two paginated enumerations
  from the full catalog instead of sweeping unrelated services and descendants.
  It retains the 60-second CLI deadline, all inventory/graph/resume assertions,
  and checks the exact two-scope, four-page, six-request budget. Optional
  `INVENTORY_E2E_TRACE=1` records per-invocation timing and bounded API-attempt
  diagnostics, including when the child process exceeds its deadline.
  The deliberate offline-graph invocation requires exit 5. Both stores verify
  SIGKILL/live-lease refusal without checkpoint corruption and Ctrl-C followed
  by immediate resume to the uninterrupted digest; the long hard-crash lease
  recovery limitation is recorded explicitly.

- The harness `probe` command reports every catalog callable in stable order,
  skips parent-bound children, and calls only the first page through the sweep's
  existing encoder, signer, status classifier and decoder. Reports retain bounded
  failure details, decoder output shapes, and outcome/protocol totals. The public
  sweep `call_once` API returns item counts and the actual HTTP response version
  without persistence. Signing credentials now clone their shared identity handle.

- Catalog kind/naming generation and regression tests follow rustfmt without
  changing mapping precedence or catalog wire formats. Formatting changes the
  embedded generator-source fingerprint, so compiled catalogs and their manifest
  hashes must be regenerated. Small helpers retain source/function size limits.

- The harness runs ordered TOML seed directories against loopback HTTPS, retaining
  rejected operations and skipped dependencies in a sorted matrix alongside a
  sorted resource/relationship oracle. Mutation authorization remains harness-only.
  Thin-slice fixtures now cover IAM, SNS/SQS, DynamoDB, KMS, Lambda signing
  configuration, S3 and CloudWatch; all 44 matrix rows are accepted by pinned
  emulator 1.0.348. Seed files support explicit request-derived identities and
  per-step services for observed cross-service references. SNS subscription
  and Lambda execution-role relationship metadata remains gated on A1.x.
  Bulk seed execution accepts `--concurrency 1..=64` (default one), retaining
  dependency barriers and deterministic oracle ordering without detached tasks.
  Five benchmark recipes honor observed service quotas and create 10,000
  resources at concurrency 64: two fresh native emulator runs completed in
  43.59 and 43.73 seconds with matching kind counts and acceptance matrices.
  IAM uses roles, policies and instance profiles to stay below per-kind limits.
  Catalog-plan classification and both product/sweep dependency tests are
  included. The sweep encoder accepts only a callable catalog plan, endpoint,
  record and input fields; protocol-spanning tests verify catalog-derived
  methods. Service rejection messages are decoded and bounded.
  Native seed integration fixtures use the widened 16-bit step identifiers in
  recipe helpers and independent oracle assertions.

- Sweep accepts PostgreSQL URLs, creates missing inventory schemas and refuses
  missing-schema resume. Collection and graph delivery share the inventory port;
  file paths retain the transitional SQLite backend. Database URLs are redacted
  from CLI argument debug output. Baseline scenarios default to isolated native
  PostgreSQL databases and retain an explicit `INVENTORY_E2E_STORE=file` mode.

- PostgreSQL now implements the complete inventory and projection ports, including
  transaction-owned delivery fencing, bounded revision-exact outbox delivery,
  repeatable-read snapshots, and paged resources, edges and tags.

- Projection delivery now uses backend-neutral inventory, snapshot, target-identity
  and delivery-ownership ports. SQLite retains its persistent sidecar generation
  and immediate cross-process contention behavior.

- PostgreSQL resolves reviewed typed reference unions in bounded batches, retracts
  stale or ambiguous matches without discarding independent proofs, and retires
  missing resources only after two complete authoritative enumerations.
  Parent retirement preserves independently owned descendants and reaggregates
  their remaining facets. Final pages, explicit outcomes and repeated tokens all
  drain completion atomically under the current lease fence.
  Concurrency debt: reviewed-catalog writes acquire advisory lock `(14, 2)`
  before sweep and kind locks, serializing reviewed-relationship writers.
  Revisit a dependency-closure protocol at B6/C6.7 after benchmarking.

- PostgreSQL schema initialization creates the inventory namespace, account-led
  resource keys and native scalar kind columns in batches of 200 tables.
  SQL timestamps retain microseconds; CBOR retains the original precision.
  The reviewed catalog is persisted with its digest so reopening retains
  current catalog authority metadata rather than restoring an incomplete registry.

- `PgStore` creates and reopens schema14 through a bounded eight-connection
  pool, checks stored catalog integrity and hides the entire URL in diagnostics.
  Connection failures never propagate raw connection details.

- PostgreSQL work seeding uses 5,000-scope array batches and a transaction
  advisory lock. Claims skip locked rows, preserve staleness ordering and reject
  exhausted generations. Resume checks manifests and locks the queue before
  fencing expired leases.

- PostgreSQL page commits batch resource, scalar, facet, tag and child-work
  writes in one lease-fenced transaction. Unchanged content is not rewritten.
  Native provenance and candidate indexes retain checked reference authority.
  Timestamp arrays use binary intervals to avoid floating-point rounding,
  and projection work collapses to a bounded full-rebuild marker on overflow.

- The sweep runtime now lives in `inventory-sweep`; CLI collection and cadence
  scheduling call its public API. Credential resolution uses `CredentialSource`
  without an AWS SDK dependency, with the profile adapter enabled by
  `inventory-aws/sweep-credentials`. Bounded IAM trust summarization is shared
  from `inventory-core` under its existing `legacy` feature.

- Sweep callers now own cancellation and optional bounded progress delivery.
  Slow listeners never block collection; full-channel drops are counted in the
  returned summary. Cancellation durably releases owned leases without changing
  scope outcomes or continuation tokens, so resume completes the same inventory.
  CLI signal handling stays outside the library and spawns no signal task.

- Sweep summaries retain their original first output line and add a `metrics`
  line with stage times, executor and transport counters, per-run decode and
  commit timing, successful claims/pages, HTTP/2 requests, and bounded `/proc`
  CPU/peak-memory readings. Missing process metrics are explicitly unavailable.

- Collection now accepts any `InventoryStore` through monomorphized APIs,
  including discovery, resume, lease settlement and page persistence. Concrete
  SQLite construction remains in CLI wiring and test fixtures.

- `InventoryStore` now exposes bounded sweep discovery, typed manifest creation,
  and generation-fenced lease release for immediate cancellation/resume.
  Queue depth is opt-in through `claim_with_depth`; ordinary claims retain their
  original UPDATE without a COUNT or RETURNING, and page commits are unchanged.

- Added the unused `inventory-graph` crate for E3.1: a backend-neutral,
  RPIT-plus-Send graph port, a bounded FalkorDB adapter and centralized
  openCypher builders. Existing projection and UI callers remain unchanged.
  `docs/GRAPH_SUBSET.md` records identity, incident-edge replacement,
  cursor ordering, limits and live conformance commands.

- Runtime catalog binding decoders live in `decode/plans/bindings.rs`, with
  separate source decoders and unchanged wire tags, limits and rejection order.
  The parent plan decoder is now below the 300-line source limit.

- Relationship metadata decoding and its existing namespace/mode tests move
  together under `decode/metadata/relations/`. Target-kind, source and scope
  decoding remain ordered as before; the parent metadata module is below 300 lines.

- Graph traversal comments move to the track E read contract. Frontier expansion
  and edge-budget calculation are separate helpers; traversal direction,
  timeout, limits and edge admission order are unchanged.
- Harness seed scenarios now load strict, bounded per-service TOML. Symbolic
  step references bind the same repeat index; nested lists, modeled structures
  and named maps reach request records without losing output bindings.
  The embedded EC2 file preserves all 12 legacy steps and six relationships,
  including input/check ordering. Expansion is limited to 4,096 steps, 65,536
  input nodes, depth eight and a conservative 64 MiB string budget.

- `cmdb-ui` gains property tests for `DagLayout::compute` (node accounting,
  non-overlap, bounds, determinism, edge direction with strictly increasing
  layers on forward edges), a criterion layout benchmark
  (`cargo bench -p cmdb-ui --bench layout`) over chain, fan-out, layered and
  cyclic graphs at 100/500/2000 nodes, an end-to-end README walkthrough, and
  a library target exposing `dag` and `data` for benchmarks. Bin test code is
  clippy-clean under `-D warnings`. The frame-time budget is not measured by
  the layout bench.

- Relationship scopes gain `arn_region`: a global source (Route 53
  key-signing key) naming a regional ARN (its us-east-1 KMS key) takes the
  target region from the ARN itself. Only same-account regional `arn:` values
  are authority; bare ids, global targets and foreign accounts fail closed.
  The catalog is regenerated accordingly.

- D–F child-collection scenarios assert native REST, AWS JSON, and RPC v2 CBOR
  requests against the pinned service models, including operation-specific
  pagination inputs and parent identifiers. FMS response fixtures use CBOR
  through the native decoder before durable node and relationship checks.

- Core loader boundary fixtures carry modeled scalar-list items, eight nested
  structure levels, full cursor tuples, matching schemas and callable decisions,
  and hashes of the actual synthetic catalog and reviewed runtime inputs.
  Eight identity keys and all 64 template parts survive the public loader.

- Reviewed identity recipes support bounded integer operands, collision-safe
  required string lists, and structural selected-value digests without
  converting typed values or copying complete selected trees. ARN operands
  retain trusted request account, partition, and region validation.
  Mandatory selected/request scalar identity components reject empty text
  before framing; typed zero and explicitly empty namespace lists remain valid.

- The ignored `sweep_e2e` suite is self-contained: it spawns the pinned
  emulator (with an isolated `HOME`) and a native `redis-server` with the
  FalkorDB module on a free loopback port, builds an isolated profile from the
  credentials the emulator issues, and tears both down on drop. The
  `AWS_INVENTORY_E2E_*` variables are gone. The FalkorDB installer is now Rust
  (`inventory-harness install-graph`); `install_graph.py` and its test are
  deleted.

- End-to-end collection path works against the live emulator:
  `aws-inventory sweep` loads the compiled runtime catalog, resolves profile
  credentials, bootstraps account identity (STS `GetCallerIdentity`) and
  enabled regions (EC2 `DescribeRegions`), enumerates the reviewed EC2 VPC and
  subnet plans through `aws-wire`, commits pages atomically into SQLite schema
  14 with fenced leases, resumable checkpoints and SQLite-indexed repeated-token
  detection, resolves reviewed `IN_VPC` relationships, and projects resources
  and edges into FalkorDB through a revision-safe outbox. Exit codes: 0
  complete, 2 degraded scopes, 1 fatal. The native Makepad viewer reads the same
  database and graph read-only.
- The ignored `sweep_e2e` suite passes against the emulator with real TLS: typed
  seeding, persisted counts and relationships, offline-then-online graph drain,
  unchanged rerun, interrupted-and-resumed sweep through a checkpoint barrier
  proxy, repeated partial runs, and a separate signed TLS 403 peer proving
  denied scopes write nothing and delete nothing.
- Catalog format 3: each service carries `ShapeOffsets` (key 94), making shape
  lookup O(1). A 16-resource debug sweep went from 31.5 s to 12.8 s (64
  resources). `models compile` builds a one-pass service index (over 10 minutes
  to 28 s) and refuses to publish a catalog the runtime reader rejects. Runtime
  extension field 60000 is documented in `docs/API_CATALOG.md`.
- The inventory database stores the catalog digest and allowed operations
  rather than the 11 MB catalog blob, which exceeded the deliberate 8 MiB
  `SQLITE_MAX_LENGTH`.
- Query-protocol XML namespace comparison ignores a trailing `/`; real EC2
  responses use `http://ec2.amazonaws.com/doc/2016-11-15/` while the model omits
  the slash.
- Public `aws_wire::response::decode` handles successful responses and modeled
  errors for all seven AWS wire protocols, writing directly into bounded
  canonical records. Published client coverage is 351 successful responses and
  57 modeled errors; 15 server-only responses are explicitly excluded.
  Shared decoding preserves bindings, defaults, sparse values and timestamps.
  XML includes modeled namespaces, flattened collections and S3 unwrapped
  output metadata; malformed inputs and resource limits have regressions.
- Request encoding covers all seven AWS wire protocols. Published REST tests
  retain exact URI, resolved-host, header and body expectations, including S3
  addressing settings. XML comparison follows modeled structures, maps and
  ordered lists rather than inferring collection semantics from element counts.
  REST and Query reject protocol selections absent from service metadata.
- Catalog defaults use checked numeric conversions and strict base64 decoding.
  Embedded model reports retain `awsQueryCompatible` counts from the dedicated
  wire flag without treating the compatibility trait as a selectable protocol.
- `inventory_core::record::RecordBuilder` accepts bounded map/list/scalar events
  without an intermediate `Value` tree. It emits the existing canonical CBOR
  representation and rejects invalid state transitions, duplicate keys and
  byte/node/depth overruns; event errors permanently invalidate the builder.
- Moon 2.2.4 CI cutover: `moon ci` replaces the old verification script.
  All 17 CI tasks pass locally, including generated-data freshness, binary-only
  core coverage, workspace checks/tests, warnings-denied source Clippy and the
  Makepad build. GitHub runs the gate on pull requests and main; version tags
  also publish Linux debug binaries. Live component tasks remain opt-in.
- M1.7: removed redundant typed-SDK inventories and their generated data,
  commands, reconciliation/audit consumers and unused dependencies. Existing
  typed collectors and independent Cloud Control/delivery evidence remain.
  All-target/all-feature workspace check, 26 retained behavior tests,
  warnings-denied production Clippy and both source guards pass.
- M1.4b: bounded reviewed-overlay parser with typed values, strict unknown-key
  rejection, Gregorian review dates and iterative binding-cycle checks.
  Parsed service metadata and operation maps are read-only to consumers;
  an external compile probe rejects all three attempted invariant mutations
  with `E0616`. After integration rebase, 427 catalog-tool tests, source
  Clippy and both source guards pass. Syntax contracts now live in
  `docs/API_CATALOG.md`. Semantic validation and callable plans remain M1.4c.
  Rust lines: 2,855 added, none deleted; production 1,454.
- `docs/VISION.md` and `docs/ROADMAP.md` now define the product and the build
  plan: direct service endpoint collection through a model-driven wire engine,
  binary records, hash-based reconciliation, full-send quota use, a laptop as
  the first runtime, and the CMDB layer in the first version. Cloud Control
  collection is retired as endpoint collection replaces it.
- The working tree was committed as the baseline (`78d3ac2`).
- M0 (`bbd0821`): `inventory_engine::store::SCHEMA_VERSION` is the single
  source of the SQLite schema version and `aws-inventory catalog` reports it
  (it reported a stale 12); both packagers exclude `.local`; `scripts/verify.sh`
  no longer invokes Python and runs the function-size check.
- M0.2/M0.3 verification after the dependency upgrade: the all-target workspace
  check and the full `bash scripts/verify.sh` gate pass. The gate ran with
  neither Python nor Python3 on `PATH`: source/function size guards, formatting,
  generated-data checks, split SDK-facade checks, Clippy, tests, doctests and
  the final build all passed.
- Preserve the concurrently merged O1 codec experiment under
  `evidence/record-codec-bench` without reverting production CBOR. Its
  `inventory-core` dependency now pins the original experiment's Git revision;
  its original measurements are explicitly historical, not current-codec results.
  Both size guards pass after integration (778 Rust files); the isolated
  benchmark's locked release run passes both fixtures and its cross-codec
  field assertions. Original result tables remain unchanged.
- M0.7a: `inventory-engine` and `cmdb-view` public items are documented; no
  `missing_docs` lint is configured for either crate. 378 public
  items are documented from their bodies, 22 are narrowed to crate visibility,
  and unused public functions are deleted (`Store::visit_edges`,
  `CoverageState::is_terminal_success`, three `LodLimits` getters and two
  `PageRequest` getters). An independent review found six comments that
  stated behavior the code does not have; all were corrected. Rust lines: 727
  added, 207 deleted.
- M0.7b (core/harness/benchmark half): documented the remaining public core
  types and errors in core, harness and benchmark; no `missing_docs` lint is
  configured for them. Legacy
  JSON types moved to a private module without changing their root API.
  Review corrected six misleading or overexposed declarations. Core's 193
  tests, all-target Clippy and warnings-denied rustdoc for these crates pass.
  The Makepad macro-generated entry point prevents a UI-wide deny without
  suppressing its diagnostic; this exception is documented at the macro.
  Catalog-tool documentation and the complete M0.7 gate remain open.
  Rust lines: 381 added, 149 deleted.
- M4.3: bounded deterministic HTTP/1.1 fault proxy with all thirteen fault,
  latency and rate-limit behaviors, TLS upstream forwarding, scoped replay,
  keep-alive and supervised shutdown. Independent code and test reviews closed
  after protocol, timing, scope and lifecycle repairs. All 52 proxy tests,
  179 harness tests and seven explicitly enabled live emulator tests passed;
  the live S3 test observes TLS forwarding and scheduled throttling.
  Snapshot eviction is arrival-ordered; retained-log replay equality applies
  before overflow. Unsupported `Expect` is rejected with 417 before body reads.
  Rust lines: 4,909 added, none deleted; production 2,483.
- M1.2c: generated and embedded the 6,951,532-byte binary API catalog.
  Compilation and exact comparison reproduce SHA-256
  `4ccb385a0d57ff48d8b957114ddbe27db4537d726368e0cc3d10b13a231bce3b`.
  Borrowed catalog access uses an inline bounded validation stack, without
  heap allocation; generic record readers retain deep-value compatibility.
  Compilation rejects dirty models, including ignored untracked files, and
  retains service/shape error context. Independent code and test reviews
  closed their findings. Rebased core/catalog tests: 571 passed; the product
  CLI test reports embedded counts without JSON. Rust lines: 3,456 added,
  69 deleted.
- Added seven classifier regressions for exact inclusive entry/byte limits,
  invalid operation keys and nonempty classification results. All seven
  public-behavior tests pass. Rust lines: 111 added, none deleted.
- Added standalone decoder-selection evidence under `evidence/decoder-bench`.
  The comparison recommends jiter and quick-xml for further protocol work;
  it does not claim a production decoder exists. Coordinator verification:
  locked Clippy and release scenarios pass, including cross-decoder binary
  parity, typed fields, retained-document bounds and chunked-reader parity.
  Recorded throughput measurements are from the benchmark worker.
  Rust lines: 2,054 added, none deleted after formatting.
- Isolated the old JSON model behind inventory-core's default `legacy`
  feature. Binary consumers disable default features; their normal dependency
  graph contains no serde or serde_json. Existing root imports and JSON
  behavior remain on the old scan path. Both independent reviews approve.
  Verification: 200 binary-only core tests, 227 legacy core tests and
  164 catalog-tool unit tests pass; both feature-specific Clippy runs and
  source guards pass. Rust lines: 397 added, 389 deleted, mostly relocation.
- Owner policy supersedes the missing-docs gate: remove Rust comments,
  including Rustdoc, and keep contracts in external Markdown. Removal is
  in progress.
- M2.3: pooled HTTP transport with keep-alive, HTTP/2, global socket bounds,
  cumulative request deadlines and independent connector deadlines covering
  admission, bounded DNS, TCP and TLS. Response limits apply before and during
  body collection; TLS errors and retryable HTTP/2 shutdowns remain typed.
  Code/test review findings are closed. After integration rebase, all 98 wire
  tests, source Clippy and source guards pass. Idle sockets retain permits until
  eviction, so another host can wait until its deadline; HTTP/2 checkout-age
  expiry can retire a sender while an active stream retains its permit.
  Rust lines: 3,313 added, none deleted; production 766.
- M1.4a: model-derived collection-plan reports, without granting callability.
  The pinned models yield 2,197 enumeration candidates, 2,940 expansions
  (337 modeled, 2,603 needing overlays), and 440 suspected missing paginators.
  An independently executed jq oracle matches all measured totals and splits.
  Both reviews approve; 392 catalog-tool tests pass after integration rebase,
  including the 512-example fixed-seed property. Regeneration comparison,
  source Clippy and source guards pass; the gate checks the committed report.
  Rust lines: 2,374 added, 21 deleted.
- `crates/aws-wire` is scaffolded, and Smithy's AWS protocol test suites
  (Smithy 1.75.0, seven protocols, 901 HTTP cases) are committed as JSON ASTs
  under `crates/aws-wire/tests/fixtures/protocol-tests/` with the conversion
  script `scripts/protocol-tests/convert.sh` and provenance digests. The
  conversion uses the released Smithy CLI and `jq`; it is not part of the
  build or the gate.
- M3.1b: `Record::content_hash` (BLAKE3 through the `blake3` library) with
  field-ID exclusion paths, `FORMAT_VERSION = 1`, the borrowed validated view
  `MapRef::from_bytes`, and typed reads (`ValueKind`, `MapRef::get_typed`).
  The hash is computed in one pass without allocation. 50 new tests. Rust
  lines: 1,350 added, 19 deleted.
- M1.2b: the append-only field-ID registry `catalog/field-ids.txt` (49,069
  member names; the line number is the ID) and `catalog-tool models fields
  [--check | --write]`. The gate now also runs the submodule check, the
  model report check and the registry check. Rust lines: 1,525 added, 17
  deleted.
- M2.2a: `aws_wire::sign`, SigV4 request signing as a thin wrapper over the
  `aws-sigv4` library, proven with 37 of AWS's published test vectors. Rust
  lines: 1,270 added, none deleted.
- M1.3a: `catalog-tool classes [--check | --write]` classifies all 19,641
  model operations (enumerate 2,197, expand 2,940, denied 11,370,
  unclassified 3,134) into `catalog/aws-classes.report`, using the must-deny
  list `catalog/overlay/must-deny.txt` (85 entries, 70 of them
  service-scoped `List`/`Describe` operations that return secrets or
  customer content, are billed, or have side effects). A class is a
  candidate: nothing is callable without a reviewed service overlay. The
  gate runs `classes --check`. Rust lines: 1,869 added, 11 deleted.
- M1.6a: `catalog-tool models endpoints [--check | --write]` evaluates every
  model's endpoint rule set and writes `catalog/aws-endpoints.report`: URL
  template, signing name and signing region for 436 services (28 global,
  402 regional, 3 needing an overlay, 3 unsupported), proven on the 4,112
  published endpoint cases in scope, with three explicit per-region
  exceptions. The gate runs `models endpoints --check`. Rust lines: 4,138
  added, 2 deleted.
- M1.5a: `catalog_tool::names`, the rule that turns a type name into a SQL
  table name (`AWS::EC2::VPCDHCPOptionsAssociation` is
  `aws_ec2_vpc_dhcp_options_association`), with the reviewed exceptions in
  `catalog/name-exceptions.tsv`. No command yet; `catalog/kinds.tsv` follows
  with M1.4. Rust lines: 1,368 added, none deleted.
- M3.1c, fixed: `Record::from_bytes` and `MapRef::from_bytes` accepted maps
  with unsorted or duplicate keys, for which lookups could report a present
  field as absent. Validation now rejects them (`KeyOutOfOrder`,
  `DuplicateKey`) and rejects unsupported key types up front. Bytes written
  by the encoder are unaffected. Rust lines: 644 added, 102 deleted.
- Fixed, found by an adversarial run against the built `inventory-harness`
  binary (41 commands, 2 defects): `assert cloud-control` exited 0 while
  printing failed checks (missing, empty, corrupt and wrong-schema databases
  all "succeeded"), and the commands that take `--flag value` pairs accepted
  unknown and repeated flags, so `install <dir> --unexpected x` created the
  destination before failing elsewhere. A failed check now exits 1 after
  printing the checks; an unknown or repeated flag is rejected before any
  work. Three tests run the binary.
- Fixed, found by an adversarial run against the built `aws-inventory`
  binary (72 scenarios, 1 defect): a scan that ends `partial` or `cancelled`
  exited 2 with nothing on standard error. It now prints one line there
  saying the report on standard output lists what failed.
- Fixed, found by an adversarial run against the built `catalog-tool`
  binary (160 commands, 5 defects, all in `package`): the archive was
  written through a predictable sibling name, so a symlink or a file already
  at that name was overwritten or destroyed; a destination inside the
  packaged root packaged itself; a `MANIFEST.sha256` that was a link to a
  payload truncated it. The archive, the manifest and generated SDK files
  are now published through an exclusively created temporary file and a
  rename, with the mode `File::create` would give; a destination inside the
  root is rejected; excluded directories are not entered; file count, depth
  and bytes are bounded before reading. The harness's second copy of the
  packager is deleted and re-exports this one. The destination directory
  must not be writable by untrusted users (the check is not atomic with
  publication). Rust lines: 1,069 added, 652 deleted.
- Fixed, found by the architecture audit: `cmdb-view` removed duplicate
  resource IDs only when they were adjacent after sorting by account, region
  and kind, so the same ID seen under two scopes was listed twice while
  lookup by ID returned one of them. A resource's identity is its ID: the
  first observation in input order is kept and every listed row is found by
  lookup. Rust lines: 245 added, 6 deleted.
- Fixed, found by two Black Hat reviews: every `catalog-tool` command that
  reads the models walked the whole source tree before applying its file
  limit, and checked a model's size from metadata before reading it without
  a bound. Discovery now stops with `TooManyEntries` after 65,536 directory
  entries, and a file is read through a 64 MiB bound that holds even if it
  grows. Rust lines: 128 added, 19 deleted.
- Fixed, found by the architecture audit, in the harness: the emulator
  process was a bare handle between spawn and readiness, so an early error
  could leave it running; `Emulator::stop` always returned success; snapshot
  documents were read without a size bound. One guard now owns the process
  from spawn to drop and kills and reaps it exactly once, `stop` returns the
  real error, readiness reads only complete log lines within a byte and
  attempt bound, and one bounded reader serves the snapshot and audit code.
  Rust lines: 742 added, 85 deleted.
- Project agents in `.omp/agents/` include the local GPU builders and
  `aws-build-sol` (ChatGPT Sol). Builders implement independent packages in
  parallel; the coordinator owns serialized compilation, formatting and tests.
- Decision V34: records are CBOR through the `minicbor` library. A codec
  benchmark (not kept in this repository) found it
  faster to build and smaller than our own format, `rkyv`, `postcard` and
  JSON. Our own format was deleted before landing; it and the benchmark are on
  the branch `evidence/o1-record-codec`.
- M1.1: the AWS API models are pinned as a shallow git submodule at
  `.local/aws-models` (commit `46135f8`, 436 files), checked by
  `scripts/check-models.sh`. Lock, verify and fetch commands of our own were
  built and discarded after review (decision V23 as changed).
- M1.2a (`cde710a`): stream the pinned Smithy JSON model files through
  `catalog-tool models report`; commit `catalog/aws-models.report` and support
  exact `--write`/`--check` workflows. The actual pinned source matches all
  expected model totals; 43 catalog-tool tests and all-target Clippy pass.
  Rust footprint: 1,494 lines added, 4 deleted.
- M3.1a (`5910de1`): canonical CBOR records through minicbor, validated scalar
  types, allocation-free borrowed lookup/iteration, generic framing validation,
  32-container ownership limits and a writer capped at 16 MiB before copying.
  122 core tests and all-target Clippy pass; independent reviews are closed.
  Rust footprint: 2,808 added including tests; 941 production physical lines,
  above the 500-line reporting target because validation and typed errors are
  retained. Content hashing and format version remain M3.1b.
- Every document now carries a status line (plan, kept and extended, baseline
  being replaced, or historical evidence) and `docs/README.md` indexes them by
  class. Statements the owner's decisions superseded are marked with the
  decision ID in `docs/ARCHITECTURE.md`, `docs/CMDB_ARCHITECTURE.md` and
  `docs/CMDB_PERFORMANCE_CONTRACT.md`. `CONTRACT.md` is declared the baseline
  contract.
- Corrected stale counts and links in `README.md`, `START_HERE.md`,
  `NEXT_AGENT_PROMPT.txt`, `docs/CMDB_ARCHITECTURE.md`, `docs/VERIFICATION.md`
  and `docs/UNIVERSAL_COVERAGE.md`. Removed the duplicate
  `docs/PERFORMANCE_CONTRACT.md`.
- M6.5 / V32: removed the obsolete `cmdb-api` crate, binary and HTTP endpoint
  tests, its workspace member, the direct `axum`/`tower`/`tower-http` workspace
  declarations, and its function-size gate path. Removed the launch command
  and updated current documentation; earlier API additions below are historical.
  `cmdb-view` still parses one JSON export and `cmdb-ui` still renders a
  synthetic fixture. In-process SQLite/FalkorDB reads and UI integration remain
  planned in M6.2/M6.3; unrelated transitive HTTP dependencies are retained.
  Rust footprint: 1,528 lines deleted.
- Refresh external Rust dependencies to the latest non-yanked stable registry
  releases, including all 437 existing exact-pinned AWS SDK crates; add
  `minicbor` 2.3.1 with `std` for `inventory-core` and regenerate all three Cargo
  lockfiles. Preserve optional/client features and migrate ZIP filename handling
  for ZIP 9.5.1 rather than retaining the older API.
- Refresh the existing SDK catalog's version/checksum/source provenance from
  upstream `release-2026-11-13` (`c88e7f9c26a66d0500bc1cc820ce73b76fa22136`),
  without adding typed service dependencies. Keep SQLx at reviewed revision
  `10462fa4414c6c7f1ed1d39bb31678e3b726878c` for SQLite 0.39 compatibility
  (published 0.9.0 excludes it), and Makepad 2.0 at
  `506ec05105b9adb08d1d21f9879df181b5244ea7` rather than downgrading to the
  registry's latest 1.0.0.
- Pair embedded and standalone AWS Emulator pins at stable `v1.0.348`, commit
  `7461ab1bbe540afe8063ce03ee5477ebb649df48`. Verify the downloaded package's
  detached AWS signature and SHA-256. All six authenticated standalone live
  tests and both authenticated embedded tests pass on this pin. Unify the embedded
  BigInt dependency at 0.12 after the initial upstream 0.12/0.14 type mismatch;
  no registry source was patched. Keep
  1.0.336 evidence historical and add the actual 1.0.348 universal capture:
  1,291 attempted, 999 listed, 292 unsupported, zero failed.
  Legacy baseline compatibility fixes retain SNS's authoritative provider
  identifiers and replace the stale 418-type selector ceiling with catalog
  membership validation; no Cloud Control collectors were added.
- Regenerate the SDK operation inventory from the actual registry and lockfile:
  24,720 operations across 468 cached versions and 437 services. Historical
  execution evidence never transfers SDK versions; receipts require matching
  source, version and module fingerprints. Current exact-version evidence
  credits 47 operations, not the former 76 cross-version rollup.
- `inventory-harness head-to-head` times repeated collector runs against one
  seeded emulator and a PostgreSQL database it creates and drops. It compares
  results only when every tool reports the seeded scope and counts.
- E2E seeds now record the nine default Connect flows the pinned emulator
  creates with each instance, and use an explicitly seeded KMS key for
  CodeArtifact domains.
- E2E coverage reports distinguish unselected kinds, and count verification
  reports unexpected kinds and pre-existing seeded identities.
- Seed validation now rejects a relationship endpoint that names a step with a
  response identity list, because such a step does not yield one resource.
- E2E count verification now fails when the run adds resources of a kind that
  the seed oracle does not list.
- Head-to-head comparisons enforce exact seeded counts, except for the EC2
  defaults documented for VPCs and subnets. They handle SIGINT, SIGTERM, SIGHUP
  and SIGQUIT throughout startup and collection, settle database creation
  before cleanup, and report a failed deletion.
- Head-to-head samples report killed leftover processes, retain leaders until
  group signalling, and stop the native stack on interruption.
- Fixture downloads retry reads interrupted by a signal.

## Unreleased — Universal listable coverage, reference graph and Rust evidence harness

- Added native Step Functions state machine and activity adapters (schema 012),
  bringing native coverage to 39 kinds. `ListStateMachines`/`ListActivities`
  paginate and enrich each item with `DescribeStateMachine`/`DescribeActivity`
  and `ListTagsForResource` in per-item child scopes. Definitions, variable
  references and execution data are never stored; tag failures retain the item
  with `tags_collected=false`. `ASSUMES_ROLE` and `ENCRYPTED_BY` edges are
  emitted only for exact in-scope IAM role and KMS key ARNs, and logging
  destinations produce no CloudWatch edge. The `step-functions` selector also
  accepts `stepfunctions` and `sfn`.

- Added native KMS key and alias adapters (schema 011), bringing native coverage
  to 37 kinds. `ListKeys`/`DescribeKey` paginate and enrich keys, with
  customer-managed rotation status and tags read in per-key child scopes;
  `ListAliases` maps exact alias ARNs and keeps the target key ID as a property.
  Rotation and tag failures retain the key with explicit collection flags, and no
  key policy body, key material or alias-to-key edge is produced.

- Added universal Cloud Control listable coverage. `--resource-type all`
  attempts every list-capable type in the pinned schema (1,291 of 1,881), not
  just the reviewed 418, through one bounded shared adapter. Against the pinned
  emulator the verified partition is 1,291 attempted, 999 listed, 292
  provider-unsupported and 0 failed, with 410 parent-scoped and 180
  handler-less types reported as explicit skips. Scan JSON carries the full
  partition and names every skip.
- Added opt-in runtime reference edges (`--universal-references`, default off).
  Resolution uses a bounded, account/region-scoped identity index with exact
  ARN, unique primary identifier and ARN trailing-component tiers, emits
  idempotent synthetic `REFERENCES` edges, and streams resource observations so
  large scans are not materialised in memory.
- Added current-state freshness views (schema 007) with deterministic source
  preference: newer authoritative observations win, equal timestamps use source
  priority, and historical/per-source evidence remains available.
- Added an explicit account/region/collector coverage registry (schema 008):
  account onboarding, role identity, status transitions and per-scope coverage
  results are durable and queryable before any collection starts.
- Added native SQS and SNS adapters (schema 009/010), bringing native coverage
  to 35 kinds. SQS maps queues, attributes and redrive `DELIVERS_TO` edges; SNS
  maps topics and account-wide subscriptions with `SUBSCRIBES_TO` edges,
  deterministic pending-subscription identities and presence-only policy
  reporting (no policy bodies).
- Added `cmdb-bench`: a streaming deterministic fleet generator and independent
  oracle. Verified 1M and 10M resources (10,000 accounts, skewed distribution,
  duplicates, reorders, deletions, empty regions, global services) with peak
  RSS under 5 MB and byte-deterministic output.
- Added `cmdb-view`, a bounded read model over exported SQLite snapshots, and
  `cmdb-api`, a read-only loopback HTTP API carrying the snapshot fingerprint
  with typed errors, bounded pagination, graph neighborhoods, LOD and coverage.
- Added `cmdb-ui`, a Makepad 2.0 viewer over the synthetic fixture: filter rail,
  virtualized table, detail rail and a deterministic account→region→kind→resource
  DAG with resource overlay edges, hover/selection, wheel zoom and drag pan.
- Added executable performance/failure contracts and the CMDB architecture
  baseline, including explicit boundaries: no auth/write API, no event
  ingestion, no tombstones/retention and no measured fleet capacity yet.
- Repaired the operation-coverage callsite scanner so it resolves clients
  imported through `use super::*` and reports verified exercised operations
  against the reviewed matrix instead of publishing an unavailable count.
- Replaced the Python emulator snapshot/assertion entry point with the Rust
  `inventory-harness` operator binary: pinned emulator installation, snapshot
  freezing, S3 and Cloud Control seeding and Cloud Control assertions.
- Replaced the Python release packager with `cargo run -p catalog-tool --
  package --destination <zip>`. Selection rules, manifest bytes and payloads
  match the retired tool; entry order is sorted and every entry carries a fixed
  1980-01-01 timestamp, so archives over an unchanged tree are byte-identical.
- Added six ignored live emulator tests that own a fresh emulator, seed through
  the real SDKs and scan with the shipped collector: native services, reviewed
  Cloud Control gaps, the universal partition, combined native+universal,
  multi-page Cloud Control listing, and an exported seeded bucket.
- Added `catalog/cloudcontrol-universal.json`: 1,881 schema types partitioned
  into 1,291 list-capable, 410 parent-required and 180 without list handlers,
  with digest and pinned source archive identity.

- Added `cmdb-api`, a read-only loopback HTTP query API over one parsed
  snapshot: typed GET endpoints for meta, filtered/paginated resources, bounded
  graph neighborhoods, LOD and coverage, all carrying the snapshot fingerprint,
  with typed JSON errors and no write path or authentication. Non-loopback
  binds are refused until an auth story exists.

Current development-tree evidence: 781 Rust tests pass with zero failures (15
ignored by default). The native live emulator phase passes with 263 attempts,
168 resources and 344 edges, including SQS/SNS/KMS/Step Functions assertions.
The prepared live baseline passes 2,664 checks with zero failures, including the
Config freeze and parent pagination gates. All-target clippy is clean, formatting
and the 300-line source/25-line function gates pass. Cloud Control listing now
sends the page-size budget as `max_results`; the live pagination gate proves a
three-page list chain at page size two. Operation-coverage `--check` verifies 76
exercised operations plus 10 reviewed additive operations and refuses to publish
when the live scan cannot resolve reviewed callsites.

## 0.6.0 — Parent-scoped collection and exact graph verification

- Added 21 reviewed parent-scoped Cloud Control types across API Gateway
  REST/V2, AppConfig, EKS, ELBv2 and CloudWatch Logs. The runtime catalog now
  has 418 types; the original 397-type descriptor file remains byte-identical.
  Child-only selections add required roots automatically and share discovery.
- Added bounded parent-model admission, independent pagination scopes,
  duplicate-model rejection, ownership validation and explicit incomplete
  prerequisite outcomes. Missing or failed parents cannot produce a false
  complete-empty child scan. Every confirmed child retains parent/model
  provenance and an evidence-bearing `CHILD_OF` edge.
- Added 169 provider columns and 20 property relationship rules. SQLite
  schema 6 adds 21 views transactionally, bringing the provider view count to
  418 and column count to 4,273. Name conflicts roll back the entire upgrade.
- Updated offline planning to distinguish known root requests from unknown
  parent-dependent request counts and expose the actual nested traversal
  bounds. Coverage overlays the parent selection on the retained 1,881-schema
  universe while keeping historical evidence separately identified.
- Added parent property tests, adversarial prerequisite/cancellation/budget
  cases, and a production SDK HTTP test for real 429 retry accounting,
  serialized models and cursor isolation. Added a fifth guarded emulator
  snapshot with 161 children, 161 parent edges and literal architecture
  expectations, including 101 EKS add-ons to force pagination.
- Extended SQLite-to-JSONL-to-FalkorDB verification to every projected resource,
  edge and scan-marker property, with exact property/label counts and strict
  JSON semantics. Added a checksum-pinned native Redis/FalkorDB installer with
  bounded downloads/extraction and owned build-process cleanup.
- Tightened snapshot provenance to bind all five commands, current source,
  both catalogs, both semantic fixtures, the executing binary and distinct
  persisted scans. Current and historical evidence cannot be interchanged.
- Fixed nested supervisor cancellation so SIGTERM drains owned process groups
  and closes receipts, including when the original group leader already exited.
  Added bounded health-failure diagnostics and real-process regressions without
  relaxing fixture health timeouts.
- Repaired mutation acceptance: the final aggregate itself requires all ten
  named live cases and two clean baselines, rejects mixed fixture identities,
  includes embedded dependency SQL/data in source closure, and keeps compiler
  infrastructure errors and test deadlines unresolved. These acceptance fixes
  do not establish a complete 95% mutation result.

The completed local run passed 530 Rust tests with zero failures or ignored
tests, 2,522 emulator assertions, and all five exact JSONL/FalkorDB projections.
Ten portable JSON/JSONL files are retained. Fresh enumeration found 2,959 mutation
candidates but executed none. See `docs/VERIFICATION.md` for receipts and gaps.

## 0.5.0 — Broader coverage, portable export and automatic graph delivery

- Expanded the reviewed regional Cloud Control selection from 200 to 397 types
  across 76 service families. Added a complete 1,881-schema capability matrix
  with explicit read/list, required-parent and unreviewed-scope exclusions, and
  a bounded offline `coverage` command.
- Expanded schema-derived architecture extraction to 945 relationship rules
  across 244 selected types. Rules validate typed ARN service/resource forms,
  preserve returned owners, label short-ID owner assumptions, and derive edges
  only after sensitive-property redaction. Corrected accountless EBS snapshot
  ARN identities and distinguished VPC Lattice target groups from ELB targets.
- Added 197 provider-column views through transactional SQLite schema 5. The
  original 200-view migration and original columns remain unchanged; all 397
  descriptors expose 4,104 provider columns. Existing observations, fingerprints
  and caller-owned objects survive the additive upgrade.
- Added streaming portable JSON and JSONL artifacts with versioned envelopes,
  normalized or all-observation modes, source/page provenance, scope outcomes,
  failures, bounded records/bytes and content hashes. Finalized partial scans
  require explicit opt-in; running scans cannot be exported.
- Connected `scan` and durable job intent to optional file/FalkorDB delivery.
  Added `export` and `deliver` commands. Delivery can resume a finalized SQLite
  scan without reissuing AWS calls, preserves completed artifact receipts across
  later failures, and records collection and delivery outcomes independently.
- Added bounded FalkorDB reconnect/retry using the shared executor. Only
  exporter-recognized idempotent plans receive automatic ambiguous-write replay;
  acknowledgements follow server success and bind the complete immutable plan.
  Cancellation and timeout paths own and drain file/checkpoint resources.
- Added independent architecture fixtures, redaction/property cases, migration
  regressions, a 10,001-resource export test and actual-server fault injection
  that drops a reply after a committed graph write. Expanded the live CLI case
  to collect, export, project and checkpoint-resume in one workflow.
- Bound coverage refresh to the executing binary's embedded catalog, exact
  collector/fixture hashes, command receipts, finalized scan and persisted
  detailed reads. A transport success without a usable observation cannot be
  promoted to resource coverage.

The historical verification receipts are in `docs/VERIFICATION_V05.md`. The SDK
dependency catalog remains 437 optional, checksum-pinned service crates. This
increment does not claim every AWS resource, live-account validation, a new
throughput result or a completed project-wide 95% mutation score.

## 0.4.0 — Global inventory, durable jobs and snapshot analysis

- Added S3 bucket metadata, IAM roles/instance profiles/customer-managed policy
  metadata, Route 53 hosted zones/records and CloudFront distributions. Seven
  new native kinds run outside the regional loop and retain bounded child
  failures, composite cursors and architecture references.
- Added Organizations account/hierarchy discovery, optional existing-role
  account fan-out and existing Config/Resource Explorer source discovery.
  Incomplete account listings stop automatic target selection.
- Added a SQLx/SQLite job queue with transactional claims, immutable lease
  fencing, bounded attempt history, idempotent submission, durable cancellation,
  expired-worker recovery and explicit retry of terminal unsuccessful jobs.
- Added worker supervision with bounded cancellation drainage, heartbeat
  renewal during drainage and bounded per-job/output summaries.
- Added streaming snapshot comparison, bounded neighborhood/path exploration,
  source-aware findings and synthetic regression fixtures.
- Added additive schema4 migration with seven native and three expanded SQL
  views, preserving old observations, custom objects and page fingerprints.
- Added a pinned, signed AWS Emulator integration harness, a complete 72-operation
  matrix plus the SDK AssumeRole boundary, populated fixtures, bounded request
  recording and independent direct-observation assertions. All 200 Cloud Control
  types are attempted; upstream gaps remain explicit.
- Added separate official SDK wire-replay tests for Resource Explorer timestamp
  conformance, terminal handling for HTTP 501, and safer diagnostics when the SDK
  rejects an HTTP success response.
- Added bounded Proptest suites for core identities/serialization, API retry and
  pagination histories, SQLx page transactions, durable job leases, discovery
  configuration and actual CLI behavior. Minimized regressions are retained.
- Fixed the public AssumeRole configuration helper to preserve the explicit
  one-attempt retry setting through SDK service defaults, including DynamoDB.
  Properties check all 15 implemented service configurations and region clones.
- Added exact emulator snapshot oracles for native, supplemental, Cloud Control
  and assumed-role collection, plus actual CLI/worker/graph round trips. The
  mutation fixture freezes its own synthetic Config recorder before collecting
  the oracle, preserving provider timestamps in comparisons.
- Added a pinned, complete-candidate mutation runner and strict 95% aggregate
  gate, with source/fixture receipts, required live baselines, bounded owned
  processes and explicit unresolved infrastructure failures. Core is complete
  at 56/56 caught; the whole-project gate is still incomplete.

This increment retains the 200 reviewed Cloud Control descriptors and all 437
pinned SDKs. Configured coverage, local validation and live AWS validation remain
separate; no live AWS collection or higher throughput claim accompanies it.

## 0.3.0 — Schema-backed breadth and collection planning

- Added a shared official Cloud Control SDK collector and 200 reviewed regional
  list/read descriptors, with bounded enrichment, provider identity provenance,
  sensitive-field redaction and explicit per-resource failures.
- Added an offline collection plan and 200 generated provider-column SQLite
  views in the aws_cc_* namespace. The view generator is Rust.
- Added ACTIVE ECS task-definition revisions, role/filesystem references, and
  ECS service primary/alternate target-group relationships.
- Added transactional schema3 migration, preserving observations and page
  fingerprints while introducing Direct4 > CloudControl3 > Config2 > Explorer1.
  Unknown added managed-table columns reject the migration before mutation.
- Added a Rust source auditor for all 437 pinned SDKs: 19,617 operation modules,
  3,779 paginators and 407 waiters, with archive checksums and reproducible output.
- Audited native retry hooks and current Rust workflow options. Retained one
  request retry owner so every explicit inventory SDK attempt re-enters the
  existing quota scheduler; no nested retry or workflow server was introduced.

The 200-type catalog is configured schema-backed capability. Live AWS regional
coverage and throughput remain unvalidated. See the current verification report.

## 0.2.0 — Architecture inventory expansion

- Added native route-table, internet-gateway, NAT-gateway, VPC-endpoint,
  network-ACL and VPC-peering collectors.
- Added native ELBv2 load-balancer, listener, ALB-rule, target-group and
  target-health membership collectors. Target groups are enumerated regionally,
  including groups with no attached load balancer.
- Added fractional request policies and shared server-directed cooldowns.
  Existing integer policy JSON and Rust constructors remain supported.
- Added a transactional SQLite schema-v2 migration, typed views, exact-snapshot
  route rows, endpoint-observation flags and load-balancer type aliases.
- Added the bounded `architecture` report command, source-aware coverage and
  explicit per-panel truncation.
- Aligned Config and Resource Explorer names for six network and four ELBv2
  resource kinds. Explorer uses ARN evidence to distinguish Classic ELB;
  supplemental records retain their source-specific detail and enrichment limits.
- Added a synthetic architecture fixture, query examples, collector design
  contracts and regression tests.

The native inventory now has 24 resource/component kinds. SDK availability is
still the pinned catalog of 437 official service SDK crates, not 437 implemented
collectors. Live AWS coverage and throughput have not been established.

## 0.1.0 — Initial working foundation

- Added 13 native resource kinds, Config aggregator and Resource Explorer input,
  a bounded request executor, SQLx/SQLite storage and replayable FalkorDB export.
- Added atomic page/checkpoint commits, immutable finalized snapshots, explicit
  partial outcomes, bounded SQL queries, a synthetic demo and SDK provenance.
- Retained original benchmark observations, including an unfavorable repeat;
  no stable throughput improvement was established.

See `docs/VERIFICATION.md` for observed test/build results and the remaining
validation boundaries for the current source tree.
