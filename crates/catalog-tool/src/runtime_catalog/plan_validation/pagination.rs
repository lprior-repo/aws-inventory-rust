use super::{RuntimeCatalogCompileError, model_path};
use inventory_core::catalog::{Operation, Service, Target};
use crate::overlay::Paginator;

pub(super) fn validate(
    service: Service<'_>,
    operation: Operation<'_>,
    overlay: &crate::overlay::OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let input = operation.input()?;
    let output = operation.output()?;
    match overlay.paginator.as_ref() {
        Some(Paginator::Token(pair)) => validate_token_pair(service, input, output, pair),
        Some(Paginator::LastListItem {
            input: request,
            output: response,
            truncated,
        }) => validate_last_item(service, input, output, request, response, truncated),
        Some(Paginator::Tokens { pairs, truncated }) => {
            validate_token_pairs(service, input, output, pairs, truncated)
        }
        Some(Paginator::None) | None => Ok(()),
    }
}

fn validate_token_pair(
    service: Service<'_>,
    input: Target,
    output: Target,
    pair: &crate::overlay::TokenPair,
) -> Result<(), RuntimeCatalogCompileError> {
    let request_str = pair.input.as_str();
    let response_str = pair.output.as_str();
    map_path_error(
        request_str,
        model_path::resolve(service, input, request_str).map(|_| ()),
    )?;
    map_path_error(
        response_str,
        model_path::resolve(service, output, response_str).map(|_| ()),
    )
}

fn validate_token_pairs(
    service: Service<'_>,
    input: Target,
    output: Target,
    pairs: &[crate::overlay::TokenPair],
    truncated: Option<&crate::overlay::MemberPath>,
) -> Result<(), RuntimeCatalogCompileError> {
    pairs
        .iter()
        .try_for_each(|pair| validate_token_pair(service, input, output, pair))?;
    validate_truncated_path(service, output, truncated)
}

fn validate_truncated_path(
    service: Service<'_>,
    output: Target,
    path: Option<&crate::overlay::MemberPath>,
) -> Result<(), RuntimeCatalogCompileError> {
    path.map_or(Ok(()), |p| {
        let path_str = p.as_str();
        map_path_error(path_str, model_path::resolve(service, output, path_str).map(|_| ()))
    })
}

fn validate_last_item(
    service: Service<'_>,
    input: Target,
    output: Target,
    request: &crate::overlay::MemberPath,
    response: &crate::overlay::MemberPath,
    truncated: &crate::overlay::MemberPath,
) -> Result<(), RuntimeCatalogCompileError> {
    let request_str = request.as_str();
    map_path_error(
        request_str,
        model_path::resolve(service, input, request_str).map(|_| ()),
    )?;
    validate_last_item_response(service, output, response, truncated)
}

fn validate_last_item_response(
    service: Service<'_>,
    output: Target,
    response: &crate::overlay::MemberPath,
    truncated: &crate::overlay::MemberPath,
) -> Result<(), RuntimeCatalogCompileError> {
    let response_str = response.as_str();
    let truncated_str = truncated.as_str();
    map_path_error(
        response_str,
        model_path::resolve(service, output, response_str).map(|_| ()),
    )?;
    let flag = map_path_error(
        truncated_str,
        model_path::resolve(service, output, truncated_str),
    )?;
    if model_path::kind(service, flag.1)? != "boolean" {
        return Err(RuntimeCatalogCompileError::Path {
            path: truncated_str.to_owned(),
        });
    }
    Ok(())
}

fn map_path_error<T>(
    path: &str,
    result: Result<T, RuntimeCatalogCompileError>,
) -> Result<T, RuntimeCatalogCompileError> {
    result.map_err(|error| match error {
        RuntimeCatalogCompileError::Path { path: inner }
            if inner != path =>
        {
            RuntimeCatalogCompileError::Path { path: path.to_owned() }
        }
        error => error,
    })
}