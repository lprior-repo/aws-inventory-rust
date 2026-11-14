use crate::error::{Error, Result, syntax};
use std::borrow::Cow;
pub fn decode(text: &str) -> Result<Cow<'_, str>> {
    if !text.contains('&') {
        return Ok(Cow::Borrowed(text));
    }
    let mut chunks = text.split('&');
    let mut out = chunks.next().map_or_else(String::new, str::to_owned);
    chunks.try_for_each(|part| {
        let (name, rest) = part
            .split_once(';')
            .ok_or(Error::Shape("XML entity terminator"))?;
        let c = match name {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            n => numeric(n)?,
        };
        out.push(c);
        out.push_str(rest);
        Ok::<_, Error>(())
    })?;
    Ok(Cow::Owned(out))
}
fn numeric(name: &str) -> Result<char> {
    let value = if let Some(n) = name.strip_prefix("#x") {
        u32::from_str_radix(n, 16).map_err(syntax)?
    } else if let Some(n) = name.strip_prefix('#') {
        n.parse().map_err(syntax)?
    } else {
        return Err(Error::Shape("XML unknown entity"));
    };
    if !matches!(value, 9 | 10 | 13 | 0x20..=0xd7ff | 0xe000..=0xfffd | 0x10000..=0x10ffff) {
        return Err(Error::Shape("XML character reference"));
    }
    char::from_u32(value).ok_or(Error::Shape("XML scalar"))
}
