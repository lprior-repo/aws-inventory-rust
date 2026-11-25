use super::{Descriptor, PageInputs, RuntimeCatalogCompileError, encode, model_path};
use crate::overlay::MemberPath;
use inventory_core::{catalog::HttpBinding, record::Value};

pub(super) fn compile(
    inputs: &PageInputs<'_, '_>,
    input: &MemberPath,
    output: &MemberPath,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<Value, RuntimeCatalogCompileError> {
    validate_descriptor(input, output, descriptor)?;
    super::validate_ancestors(inputs.service, inputs.operation.input()?, input.as_str())?;
    super::validate_ancestors(inputs.service, inputs.operation.output()?, output.as_str())?;
    let request = model_path::terminal_member(inputs.service, inputs.operation.input()?, input.as_str())?;
    let response = model_path::terminal_member(inputs.service, inputs.operation.output()?, output.as_str())?;
    let value_type = super::token_type(model_path::kind(inputs.service, request.1.target)?)?;
    let element = model_path::list_element(inputs.service, response.1.target)?;
    if value_type != super::token_type(model_path::kind(inputs.service, element)?)?
        || !matches!(response.1.wire()?.binding()?, HttpBinding::Document | HttpBinding::Payload)
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: input.as_str().to_owned(),
        });
    }
    encode::map([
        (1, encode::path(inputs.services, input.as_str())?),
        (2, encode::path(inputs.services, output.as_str())?),
        (3, Value::Int(value_type)),
        (4, super::super::placement(request.1, request.0, inputs.protocol)?),
        (5, super::super::response_placement(response.1, response.0, inputs.protocol)?),
        (6, Value::Bool(false)),
        (7, Value::Int(1)),
    ])
}

fn validate_descriptor(
    input: &MemberPath,
    output: &MemberPath,
    descriptor: Option<&Descriptor<'_>>,
) -> Result<(), RuntimeCatalogCompileError> {
    let valid = descriptor.is_none_or(|expected| {
        expected.input_token.is_none_or(|path| path == input.as_str())
            && expected.output_token.is_none_or(|path| {
                path == output.as_str() || path.strip_suffix("[-1]") == Some(output.as_str())
            })
    });
    if valid {
        Ok(())
    } else {
        Err(RuntimeCatalogCompileError::Path {
            path: "page last_list_item descriptor".to_owned(),
        })
    }
}
