# Capacity and performance acceptance contract

These are qualification targets, not observed system capacity. The existing
release stage baseline is historical context, not evidence that this CMDB passes.

## Required clocks and denominators

T0 = provider change time where available; T1 = authenticated server receipt
before queue admission; T2 = durable receipt; T3 = accepted refreshed observation;
T4 = required SQL/relationship projection visible; T5 = API response; T6 = browser
frame reflecting the change. Preserve clock uncertainty and distinguish source
capture from local observation. Do not restart the clock after durable queueing.

The five-second deadline is T4-T1 for accepted valid supported change hints.
Target at least 99.9 percent within five seconds over the healthy declared steady
workload, while reporting every miss and also p50/p95/p99/max. Unknown mappings,
refresh failures and incomplete results never count as successful fresh state.
Provider T1-T0 is reported separately, including unavailable timestamps. Graph
or provider outage is a visible freshness breach; it is not silently excluded
from the operational dashboard even when fault-mode results are analyzed separately.

Every expected supported scope must finish complete successful reconciliation
at least every 3,600 seconds, including empty scopes. Publish the entire fleet
denominator, failed/denied/unsupported/product-gap counts, last complete time,
and maximum overdue age. This is an aggressive product objective; insufficient
permissions or external AWS outages must remain visible violations/limitations,
not guaranteed freshness invented by software. Config capture age remains separate.

## Frozen workload profiles

| Profile | Population and purpose |
| --- | --- |
| Functional | 2 accounts, cross-account/shared identities, global/regional/parent resources, exact expected state and faults |
| Pilot | 20 then 100 explicitly authorized real accounts; record actual regions, population and quotas |
| Fleet qualification | 3,000 simulated accounts, 10 million resources, 30 million edges, all accepted collector definitions and declared regional/global scope populations |
| Stress characterization | 10,000 simulated accounts; increase resource/edge and change density explicitly until the supported envelope is established |
| UI/API | Same 10-million-resource read model; bounded view 5,000 nodes/20,000 edges; lookup/filter/one-hop and historical queries with declared selectivity |

The fleet fixture includes account skew (top one percent owns half the resources),
empty accounts/regions, all enabled-region rules, high-degree nodes, global and
parent-only resources, realistic payload-size percentiles, tags/policy enrichment,
alias collisions and unresolved references. Publish counts per account/region/
collector and deterministic seeds. Resource count alone is not a workload spec.

Proposed steady change rate: 1,000 valid changes/second across the fleet; burst:
10,000/second for five minutes. Run full hourly reconciliation, interactive
queries, graph projection, history retention and normal observability concurrently.
Publish hot-quota-domain concentration rather than hiding skew with one bucket
per request. Qualify backpressure/recovery separately when offered load exceeds
the declared envelope. These rates are targets to test, not current capacity.

## Performance budgets

- Receipt-to-query-visible: five-second contract above; no errors counted fresh.
- Hourly reconciliation: zero unexplained deadline misses in the healthy fleet
  profile; report bounds and any source/provider limits per collector.
- Indexed lookup/filter/one-hop: p95 at most 250 ms, p99 at most 1 second;
  interactive maximum 5-second statement budget and explicit timeout.
- UI bounded view: p95 first useful view at most 2 seconds, frame time at most
  16.7 ms, input response at most 100 ms. Reference client: 4+ CPU cores, 16 GiB
  RAM, WebGL2-capable integrated graphics; network shaped to 50 Mbit/s and 40 ms
  round trip. Record actual browser, GPU, viewport, labels and rendering settings.
- Backlog: finite configured count/bytes; after a finite burst it must decrease
  and reach the predeclared healthy threshold within a measured recovery window.
  Freeze that window from load/capacity arithmetic before accepting the run.
- Memory, CPU, API attempts, write/WAL amplification, bytes, disk growth and total
  infrastructure/provider costs: report per stage and per valid changed resource.

## Capacity arithmetic before deployment

Estimate hourly calls as sum over account/region/collector of enumeration pages
plus detail/tag/policy/relationship calls and expected retries. Include known-empty
populations, parent fanout and global work. Divide by 3,600 to get the minimum
sustained rate, then compare each actual quota domain and request latency to
required active concurrency. Add event refresh without double-counting coalesced
work. If a domain's required calls exceed its safe quota, the spec is infeasible
for that population until collection strategy, quota or scope changes explicitly.

Estimate storage from actual serialized observation/edge sizes, update churn,
retention, indexes, WAL, replicas, backups, archives, and simultaneous rebuilds.
An hourly unchanged enumeration need not duplicate identical bodies, but must
retain observation/coverage timing evidence. Content deduplication must preserve
source, visibility, schema, ordering and per-epoch evidence.

## Measurement protocol

Freeze source/binary/lockfile hashes, toolchain, hardware/topology/cost, filesystem,
durability/replication, quotas, fields/relationships, query suite, capture age,
cache mode, dataset seed, error distribution and observability. Warm up deliberately;
run enough independent repetitions and request samples for tail estimates;
randomize/interleave comparison order. Retain raw histograms, counts and errors.
Include cold bootstrap, warm steady state, mixed contention, faults and recovery.

Compare released unmodified Steampipe with both cold and allowed-cache queries,
and CloudQuery with full, supported incremental, and supported event-driven modes.
Use identical equivalent AWS populations, fields, permissions, freshness and
durability. A preloaded stale database versus fresh API query is not equal work.
Do not have competitors consume the same AWS quotas simultaneously. Verify final
resources, fields, relationships, deletions, source times and scope outcomes with
an independent oracle before accepting the faster result.

Profile CPU, allocation, async waiting, SQL/locks/I/O, payload and GPU stages
before optimization. Keep an optimization only with passing behavior evidence
and repeatable before/after improvement. Do not disable checks/durability, bypass
quotas, omit hard scopes or remove observability to meet a target. Claim only
named measured workloads/competitors, never an unsupported market-wide maximum.
