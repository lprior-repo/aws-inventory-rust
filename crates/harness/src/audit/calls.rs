//! Enumerate empty SDK `send()` call sites under `crates/aws/src`.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::HarnessError;

use super::mask::{location, mask, source_error};
use super::resolve::{function_at, is_identifier, receiver_service};
use super::tokens::{Token, tokenize};

/// Maximum SDK callsites accepted in one source file.
pub const MAX_CALLS: usize = 4_096;

/// One resolved SDK send call site.
#[derive(Clone, Debug)]
pub struct CallSite {
    /// Resolved `service.operation` identity.
    pub id: String,
    /// Workspace-relative source path.
    pub path: String,
    /// Enclosing function name.
    pub function: String,
    /// SDK service namespace.
    pub sdk_service: String,
    /// SDK operation method.
    pub sdk_operation: String,
    /// Receiver identifier.
    pub receiver: String,
    /// Line of the operation method.
    pub operation_line: usize,
    /// Line of the `send` token.
    pub send_line: usize,
}

impl CallSite {
    /// Render the call site with the keys the Python audit records.
    pub fn to_value(&self) -> Value {
        json!({
            "id": self.id,
            "path": self.path,
            "function": self.function,
            "sdk_service": self.sdk_service,
            "sdk_operation": self.sdk_operation,
            "receiver": self.receiver,
            "operation_line": self.operation_line,
            "send_line": self.send_line,
        })
    }
}

/// Masked source, its tokens and the delimiter pair map.
struct Lexed<'a> {
    path: &'a str,
    characters: &'a [char],
    masked: &'a [char],
    tokens: &'a [Token],
    pairs: &'a BTreeMap<usize, usize>,
    hint: Option<&'a str>,
}

/// Masked characters, tokens and bracket pairs of one source file.
type LexedSource = (Vec<char>, Vec<Token>, BTreeMap<usize, usize>);

/// Resolved method chain and its optional call offset.
type MethodChain = (Vec<(String, usize)>, Option<usize>);

/// Calls found in one source file and the non-empty sends skipped.
#[derive(Default)]
struct Found {
    calls: Vec<CallSite>,
    nonempty: usize,
}

fn token_text(tokens: &[Token], index: usize) -> Option<&str> {
    tokens.get(index).map(|token| token.text.as_str())
}

fn at_send(tokens: &[Token], index: usize) -> bool {
    token_text(tokens, index) == Some(".")
        && token_text(tokens, index.saturating_add(1)) == Some("send")
        && token_text(tokens, index.saturating_add(2)) == Some("(")
}

fn lex(path: &str, characters: &[char]) -> Result<LexedSource, HarnessError> {
    let masked = mask(characters)
        .map_err(|error| source_error(path, characters, error.offset, &error.message))?;
    let (tokens, pairs) = tokenize(&masked)
        .map_err(|error| source_error(path, characters, error.offset, &error.message))?;
    Ok((masked, tokens, pairs))
}

/// Enumerate the empty SDK send calls in one source file.
pub fn calls_in_source(path: &str, source: &str) -> Result<(Vec<CallSite>, usize), HarnessError> {
    calls_in_source_with_hint(path, source, None)
}

/// Enumerate calls with an ancestor-module SDK service hint.
pub fn calls_in_source_with_hint(
    path: &str,
    source: &str,
    hint: Option<&str>,
) -> Result<(Vec<CallSite>, usize), HarnessError> {
    let characters: Vec<char> = source.chars().collect();
    let (masked, tokens, pairs) = lex(path, &characters)?;
    let lexed = Lexed {
        path,
        characters: &characters,
        masked: &masked,
        tokens: &tokens,
        pairs: &pairs,
        hint,
    };
    let mut found = Found::default();
    for index in 0..tokens.len().saturating_sub(3) {
        if at_send(&tokens, index) {
            scan_send(&lexed, index, &mut found)?;
        }
    }
    Ok((found.calls, found.nonempty))
}

fn scan_send(lexed: &Lexed<'_>, index: usize, found: &mut Found) -> Result<(), HarnessError> {
    let send_open = index.saturating_add(2);
    let send_close = index.saturating_add(3);
    if lexed.pairs.get(&send_open).copied() != Some(send_close) {
        found.nonempty = found.nonempty.saturating_add(1);
        return Ok(());
    }
    found.calls.push(resolve_call(lexed, index)?);
    if found.calls.len() > MAX_CALLS {
        return Err(HarnessError::invalid("SDK send call budget exceeded"));
    }
    Ok(())
}

fn chain_error(lexed: &Lexed<'_>, offset: usize) -> HarnessError {
    source_error(
        lexed.path,
        lexed.characters,
        offset,
        "unrecognized empty send call chain",
    )
}

