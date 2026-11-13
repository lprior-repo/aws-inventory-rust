# Sources and evidence policy

Primary public sources checked 2026-11-13. Public documentation establishes
documented behavior, not independent competitor benchmarks or our implementation.

| Source | Design relevance |
| --- | --- |
| [Steampipe documentation](https://steampipe.io/docs) | API-backed SQL distributions and interfaces |
| [Steampipe FAQ](https://steampipe.io/docs/faq/overview) | Parallel connections, caching, selected columns and server filtering |
| [CloudQuery syncs](https://www.cloudquery.io/docs/cli/core-concepts/syncs) | Streamed destination writes; full versus supported incremental tables |
| [CloudQuery AWS source](https://www.cloudquery.io/hub/plugins/source/cloudquery/aws/latest/docs) | Current AWS source configuration and supported event-triggered selective sync |
| [CloudQuery parallel execution](https://www.cloudquery.io/docs/cli/managing-cloudquery/running-in-parallel) | Non-overlapping shards and unique source identity |
| [AWS Organizations](https://docs.aws.amazon.com/organizations/latest/userguide/orgs_introduction.html) | Account/OU discovery and enterprise organization boundary |
| [Cloud Control supported resources](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/supported-resources.html) | Per-type handler support rather than universal enumeration |
| [Config aggregation](https://docs.aws.amazon.com/config/latest/developerguide/aggregate-data.html) | Aggregates existing recorded configuration across enabled accounts/regions |
| [EventBridge delivery levels](https://docs.aws.amazon.com/eventbridge/latest/ref/event-delivery-level.html) | Best-effort versus at-least-once service delivery, motivating reconciliation |
| [StackSets auto-deployment](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/stacksets-orgs-manage-auto-deployment.html) | Explicit administrator-enabled onboarding to new OU accounts |
| [PostgreSQL row security](https://www.postgresql.org/docs/current/ddl-rowsecurity.html) | Default-deny policies and owner/superuser/BYPASSRLS caveats |
| [PostgreSQL transaction isolation](https://www.postgresql.org/docs/current/transaction-iso.html) | Actual snapshot/isolation behavior; multi-statement consistency is explicit |
| [PostgreSQL client defaults](https://www.postgresql.org/docs/current/runtime-config-client.html) | Session/search-path/statement controls, not SQL-prefix security |
| [PostgreSQL PITR](https://www.postgresql.org/docs/current/continuous-archiving.html) | Base backups and WAL recovery; restore execution still required |
| [Sigma rendering documentation](https://www.sigmajs.org/concepts/rendering/) | Established GPU graph-rendering approach; not a chosen/benchmarked dependency |

## Local sources

Read the existing architecture, collectors, storage, scheduler, durable jobs,
graph delivery, SDK manifest/auditor, CLI catalog, prior strict review, and release
benchmark artifacts. Exact baseline source is recorded in `current-state.md`.
Catalog source hashes and refreshed SDK-source audit bind the seed registers.
Do not treat an old document's self-description as evidence for the current pin.

## Skills used in specification work

- `arch-design-qa`: requirements/contract/failure/consequence/pre-mortem process;
  finalization remains subject to its explicit hardening boundary.
- `rust-contract`: domain model, type/workflow contracts, errors, boundary map,
  hazards and schema-conforming proof seeds; no proof plan/implementation claim.
- `architectural-drift` and `black-hat-reviewer`: strict structural/DDD review
  criteria carried into release requirements, not a new global code approval.
- Holzmann performance references from the preceding baseline work: measured
  release baselines, bounded resources and no unproven performance claims.

Async, Holzmann, Scott DDD, manual QA, Red Queen, local GPU, proof/testing and
evidence workflows are required future implementation/release gates. Their names
in a document do not mean those agents or tools executed or passed this spec.
