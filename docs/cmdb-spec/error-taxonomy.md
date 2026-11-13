# Error taxonomy

Owned API/workflow boundaries shall use the closed semantic categories below.
Provider codes and bounded diagnostics are attached evidence; new provider codes
map to ProviderUnknown until reviewed, rather than being assumed retryable.
This is exhaustive for owned boundary classes, not a claim to enumerate every
future AWS service error string.

| Error family / required variants | Disposition |
| --- | --- |
| Input: MalformedEnvelope, InvalidIdentifier, InvalidArn, InvalidRegion, NonCommercialPartition, InvalidFilter, InvalidCursor, InvalidConfiguration, UnsupportedSchemaVersion | Reject at boundary; no domain mutation |
| Bounds: PayloadTooLarge, PageTooLarge, TooManyChildren, TooManyEdges, DepthExceeded, TokenCycle, PageBudgetExhausted, QueryBudgetExceeded, ExportBudgetExceeded | Explicit partial/rejection; never complete enumeration |
| Identity: CallerAccountMismatch, PrincipalMismatch, OwnerUnresolved, AliasConflict, IncarnationAmbiguous, SourceBindingMismatch | Quarantine or explicit unresolved evidence; no guessed identity |
| Credentials: CredentialUnavailable, CredentialExpired, CredentialRefreshFailed, AssumeRoleDenied, ExternalIdMismatch | Bounded refresh where appropriate; otherwise degrade target |
| Provider: AccessDenied, ResourceNotFound, UnsupportedOperation, RegionDisabled, SubscriptionRequired, Throttled, RetryHintInvalid, RetryHintExcessive, ProviderUnavailable, ProviderUnknown | Per-operation policy; NotFound is context-dependent and not universal deletion |
| Transport: DnsFailure, TlsFailure, ConnectTimeout, RequestTimeout, ConnectionLost, TruncatedResponse, ProtocolViolation | Bounded classified retry; TLS/protocol policy failures are terminal until repaired |
| Work: QueueAtCapacity, AdmissionClosed, WorkConflict, LeaseLost, LeaseExpired, LeaseGenerationMismatch, WorkDeadlineExceeded, CancelRequested, DrainDeadlineExceeded | Stop new effects; preserve accepted work; visible retry/failure outcome |
| Enumeration: CheckpointConflict, PageFingerprintMismatch, PopulationChanged, InvalidContinuationToken, ParentDiscoveryIncomplete, MissingTerminalEvidence, ScopeSuperseded | Retain evidence; reject completion; restart only under reviewed policy |
| Reconciliation: IncomparableSourceOrder, StaleObservation, FieldRedacted, VisibilityChanged, InsufficientDeletionEvidence, ConflictingOwnership | Retain evidence and explicit uncertainty; do not invent empty fields/deletions |
| Storage: Unavailable, TransactionConflict, CommitOutcomeUnknown, DiskFull, CorruptRecord, MigrationConflict, UnsupportedDatabaseVersion, BackupUnavailable | Retry known-safe transactions only; ambiguous commits resolve through idempotency lookup |
| Projection: ProjectionUnavailable, ProjectionGap, ProjectionVersionConflict, ProjectionLagExceeded, RebuildRequired, InvalidProjectionPlan | Keep durable outbox; gate freshness claims; rebuild from accepted state |
| Authorization: Unauthenticated, TokenInvalid, TokenExpired, WorkspaceForbidden, OperationForbidden, PolicyVersionChanged, FieldForbidden | Deny; audit without leaking hidden object existence |
| Query: UnsupportedSql, UnsafeSql, StatementTimeout, ResultTooLarge, SnapshotExpired, CursorExpired, InvalidTraversal, GraphTruncated | Return bounded typed result/error; label truncation and freshness |
| CMDB edit: RevisionConflict, InvalidCustomField, ReferencedObjectMissing, DuplicateExternalKey | Preserve current revision; return recoverable conflict |
| Delivery: DestinationUnavailable, DestinationDenied, DeliveryConflict, ReceiverRateLimited, ReceiverRejected, ExportPublicationFailed | Idempotent bounded retries and explicit dead letter; independent from inventory success |
| Runtime: TaskPanicked, TaskCancelledUnexpectedly, BlockingTaskFailed, ClockUncertain, InvariantViolation | Fail affected work safely, surface incident, and retain bounded diagnostics |

## Retry contract

Exactly one layer owns provider-operation retry policy. Every attempt consumes
the applicable rate/concurrency budget, including retries and timeout retries.
Credential refresh has its own observable bounded budget. Retry ceilings include
attempts, elapsed time, jitter, server cooldown, and job deadline. HTTP status
alone does not determine retryability. A job retry cannot silently multiply an
exhausted inner attempt budget.

Database serialization/deadlock retries replay only pure transaction decisions,
with a fixed cap and no AWS effects inside the closure. CommitOutcomeUnknown
requires lookup by immutable idempotency key before retrying publication.

## Public error envelope

Required fields: stable code, safe message, correlation ID, retry disposition,
workspace-scoped diagnostic reference, and relevant scope/projection status.
No credentials, unredacted payloads, raw database connection strings, stack
frames, hidden workspace IDs, or provider secrets appear in user error text.
Return 400 for invalid requests, 401 for absent/invalid authentication, 403 for
authorized-principal policy denial where existence is public, 404 for hidden or
missing objects, 409/412 for revision conflicts, 413 for byte limits, 429 for
admission limits, and 503 for unavailable required dependencies. Stream errors
carry a resume/reset disposition. Internal provider codes never become the HTTP
authorization decision.
