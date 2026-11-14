use super::{EncodeError, RequestContext};
use http::{HeaderMap, HeaderValue, header::ACCEPT};
use inventory_core::record::ValueRef;

pub(super) fn headers(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
) -> Result<(), EncodeError> {
    if context.service.sdk_id()? == Some("API Gateway") && !headers.contains_key(ACCEPT) {
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    }
    Ok(())
}

pub(super) fn label<'a>(
    context: RequestContext<'a>,
    name: &str,
    value: Option<ValueRef<'a>>,
) -> Result<Option<ValueRef<'a>>, EncodeError> {
    let absent = matches!(value, None | Some(ValueRef::Null | ValueRef::Text("")));
    if absent && name == "accountId" && context.service.endpoint_prefix()? == Some("glacier") {
        return Ok(Some(ValueRef::Text("-")));
    }
    Ok(value)
}
