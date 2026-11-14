use super::{MAX_HEAD_BYTES, ProxyError};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(crate) enum Coding {
    #[default]
    Absent,
    Chunked,
    Close,
}

#[derive(Default)]
struct List {
    framing: Coding,
    chunked_seen: bool,
}

pub(crate) fn parse(headers: &[httparse::Header<'_>]) -> Result<Coding, ProxyError> {
    let mut list = List::default();
    for header in headers
        .iter()
        .filter(|h| h.name.eq_ignore_ascii_case("transfer-encoding"))
    {
        let text = std::str::from_utf8(header.value).map_err(|_| ProxyError::Framing)?;
        // Each coding consumes input; the complete head has MAX_HEAD_BYTES bytes.
        for part in text.split(',').take(MAX_HEAD_BYTES) {
            list.push(part)?;
        }
    }
    Ok(list.framing)
}

impl List {
    fn push(&mut self, part: &str) -> Result<(), ProxyError> {
        let name = part.split(';').next().ok_or(ProxyError::Framing)?.trim();
        if name.is_empty() || !name.bytes().all(token_byte) {
            return Err(ProxyError::Framing);
        }
        if name.eq_ignore_ascii_case("chunked") {
            if self.chunked_seen {
                return Err(ProxyError::Framing);
            }
            self.chunked_seen = true;
            self.framing = Coding::Chunked;
        } else {
            self.framing = Coding::Close;
        }
        Ok(())
    }
}

fn token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}
