# Kickoff prompt for a worker

> Status: Replaces `NEXT_AGENT_PROMPT.txt` for the end-to-end milestone. Give
> the block below to a worker together with one package ID.

```text
Continue the Rust AWS inventory project in this directory. Your package is: <ID>.

Read, in this order: docs/e2e/README.md, docs/e2e/ARCHITECTURE.md,
docs/e2e/DECISIONS.md, docs/e2e/BUILD_PLAN.md, your step in docs/e2e/build/,
then docs/VISION.md. For a step that touches tables, also read
docs/e2e/DATA_MODEL.md. For a UI step also read
docs/e2e/MAKEPAD_UI.md, and read the pinned Makepad source for any call you
use; do not write Makepad code from memory. Where DECISIONS and VISION
differ, DECISIONS wins. docs/ROADMAP.md is background; its order is replaced by
docs/e2e/README.md until the exit test there passes.

What we are building first: one run, from one CLI command or one UI action,
that seeds the pinned AWS emulator, sweeps every callable operation of every
service including child resources, stores typed SQL tables in PostgreSQL,
projects the graph
and renders the whole account in the Makepad UI at 120 frames per second.

Owner's rules:
- End to end first. Do not widen scope beyond your package.
- Everything in the account, children included. Never content, secrets or paid
  calls. The deny list is applied first and always wins. A rule may deny on a
  name and may never allow on one.
- Collection goes through our wire engine, driven by the pinned AWS API models.
  No new typed AWS SDK service crates.
- A service is data (models plus overlay), never Rust.
- Cloud Control is a cross-check only. It is never on the collection path and
  never a source of stored facts.
- No JSON on the hot path unless a recorded measurement shows it is needed.
- Performance is the critical part of this product. If your step is on the
  sweep, write, read or render path it has a budget in the build plan: record
  the number before and after your change with the named benchmark command,
  from a release build. A regression needs a written reason or the step does
  not land. A performance claim without a measurement is not made.
- The database has two roles. The collector writes; the reader only reads.
  SQL typed by a person, and every read, uses the reader. Never hand a
  writable connection to a read path.
- Durable by default: synchronous commits and logged tables. A relaxed
  setting is an option only after the database-crash test (B4.6) passes
  with it.
- Missing is not absent. Only a complete enumeration or an authoritative
  not-found counts toward deletion.
- PostgreSQL is the only store. Build for PostgreSQL only. The current store's
  code is left alone until step C6.12 deletes it; add nothing to it.
- Collection is read-only. The write capability used for seeding exists only
  in the harness crate.
- The mutation gate is suspended. Tests and both reviews are still required.
- Everything is in one private repository for now. UI and graph crates may
  depend on collector, store and SQL crates, never the reverse.
- Python under scripts/ and catalog/upstream/ is frozen. Do not extend, port or
  plan around it.

Engineering rules:
- Workspace lints apply: no unsafe, unwrap, expect, panic, todo, unchecked
  indexing or lossy casts. Files at or under 1000 lines, functions under 60
  lines. No Rust comments.
- Every queue, batch, retry, response and traversal is bounded. No detached
  tasks. ApiExecutor is the only retry owner.
- State that matters lives in the store. A killed process resumes.
- Take numbers from the tree and the catalog, not from documents.
- Prove behavior against the pinned emulator with real signed requests, in the
  same owned process lifetime as the test. An operation is emulator-verified
  only when it returned at least one seeded resource.
- Implemented, emulator-verified and live-verified are separate facts.

How work lands: unchanged from VISION section 10. You implement in your own
worktree and do not run Cargo; the coordinator integrates, runs the narrow
checks, obtains the code review and the test review, and sends findings back
as one repair packet. Land with explicit paths.

Finish the package to its exit evidence. Mark it done in its track file with
the command that proved it, state the Rust lines added and deleted, update the
documents your change makes wrong in the same change, and report exact
commands, results, failed attempts and what was not run.
```

## Packages that can start today

No package below waits on another.

| Package | Track file |
| --- | --- |
| A0 catalog gate (first: every later catalog change needs it) | [A_COVERAGE.md](A_COVERAGE.md) |
| A4 seed files | [A_COVERAGE.md](A_COVERAGE.md) |
| B5 benchmark harness, then B0 baseline; B1 buckets | [B_SPEED.md](B_SPEED.md) |
| C5 store interface (first in its track; C6, the PostgreSQL store, follows it) | [C_SQL.md](C_SQL.md) |
| D1 sweep library, D4 bulk load | [D_LIBRARY_UI.md](D_LIBRARY_UI.md) |
| E3 graph interface | [E_GRAPH.md](E_GRAPH.md) |

B3 and D1 both change `src/sweep_runtime/`. Give them to one worker, or land
D1 first. The wave order in [BUILD_PLAN.md](BUILD_PLAN.md) is authoritative
where it differs from this table.
