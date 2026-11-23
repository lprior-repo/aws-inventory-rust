use inventory_core::record::{FieldId, MemberExclusions, RecordError};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn given_empty_policy_when_matching_then_no_member_is_excluded() -> TestResult {
    assert!(!MemberExclusions::default().contains(FieldId::new(1)?));
    Ok(())
}

#[test]
fn given_eight_members_when_matching_then_only_reviewed_ids_are_excluded() -> TestResult {
    let ids = (1..=8).map(FieldId::new).collect::<Result<Vec<_>, _>>()?;
    let exclusions = MemberExclusions::new(ids)?;
    for id in 1..=8 {
        assert!(exclusions.contains(FieldId::new(id)?));
    }
    assert!(!exclusions.contains(FieldId::new(9)?));
    Ok(())
}

#[test]
fn given_nine_distinct_members_when_constructing_then_policy_is_rejected() -> TestResult {
    let ids = (1..=9).map(FieldId::new).collect::<Result<Vec<_>, _>>()?;
    assert!(matches!(
        MemberExclusions::new(ids),
        Err(RecordError::SizeLimit)
    ));
    Ok(())
}

#[test]
fn given_duplicate_members_when_constructing_then_duplicates_do_not_consume_capacity() -> TestResult
{
    let field = FieldId::new(1)?;
    assert_eq!(
        MemberExclusions::new([field; 9])?,
        MemberExclusions::new([field])?
    );
    Ok(())
}
