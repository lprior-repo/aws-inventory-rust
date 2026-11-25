use super::*;
use crate::overlay::{MemberPath, NameError};

fn overlay(count: usize) -> Result<OperationOverlay, NameError> {
    let exclude_response_members = (0..count)
        .map(|index| MemberPath::new(&format!("Member{index}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OperationOverlay {
        exclude_response_members,
        ..OperationOverlay::default()
    })
}

#[test]
fn given_nine_excluded_members_when_bounded_then_the_exclusion_list_is_named()
-> Result<(), NameError> {
    let result = bounded(&overlay(9)?);

    assert!(matches!(
        result,
        Err(RuntimeCatalogCompileError::Path { path }) if path == "exclude_response_members"
    ));
    Ok(())
}

#[test]
fn given_eight_excluded_members_when_bounded_then_the_list_is_accepted() -> Result<(), NameError> {
    assert!(bounded(&overlay(8)?).is_ok());
    Ok(())
}
