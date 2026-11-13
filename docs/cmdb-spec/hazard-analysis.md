# Hazard analysis and blast radius

These are domain hazards and mitigation contracts. They are not executed proof
or test evidence. Scenario IDs identify future acceptance behavior requirements.

| Hazard | Cascade / violated invariant | Required containment | Scenario |
| --- | --- | --- | --- |
| Partial Organizations listing | Missing accounts appear fully covered | Complete-list evidence; preserve expected prior membership and mark discovery incomplete | A01/A03 |
| Wrong account/partition credentials | Cross-account corruption and exposure | Verify identity before acceptance; capability-bound target | A01 |
| Global service in regional loop | Duplicates, quota waste and false ownership | Reviewed global endpoint/population definition | A04 |
| Parent traversal denied or capped | Children disappear and are falsely deleted | Incomplete parent/child coverage; no deletion evidence | A05/A08 |
| Empty page with token | Premature successful completion | Continue validated token chain | A05 |
| Token invalid/cyclic | Infinite loops or false terminal population | Bounded explicit outcome; new epoch on reviewed restart | A05 |
| Worker crashes after commit before ACK | Duplicate effects and state | Unique page/receipt keys; idempotency lookup and replay | A06 |
| Split-brain/paused owner resumes | Stale updates, duplicate quota allocation | Generation-checked storage writes; bounded dispatch overlap/headroom | A06/A10 |
| Quota domains split by workspace | AWS throttle storm affects all registrations | Coordinate real account/service quota boundary | A10 |
| Retry layers multiply | Self-amplifying AWS failure and deadlines | One retry owner, finite attempts/time, preserved cooldown | A10 |
| Hot account/event storm | Hourly work starves | Fair per-class/account scheduling and bounded coalescing | A12/A13 |
| Event delayed/dropped upstream | Stale inventory despite healthy consumer | Hourly authoritative reconciliation and provider-delivery telemetry | A11/A12 |
| Event duplicated/reordered | Resurrection or stale overwrite | Receipt linkage, source ordering, monotonic entity versions and confirmation reads | A07/A08 |
| Database outage/disk full | Loss after premature acknowledgement | No upstream ACK before durable receipt; bounded backpressure and alerts | A13/A27 |
| Current source priority ignores age | Permanently stale direct rows hide newer facts | Freshness/authority/ordering-aware field policy | A07 |
| Arrival time mistaken for capture | Cached source appears fresh | Distinct timestamp types and per-field source age | A07 |
| Absence races with newer observation | Newly created/updated resource deleted | Epoch cutoff, conditional lifecycle update, reobservation reset | A08 |
| Restricted role narrows visibility | Hidden resources called deleted | Principal visibility class and complete evidence policy | A03/A08 |
| ID reused or shared owner guessed | Incorrect CMDB joins and exposure | Incarnation/alias evidence; unresolved references explicit | A04 |
| One edge source incomplete | Other sources' dependencies erased | Relationship population/source ownership | A08 |
| Outbox cursor skips gap | Projection falsely claims completeness | Contiguous watermark or explicit gap accounting | A09 |
| Graph offline / rebuild on live namespace | Collection stalls or users see partial graph | Bounded outbox; independent rebuild, validation, atomic switch | A09 |
| SQL RLS owner/pool bypass | Cross-workspace leak | Non-owner/non-BYPASSRLS roles, context reset, negative tests | A16 |
| Hidden count/path/cache leak | Indirect privilege bypass | Authorization across every derived read model and cache key | A16 |
| Tag/annotation injection or SSRF | Browser execution or network exfiltration | Escaping, CSP, destination controls, bounded parsing | A17 |
| Secret-bearing config retained | Persistent exfiltration across sinks/backups | Pre-storage classification/redaction and canary-secret scans | A17 |
| Gigantic/compressed payload | Memory exhaustion and worker starvation | Preallocation/decompression/depth/byte/time bounds | A13/A17 |
| Entire estate rendered in browser | Frozen UI and excessive transfer | Server-bounded neighborhoods, LOD, background layout | A21/A23 |
| Obsolete layout/query completes late | Wrong graph replaces current selection | Query/layout generation checks and cancellation | A21 |
| GPU context loss / nonfinite coordinates | Broken UI or shader instability | Finite coordinate parsing, bounded fallback and context recovery | A21 |
| Retention expires dedup/tombstones early | Old events resurrect deleted resources | Replay horizon and stale-generation rejection independent of receipt dedup | A20 |
| Archive deleted before verified | Irrecoverable history loss | Manifest/hash verification and holds before expiry | A20 |
| Backup exists but cannot restore | False recovery promise | Automated restore drill and measured RPO/RTO | A27 |
| Incompatible schema/collector rollout | Old workers corrupt new state | Version gates, expand/contract, canary and rollback limits | A19 |
| Clock uncertainty | False lease/deadline/source ordering | Distinct monotonic/wall clocks, uncertainty budgets, safe admission closure | A06 |
| Synthetic test mistaken for AWS evidence | Premature production release | Qualification states and immutable source/test provenance | A24/A28 |
| Benchmark drops coverage/durability | False fastest claim | Frozen equal-work manifest plus correctness oracle | A29 |

## Dominant systemic risk

The most likely coupled failure is a provider throttle or database slowdown that
creates event backlog, consumes reconciliation capacity, and makes old inventory
look healthy. The countermeasure is a single visible contract across intake,
durable work, observed state and query projection: bounded admission, age and
deadline metrics, fair scheduling, explicit stale state, and recovery drills.
Increasing worker count without respecting quota and storage feedback can worsen
this failure. The spec must forbid that automatic response.
