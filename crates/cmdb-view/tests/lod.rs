mod support;

use cmdb_view::{LodLimits, ViewError};

#[test]
fn counts_resources_per_kind_and_account() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let summary = snapshot.lod(LodLimits::default());
    assert_eq!(summary.total_resources, 4);
    let kinds: Vec<(&str, usize)> = summary
        .kinds
        .iter()
        .map(|bucket| (bucket.key.as_str(), bucket.count))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("aws_ec2_instance", 2),
            ("aws_iam_role", 1),
            ("aws_s3_bucket", 1)
        ]
    );
    let accounts: Vec<(&str, usize)> = summary
        .accounts
        .iter()
        .map(|bucket| (bucket.key.as_str(), bucket.count))
        .collect();
    assert_eq!(
        accounts,
        vec![(support::ACCOUNT_A, 2), (support::ACCOUNT_B, 2)]
    );
    assert!(!summary.truncated_kinds);
    assert!(!summary.truncated_accounts);
    Ok(())
}

#[test]
fn top_degree_is_ranked_and_capped() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = support::snapshot()?;
    let summary = snapshot.lod(LodLimits::default());
    let ranked: Vec<(&str, usize)> = summary
        .top_degree
        .iter()
        .map(|entry| (entry.id, entry.degree))
        .collect();
    assert_eq!(
        ranked,
        vec![
            (support::INSTANCE_ONE, 3),
            (support::INSTANCE_TWO, 3),
            (support::BUCKET, 2),
            (support::ROLE, 1)
        ]
    );
    assert_eq!(
        summary.top_degree.first().map(|entry| entry.kind),
        Some("aws_ec2_instance")
    );

    let capped = snapshot.lod(LodLimits::new(1, 1, 2)?);
    assert_eq!(capped.kinds.len(), 1);
    assert_eq!(capped.accounts.len(), 1);
    assert_eq!(capped.top_degree.len(), 2);
    assert!(capped.truncated_kinds);
    assert!(capped.truncated_accounts);
    assert!(capped.truncated_top_degree);
    Ok(())
}

#[test]
fn rejects_invalid_limits() {
    assert!(matches!(
        LodLimits::new(0, 1, 1),
        Err(ViewError::InvalidLodLimit)
    ));
    assert!(matches!(
        LodLimits::new(1, 0, 1),
        Err(ViewError::InvalidLodLimit)
    ));
    assert!(matches!(
        LodLimits::new(1, 1, 0),
        Err(ViewError::InvalidLodLimit)
    ));
    assert!(matches!(
        LodLimits::new(1, usize::MAX, 1),
        Err(ViewError::InvalidLodLimit)
    ));
}
