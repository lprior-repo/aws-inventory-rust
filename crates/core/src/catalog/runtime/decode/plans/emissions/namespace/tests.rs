use super::*;
use crate::{
    catalog::runtime::{IdentityBranchPlan, IdentityPredicatePlan},
    record::FieldId,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
fn field(
    service: &str,
    operation: &str,
    source: ContextSource,
) -> Result<IdentityFieldPlan, crate::record::RecordError> {
    Ok(IdentityFieldPlan::Context {
        service: service.into(),
        operation: operation.into(),
        source,
        path: vec![FieldId::new(1)?].into_boxed_slice(),
    })
}

#[test]
fn given_current_discovery_context_when_extra_identity_decodes_then_implicit_row_pairing_is_rejected()
-> TestResult {
    let identity = IdentityPlan::CorrelatedKey(
        vec![field("service", "GetThings", ContextSource::SelectedItem)?].into_boxed_slice(),
    );
    assert!(matches!(
        validate(&identity, "service", "GetThings"),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_explicit_current_request_response_or_distinct_producer_when_decoded_then_supported_context_is_not_narrowed()
-> TestResult {
    [
        field("service", "GetThings", ContextSource::Request)?,
        field("service", "GetThings", ContextSource::Response)?,
        field("service", "ListParents", ContextSource::SelectedItem)?,
        field("other", "GetThings", ContextSource::SelectedItem)?,
    ]
    .into_iter()
    .try_for_each(|value| -> TestResult {
        let identity = IdentityPlan::CorrelatedKey(vec![value].into_boxed_slice());
        assert!(matches!(
            validate(&identity, "service", "GetThings"),
            Ok(())
        ));
        Ok(())
    })?;
    Ok(())
}

#[test]
fn given_choice_identity_when_ambiguous_fields_hide_in_any_arm_then_decoding_still_rejects()
-> TestResult {
    let fields = IdentityLeafPlan::Fields(
        vec![field("service", "GetThings", ContextSource::SelectedItem)?].into_boxed_slice(),
    );
    let path = vec![FieldId::new(1)?].into_boxed_slice();
    let prefer = IdentityPlan::Choice(IdentityChoicePlan::PreferArn {
        path: path.clone(),
        fallback: fields.clone(),
        review: "reviewed".into(),
    });
    let branches = IdentityPlan::Choice(IdentityChoicePlan::Discriminated(
        vec![IdentityBranchPlan {
            predicate: IdentityPredicatePlan::Present(path),
            identity: fields,
        }]
        .into_boxed_slice(),
    ));
    [prefer, branches].iter().for_each(|identity| {
        assert!(matches!(
            validate(identity, "service", "GetThings"),
            Err(RuntimeCatalogError::Capability)
        ))
    });
    Ok(())
}
