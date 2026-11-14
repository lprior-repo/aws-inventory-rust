use crate::request::{EncodeError, RequestContext};
use inventory_core::{catalog::Target, record::ValueRef};

pub(super) fn resolve(context: RequestContext<'_>) -> Result<Option<&str>, EncodeError> {
    if context.service.endpoint_prefix()? != Some("machinelearning")
        || context.operation.name()? != "Predict"
    {
        return Ok(None);
    }
    prediction_endpoint(context).map(Some)
}

fn prediction_endpoint(context: RequestContext<'_>) -> Result<&str, EncodeError> {
    let Target::Shape(index) = context.operation.input()? else {
        return Err(EncodeError::MissingInput);
    };
    let input = context
        .service
        .shape(index)?
        .ok_or(EncodeError::MissingInput)?;
    for entry in input.members()? {
        let member = entry?;
        if member.name()? != "PredictEndpoint" {
            continue;
        }
        return match context.input.get_field(member.id)? {
            Some(ValueRef::Text(endpoint)) => Ok(endpoint),
            _ => Err(EncodeError::InvalidValue {
                field: Some(member.id),
                reason: "missing or non-string prediction endpoint",
            }),
        };
    }
    Err(EncodeError::MissingInput)
}
