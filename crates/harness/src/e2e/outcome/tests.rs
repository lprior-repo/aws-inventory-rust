use super::*;

const COMPLETE: &str = "resource_writes_scope=invocation resource_writes=6 rows=6 complete=8 partial=0 unsupported=0 failed=0 work_remaining=0 graph_pending=0";
const EMULATOR: &str = "resource_writes_scope=invocation resource_writes=96 rows=96 complete=141 partial=0 unsupported=19 failed=5 work_remaining=0 graph_pending=0";

#[test]
fn given_collector_exit_table_when_classified_then_only_completed_invocations_pass() {
    [Some(0), Some(2)]
        .iter()
        .for_each(|code| assert!(accept(*code, EMULATOR).is_ok()));
    [Some(1), Some(3), Some(4), Some(5), Some(9), None]
        .iter()
        .for_each(|code| assert!(accept(*code, COMPLETE).is_err()));
}

#[test]
fn given_usage_or_incomplete_summary_when_exit_is_two_then_invocation_still_fails() {
    [
        "usage: collector",
        &EMULATOR.replace("partial=0", "partial=1"),
        &EMULATOR.replace("work_remaining=0", "work_remaining=1"),
        &EMULATOR.replace("graph_pending=0", "graph_pending=1"),
    ]
    .iter()
    .for_each(|summary| assert!(accept(Some(2), summary).is_err()));
    assert!(
        accept(
            Some(0),
            &COMPLETE.replace("work_remaining=0", "work_remaining=1")
        )
        .is_err()
    );
    assert!(accept(Some(0), COMPLETE).is_ok());
}
