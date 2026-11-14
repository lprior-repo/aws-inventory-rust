mod body;
mod binding;
mod uri;
mod timestamp;

use super::common::finish;
use super::{EncodeError, Protocol, RequestContext};
use http::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use inventory_core::{catalog::Member, record::ValueRef};
pub fn encode(context: RequestContext<'_>) -> Result<http::Request<bytes::Bytes>, EncodeError> {
    if !matches!(context.protocol, Protocol::RestJson1 | Protocol::RestXml) {
        return Err(EncodeError::Unsupported("non-REST protocol"));
    }
    let operation = context.operation.http()?.ok_or(EncodeError::Unsupported("operation without HTTP binding"))?;
    if context.operation.checksum_required()? {
        return Err(EncodeError::Unsupported("required request checksum is unavailable"));
    }
    let mut target = uri::target(context)?;
    let mut headers = HeaderMap::new();
    binding::headers(context, &mut target, &mut headers)?;
    let body = body::encode(context, &mut headers)?;
    let endpoint = uri::endpoint(context)?;
    let adjusted = RequestContext { endpoint: &endpoint, ..context };
    finish(adjusted, operation.method, Some(&target), headers, body)
}
fn field_value<'a>(context: RequestContext<'a>, member: Member<'a>) -> Result<Option<ValueRef<'a>>, EncodeError> {
    Ok(context.input.get_field(member.id)?)
}




fn write_header(headers: &mut HeaderMap, name: &str, value: &str) -> Result<(), EncodeError> {
    let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| EncodeError::InvalidValue { field: None, reason: "invalid header name" })?;
    let value = HeaderValue::from_str(value)?;
    headers.append(name, value);
    Ok(())
}



fn set_content_type(headers: &mut HeaderMap, value: &'static str) {
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(value));
}

#[cfg(test)]
mod tests;
