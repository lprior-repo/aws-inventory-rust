use super::{
    Decision, Fault, MAX_BODY_BYTES, MAX_CHUNK_BYTES, MAX_CHUNKS, MAX_HEAD_BYTES, MAX_HEADERS,
    ProxyError, deadline, http,
    message::{self, Framing, Metadata},
    protocol::{BodyPolicy, Connection},
};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

/// Wire overhead (chunk lines, delimiters and trailers) has its own hard cap.
const MAX_WIRE_BODY_BYTES: usize = MAX_BODY_BYTES.saturating_add(MAX_HEAD_BYTES);
const MAX_CHUNK_LINE: usize = 1_024;
const MAX_INFORMATIONAL: usize = 16;

pub(crate) struct Response {
    pub head: Vec<u8>,
    pub body: Vec<u8>,
    pub connection: Connection,
}

pub(crate) async fn read<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    policy: BodyPolicy,
    decision: Decision,
) -> Result<Response, ProxyError> {
    for _ in 0..MAX_INFORMATIONAL {
        let head = http::head(reader).await?.ok_or(ProxyError::Framing)?;
        let Metadata::Final(final_body) = message::metadata(&head, policy)? else {
            continue;
        };
        let body = read_body(reader, final_body.framing(), decision).await?;
        return Ok(Response {
            head,
            body,
            connection: final_body.connection(),
        });
    }
    Err(ProxyError::Limit("informational responses"))
}

async fn read_body<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    framing: Framing,
    decision: Decision,
) -> Result<Vec<u8>, ProxyError> {
    match framing {
        Framing::Empty => Ok(Vec::new()),
        Framing::Length(size) => Ok(corrupt(http::fixed(reader, size).await?, decision)),
        Framing::Chunked => chunks(reader, decision).await,
        Framing::Close => Ok(corrupt(close_body(reader).await?, decision)),
    }
}

async fn close_body<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> Result<Vec<u8>, ProxyError> {
    let mut bytes = Vec::new();
    super::reserve(&mut bytes, MAX_BODY_BYTES.saturating_add(1))?;
    let cap =
        u64::try_from(MAX_BODY_BYTES.saturating_add(1)).map_err(|_| ProxyError::Allocation)?;
    deadline(reader.take(cap).read_to_end(&mut bytes)).await?;
    if bytes.len() > MAX_BODY_BYTES {
        return Err(ProxyError::Limit("close-delimited body"));
    }
    Ok(bytes)
}

async fn line<R: AsyncRead + Unpin>(reader: &mut BufReader<R>) -> Result<Vec<u8>, ProxyError> {
    let mut line = Vec::new();
    super::reserve(&mut line, MAX_CHUNK_LINE)?;
    for _ in 0..MAX_CHUNK_LINE {
        line.push(deadline(reader.read_u8()).await?);
        if line.ends_with(b"\r\n") {
            return Ok(line);
        }
    }
    Err(ProxyError::Limit("chunk line"))
}

async fn chunks<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    decision: Decision,
) -> Result<Vec<u8>, ProxyError> {
    let mut wire = Vec::new();
    let mut decoded = 0usize;
    for _ in 0..MAX_CHUNKS {
        let line = line(reader).await?;
        let size = chunk_size(&line)?;
        decoded = decoded_size(decoded, size)?;
        append(&mut wire, &line)?;
        if size == 0 {
            trailers(reader, &mut wire).await?;
            return Ok(wire);
        }
        chunk_payload(reader, &mut wire, size, decision).await?;
    }
    Err(ProxyError::Limit("chunk count"))
}

fn chunk_size(line: &[u8]) -> Result<usize, ProxyError> {
    let size = match httparse::parse_chunk_size(line).map_err(|_| ProxyError::Framing)? {
        httparse::Status::Complete((_, size)) => {
            usize::try_from(size).map_err(|_| ProxyError::Framing)?
        }
        httparse::Status::Partial => return Err(ProxyError::Framing),
    };
    if size > MAX_CHUNK_BYTES {
        return Err(ProxyError::Limit("chunk payload"));
    }
    Ok(size)
}

fn decoded_size(total: usize, size: usize) -> Result<usize, ProxyError> {
    let decoded = total
        .checked_add(size)
        .ok_or(ProxyError::Limit("chunked body"))?;
    if decoded > MAX_BODY_BYTES {
        return Err(ProxyError::Limit("chunked body"));
    }
    Ok(decoded)
}

async fn chunk_payload<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    wire: &mut Vec<u8>,
    size: usize,
    decision: Decision,
) -> Result<(), ProxyError> {
    let payload = corrupt(http::fixed(reader, size).await?, decision);
    append(wire, &payload)?;
    if http::fixed(reader, 2).await? != b"\r\n" {
        return Err(ProxyError::Framing);
    }
    append(wire, b"\r\n")
}

async fn trailers<R: AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
    wire: &mut Vec<u8>,
) -> Result<(), ProxyError> {
    let mut total = 0usize;
    for _ in 0..=MAX_HEADERS {
        let line = line(reader).await?;
        total = total
            .checked_add(line.len())
            .ok_or(ProxyError::Limit("trailers"))?;
        if total > MAX_HEAD_BYTES {
            return Err(ProxyError::Limit("trailers"));
        }
        append(wire, &line)?;
        if line == b"\r\n" {
            return Ok(());
        }
        if !line.contains(&b':') {
            return Err(ProxyError::Framing);
        }
    }
    Err(ProxyError::Limit("trailer count"))
}

fn append(wire: &mut Vec<u8>, bytes: &[u8]) -> Result<(), ProxyError> {
    let size = wire
        .len()
        .checked_add(bytes.len())
        .ok_or(ProxyError::Limit("wire body"))?;
    if size > MAX_WIRE_BODY_BYTES {
        return Err(ProxyError::Limit("wire body"));
    }
    super::reserve(wire, bytes.len())?;
    wire.extend_from_slice(bytes);
    Ok(())
}

fn corrupt(mut bytes: Vec<u8>, decision: Decision) -> Vec<u8> {
    if decision.fault == Some(Fault::MalformedBody) {
        let [pattern, ..] = decision.entropy.to_le_bytes();
        bytes.fill(pattern);
        if let Some(first) = bytes.first_mut() {
            *first = 0xff;
        }
    }
    bytes
}

pub(crate) fn prefix(body: &[u8], entropy: u64) -> &[u8] {
    let fraction = usize::try_from(entropy % 3).map_or(1, |n| n.saturating_add(1));
    let length = body
        .len()
        .checked_mul(fraction)
        .map_or(0, |n| n / 4)
        .min(body.len().saturating_sub(1));
    match body.get(..length) {
        Some(prefix) => prefix,
        None => &[],
    }
}
