# Boundary map

| Boundary | Inputs | Trusted output / ownership |
| --- | --- | --- |
| CLI/config/HTTP | Untrusted strings, JSON, tokens, paths | Parsed configuration, identities, byte/count budgets, authorization context |
| AWS SDK | Provider pages and typed transport errors | Validated observations, explicit partial coverage, operation evidence |
| Event transport | Duplicate/reordered/oversized/forged envelopes | Authenticated durable receipts and bounded refresh intents |
| Pure domain | Trusted facts, policy versions, explicit clock facts | Deterministic transitions, source decisions, reconciliation and effect intents |
| Async orchestration | Durable work and capability grants | Region-owned bounded tasks, cancellation/drain outcomes, no detached effects |
| Storage adapter | Parsed domain commands | Atomic observations/checkpoints/outbox, conditional ownership, migrations |
| Projectors | Durable accepted sequences | SQL/graph/search versions, lag, deterministic replay and rebuild |
| Query gateway | Authorized query AST/filter/traversal | Bounded results with coverage, sequence, freshness, truncation, and provenance |
| UI | Authorized versioned DTOs and user actions | Accessible tables, bounded GPU views, visible coverage and conflict recovery |
| Operator/bootstrap | Reviewed deployment plans | Explicit audited infrastructure changes using separate credentials |

## Proposed component organization

Retain the existing inventory-core, inventory-engine, inventory-aws, sdk-catalog,
and CLI boundaries where they fit. Add domain/control/query/UI adapters only for
concrete owned responsibilities. Crate boundaries are implementation decisions;
do not introduce generic plugin frameworks or one-implementer traits solely to
match this diagram. Collector definitions and compatibility views are versioned
data; provider calls remain typed and auditable.

Production application logic remains Rust. The browser application uses a Rust
Wasm shell and a measured WebGL2-capable renderer; a narrow JavaScript binding
is permissible only if its necessity and measured tradeoff are recorded.
No first-party unsafe code, ad hoc coding harness, or unchecked parsing is
introduced for speed. Existing native dependencies retain explicit trust scope.

Time, randomness, identifiers, persistence, network, and credential acquisition
are capabilities supplied to the shell. Domain calculations receive explicit
values and are deterministic. Diagnostic profiling may be disabled for a timed
run only when the production observability configuration and comparison policy
are recorded; correctness/durability checks remain enabled.

## Runtime ownership

Each service region owns tasks, cancellation, bounded channels, and shutdown.
No lock guard spans await. Blocking serialization/compression or driver work is
isolated behind bounded admission and tracked completion. No unbounded task per
resource, entire-account vector, or detached retry loop is permitted. Outbound
network destinations, filesystem paths, and credentials are capability-gated.

## Authorization propagation

Authorization is enforced before query planning and at storage boundaries.
Workspace selection in a URL is not a capability. Connection-pool checkout sets
and verifies transaction-local policy context; return/reset cannot leak a prior
request's workspace. Projectors and background workers use narrowly scoped
service identities, not the API owner's unrestricted database role.
