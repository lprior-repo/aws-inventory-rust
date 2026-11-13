use super::*;
use aws_sdk_organizations::types::AccountState;

#[test]
fn account_state_controls_eligibility_and_missing_state_fails_closed() -> Result<(), AwsScanError> {
    let active = Account::builder()
        .id("123456789012")
        .name("Production")
        .state(AccountState::Active)
        .build();
    let suspended = Account::builder()
        .id("223456789012")
        .state(AccountState::Suspended)
        .build();
    let unknown = Account::builder().id("323456789012").build();
    assert!(map_account(active)?.eligible_for_scan);
    assert!(!map_account(suspended)?.eligible_for_scan);
    assert!(!map_account(unknown)?.eligible_for_scan);
    Ok(())
}

#[test]
fn malformed_account_id_is_rejected_and_email_is_not_retained() -> Result<(), AwsScanError> {
    assert!(map_account(Account::builder().id("bad").build()).is_err());
    let account = map_account(
        Account::builder()
            .id("123456789012")
            .email("sensitive@example.test")
            .state(AccountState::Active)
            .build(),
    )?;
    let json = serde_json::to_value(account).map_err(|error| data_error(&error.to_string()))?;
    assert!(json.get("email").is_none());
    Ok(())
}

#[test]
fn root_unit_and_membership_id_types_are_checked() -> Result<(), AwsScanError> {
    assert!(map_root(Root::builder().id("ou-test-unit").build()).is_err());
    assert!(map_root(Root::builder().id("r-").build()).is_err());
    assert!(map_root(Root::builder().id("r-a").build()).is_err());
    assert!(map_root(Root::builder().id("r-root-extra").build()).is_err());
    assert!(validate_node_id("ou-root-a", "ou-").is_err());
    assert!(validate_node_id("ou-root-12345678-extra", "ou-").is_err());
    validate_node_id("ou-root-12345678", "ou-")?;
    assert!(
        map_unit(
            OrganizationalUnit::builder().id("r-root").build(),
            "r-root",
            1
        )
        .is_err()
    );
    assert!(
        map_member(
            Child::builder()
                .id("123456789012")
                .r#type(ChildType::OrganizationalUnit)
                .build()
        )
        .is_err()
    );
    assert_eq!(
        map_member(
            Child::builder()
                .id("123456789012")
                .r#type(ChildType::Account)
                .build()
        )?,
        "123456789012"
    );
    Ok(())
}

fn root() -> OrganizationNode {
    OrganizationNode {
        id: "r-root".into(),
        arn: None,
        name: Some("Root".into()),
        parent_id: None,
        depth: 0,
    }
}

#[test]
fn hierarchy_rejects_cycles_multiple_parents_and_duplicate_memberships() {
    let mut cursor = HierarchyCursor::new(Listing {
        items: vec![root()],
        coverage: DiscoveryCoverage::success(),
    });
    let parent = root();
    let unit = OrganizationNode {
        id: "ou-root-production".into(),
        arn: None,
        name: None,
        parent_id: Some(parent.id.clone()),
        depth: 1,
    };
    cursor.accept_unit(&parent, unit.clone());
    cursor.accept_unit(&parent, unit);
    cursor.accept_member(&parent, "123456789012".into());
    cursor.accept_member(&parent, "123456789012".into());
    assert_eq!(cursor.result.units.len(), 1);
    assert_eq!(cursor.result.memberships.len(), 1);
    assert_eq!(cursor.result.coverage.failures.len(), 2);
    assert!(!cursor.result.coverage.complete);
}

#[test]
fn empty_roots_and_depth_limit_are_explicit_partial_coverage() {
    let empty = HierarchyCursor::new(Listing {
        items: vec![],
        coverage: DiscoveryCoverage::success(),
    });
    assert!(!empty.result.coverage.complete);
    let mut cursor = HierarchyCursor::new(Listing {
        items: vec![root()],
        coverage: DiscoveryCoverage::success(),
    });
    cursor.accept_unit(
        &root(),
        OrganizationNode {
            id: "ou-root-deep".into(),
            arn: None,
            name: None,
            parent_id: Some("r-root".into()),
            depth: MAX_DEPTH.saturating_add(1),
        },
    );
    assert!(cursor.result.units.is_empty());
    assert!(!cursor.result.coverage.complete);
}
