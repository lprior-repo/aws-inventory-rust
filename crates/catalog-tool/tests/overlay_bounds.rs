use catalog_tool::overlay::{
    MAX_FILE_BYTES, MAX_OPERATIONS, MemberPath, OperationName, Rule, ServiceOverlay,
};
pub mod overlay_support;
use overlay_support::{HEADER, TestResult, error, name, parse, reject};

#[test]
fn page_and_batch_sizes_accept_extremes_and_reject_one_past() -> TestResult {
    [1, 100_000].into_iter().try_for_each(|n| -> TestResult {
        let o = parse(&format!("[operation.ListThings]\npage_size={n}"))?;
        assert_eq!(
            o.operations()
                .get(&name("ListThings")?)
                .and_then(|op| op.page_size),
            Some(n)
        );
        Ok(())
    })?;
    [0, -1, 100_001].into_iter().for_each(|n| {
        reject(
            &format!("page_size={n}"),
            "page_size",
            &n.to_string(),
            Rule::Range {
                min: 1,
                max: 100_000,
            },
        )
    });
    [1, 10_000].into_iter().try_for_each(|n| -> TestResult {
        let o = parse(&format!(
            "[operation.ListThings.batch]\nmember='Ids'\nmax={n}\nreplaces='DescribeThing'"
        ))?;
        assert_eq!(
            o.operations()
                .get(&name("ListThings")?)
                .and_then(|op| op.batch.as_ref())
                .map(|b| b.max),
            Some(n)
        );
        Ok(())
    })?;
    [0, -1, 10_001].into_iter().for_each(|n| {
        reject(
            &format!("batch={{member='Ids',max={n},replaces='DescribeThing'}}"),
            "batch.max",
            &n.to_string(),
            Rule::Range {
                min: 1,
                max: 10_000,
            },
        )
    });
    Ok(())
}
#[test]
fn paths_enforce_byte_segment_and_initial_character_bounds() -> TestResult {
    let exact = "A".repeat(256);
    assert_eq!(MemberPath::new(&exact)?.to_string(), exact);
    let over = "A".repeat(257);
    assert_eq!(
        MemberPath::new(&over).err().map(|e| e.value),
        Some(over.clone())
    );
    reject(&format!("items='{over}'"), "items", &over, Rule::MemberPath);
    let eight = std::iter::repeat_n("A", 8).collect::<Vec<_>>().join(".");
    assert_eq!(MemberPath::new(&eight)?.to_string(), eight);
    let nine = format!("{eight}.A");
    reject(&format!("tags='{nine}'"), "tags", &nine, Rule::MemberPath);
    assert_eq!(MemberPath::new("A_0.B9")?.as_str(), "A_0.B9");
    assert_eq!(OperationName::new("A0")?.to_string(), "A0");
    ["", "0A", "A_B", "A.B"]
        .into_iter()
        .for_each(|s| assert_eq!(OperationName::new(s).err().map(|e| e.value), Some(s.into())));
    Ok(())
}
#[test]
fn quota_and_reviewer_byte_limits_are_inclusive() -> TestResult {
    let quota = format!("a{}", "-".repeat(31));
    let o = parse(&format!("[operation.ListThings]\nquota_group='{quota}'"))?;
    assert_eq!(
        o.operations()
            .get(&name("ListThings")?)
            .and_then(|op| op.quota_group.as_deref()),
        Some(quota.as_str())
    );
    let over = format!("{quota}a");
    reject(
        &format!("quota_group='{over}'"),
        "quota_group",
        &over,
        Rule::Value,
    );
    let o = ServiceOverlay::parse("a", "format=1\nreviewed='2026-01-01'\nreviewer='x'")?;
    assert_eq!(o.reviewer(), "x");
    Ok(())
}
#[test]
fn file_and_operation_counts_accept_exact_limit_and_reject_one_past() -> TestResult {
    let padding = MAX_FILE_BYTES
        .checked_sub(HEADER.len())
        .ok_or("header exceeds cap")?;
    let exact = format!("{HEADER}#{}", "x".repeat(padding.saturating_sub(1)));
    assert_eq!(ServiceOverlay::parse("ec2", &exact)?.operations().len(), 0);
    let over = format!("{exact}@");
    error(
        &over,
        None,
        "",
        &MAX_FILE_BYTES.saturating_add(1).to_string(),
        Rule::Limit {
            max: MAX_FILE_BYTES,
        },
    );
    let rows = (0..MAX_OPERATIONS)
        .map(|i| format!("[operation.A{i}]\n"))
        .collect::<String>();
    assert_eq!(parse(&rows)?.operations().len(), MAX_OPERATIONS);
    error(
        &format!("{HEADER}{rows}[operation.Excess]"),
        None,
        "operation",
        &MAX_OPERATIONS.saturating_add(1).to_string(),
        Rule::Limit {
            max: MAX_OPERATIONS,
        },
    );
    Ok(())
}
