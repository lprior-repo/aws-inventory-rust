use super::{
    MAX_BODY_BYTES, MAX_HEAD_BYTES, MAX_HEADERS, ProxyError, Scope, deadline,
    protocol::{BodyPolicy, Connection},
    validated::ValidatedConfig,
};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) struct Request {
    pub bytes: Vec<u8>,
    pub scope: Scope,
    pub body_policy: BodyPolicy,
    pub connection: Connection,
}

pub(crate) struct RequestError {
    pub error: ProxyError,
    pub scope: Option<Scope>,
}
pub(crate) type RequestResult<T> = Result<T, RequestError>;

impl RequestError {
    pub(crate) fn unparsed(error: ProxyError) -> Self {
        Self { error, scope: None }
    }
    pub(crate) fn scoped(error: ProxyError, scope: Scope) -> Self {
        Self {
            error,
            scope: Some(scope),
        }
    }
}

struct Metadata {
    size: usize,
    scope: Scope,
    body_policy: BodyPolicy,
    connection: Connection,
}
impl Metadata {
    fn reject(self, error: ProxyError) -> RequestError {
        RequestError::scoped(error, self.scope)
    }
}

pub(crate) fn header<'a>(headers: &'a [httparse::Header<'a>], name: &str) -> Option<&'a [u8]> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value)
}

pub(crate) fn has_token(headers: &[httparse::Header<'_>], name: &str, token: &str) -> bool {
    headers
        .iter()
        .filter(|h| h.name.eq_ignore_ascii_case(name))
        .any(|h| {
            std::str::from_utf8(h.value)
                .is_ok_and(|s| s.split(',').any(|v| v.trim().eq_ignore_ascii_case(token)))
        })
}

pub(crate) fn length(headers: &[httparse::Header<'_>]) -> Result<Option<usize>, ProxyError> {
    let mut lengths = headers
        .iter()
        .filter(|h| h.name.eq_ignore_ascii_case("content-length"));
    let Some(first) = lengths.next() else {
        return Ok(None);
    };
    if lengths.next().is_some() {
        return Err(ProxyError::Framing);
    }
    let text = std::str::from_utf8(first.value).map_err(|_| ProxyError::Framing)?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ProxyError::Framing);
    }
    text.parse::<usize>()
        .map(Some)
        .map_err(|_| ProxyError::Framing)
}

pub(crate) async fn head<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> Result<Option<Vec<u8>>, ProxyError> {
    let mut bytes = Vec::new();
    super::reserve(&mut bytes, MAX_HEAD_BYTES)?;
    for _ in 0..MAX_HEAD_BYTES {
        match deadline(reader.read_u8()).await {
            Ok(byte) => push_head_byte(&mut bytes, byte)?,
            Err(ProxyError::Io(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof && bytes.is_empty() =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        }
        if bytes.ends_with(b"\r\n\r\n") {
            return Ok(Some(bytes));
        }
    }
    Err(ProxyError::Limit("HTTP head"))
}

pub(crate) async fn fixed<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    size: usize,
) -> Result<Vec<u8>, ProxyError> {
    if size > MAX_BODY_BYTES {
        return Err(ProxyError::Limit("body"));
    }
    let mut bytes = Vec::new();
    super::reserve(&mut bytes, size)?;
    bytes.resize(size, 0);
    deadline(reader.read_exact(&mut bytes)).await?;
    Ok(bytes)
}

pub(crate) async fn request<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    config: &ValidatedConfig,
) -> RequestResult<Option<Request>> {
    let Some(bytes) = head(reader).await.map_err(RequestError::unparsed)? else {
        return Ok(None);
    };
    let metadata = request_metadata(&bytes, config)?;
    finish_request(reader, bytes, metadata).await
}

fn request_metadata(bytes: &[u8], config: &ValidatedConfig) -> RequestResult<Metadata> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut parsed = httparse::Request::new(&mut headers);
    let status = parsed
        .parse(bytes)
        .map_err(|_| RequestError::unparsed(ProxyError::Framing))?;
    complete(status, bytes.len()).map_err(RequestError::unparsed)?;
    if parsed.version != Some(1) {
        return Err(RequestError::unparsed(ProxyError::Framing));
    }
    metadata(&parsed, config)
}

pub(crate) fn rejection(error: &ProxyError) -> (u16, &'static str) {
    match error {
        ProxyError::UnsupportedExpectation => (417, "expectation-failed"),
        ProxyError::Limit("chunked request") => (501, "chunked-request"),
        ProxyError::Limit("request body") => (413, "request-body"),
        ProxyError::Limit("HTTP head") => (431, "request-head"),
        ProxyError::Limit("scope capacity") => (503, "scope-capacity"),
        ProxyError::Timeout => (408, "idle-timeout"),
        _ => (400, "invalid-request"),
    }
}

fn request_body_length(headers: &[httparse::Header<'_>]) -> Result<usize, ProxyError> {
    if header(headers, "expect").is_some() {
        return Err(ProxyError::UnsupportedExpectation);
    }
    if header(headers, "transfer-encoding").is_some() {
        return Err(ProxyError::Limit("chunked request"));
    }
    let Some(size) = length(headers)? else {
        return Ok(0);
    };
    if size > MAX_BODY_BYTES {
        return Err(ProxyError::Limit("request body"));
    }
    Ok(size)
}

fn push_head_byte(bytes: &mut Vec<u8>, byte: u8) -> Result<(), ProxyError> {
    if (byte == b'\n' && bytes.last() != Some(&b'\r'))
        || (bytes.last() == Some(&b'\r') && byte != b'\n')
    {
        return Err(ProxyError::Framing);
    }
    bytes.push(byte);
    Ok(())
}

pub(crate) fn complete(status: httparse::Status<usize>, size: usize) -> Result<(), ProxyError> {
    if !matches!(status, httparse::Status::Complete(consumed) if consumed == size) {
        return Err(ProxyError::Framing);
    }
    Ok(())
}

fn metadata(
    parsed: &httparse::Request<'_, '_>,
    config: &ValidatedConfig,
) -> RequestResult<Metadata> {
    let scope = super::plan::signed_scope(header(parsed.headers, "authorization"), config);
    let size = match request_body_length(parsed.headers) {
        Ok(size) => size,
        Err(error) => return Err(RequestError::scoped(error, scope)),
    };
    Ok(Metadata {
        size,
        scope,
        body_policy: if parsed.method == Some("HEAD") {
            BodyPolicy::Head
        } else {
            BodyPolicy::Normal
        },
        connection: if has_token(parsed.headers, "connection", "close") {
            Connection::Close
        } else {
            Connection::KeepAlive
        },
    })
}

async fn finish_request<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    mut bytes: Vec<u8>,
    metadata: Metadata,
) -> RequestResult<Option<Request>> {
    let body = match fixed(reader, metadata.size).await {
        Ok(body) => body,
        Err(error) => return Err(metadata.reject(error)),
    };
    if let Err(error) = super::reserve(&mut bytes, body.len()) {
        return Err(metadata.reject(error));
    }
    bytes.extend_from_slice(&body);
    Ok(Some(Request {
        bytes,
        scope: metadata.scope,
        body_policy: metadata.body_policy,
        connection: metadata.connection,
    }))
}