fn token_offset(lexed: &Lexed<'_>, index: usize) -> usize {
    lexed
        .tokens
        .get(index)
        .map(|token| token.offset)
        .unwrap_or(0)
}

fn is_method_prefix(lexed: &Lexed<'_>, opening: usize) -> bool {
    let dot = opening.checked_sub(2);
    let name = opening.checked_sub(1);
    opening >= 2
        && dot.and_then(|value| token_text(lexed.tokens, value)) == Some(".")
        && name
            .and_then(|value| token_text(lexed.tokens, value))
            .map(is_identifier)
            == Some(true)
}

fn method_at(
    lexed: &Lexed<'_>,
    opening: usize,
    call_offset: usize,
) -> Result<Option<(String, usize)>, HarnessError> {
    if !is_method_prefix(lexed, opening) {
        return Ok(None);
    }
    let name = opening.checked_sub(1);
    let method = name
        .and_then(|value| lexed.tokens.get(value))
        .ok_or_else(|| {
            source_error(
                lexed.path,
                lexed.characters,
                call_offset,
                "missing SDK operation",
            )
        })?;
    Ok(Some((method.text.clone(), method.offset)))
}

fn opening_pair(
    lexed: &Lexed<'_>,
    current: usize,
    call_offset: usize,
) -> Result<usize, HarnessError> {
    lexed.pairs.get(&current).copied().ok_or_else(|| {
        source_error(
            lexed.path,
            lexed.characters,
            call_offset,
            "unpaired Rust delimiter",
        )
    })
}

fn method_chain(
    lexed: &Lexed<'_>,
    index: usize,
    call_offset: usize,
) -> Result<MethodChain, HarnessError> {
    let mut cursor = index.checked_sub(1);
    let mut methods: Vec<(String, usize)> = Vec::new();
    while let Some(current) = cursor {
        if token_text(lexed.tokens, current) != Some(")") {
            break;
        }
        let opening = opening_pair(lexed, current, call_offset)?;
        let Some(method) = method_at(lexed, opening, call_offset)? else {
            break;
        };
        methods.push(method);
        cursor = opening.checked_sub(3);
    }
    Ok((methods, cursor))
}

fn receiver_name(
    lexed: &Lexed<'_>,
    cursor: Option<usize>,
    call_offset: usize,
) -> Result<String, HarnessError> {
    let index = cursor.ok_or_else(|| chain_error(lexed, call_offset))?;
    match token_text(lexed.tokens, index) {
        Some(value) if is_identifier(value) => Ok(value.to_owned()),
        _ => Err(chain_error(lexed, call_offset)),
    }
}

fn send_offset(lexed: &Lexed<'_>, index: usize, offset: usize) -> Result<usize, HarnessError> {
    lexed
        .tokens
        .get(index.saturating_add(1))
        .map(|token| token.offset)
        .ok_or_else(|| source_error(lexed.path, lexed.characters, offset, "missing send offset"))
}

fn enclosing_function(lexed: &Lexed<'_>, offset: usize) -> Result<(String, usize), HarnessError> {
    function_at(lexed.masked, offset)
        .map_err(|error| source_error(lexed.path, lexed.characters, offset, &error.to_string()))
}

fn service_of(
    lexed: &Lexed<'_>,
    receiver: &str,
    function_start: usize,
    offset: usize,
) -> Result<String, HarnessError> {
    receiver_service(lexed.masked, receiver, function_start, offset, lexed.hint)
        .map_err(|error| source_error(lexed.path, lexed.characters, offset, &error.to_string()))
}

fn resolve_call(lexed: &Lexed<'_>, index: usize) -> Result<CallSite, HarnessError> {
    let call_offset = token_offset(lexed, index);
    let (methods, cursor) = method_chain(lexed, index, call_offset)?;
    let (operation, operation_offset) = methods
        .last()
        .cloned()
        .ok_or_else(|| chain_error(lexed, call_offset))?;
    let receiver = receiver_name(lexed, cursor, call_offset)?;
    let send_offset = send_offset(lexed, index, call_offset)?;
    let (function, function_start) = enclosing_function(lexed, call_offset)?;
    let service = service_of(lexed, &receiver, function_start, call_offset)?;
    Ok(CallSite {
        id: format!("{service}.{operation}"),
        path: lexed.path.to_owned(),
        function,
        sdk_service: service,
        sdk_operation: operation,
        receiver,
        operation_line: location(lexed.characters, operation_offset).0,
        send_line: location(lexed.characters, send_offset).0,
    })
}

#[cfg(test)]
mod tests;
