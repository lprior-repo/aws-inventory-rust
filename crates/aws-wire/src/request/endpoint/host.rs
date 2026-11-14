use super::EncodeError;

pub(super) fn prepend_host(
    endpoint: &str,
    prefix: &str,
    limit: usize,
) -> Result<String, EncodeError> {
    let uri = endpoint
        .parse::<http::Uri>()
        .map_err(|_| EncodeError::InvalidEndpoint)?;
    let scheme = uri.scheme_str().ok_or(EncodeError::InvalidEndpoint)?;
    let authority = uri.authority().ok_or(EncodeError::InvalidEndpoint)?;
    let path = uri
        .path_and_query()
        .map_or("", http::uri::PathAndQuery::as_str);
    let length = host_length(scheme, prefix, authority.as_str(), path)?;
    if length > limit {
        return Err(EncodeError::Limit("URI"));
    }
    compose_host(scheme, prefix, authority.as_str(), path, length)
}

fn host_length(scheme: &str, prefix: &str, host: &str, path: &str) -> Result<usize, EncodeError> {
    scheme
        .len()
        .checked_add(3)
        .and_then(|size| size.checked_add(prefix.len()))
        .and_then(|size| size.checked_add(host.len()))
        .and_then(|size| size.checked_add(path.len()))
        .ok_or(EncodeError::Limit("URI"))
}

fn compose_host(
    scheme: &str,
    prefix: &str,
    host: &str,
    path: &str,
    length: usize,
) -> Result<String, EncodeError> {
    let mut result = String::new();
    result
        .try_reserve_exact(length)
        .map_err(EncodeError::Allocation)?;
    result.push_str(scheme);
    result.push_str("://");
    result.push_str(prefix);
    result.push_str(host);
    result.push_str(path);
    Ok(result)
}
