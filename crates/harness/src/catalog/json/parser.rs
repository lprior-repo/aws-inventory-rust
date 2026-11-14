use super::Json;
use crate::HarnessError;

pub(super) fn parse(text: &str) -> Result<Json, HarnessError> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        index: 0,
    };
    let value = parser.value()?;
    parser.whitespace();
    if parser.index < parser.bytes.len() {
        return Err(parser.failure("trailing characters"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl Parser<'_> {
    fn failure(&self, reason: &str) -> HarnessError {
        HarnessError::invalid(format!("invalid JSON at byte {}: {reason}", self.index))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.index).copied()
    }

    fn bump(&mut self) {
        self.index = self.index.saturating_add(1);
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn value(&mut self) -> Result<Json, HarnessError> {
        self.whitespace();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.failure("expected a value")),
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, HarnessError> {
        let end = self.index.saturating_add(word.len());
        if self.bytes.get(self.index..end) == Some(word.as_bytes()) {
            self.index = end;
            Ok(value)
        } else {
            Err(self.failure("invalid literal"))
        }
    }

    fn object(&mut self) -> Result<Json, HarnessError> {
        self.bump();
        let mut entries: Vec<(String, Json)> = Vec::new();
        self.whitespace();
        if self.peek() == Some(b'}') {
            self.bump();
            return Ok(Json::Object(entries));
        }
        loop {
            self.whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.failure("expected an object key"));
            }
            let key = self.string()?;
            self.whitespace();
            if self.peek() != Some(b':') {
                return Err(self.failure("expected ':'"));
            }
            self.bump();
            let value = self.value()?;
            if let Some((_, slot)) = entries.iter_mut().find(|(name, _)| *name == key) {
                *slot = value;
            } else {
                entries.push((key, value));
            }
            self.whitespace();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b'}') => {
                    self.bump();
                    return Ok(Json::Object(entries));
                }
                _ => return Err(self.failure("expected ',' or '}'")),
            }
        }
    }

    fn array(&mut self) -> Result<Json, HarnessError> {
        self.bump();
        let mut items = Vec::new();
        self.whitespace();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.whitespace();
            match self.peek() {
                Some(b',') => self.bump(),
                Some(b']') => {
                    self.bump();
                    return Ok(Json::Array(items));
                }
                _ => return Err(self.failure("expected ',' or ']'")),
            }
        }
    }

    fn string(&mut self) -> Result<String, HarnessError> {
        self.bump();
        let mut out = String::new();
        loop {
            self.push_segment(&mut out)?;
            match self.peek() {
                Some(b'"') => {
                    self.bump();
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.bump();
                    out.push(self.escape_character()?);
                }
                _ => return Err(self.failure("unterminated string")),
            }
        }
    }

    fn push_segment(&mut self, out: &mut String) -> Result<(), HarnessError> {
        let start = self.index;
        while let Some(byte) = self.peek() {
            if byte == b'"' || byte == b'\\' || byte < 0x20 {
                break;
            }
            self.bump();
        }
        let segment = self
            .bytes
            .get(start..self.index)
            .and_then(|raw| std::str::from_utf8(raw).ok())
            .ok_or_else(|| self.failure("invalid UTF-8 in string"))?;
        out.push_str(segment);
        Ok(())
    }

    fn escape_character(&mut self) -> Result<char, HarnessError> {
        let byte = self
            .peek()
            .ok_or_else(|| self.failure("unterminated escape"))?;
        self.bump();
        Ok(match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode_escape(),
            _ => return Err(self.failure("invalid escape")),
        })
    }

    fn unicode_escape(&mut self) -> Result<char, HarnessError> {
        let first = self.hex4()?;
        let code = if (0xd800..=0xdbff).contains(&first) {
            if self.bytes.get(self.index..self.index.saturating_add(2)) != Some(b"\\u") {
                return Err(self.failure("unpaired surrogate"));
            }
            self.index = self.index.saturating_add(2);
            let second = self.hex4()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(self.failure("unpaired surrogate"));
            }
            first
                .wrapping_sub(0xd800)
                .saturating_mul(0x400)
                .saturating_add(second.wrapping_sub(0xdc00))
                .saturating_add(0x1_0000)
        } else {
            first
        };
        char::from_u32(code).ok_or_else(|| self.failure("invalid code point"))
    }

    fn hex4(&mut self) -> Result<u32, HarnessError> {
        let end = self.index.saturating_add(4);
        let raw = self
            .bytes
            .get(self.index..end)
            .ok_or_else(|| self.failure("truncated escape"))?;
        let text = std::str::from_utf8(raw).map_err(|_| self.failure("truncated escape"))?;
        let value = u32::from_str_radix(text, 16).map_err(|_| self.failure("invalid escape"))?;
        self.index = end;
        Ok(value)
    }

    fn number(&mut self) -> Result<Json, HarnessError> {
        let start = self.index;
        while let Some(byte) = self.peek() {
            if byte.is_ascii_digit() || matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E') {
                self.bump();
            } else {
                break;
            }
        }
        let raw = self
            .bytes
            .get(start..self.index)
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .ok_or_else(|| self.failure("invalid number"))?;
        serde_json::from_str::<serde_json::Value>(raw)
            .map_err(|_| self.failure("invalid number"))?;
        Ok(Json::Number(raw.to_string()))
    }
}
