//! Bounded tokenisation and delimiter pairing of masked Rust text.

use std::collections::BTreeMap;

use super::mask::{LexError, location};

/// Maximum tokens accepted for one audited source file.
pub const MAX_TOKENS: usize = 500_000;

/// One token of the masked source.
#[derive(Clone, Debug)]
pub struct Token {
    /// Token text.
    pub text: String,
    /// Character offset of the token start.
    pub offset: usize,
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn opening_for(closing: &str) -> Option<&'static str> {
    match closing {
        ")" => Some("("),
        "]" => Some("["),
        "}" => Some("{"),
        _ => None,
    }
}

fn closing_for(opening: &str) -> &'static str {
    match opening {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        _ => "",
    }
}

/// Tokenise masked text and pair every delimiter.
pub fn tokenize(masked: &[char]) -> Result<(Vec<Token>, BTreeMap<usize, usize>), LexError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut pairs: BTreeMap<usize, usize> = BTreeMap::new();
    let mut position = 0_usize;
    while position < masked.len() {
        let character = *masked
            .get(position)
            .ok_or_else(|| LexError::new("missing token", position))?;
        if character.is_whitespace() {
            position = position.saturating_add(1);
            continue;
        }
        let (text, next) =
            if character == ':' && masked.get(position.saturating_add(1)) == Some(&':') {
                ("::".to_owned(), position.saturating_add(2))
            } else if is_identifier_start(character) {
                let mut end = position.saturating_add(1);
                while masked.get(end).map(|value| is_identifier_continue(*value)) == Some(true) {
                    end = end.saturating_add(1);
                }
                let slice = masked
                    .get(position..end)
                    .ok_or_else(|| LexError::new("token bounds", position))?;
                (slice.iter().collect(), end)
            } else {
                (character.to_string(), position.saturating_add(1))
            };
        if tokens.len() >= MAX_TOKENS {
            return Err(LexError::new("Rust token budget exceeded", position));
        }
        let index = tokens.len();
        tokens.push(Token {
            text: text.clone(),
            offset: position,
        });
        if matches!(text.as_str(), "(" | "[" | "{") {
            stack.push(index);
        } else if let Some(opening_text) = opening_for(&text) {
            let opening_index = *stack.last().ok_or_else(|| {
                LexError::new(
                    format!("unexpected closing Rust delimiter '{text}'"),
                    position,
                )
            })?;
            let opening = tokens
                .get(opening_index)
                .ok_or_else(|| LexError::new("missing opening delimiter", position))?;
            if opening.text != opening_text {
                let (line, column) = location(masked, opening.offset);
                return Err(LexError::new(
                    format!(
                        "unbalanced Rust delimiters: '{text}' closes '{}' opened at {line}:{column}; expected '{}'",
                        opening.text,
                        closing_for(&opening.text)
                    ),
                    position,
                ));
            }
            stack.pop();
            pairs.insert(opening_index, index);
            pairs.insert(index, opening_index);
        }
        position = next;
    }
    if let Some(opening_index) = stack.last().copied() {
        let opening = tokens
            .get(opening_index)
            .ok_or_else(|| LexError::new("missing opening delimiter", masked.len()))?;
        return Err(LexError::new(
            format!(
                "unclosed Rust delimiter '{}'; expected '{}' before end of file",
                opening.text,
                closing_for(&opening.text)
            ),
            opening.offset,
        ));
    }
    Ok((tokens, pairs))
}
