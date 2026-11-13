use std::error::Error;

use super::*;
use crate::dist;
use crate::generator::generate;
use crate::options::{Options, SizeSpec};
use crate::verify::verify;

fn options(accounts: usize, size: SizeSpec) -> Options {
    Options {
        accounts,
        size,
        seed: 7,
        empty_regions: 0,
        global_share: 0,
        payload_bytes: 16,
        duplicate_pct: 0,
        reorder_pct: 0,
        deletion_bursts: 0,
        deletion_burst_size: 0,
    }
}

#[test]
fn streaming_generation_is_byte_deterministic() -> Result<(), Box<dyn Error>> {
    let left = tempfile::tempdir()?;
    let right = tempfile::tempdir()?;
    let mut options = options(5, SizeSpec::AssetsTotal { assets_total: 900 });
    options.empty_regions = 1;
    options.global_share = 40;
    options.duplicate_pct = 10;
    options.reorder_pct = 10;
    options.deletion_bursts = 2;
    options.deletion_burst_size = 3;
    let first = generate(left.path(), options)?;
    let second = generate(right.path(), options)?;
    assert_eq!(first, second);
    assert_eq!(first.resources, 900);
    assert_eq!(first.page_size, 100);
    assert_eq!(first.deletions.count, 6);
    assert_same_files(left.path(), right.path())?;
    assert_eq!(verify(left.path())?, first);
    Ok(())
}

fn assert_same_files(
    left: &std::path::Path,
    right: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    for name in ["events.jsonl", "deletions.jsonl", "oracle.json"] {
        let left_bytes = std::fs::read(left.join(name))?;
        let right_bytes = std::fs::read(right.join(name))?;
        assert_eq!(
            left_bytes, right_bytes,
            "{name} differs between identical runs"
        );
    }
    Ok(())
}

#[test]
fn uniform_mode_keeps_the_existing_command_working() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let oracle = generate(
        root.path(),
        options(3, SizeSpec::Uniform { per_account: 12 }),
    )?;
    assert_eq!(oracle.accounts, 3);
    assert_eq!(oracle.resources, 36);
    assert!(oracle.edges > 0);
    assert_eq!(oracle.per_account.len(), 3);
    assert_eq!(verify(root.path())?, oracle);
    Ok(())
}

#[test]
fn skewed_distribution_sums_exactly_and_is_tail_heavy() -> Result<(), Box<dyn Error>> {
    let options = options(
        200,
        SizeSpec::AssetsTotal {
            assets_total: 50_000,
        },
    );
    let sizes = dist::account_sizes(&options)?;
    assert_eq!(sizes.len(), 200);
    assert_eq!(sizes.iter().copied().sum::<usize>(), 50_000);
    assert!(sizes.iter().all(|size| *size >= 1));
    let largest = sizes.iter().copied().max().ok_or("empty sizes")?;
    let smallest = sizes.iter().copied().min().ok_or("empty sizes")?;
    assert!(largest > smallest.saturating_mul(10));
    assert_eq!(dist::account_sizes(&options)?, sizes);
    let root = tempfile::tempdir()?;
    let oracle = generate(root.path(), options)?;
    assert_eq!(oracle.resources, 50_000);
    assert_eq!(verify(root.path())?, oracle);
    Ok(())
}

#[test]
fn empty_regions_are_reported_and_never_populated() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let mut options = options(10, SizeSpec::Uniform { per_account: 7 });
    options.empty_regions = 2;
    let oracle = generate(root.path(), options)?;
    assert_eq!(
        oracle.empty_regions,
        vec!["empty-1".to_owned(), "empty-2".to_owned()]
    );
    for name in &oracle.empty_regions {
        assert_eq!(
            oracle.regions.get(name).copied().ok_or("missing region")?,
            0
        );
    }
    let text = std::fs::read_to_string(root.path().join("events.jsonl"))?;
    assert!(!text.contains("\"region\":\"empty-"));
    assert_eq!(verify(root.path())?, oracle);
    Ok(())
}

#[test]
fn duplicate_and_reordered_events_keep_oracle_state() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let mut options = options(3, SizeSpec::AssetsTotal { assets_total: 240 });
    options.duplicate_pct = 100;
    options.reorder_pct = 100;
    let oracle = generate(root.path(), options)?;
    assert!(oracle.duplicates > 0);
    assert!(oracle.reorders > 0);
    let text = std::fs::read_to_string(root.path().join("events.jsonl"))?;
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.windows(2).any(|pair| pair.first() == pair.get(1)));
    assert_eq!(verify(root.path())?, oracle);
    Ok(())
}

#[test]
fn deletion_oracle_detects_mismatch() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let mut options = options(4, SizeSpec::AssetsTotal { assets_total: 400 });
    options.deletion_bursts = 3;
    options.deletion_burst_size = 5;
    let oracle = generate(root.path(), options)?;
    assert_eq!(oracle.deletions.count, 15);
    assert_eq!(verify(root.path())?, oracle);
    let path = root.path().join("deletions.jsonl");
    let text = std::fs::read_to_string(&path)?;
    let mut lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() > 1);
    lines.remove(0);
    std::fs::write(&path, format!("{}\n", lines.join("\n")))?;
    assert!(verify(root.path()).is_err());
    Ok(())
}

#[test]
fn changed_events_fail_verification() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    generate(
        root.path(),
        options(1, SizeSpec::Uniform { per_account: 5 }),
    )?;
    let path = root.path().join("events.jsonl");
    let text = std::fs::read_to_string(&path)?;
    let (_, rest) = text.split_once('\n').ok_or("workload has no events")?;
    std::fs::write(&path, rest)?;
    assert!(verify(root.path()).is_err());
    Ok(())
}

#[test]
fn global_services_are_seeded_and_verifiable() -> Result<(), Box<dyn Error>> {
    let root = tempfile::tempdir()?;
    let mut options = options(
        50,
        SizeSpec::AssetsTotal {
            assets_total: 5_000,
        },
    );
    options.global_share = 40;
    let oracle = generate(root.path(), options)?;
    assert!(oracle.global_services > 0);
    assert!(oracle.global_services < oracle.accounts);
    let global = oracle
        .regions
        .get("global")
        .copied()
        .ok_or("global region")?;
    assert!(global >= oracle.global_services);
    assert_eq!(verify(root.path())?, oracle);
    Ok(())
}

#[test]
fn cli_rejects_contradictory_modes() {
    let contradictory = vec![
        "workload".to_owned(),
        "--accounts".to_owned(),
        "2".to_owned(),
        "--per-account".to_owned(),
        "5".to_owned(),
        "--assets-total".to_owned(),
        "10".to_owned(),
    ];
    assert!(run(contradictory).is_err());
}

#[test]
fn cli_rejects_bad_bounds() {
    let mut opts = options(1, SizeSpec::Uniform { per_account: 1 });
    opts.global_share = 101;
    assert!(opts.validate().is_err());

    let opts = options(10, SizeSpec::AssetsTotal { assets_total: 9 });
    assert!(opts.validate().is_err());
    assert!(
        options(0, SizeSpec::Uniform { per_account: 1 })
            .validate()
            .is_err()
    );

    let mut opts = options(1, SizeSpec::Uniform { per_account: 1 });
    opts.deletion_bursts = 1;
    assert!(opts.validate().is_err());

    let mut opts = options(1, SizeSpec::Uniform { per_account: 1 });
    opts.payload_bytes = crate::options::MAX_PAYLOAD_BYTES.saturating_add(1);
    assert!(opts.validate().is_err());
}
