# Native local GPU implementation audit

The user explicitly approved using the installed OMP CLI as a bridge because
this Codex chat does not expose OMP's native task tool. OMP18.20.0 dispatched
three substantive Rust SQL-policy implementation/correction jobs through native
`task`, each with explicit model
`local-3090-llama/qwen3.8-27b-uncensored:high`. The coordinator used the local5090
lane for orchestration. No cloud worker was relabelled as a3090 worker.

All three worker transcripts record model identity
`local-3090-llama/qwen3.8-27b-uncensored` and
`resolvedModelIsFallback:false`. Provider configuration, the11001 listener PID,
and the same PID's RTX3090 allocation were checked locally. See
`hardware-proof.txt` and the three filtered worker metadata JSON files.
The initial malformed `agent` argument was rejected before inference; the
coordinator corrected it and dispatched the actual native task. That preflight
failure is not counted as GPU implementation work.

Workers had exclusive ownership of the SQL-policy module in the isolated
`work/local-gpu-candidate` checkout. They performed actual Rust writes. The
coordinator preserved every complete submission before formatting/verification
and kept the production checkout on the working policy. Workers ran no builds,
tests, linters, formatters, Python validators, or publication commands. Captured
shell commands were filesystem operations and source inspection only.

| Submission | Actual coordinator result | Disposition |
| --- | --- | --- |
| Rust3090SqlPolicy | Two public behavior tests failed; extended strict Clippy reported23 errors | REJECTED |
| Rust3090SqlPolicyRepair | Three expanded public behavior tests failed; strict Clippy reported two string-slice panic surfaces; required reference reads were also incomplete | REJECTED |
| Rust3090SqlPolicyCorrection | Compilation failed with E0282 before tests could execute; scoped source-size/function-size checks alone did not establish correctness | REJECTED |

The last worker successfully read the exact mandatory skills/references before
editing. The earlier repair's nonexistent shortened reference paths and missing
reads remain visible in its filtered metadata; they were not treated as a pass.

**No rejected parser implementation was integrated or pushed to main.** The
production SQL policy passed the coordinator's frozen three-test bank covering45
consumer-visible cases: quoted values/identifiers, comment boundaries, completed
statements, exact policy errors, and native syntax-error classification. Those
regressions are now on main in`query_lexical_contract.rs`. Main then passed539
Rust tests with all guarded live tests explicitly enabled.

`rejected-*` retains failed command output and first-submission source hashes.
Complete raw native sessions and preserved source snapshots remain locally under
`work/local-gpu-sessions` and`work/local-gpu-packets/submission-*`; their transcript
hashes are recorded here. Full skill-library contents were not duplicated into
this repository. The isolated code remains rejected WIP for review, not a landed
or accepted contribution. Worker prose claiming completion is not test evidence.
