use crate::{SweepError, record};
use inventory_core::{
    catalog::runtime::ArnResourceKind,
    inventory::{Region, ScopeKey},
    record::{FieldId, Record, Value, ValueRef},
};

pub(super) fn value(
    row: &Record,
    path: &[FieldId],
    expected_service: &str,
    kind: ArnResourceKind,
    scope: &ScopeKey,
) -> Result<Option<Value>, SweepError> {
    let Some(ValueRef::Text(text)) = record::path_value(row.root(), path)? else {
        return Err(SweepError::Operation(
            "reviewed parent ARN is missing or not text",
        ));
    };
    if matches!(kind, ArnResourceKind::SnsSubscription)
        && matches!(text, "PendingConfirmation" | "Deleted")
    {
        return Ok(None);
    }
    validate(text, expected_service, &kind, scope)?;
    Ok(Some(Value::Text(text.to_owned())))
}

fn validate(
    text: &str,
    expected_service: &str,
    kind: &ArnResourceKind,
    scope: &ScopeKey,
) -> Result<(), SweepError> {
    let Region::Regional(region) = scope.region() else {
        return Err(SweepError::Operation(
            "reviewed parent ARN has no trusted request region",
        ));
    };
    let trusted = record::IdentityContext {
        account: scope.account(),
        region,
    };
    if expected_service != record::trusted_arn::resource_service(kind)
        || record::trusted_arn::validate_resource_arn(text, kind, &trusted, None).is_err()
    {
        return Err(SweepError::Operation(
            "reviewed parent ARN is invalid or outside its trusted scope",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
