# Coverage seed catalogs and provenance

These registers expose the known starting surface and unresolved classification.
They are not a completed collector plan or a claim that every SDK API is inventory.

| File | Rows / scope |
| --- | --- |
| current-collector-descriptors.jsonl | 450: 32 native kinds + 397 Cloud Control roots + 21 parent-dependent types |
| sdk-service-review-register.jsonl | 437 exact current SDK service versions, all requiring inventory-surface review |
| sdk-operation-review-register.jsonl | 19,643 current SDK operation modules, all initially unreviewed |
| current-sdk-source-audit.json | Fresh offline lexical audit of exact current published packages and archives |
| source-hashes.txt | Exact source-input file digests; repository-relative paths identify source catalogs |
| provenance.json | Machine-readable source pin, counts, input hashes, audit identity and limitations |

The source auditor inspected 437 services, verified all 437 archive checksums,
read 647,856,058 source bytes, hashed 242,234,008 archive bytes, and found 19,643
operation modules, 3,784 paginator modules and 407 waiter modules. It matches
the current manifest version for every service and official revision
`c8e11bea896b54530c587c29ce30ebd6619af62a` / release-2026-11-12.

The first audit invocation used the unused default Cargo registry path and failed
with a missing-file error. The successful invocation used the actual configured
`/cache/cargo-shared` registry, with no network calls, AWS credentials, service
clients or SDK feature execution. Both outcomes are retained in audit receipts;
only the successful result supplies the register. The existing auditor was built
without production source edits.

SDK operation names include writes, application-data reads and opt-in services.
All classifications remain `unreviewed`, with no reviewer/evidence assigned.
No regex based on Get/List/Describe may silently mark them accepted inventory.
Collector rows represent descriptor availability and selected fields, not new
CMDB acceptance. Different adapters overlap normalized resource identities.

To regenerate, run the repository's existing `inventory-sdk-catalog` example
`audit` against the pinned manifest and actual Cargo registry source/cache,
then enumerate its services/operations into the review schema. Keep exact
manifest/archive hashes and all errors. Classification, coverage-universe union,
permission mapping and production qualification are subsequent reviewed work.
