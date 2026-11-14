use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SqlLexState {
    Normal,
    Single,
    Double,
    Backtick,
    Bracket,
    LineComment,
    BlockComment,
}

pub(super) fn validate_query(sql: &str, row_limit: usize) -> Result<(), StoreError> {
    if row_limit == 0 || row_limit > MAX_QUERY_ROWS {
        return Err(StoreError::Limit("query row limit"));
    }
    if sql.is_empty() || sql.len() > MAX_SQL_BYTES || sql.contains('\0') {
        return Err(StoreError::QueryDenied(
            "empty, oversized, or NUL-containing SQL",
        ));
    }
    validate_query_prefix(sql)?;
    let mut state = SqlLexState::Normal;
    let mut ended = false;
    let mut skip = false;
    for (index, byte) in sql.bytes().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        let next = index
            .checked_add(1)
            .and_then(|position| sql.as_bytes().get(position))
            .copied();
        match state {
            SqlLexState::Normal => lex_normal(byte, next, &mut state, &mut ended, &mut skip)?,
            SqlLexState::LineComment if byte == b'\n' || byte == b'\r' => {
                state = SqlLexState::Normal
            }
            SqlLexState::BlockComment if byte == b'*' && next == Some(b'/') => {
                state = SqlLexState::Normal;
                skip = true;
            }
            SqlLexState::Bracket if byte == b']' => state = SqlLexState::Normal,
            SqlLexState::Single if byte == b'\'' => close_quote(next, byte, &mut state, &mut skip),
            SqlLexState::Double if byte == b'"' => close_quote(next, byte, &mut state, &mut skip),
            SqlLexState::Backtick if byte == b'`' => close_quote(next, byte, &mut state, &mut skip),
            _ => {}
        }
    }
    if !matches!(state, SqlLexState::Normal | SqlLexState::LineComment) {
        return Err(StoreError::QueryDenied(
            "unterminated SQL literal or comment",
        ));
    }
    Ok(())
}

pub(super) fn close_quote(next: Option<u8>, quote: u8, state: &mut SqlLexState, skip: &mut bool) {
    if next == Some(quote) {
        *skip = true;
    } else {
        *state = SqlLexState::Normal;
    }
}

pub(super) fn lex_normal(
    byte: u8,
    next: Option<u8>,
    state: &mut SqlLexState,
    ended: &mut bool,
    skip: &mut bool,
) -> Result<(), StoreError> {
    if byte.is_ascii_whitespace() {
        return Ok(());
    }
    if byte == b'-' && next == Some(b'-') {
        *state = SqlLexState::LineComment;
        *skip = true;
        return Ok(());
    }
    if byte == b'/' && next == Some(b'*') {
        *state = SqlLexState::BlockComment;
        *skip = true;
        return Ok(());
    }
    if *ended {
        return Err(StoreError::QueryDenied(
            "multiple SQL statements are forbidden",
        ));
    }
    match byte {
        b';' => *ended = true,
        b'\'' => *state = SqlLexState::Single,
        b'"' => *state = SqlLexState::Double,
        b'`' => *state = SqlLexState::Backtick,
        b'[' => *state = SqlLexState::Bracket,
        _ => {}
    }
    Ok(())
}

pub(super) fn validate_query_prefix(sql: &str) -> Result<(), StoreError> {
    let mut state = SqlLexState::Normal;
    let mut skip = false;
    for (index, byte) in sql.bytes().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        let next = index
            .checked_add(1)
            .and_then(|position| sql.as_bytes().get(position))
            .copied();
        match state {
            SqlLexState::LineComment => {
                if byte == b'\n' || byte == b'\r' {
                    state = SqlLexState::Normal;
                }
            }
            SqlLexState::BlockComment => {
                if byte == b'*' && next == Some(b'/') {
                    state = SqlLexState::Normal;
                    skip = true;
                }
            }
            _ if byte.is_ascii_whitespace() => {}
            _ if byte == b'-' && next == Some(b'-') => {
                state = SqlLexState::LineComment;
                skip = true;
            }
            _ if byte == b'/' && next == Some(b'*') => {
                state = SqlLexState::BlockComment;
                skip = true;
            }
            _ => {
                let tail = sql
                    .get(index..)
                    .ok_or(StoreError::QueryDenied("invalid SQL prefix"))?;
                let keyword = tail
                    .split(|ch: char| !ch.is_ascii_alphabetic())
                    .next()
                    .ok_or(StoreError::QueryDenied("missing SQL keyword"))?;
                if keyword.eq_ignore_ascii_case("select") || keyword.eq_ignore_ascii_case("with") {
                    return Ok(());
                }
                return Err(StoreError::QueryDenied(
                    "only SELECT or WITH queries are permitted",
                ));
            }
        }
    }
    Err(StoreError::QueryDenied("missing SELECT or WITH query"))
}
