use super::{EncodeError, RequestContext, RequestLimits};
use bytes::Bytes;
use http::{
    HeaderMap, HeaderValue, Request, Uri,
    header::{CONTENT_LENGTH, HOST},
};

pub(crate) struct BoundedBody {
    bytes: Vec<u8>,
    limit: usize,
}

impl BoundedBody {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let needed = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(EncodeError::Limit("body"))?;
        if needed > self.limit {
            return Err(EncodeError::Limit("body"));
        }
        self.reserve(needed)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn reserve(&mut self, needed: usize) -> Result<(), EncodeError> {
        if needed <= self.bytes.capacity() {
            return Ok(());
        }
        let grown = match self.bytes.capacity().checked_mul(2) {
            Some(capacity) => capacity.min(self.limit),
            None => self.limit,
        };
        let target = needed.max(grown).max(4096.min(self.limit));
        let additional = target
            .checked_sub(self.bytes.len())
            .ok_or(EncodeError::Limit("body"))?;
        self.bytes.try_reserve_exact(additional)?;
        Ok(())
    }

    pub(crate) fn remaining(&self) -> Result<usize, EncodeError> {
        self.limit
            .checked_sub(self.bytes.len())
            .ok_or(EncodeError::Limit("body"))
    }

    pub(crate) fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

pub(crate) fn finish(
    context: RequestContext<'_>,
    method: &str,
    target: Option<&str>,
    mut headers: HeaderMap,
    body: Vec<u8>,
) -> Result<Request<Bytes>, EncodeError> {
    if body.len() > context.limits.body_bytes() {
        return Err(EncodeError::Limit("body"));
    }
    let endpoint = endpoint(context.endpoint, context.limits)?;
    let uri = request_target(&endpoint, target, context.limits)?;
    let authority = endpoint.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let body = prepare_headers(context, &mut headers, authority.as_str(), body)?;
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Bytes::from(body))?;
    *request.headers_mut() = headers;
    Ok(request)
}

fn prepare_headers(
    context: RequestContext<'_>,
    headers: &mut HeaderMap,
    authority: &str,
    body: Vec<u8>,
) -> Result<Vec<u8>, EncodeError> {
    install_host(headers, authority, context.limits.header_bytes())?;
    super::customization::headers(context, headers)?;
    let body = super::compression::apply(context, headers, body)?;
    super::checksum::apply(context, headers, &body)?;
    if !body.is_empty() {
        headers.insert(CONTENT_LENGTH, HeaderValue::from(body.len()));
    }
    check_headers(headers, context.limits.header_bytes())?;
    Ok(body)
}

fn endpoint(value: &str, limits: RequestLimits) -> Result<Uri, EncodeError> {
    if value.len() > limits.uri_bytes() {
        return Err(EncodeError::Limit("URI"));
    }
    if value.contains('#') {
        return Err(EncodeError::InvalidEndpoint);
    }
    let uri: Uri = value.parse().map_err(|_| EncodeError::InvalidEndpoint)?;
    let authority = uri.authority().ok_or(EncodeError::InvalidEndpoint)?;
    if !matches!(uri.scheme_str(), Some("http" | "https"))
        || authority.as_str().contains('@')
        || uri.query().is_some()
    {
        return Err(EncodeError::InvalidEndpoint);
    }
    Ok(uri)
}

fn request_target(
    endpoint: &Uri,
    target: Option<&str>,
    limits: RequestLimits,
) -> Result<Uri, EncodeError> {
    let Some(target) = target else {
        return Ok(endpoint.clone());
    };
    if !target.starts_with('/') || target.contains('#') {
        return Err(EncodeError::InvalidEndpoint);
    }
    let path = target_path(endpoint, target, limits)?;
    uri_with_path(endpoint, path)
}

fn target_path(endpoint: &Uri, target: &str, limits: RequestLimits) -> Result<String, EncodeError> {
    let prefix = match endpoint.path().strip_suffix('/') {
        Some(prefix) => prefix,
        None => endpoint.path(),
    };
    let length = prefix
        .len()
        .checked_add(target.len())
        .ok_or(EncodeError::Limit("URI"))?;
    check_uri_size(endpoint, length, limits.uri_bytes())?;
    let mut path = String::new();
    path.try_reserve_exact(length)?;
    path.push_str(prefix);
    path.push_str(target);
    Ok(path)
}

fn uri_with_path(endpoint: &Uri, path: String) -> Result<Uri, EncodeError> {
    Ok(Uri::builder()
        .scheme(endpoint.scheme_str().ok_or(EncodeError::InvalidEndpoint)?)
        .authority(
            endpoint
                .authority()
                .ok_or(EncodeError::InvalidEndpoint)?
                .clone(),
        )
        .path_and_query(path)
        .build()?)
}

fn check_uri_size(endpoint: &Uri, path_bytes: usize, limit: usize) -> Result<(), EncodeError> {
    let scheme = endpoint.scheme_str().ok_or(EncodeError::InvalidEndpoint)?;
    let authority = endpoint.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let size = scheme
        .len()
        .checked_add(3)
        .and_then(|length| length.checked_add(authority.as_str().len()))
        .and_then(|length| length.checked_add(path_bytes))
        .ok_or(EncodeError::Limit("URI"))?;
    if size > limit {
        return Err(EncodeError::Limit("URI"));
    }
    Ok(())
}

fn install_host(headers: &mut HeaderMap, authority: &str, limit: usize) -> Result<(), EncodeError> {
    let existing = check_headers(headers, limit)?;
    if let Some(host) = headers.get(HOST) {
        if host.as_bytes() != authority.as_bytes() || headers.get_all(HOST).iter().count() != 1 {
            return Err(EncodeError::InvalidEndpoint);
        }
    } else {
        check_host_size(existing, authority.len(), limit)?;
        headers.insert(HOST, authority.parse()?);
    }
    Ok(())
}

fn check_host_size(existing: usize, authority: usize, limit: usize) -> Result<(), EncodeError> {
    let needed = existing
        .checked_add(4)
        .and_then(|bytes| bytes.checked_add(authority))
        .and_then(|bytes| bytes.checked_add(4))
        .ok_or(EncodeError::Limit("headers"))?;
    if needed > limit {
        return Err(EncodeError::Limit("headers"));
    }
    Ok(())
}

pub(super) fn check_headers(headers: &HeaderMap, limit: usize) -> Result<usize, EncodeError> {
    let mut bytes = 0usize;
    for (name, value) in headers {
        bytes = bytes
            .checked_add(name.as_str().len())
            .and_then(|length| length.checked_add(value.as_bytes().len()))
            .and_then(|length| length.checked_add(4))
            .ok_or(EncodeError::Limit("headers"))?;
        if bytes > limit {
            return Err(EncodeError::Limit("headers"));
        }
    }
    Ok(bytes)
}

pub(super) fn ensure_service_protocol(
    context: RequestContext<'_>,
    expected: &str,
) -> Result<(), EncodeError> {
    let found = context
        .service
        .protocols()?
        .try_fold(false, |found, protocol| {
            Ok::<bool, EncodeError>(found || protocol? == expected)
        })?;
    if found {
        return Ok(());
    }
    Err(EncodeError::Unsupported(
        "protocol does not match service metadata",
    ))
}

#[cfg(test)]
mod tests;
