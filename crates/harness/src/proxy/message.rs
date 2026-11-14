use super::{
    MAX_HEADERS, ProxyError,
    coding::{self, Coding},
    http,
    protocol::{BodyPolicy, Connection},
};

#[derive(Clone, Copy)]
pub(crate) enum Framing {
    Empty,
    Length(usize),
    Chunked,
    Close,
}
pub(crate) struct FinalBody {
    framing: Framing,
    connection: Connection,
}
impl FinalBody {
    fn new(framing: Framing, connection: Connection) -> Self {
        Self {
            framing,
            connection,
        }
    }
    pub(crate) fn framing(&self) -> Framing {
        self.framing
    }
    pub(crate) fn connection(&self) -> Connection {
        self.connection
    }
}
pub(crate) enum Metadata {
    Informational,
    Final(FinalBody),
}
#[derive(Eq, PartialEq)]
enum Status {
    Informational,
    Bodyless,
    Body,
}

pub(crate) fn metadata(bytes: &[u8], policy: BodyPolicy) -> Result<Metadata, ProxyError> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut parsed = httparse::Response::new(&mut headers);
    http::complete(
        parsed.parse(bytes).map_err(|_| ProxyError::Framing)?,
        bytes.len(),
    )?;
    final_metadata(&parsed, policy)
}

fn final_metadata(
    parsed: &httparse::Response<'_, '_>,
    policy: BodyPolicy,
) -> Result<Metadata, ProxyError> {
    let status = response_status(parsed.code)?;
    let connection =
        if parsed.version != Some(1) || http::has_token(parsed.headers, "connection", "close") {
            Connection::Close
        } else {
            Connection::KeepAlive
        };
    let body = body_framing(parsed.headers, connection)?;
    if status == Status::Informational {
        return Ok(Metadata::Informational);
    }
    if policy == BodyPolicy::Head || status == Status::Bodyless {
        return Ok(Metadata::Final(FinalBody::new(Framing::Empty, connection)));
    }
    Ok(Metadata::Final(body))
}

fn response_status(code: Option<u16>) -> Result<Status, ProxyError> {
    match code.ok_or(ProxyError::Framing)? {
        101 => Err(ProxyError::Framing),
        100..200 => Ok(Status::Informational),
        204 | 304 => Ok(Status::Bodyless),
        _ => Ok(Status::Body),
    }
}

fn body_framing(
    headers: &[httparse::Header<'_>],
    connection: Connection,
) -> Result<FinalBody, ProxyError> {
    let length = http::length(headers)?;
    let coding = coding::parse(headers)?;
    if coding != Coding::Absent && length.is_some() {
        return Err(ProxyError::Framing);
    }
    match coding {
        Coding::Chunked => return Ok(FinalBody::new(Framing::Chunked, connection)),
        Coding::Close => return Ok(close_body()),
        Coding::Absent => {}
    }
    Ok(match length {
        Some(size) => FinalBody::new(Framing::Length(size), connection),
        None => close_body(),
    })
}

fn close_body() -> FinalBody {
    FinalBody::new(Framing::Close, Connection::Close)
}
