use super::{DecodeError, ResponseContext, Token};
use quick_xml::events::{BytesRef, BytesStart};

pub(super) fn push_leaf<'a>(
    token: Token<'a>,
    tokens: &mut Vec<Token<'a>>,
    parents: &mut Vec<Option<usize>>,
    mates: &mut Vec<usize>,
    stack: &[usize],
) -> Result<(), DecodeError> {
    reserve(tokens, parents, mates)?;
    let index = tokens.len();
    tokens.push(token);
    parents.push(stack.last().copied());
    mates.push(index);
    Ok(())
}

pub(super) fn reserve<'a>(
    tokens: &mut Vec<Token<'a>>,
    parents: &mut Vec<Option<usize>>,
    mates: &mut Vec<usize>,
) -> Result<(), DecodeError> {
    tokens.try_reserve(1)?;
    parents.try_reserve(1)?;
    mates.try_reserve(1)?;
    Ok(())
}

pub(super) fn count_node(
    count: &mut usize,
    context: ResponseContext<'_>,
) -> Result<(), DecodeError> {
    *count = count
        .checked_add(1)
        .ok_or(DecodeError::Limit("response nodes"))?;
    if *count > context.limits.nodes() {
        return Err(DecodeError::Limit("response nodes"));
    }
    Ok(())
}

pub(super) fn count_attributes(
    start: &BytesStart<'_>,
    count: &mut usize,
    context: ResponseContext<'_>,
) -> Result<(), DecodeError> {
    for attribute in start.attributes().with_checks(true) {
        let attribute = attribute.map_err(|_| DecodeError::Invalid("XML attribute"))?;
        count_node(count, context)?;
        attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(|_| DecodeError::Invalid("XML attribute value"))?;
    }
    Ok(())
}

pub(super) fn reference(reference: BytesRef<'_>) -> Result<char, DecodeError> {
    if let Some(value) = reference
        .resolve_char_ref()
        .map_err(|_| DecodeError::Invalid("XML reference"))?
    {
        return if valid_character(value) {
            Ok(value)
        } else {
            Err(DecodeError::Invalid("XML character"))
        };
    }
    match reference.as_ref() {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        _ => Err(DecodeError::Invalid("XML entity")),
    }
}

fn valid_character(character: char) -> bool {
    matches!(u32::from(character), 9 | 10 | 13 | 0x20..=0xd7ff | 0xe000..=0xfffd | 0x10000..=0x10ffff)
}

pub(super) fn validate_outside_text(
    tokens: &[Token<'_>],
    parents: &[Option<usize>],
    root: usize,
) -> Result<(), DecodeError> {
    for (index, token) in tokens.iter().enumerate() {
        if index < root || parents.get(index).copied().flatten().is_none() && index != root {
            match token {
                Token::Text(text)
                    if !text.as_ref().bytes().all(|byte| byte.is_ascii_whitespace()) =>
                {
                    return Err(DecodeError::Invalid("XML text outside root"));
                }
                Token::CData(_) | Token::Reference(_) => {
                    return Err(DecodeError::Invalid("XML content outside root"));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
