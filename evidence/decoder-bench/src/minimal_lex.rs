use crate::error::{Error, Result, syntax};
use std::borrow::Cow;
pub struct Lexer<'a> {
    pub input: &'a [u8],
    pub pos: usize,
}
impl<'a> Lexer<'a> {
    pub fn peek(&mut self) -> Option<u8> {
        let count = self
            .input
            .get(self.pos..)?
            .iter()
            .take_while(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t'))
            .count();
        self.pos = self.pos.saturating_add(count);
        self.input.get(self.pos).copied()
    }
    pub fn eat(&mut self, want: u8) -> Result<()> {
        if self.peek() != Some(want) {
            return Err(Error::Shape("baseline delimiter"));
        }
        self.pos = self.pos.saturating_add(1);
        Ok(())
    }
    pub fn literal(&mut self, want: &[u8]) -> Result<()> {
        let end = self.pos.checked_add(want.len()).ok_or(Error::Bound)?;
        if self.input.get(self.pos..end) != Some(want) {
            return Err(Error::Shape("baseline literal"));
        }
        self.pos = end;
        Ok(())
    }
    pub fn string(&mut self) -> Result<Cow<'a, str>> {
        self.eat(b'"')?;
        let start = self.pos;
        let mut escaped = false;
        let end = (start..self.input.len())
            .find(|i| {
                let byte = self.input.get(*i).copied();
                if escaped {
                    escaped = false;
                    return false;
                }
                if byte == Some(b'\\') {
                    escaped = true;
                    return false;
                }
                byte == Some(b'"')
            })
            .ok_or(Error::Shape("unterminated string"))?;
        self.pos = end.saturating_add(1);
        let raw =
            std::str::from_utf8(self.input.get(start..end).ok_or(Error::Bound)?).map_err(syntax)?;
        if raw.bytes().any(|b| b < 32) {
            return Err(Error::Shape("control in string"));
        }
        if !raw.contains('\\') {
            return Ok(Cow::Borrowed(raw));
        }
        unescape(raw).map(Cow::Owned)
    }
    pub fn number(&mut self) -> Result<&'a str> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos = self.pos.saturating_add(1);
        }
        let first = self
            .input
            .get(self.pos)
            .copied()
            .ok_or(Error::Shape("number EOF"))?;
        if first == b'0' {
            self.pos = self.pos.saturating_add(1);
        } else {
            self.digits()?;
        }
        if self.input.get(self.pos) == Some(&b'.') {
            self.pos = self.pos.saturating_add(1);
            self.digits()?;
        }
        if matches!(self.input.get(self.pos), Some(b'e' | b'E')) {
            self.exponent()?;
        }
        std::str::from_utf8(self.input.get(start..self.pos).ok_or(Error::Bound)?).map_err(syntax)
    }
    fn digits(&mut self) -> Result<()> {
        let count = self
            .input
            .get(self.pos..)
            .ok_or(Error::Bound)?
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        if count == 0 {
            return Err(Error::Shape("number digit"));
        }
        self.pos = self.pos.saturating_add(count);
        Ok(())
    }
    fn exponent(&mut self) -> Result<()> {
        self.pos = self.pos.saturating_add(1);
        if matches!(self.input.get(self.pos), Some(b'+' | b'-')) {
            self.pos = self.pos.saturating_add(1);
        }
        self.digits()
    }
}
fn unescape(raw: &str) -> Result<String> {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    std::iter::from_fn(|| {
        chars.next().map(|c| {
            if c != '\\' {
                out.push(c);
                return Ok(());
            }
            let next = chars.next().ok_or(Error::Shape("escape EOF"))?;
            let decoded = match next {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{8}',
                'f' => '\u{c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => unicode(&mut chars)?,
                _ => return Err(Error::Shape("bad escape")),
            };
            out.push(decoded);
            Ok(())
        })
    })
    .take(raw.len())
    .try_for_each(|r| r)?;
    Ok(out)
}
fn hex(chars: &mut std::str::Chars<'_>) -> Result<u16> {
    (0..4).try_fold(0u16, |value, _| {
        let digit = chars
            .next()
            .and_then(|c| c.to_digit(16))
            .ok_or(Error::Shape("unicode hex"))?;
        let digit = u16::try_from(digit).map_err(|_| Error::Bound)?;
        value
            .checked_mul(16)
            .and_then(|v| v.checked_add(digit))
            .ok_or(Error::Bound)
    })
}
fn unicode(chars: &mut std::str::Chars<'_>) -> Result<char> {
    let high = hex(chars)?;
    if !(0xd800..=0xdbff).contains(&high) {
        return char::from_u32(u32::from(high)).ok_or(Error::Shape("unicode scalar"));
    }
    if chars.next() != Some('\\') || chars.next() != Some('u') {
        return Err(Error::Shape("surrogate pair"));
    }
    let low = hex(chars)?;
    char::decode_utf16([high, low])
        .next()
        .ok_or(Error::Shape("unicode pair"))?
        .map_err(syntax)
}
