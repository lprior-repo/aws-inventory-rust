use super::*;
use crate::overlay::{IdentityBranch, IdentityPredicate, MemberPath, OperationName};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn context(
    operation: &str,
    source: ContextSource,
) -> Result<IdentityField, crate::overlay::NameError> {
    Ok(IdentityField::Context {
        operation: OperationName::new(operation)?,
        source,
        path: MemberPath::new("Name")?,
    })
}

#[test]
fn given_additional_selection_when_identity_asks_for_current_discovery_row_then_ambiguous_correlation_is_rejected()
-> TestResult {
    let identity =
        Identity::CorrelatedKey(vec![context("GetThings", ContextSource::SelectedItem)?]);
    assert!(matches!(
        validate(&identity, "GetThings"),
        Err(RuntimeCatalogCompileError::Path { .. })
    ));
    Ok(())
}

#[test]
fn given_explicit_selected_request_response_or_ancestor_sources_when_validated_then_independent_identities_remain_available()
-> TestResult {
    [
        IdentityField::Selected(MemberPath::new("Name")?),
        IdentityField::Request(MemberPath::new("Name")?),
        context("GetThings", ContextSource::Response)?,
        context("GetThings", ContextSource::Request)?,
        context("ListParents", ContextSource::SelectedItem)?,
    ]
    .into_iter()
    .try_for_each(|field| -> TestResult {
        assert!(matches!(
            validate(&Identity::CorrelatedKey(vec![field]), "GetThings"),
            Ok(())
        ));
        Ok(())
    })?;
    Ok(())
}

#[test]
fn given_prefer_arn_fallback_when_its_fields_request_the_current_discovery_row_then_the_hidden_ambiguous_arm_is_rejected()
-> TestResult {
    let identity = Identity::Choice(IdentityChoice::PreferArn {
        path: MemberPath::new("Arn")?,
        fallback: IdentityLeaf::Fields(vec![context("GetThings", ContextSource::SelectedItem)?]),
        review: "reviewed".to_owned(),
    });
    assert!(matches!(
        validate(&identity, "GetThings"),
        Err(RuntimeCatalogCompileError::Path { .. })
    ));
    Ok(())
}

#[test]
fn given_discriminated_identity_when_a_later_arm_requests_current_discovery_row_then_that_ambiguous_arm_is_rejected()
-> TestResult {
    let identity = Identity::Choice(IdentityChoice::Discriminated(vec![
        IdentityBranch::new(
            IdentityPredicate::Present(MemberPath::new("Arn")?),
            IdentityLeaf::Arn(MemberPath::new("Arn")?),
        )?,
        IdentityBranch::new(
            IdentityPredicate::Present(MemberPath::new("Name")?),
            IdentityLeaf::Fields(vec![context("GetThings", ContextSource::SelectedItem)?]),
        )?,
    ]));
    assert!(matches!(
        validate(&identity, "GetThings"),
        Err(RuntimeCatalogCompileError::Path { .. })
    ));
    Ok(())
}
