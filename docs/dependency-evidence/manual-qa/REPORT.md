# Manual invocation QA

## Target

Binary: `/home/lewis/Documents/Codex/2026-11-12/let/aws-inventory-rs/target/debug/aws-inventory`

Version output (`version.stdout`):

```text
aws-inventory 0.6.0
```

Verified SHA256 before execution and after completion:
`8fe9df4f994c84de546cf3f2c1614895f17ff9cab0fdfaa3e533aef02bf867e8`.
No Rust builds or source edits were performed during manual QA. Databases and exports created by these commands are confined to this QA directory. AWS commands used the authenticated loopback wrapper exclusively.

## Interface surface and evidence

The main help, all 20 listed top-level command helps, and all six job subcommand helps were executed. Their unabridged output is retained in `help*.stdout`. Each invocation has an exact shell-quoted `.command`, verbatim `.stdout` and `.stderr`, and numeric `.result.json`. `results.json` contains all 120 records; `matrix.tsv` is the compact exit-code matrix.

This is a manual workflow review, not a claim that every value of every flag has been exhaustively tested. Parent workspace/live tests and the separate Red Queen campaign cover additional cases. Role assumption, real AWS accounts, abrupt process death/lease expiry, and every numeric upper boundary were not exercised manually here.

## Executed workflows

- SDK catalog: all 437 entries, November 12 release/revision identity, EC2 1.279.0 and default capability metadata, S3 package-name filter, and unknown-service rejection. General catalog reports 32 native kinds and the AWS feature enabled. SDK availability remains distinct from collection coverage.
- Synthetic demo: seven resources/four edges; report, documented SQL view, architecture, findings, self-diff, bounded neighbors/path, and JSON plus all-observation JSONL exports.
- Durable queue: submit, identical replay, conflicting specification refusal, listing/show, queued cancellation without an attempt, empty-worker return, explicit retry, native local EC2 execution, persisted successful attempt, and subsequent empty-worker return. Native worker report and direct SQLite counts agree on 62 resources, 127 edges, 12 complete scopes, zero failed scopes, one scan.
- Queue failures: zero/oversized claim limits, too-short lease, oversized list limit, missing job/queue, zero retry allowance, zero initial attempt budget, excessive delay, same queue/inventory path, invalid service, successful-job retry, and queue/inventory application identity collisions.
- Offline intent: a delayed Unicode job ID with a nonexistent credential profile was submitted/shown/cancelled without creating its inventory file or resolving that profile. Delayed worker executed zero jobs.
- Durable running cancellation: waited until a real worker held the job, requested cancellation, observed persisted cancelled attempt and zero API calls in flight. Explicitly retried and repeated with the correct cancelled exit-code expectation. Both attempts remain in history.
- Worker signal: GNU timeout sent SIGINT to the task-owned worker after one second, preserving its process status. The worker returned JSON cancellation with exit 2, persisted a cancelled attempt, drained API in-flight count to zero, and the next worker found an empty queue.
- Local discovery: existing Config aggregator discovery completed with one selected candidate, two successful API attempts, and zero calls in flight. Invalid pagination was rejected before AWS execution.
- Local stack: readiness was required before AWS commands; final health returned READY. Main PID and loopback endpoints are retained in `stack-health-final.stdout`.

## Exact evidence excerpts

`query-worker-counts.stdout`:

```json
[
  {
    "edges": 127,
    "resources": 62,
    "scans": 1
  }
]
```

`job-run-signal.stdout`:

```json
{
  "executed": 1,
  "jobs": [
    {
      "attempt": 1,
      "error": null,
      "error_truncated": false,
      "job_id": "qa-worker-signal",
      "result_available": true,
      "result_truncated": false,
      "scan_count": 1,
      "state": "cancelled"
    }
  ],
  "mode": "bounded worker invocation; no background daemon remains",
  "results": "Terminal results are retained by job ID in the queue; oversized results declare result_truncated. This response contains bounded summaries.",
  "status": "cancelled",
  "stop_reason": "cancelled_job",
  "worker": "manual-qa-signal"
}
```

`signal-drain-invariant.stdout`:

```text
true
```

That check requires the actual persisted job and attempt to be cancelled, attempt count one, and retained result API in-flight count zero. It does not infer cleanup solely from the exit code.

## Findings and expectation corrections

120 invocations were captured: 115 matched their initial expected exit and five did not. Original evidence was retained without rewriting expected values.

| Original record | Actual behavior | Disposition and confirming evidence |
| --- | --- | --- |
| `query` | Exit 1: `no such table: resources` | QA command used a nonexistent table. Corrected documented `aws_ec2_instance` and actual `latest_resources` queries both exit 0 (`query-instances`, `query-resources`). No product defect. |
| `job-run-running-cancel` | Exit 2 with cancelled JSON and persisted cancelled attempt | Initial QA expectation of zero was incorrect. CLI intentionally returns 2 for partial/cancelled output. Repeated cancellation (`job-run-cancel-confirm`) and SIGINT (`job-run-signal`) both matched expected 2 and passed persisted drain/history checks. No product defect. |
| `stack-status`, `stack-status-second`, `stack-status-third` | Exit 1 during fixture initialization; first two also printed a curl CA argument error from prior-session environment | AWS work was held until READY. Parent corrected startup session/PID checks in the stack wrapper; subsequent `stack-status-ready` and `stack-health-final` return 0/READY. Corrected with observed readiness evidence; no Rust binary change. |

No crash or unresolved functional failure was observed in the executed workflows. This report does not grant blanket architectural, mutation, formal-verification, or all-interface acceptance.

## Developer toolbox

Inspect the retained `.command`, `.stdout`, `.stderr`, and `.result.json` files for any record ID. `jq` checks, `timeout` supervision, direct CLI invocation, `sha256sum`, `file`, `ldd`, and the local stack health command were used. All task-owned database files and exports are retained for inspection; no external credentials were printed or modified.
