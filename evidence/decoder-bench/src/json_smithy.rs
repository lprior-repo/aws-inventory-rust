use crate::{
    error::{Error, Result, syntax},
    scalar::Kind,
    sink::Sink,
};
use aws_smithy_json::deserialize::{Token, json_token_iter};
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    json_token_iter(input).try_for_each(|token| match token.map_err(syntax)? {
        Token::StartObject { .. } => sink.begin(true),
        Token::EndObject { .. } => sink.end(true),
        Token::StartArray { .. } => sink.begin(false),
        Token::EndArray { .. } => sink.end(false),
        Token::ObjectKey { key, .. } => sink.key(&key.to_unescaped().map_err(syntax)?),
        Token::ValueString { value, .. } => {
            sink.scalar(&value.to_unescaped().map_err(syntax)?, Kind::Text)
        }
        Token::ValueBool { value, .. } => {
            sink.scalar(if value { "true" } else { "false" }, Kind::Bool)
        }
        Token::ValueNull { .. } => sink.scalar("", Kind::Null),
        Token::ValueNumber { offset, .. } => {
            sink.scalar(raw_number(input, offset.0)?, Kind::Number)
        }
    })
}
fn raw_number(input: &[u8], start: usize) -> Result<&str> {
    let tail = input.get(start..).ok_or(Error::Bound)?;
    let len = tail
        .iter()
        .take_while(|b| matches!(b, b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E'))
        .count();
    std::str::from_utf8(tail.get(..len).ok_or(Error::Bound)?).map_err(syntax)
}
