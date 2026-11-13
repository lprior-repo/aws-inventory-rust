# Type contracts

These are required semantic types and constructors, not production Rust code.
External Serde/SQL/HTTP DTOs may contain primitives; parse them exactly once into
trusted domain values. Private fields prevent bypassing constructors. Preserve
existing CLI/export DTOs through explicit versioned adapters.

| Type family | Required constraints |
| --- | --- |
| WorkspaceId, RegistrationId, WorkId, EpochId, ReceiptId | Distinct opaque identifiers; no implicit cross-type conversion |
| AwsAccountId | Exactly twelve ASCII digits; preserve leading zeroes |
| Partition | CommercialAws in launch domain; reject noncommercial registrations explicitly |
| Region | Reviewed current region registry plus explicit endpoint metadata; Global is a separate sum variant |
| ResourceKind, CollectorVersion, SchemaVersion | Registry-backed values with explicit Unknown/Unsupported boundary outcome |
| CanonicalResourceId | Validated namespaced tuple; cannot be constructed by arbitrary string concatenation |
| ResourceReference | Resolved identity with evidence, or UnresolvedReference with raw bounded identifier and resolution reason |
| ResourceIncarnation | Provider lifetime marker or explicit continuity-unknown marker |
| SourceTime | ProviderCapturedAt, ProviderEventAt, LocalObservedAt, and ReceivedAt are distinct units/semantics |
| Duration, Deadline, Rate, Capacity, ByteSize | Bounded positive units; checked conversions; no raw integer time or rate parameters |
| ScopePopulation | Immutable collector/filter/parent/visibility definition with canonical digest |
| LeaseGeneration | Monotonic fencing value minted by durable control storage; never chosen by a worker |
| CommitSequence, ProjectionSequence | Monotonic storage ordering; never equated with provider causal order |
| CompleteScopeEvidence | Constructible only from validated terminal pagination and permitted outcome rules |
| ConfirmedDeletionEvidence | Reviewed collector policy plus qualifying absence/lookup evidence; cannot be made from a failed scan |
| AuthorizedWorkspace | Derived from verified identity and current policy, never solely from request path/body |
| RedactedPayload | Policy-classified field representation; raw secret values cannot satisfy this type |
| BoundedPage, BoundedEvent, BoundedGraphView | Checked counts and aggregate byte budgets before allocation/processing |

## Sum types and legal states

AccountState = Discovered | AwaitingAccess | Verifying | Enabled | Degraded |
Suspended | Removed. Each variant carries only its required timestamps/reasons.

EpochState = Planned | Running(lease, checkpoint, cutoff) |
Complete(complete_evidence, counts, end_sequence) |
Incomplete(reason, committed_checkpoint) | Cancelled(checkpoint) |
Superseded(new_epoch). There is no optional completed_at that defines state.

WorkState = Ready | Leased(owner, generation, expiry) |
RetryScheduled(next_eligible, budget) | Succeeded(result) |
Failed(typed_error) | Cancelled(reason) | DeadLetter(diagnostics_reference).
Storage excludes terminal payloads from active work claims by construction.

Presence = Observed(version) | SuspectedAbsent(evidence) |
ConfirmedDeleted(tombstone) | Reobserved(new_version, continuity_decision).
Visibility = Complete | Partial(missing_fields) | Denied(reason) |
Unsupported(reason). Presence and visibility are orthogonal.

Freshness = Fresh(age, budget) | Overdue(age, budget) |
Unknown(reason). A valid observation may be Overdue; this is not deletion.

ExecutionOutcome = Success(value) | DomainFailure(error) |
Cancelled(reason, drain_result) | DeadlineExceeded(checkpoint, drain_result) |
RuntimeFailure(bounded_diagnostic). Panic is a runtime fault, not ordinary success.

## Operation contracts

AcceptPage requires AuthorizedWorker, CurrentLease, RunningEpoch, MatchingScope,
NextPageOrExactReplay, and BoundedPage. Its successful result contains the
durable checkpoint and commit sequence, not just an in-memory acknowledgement.

CompleteEpoch consumes RunningEpoch and validated terminal evidence. It cannot
be invoked with partial child discovery, exhausted bounds, denied pages, repeated
tokens, expired ownership, or an unexplained missing page.

ConfirmDeletion consumes a deletion candidate and ConfirmedDeletionEvidence.
Normal source priority, wall-clock age, a missing event, or a disconnected account
cannot satisfy the constructor.

EditAnnotation requires AuthorizedEditor and ExpectedRevision. An outdated
revision returns RevisionConflict and preserves the stored revision unchanged.

ApplyProjection requires an accepted observation/outbox record and a version
newer than the affected projected entity. Replay of the same version is a no-op;
an older version is retained as evidence without regressing current state.

## Remaining representability risk

The existing Resource/Edge/CollectionScope structs expose primitive strings and
separate validation. They are boundary DTOs for migration purposes; they are not
the trusted CMDB domain. Implementing the new contract requires adapters and
source-level type migration, not renaming the existing structs in documentation.
