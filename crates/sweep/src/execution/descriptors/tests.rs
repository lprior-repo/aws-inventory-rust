use super::*;

const TYPICAL: &str = "\
Limit                     Soft Limit           Hard Limit           Units
Max cpu time              unlimited            unlimited            seconds
Max open files            1024                 524288               files";

#[test]
fn given_typical_limits_when_parsing_then_files_value() {
    assert_eq!(parse_soft_open_files(TYPICAL), Some(SoftLimit::Files(1024)));
}

#[test]
fn given_unlimited_line_when_parsing_then_unlimited() {
    let text = "Max open files            unlimited            unlimited            files";
    assert_eq!(parse_soft_open_files(text), Some(SoftLimit::Unlimited));
}

#[test]
fn given_no_max_open_files_line_when_parsing_then_none() {
    let text = "Max cpu time              unlimited            unlimited            seconds";
    assert_eq!(parse_soft_open_files(text), None);
}

#[test]
fn given_non_numeric_soft_value_when_parsing_then_none() {
    let text = "Max open files            many                 524288               files";
    assert_eq!(parse_soft_open_files(text), None);
}

#[test]
fn given_empty_soft_value_when_parsing_then_none() {
    let text = "Max open files";
    assert_eq!(parse_soft_open_files(text), None);
}

#[test]
fn given_hard_limit_unlimited_when_parsing_then_soft_value() {
    let text = "Max open files            1024                 unlimited            files";
    assert_eq!(parse_soft_open_files(text), Some(SoftLimit::Files(1024)));
}

#[test]
fn given_small_concurrency_when_bounded_then_concurrency() {
    assert_eq!(connection_bound(16, Some(SoftLimit::Files(1024))), 16);
}

#[test]
fn given_large_concurrency_when_bounded_then_cap() {
    assert_eq!(connection_bound(1024, Some(SoftLimit::Files(1024))), 768);
}

#[test]
fn given_tight_limit_when_bounded_then_one() {
    assert_eq!(connection_bound(64, Some(SoftLimit::Files(256))), 1);
    assert_eq!(connection_bound(64, Some(SoftLimit::Files(100))), 1);
}

#[test]
fn given_unlimited_limit_when_bounded_then_concurrency() {
    assert_eq!(connection_bound(4096, Some(SoftLimit::Unlimited)), 4096);
}

#[test]
fn given_missing_limit_when_bounded_then_fallback() {
    assert_eq!(connection_bound(4096, None), 256);
    assert_eq!(connection_bound(16, None), 16);
}

#[test]
fn given_zero_concurrency_when_bounded_then_one() {
    assert_eq!(connection_bound(0, Some(SoftLimit::Unlimited)), 1);
}

#[test]
fn given_linux_environment_when_reading_then_some() {
    assert!(soft_open_files().is_some());
}
