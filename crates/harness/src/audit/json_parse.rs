//! Recursive-descent parser for the order-preserving JSON value.

use crate::HarnessError;

use super::json_ordered::Json;

fn malformed() -> HarnessError {
    HarnessError::invalid("invalid JSON document")
}

/// Parse one complete JSON document.
pub(super) fn parse(text: &str) -> Result<Json, HarnessError> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
    };
    let value = parser.value()?;
    parser.whitespace();
    if parser.position != parser.bytes.len() {
        return Err(malformed());
    }
    Ok(value)
}

fn number_from_text(text: &str, integral: bool) -> Result<Json, HarnessError> {
    if integral {
        if let Ok(value) = text.parse::<i64>() {
            return Ok(Json::Int(value));
        }
        return text.parse::<u64>().map(Json::UInt).map_err(|_| malformed());
    }
    text.parse::<f64>()
        .map(Json::Float)
        .map_err(|_| malformed())
}

fn insert_entry(
    entries: &mut Vec<(String, Json)>,
    key: &str,
    value: Json,
) -> Result<(), HarnessError> {
    let mut document = Json::Object(std::mem::take(entries));
    document.insert(key, value);
    match document {
        Json::Object(updated) => {
            *entries = updated;
            Ok(())
        }
        _ => Err(malformed()),
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn value(&mut self) -> Result<Json, HarnessError> {
        self.whitespace();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::String),
            Some(b't') => self.literal(b"true", Json::Bool(true)),
            Some(b'f') => self.literal(b"false", Json::Bool(false)),
            Some(b'n') => self.literal(b"null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(malformed()),
        }
    }

    fn object(&mut self) -> Result<Json, HarnessError> {
        self.expect(b'{')?;
        let mut entries: Vec<(String, Json)> = Vec::new();
        self.whitespace();
        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(Json::Object(entries));
        }
        loop {
            self.object_entry(&mut entries)?;
            if self.object_boundary()? {
                return Ok(Json::Object(entries));
            }
        }
    }

    fn object_entry(&mut self, entries: &mut Vec<(String, Json)>) -> Result<(), HarnessError> {
        self.whitespace();
        let key = self.string()?;
        self.whitespace();
        self.expect(b':')?;
        let value = self.value()?;
        insert_entry(entries, &key, value)
    }

    fn object_boundary(&mut self) -> Result<bool, HarnessError> {
        self.whitespace();
        match self.peek() {
            Some(b',') => {
                self.advance();
                Ok(false)
            }
            Some(b'}') => {
                self.advance();
                Ok(true)
            }
            _ => Err(malformed()),
        }
    }

    fn array(&mut self) -> Result<Json, HarnessError> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.whitespace();
        if self.peek() == Some(b']') {
            self.position = self.position.saturating_add(1);
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.whitespace();
            match self.peek() {
                Some(b',') => self.position = self.position.saturating_add(1),
                Some(b']') => {
                    self.position = self.position.saturating_add(1);
                    return Ok(Json::Array(items));
                }
                _ => return Err(malformed()),
            }
        }
    }

    fn string(&mut self) -> Result<String, HarnessError> {
        self.expect(b'"')?;
        let mut output: Vec<u8> = Vec::new();
        loop {
            let byte = self.peek().ok_or_else(malformed)?;
            self.advance();
            match byte {
                b'"' => break,
                b'\\' => self.escape(&mut output)?,
                0x00..=0x1f => return Err(malformed()),
                other => output.push(other),
            }
        }
        String::from_utf8(output).map_err(|_| malformed())
    }

    fn escape(&mut self, output: &mut Vec<u8>) -> Result<(), HarnessError> {
        let escaped = self.peek().ok_or_else(malformed)?;
        self.advance();
        match escaped {
            b'"' | b'\\' | b'/' => output.push(escaped),
            b'b' => output.push(0x08),
            b'f' => output.push(0x0c),
            b'n' => output.push(b'\n'),
            b'r' => output.push(b'\r'),
            b't' => output.push(b'\t'),
            b'u' => self.unicode(output)?,
            _ => return Err(malformed()),
        }
        Ok(())
    }

    fn unicode(&mut self, output: &mut Vec<u8>) -> Result<(), HarnessError> {
        let mut code = u32::from(self.hex4()?);
        if (0xd800..=0xdbff).contains(&code) {
            code = self.surrogate_pair(code)?;
        }
        let character = char::from_u32(code).ok_or_else(malformed)?;
        let mut buffer = [0_u8; 4];
        output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        Ok(())
    }

    fn surrogate_pair(&mut self, high: u32) -> Result<u32, HarnessError> {
        if self.peek() != Some(b'\\') {
            return Err(malformed());
        }
        self.advance();
        self.expect(b'u')?;
        let low = u32::from(self.hex4()?);
        if !(0xdc00..=0xdfff).contains(&low) {
            return Err(malformed());
        }
        let high_part = high.saturating_sub(0xd800).saturating_mul(0x400);
        let low_part = low.saturating_sub(0xdc00);
        0x10000_u32
            .checked_add(high_part)
            .and_then(|value| value.checked_add(low_part))
            .ok_or_else(malformed)
    }

    fn hex4(&mut self) -> Result<u16, HarnessError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            let byte = self.peek().ok_or_else(malformed)?;
            self.position = self.position.saturating_add(1);
            let digit = match byte {
                b'0'..=b'9' => u16::from(byte.saturating_sub(b'0')),
                b'a'..=b'f' => u16::from(byte.saturating_sub(b'a')).saturating_add(10),
                b'A'..=b'F' => u16::from(byte.saturating_sub(b'A')).saturating_add(10),
                _ => return Err(malformed()),
            };
            value = value.saturating_mul(16).saturating_add(digit);
        }
        Ok(value)
    }

    fn number(&mut self) -> Result<Json, HarnessError> {
        let start = self.position;
        self.sign();
        self.digits();
        let mut integral = true;
        if self.peek() == Some(b'.') {
            integral = false;
            self.advance();
            self.digits();
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            integral = false;
            self.exponent();
        }
        number_from_text(self.token_text(start)?, integral)
    }

    fn sign(&mut self) {
        if self.peek() == Some(b'-') {
            self.advance();
        }
    }

    fn digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.advance();
        }
    }

    fn exponent(&mut self) {
        self.advance();
        if matches!(self.peek(), Some(b'+' | b'-')) {
            self.advance();
        }
        self.digits();
    }

    fn token_text(&self, start: usize) -> Result<&str, HarnessError> {
        let slice = self.bytes.get(start..self.position).ok_or_else(malformed)?;
        std::str::from_utf8(slice).map_err(|_| malformed())
    }

    fn literal(&mut self, text: &[u8], value: Json) -> Result<Json, HarnessError> {
        for expected in text {
            if self.peek() != Some(*expected) {
                return Err(malformed());
            }
            self.advance();
        }
        Ok(value)
    }

    fn expect(&mut self, expected: u8) -> Result<(), HarnessError> {
        if self.peek() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(malformed())
        }
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.advance();
        }
    }

    fn advance(&mut self) {
        self.position = self.position.saturating_add(1);
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }
}
