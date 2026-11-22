# AWS CMDB full specification bundle

> [../VISION.md](../VISION.md) is the product definition and wins where this
> bundle differs. Its decisions of 2026-11-13 change this draft: collection uses
> service endpoint APIs through our own wire engine, not Cloud Control; the
> first runtime is one laptop with about 10 accounts, with the enterprise
> deployment later; freshness comes from repeated sweeps on an interval, with
> the five-second event path later; no history is kept in the first version;
> owner, team and application come from tags only; and the UI is a native
> Makepad application.

Start with [the full specification index](architecture-spec.draft.md), then read
[the decisions and pending choices](decisions.md). This is a comprehensive
implementation design draft for commercial AWS, self-hosted enterprise use,
thousands of continuously monitored accounts, five-second change processing and
hourly reconciliation, with confirmed deletion/history and a GPU-assisted UI.

Includes 135 EARS requirements, 29 acceptance scenario families, domain/type/
workflow/error contracts, storage/collection/API/UI/security/operations designs.
All requirements are specified/unverified; no executed product acceptance tests
are claimed by this document set.

Pending overload/retention answers and the final architectural pre-mortem prevent
this draft from being called the final accepted baseline. No production Rust
implementation, real AWS deployment/collection, task decomposition, formal proof
or independent approval is claimed by this bundle.
