use super::{MAX_SQL_BYTES, SqlError};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dialect {
    Legacy,
    Postgres,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SqlLexState<'a> {
    Normal,
    Single,
    Escape,
    Double(usize),
    Backtick,
    Bracket,
    Dollar(&'a [u8]),
    LineComment,
    BlockComment(usize),
}

struct Lexer<'a> {
    state: SqlLexState<'a>,
    skip: usize,
    ended: bool,
    dialect: Dialect,
}

/// Check a single PostgreSQL statement's size, quoting and read-query prefix.
pub fn validate(sql: &str) -> Result<(), SqlError> {
    validate_with(sql, Dialect::Postgres)
}

pub(crate) fn validate_with(sql: &str, dialect: Dialect) -> Result<(), SqlError> {
    if sql.is_empty() || sql.len() > MAX_SQL_BYTES || sql.contains('\0') {
        return Err(SqlError::QueryDenied(
            "empty, oversized, or NUL-containing SQL",
        ));
    }
    validate_query_prefix(sql, dialect)?;
    let lexer = Lexer {
        state: SqlLexState::Normal,
        skip: 0,
        ended: false,
        dialect,
    };
    let lexer = sql
        .bytes()
        .enumerate()
        .try_fold(lexer, |mut lexer, (index, byte)| {
            lexer.step(sql.as_bytes(), index, byte)?;
            Ok::<_, SqlError>(lexer)
        })?;
    if !matches!(lexer.state, SqlLexState::Normal | SqlLexState::LineComment) {
        return Err(SqlError::QueryDenied("unterminated SQL literal or comment"));
    }
    Ok(())
}

impl<'a> Lexer<'a> {
    fn step(&mut self, sql: &'a [u8], index: usize, byte: u8) -> Result<(), SqlError> {
        if self.skip > 0 {
            self.skip = self.skip.saturating_sub(1);
            return Ok(());
        }
        let next = sql.get(index.saturating_add(1)).copied();
        match self.state {
            SqlLexState::Normal => self.lex_normal(sql, index, byte, next)?,
            SqlLexState::LineComment if matches!(byte, b'\n' | b'\r') => {
                self.state = SqlLexState::Normal;
            }
            SqlLexState::BlockComment(depth) => self.comment(byte, next, depth),
            SqlLexState::Bracket if byte == b']' => self.state = SqlLexState::Normal,
            SqlLexState::Single | SqlLexState::Escape if byte == b'\'' => {
                self.close_quote(next, byte);
            }
            SqlLexState::Escape if byte == b'\\' => self.skip = 1,
            SqlLexState::Double(start) if byte == b'"' => {
                self.close_identifier(sql, start, index, next)?;
            }
            SqlLexState::Backtick if byte == b'`' => self.close_quote(next, byte),
            SqlLexState::Dollar(delimiter)
                if byte == b'$'
                    && sql
                        .get(index..)
                        .is_some_and(|tail| tail.starts_with(delimiter)) =>
            {
                self.state = SqlLexState::Normal;
                self.skip = delimiter.len().saturating_sub(1);
            }
            _ => {}
        }
        Ok(())
    }

    fn comment(&mut self, byte: u8, next: Option<u8>, depth: usize) {
        if byte == b'*' && next == Some(b'/') {
            self.state = if depth == 1 {
                SqlLexState::Normal
            } else {
                SqlLexState::BlockComment(depth.saturating_sub(1))
            };
            self.skip = 1;
        } else if self.dialect == Dialect::Postgres && byte == b'/' && next == Some(b'*') {
            self.state = SqlLexState::BlockComment(depth.saturating_add(1));
            self.skip = 1;
        }
    }

    fn close_quote(&mut self, next: Option<u8>, quote: u8) {
        if next == Some(quote) {
            self.skip = 1;
        } else {
            self.state = SqlLexState::Normal;
        }
    }

    fn close_identifier(
        &mut self,
        sql: &[u8],
        start: usize,
        end: usize,
        next: Option<u8>,
    ) -> Result<(), SqlError> {
        if next == Some(b'"') {
            self.skip = 1;
            return Ok(());
        }
        if self.dialect == Dialect::Postgres {
            let identifier = sql
                .get(start..end)
                .ok_or(SqlError::QueryDenied("invalid quoted identifier"))?;
            reject_control_name(quoted_bytes(identifier))?;
        }
        self.state = SqlLexState::Normal;
        Ok(())
    }

    fn lex_normal(
        &mut self,
        sql: &'a [u8],
        index: usize,
        byte: u8,
        next: Option<u8>,
    ) -> Result<(), SqlError> {
        if byte.is_ascii_whitespace() {
            return Ok(());
        }
        if byte == b'-' && next == Some(b'-') {
            self.state = SqlLexState::LineComment;
            self.skip = 1;
            return Ok(());
        }
        if byte == b'/' && next == Some(b'*') {
            self.state = SqlLexState::BlockComment(1);
            self.skip = 1;
            return Ok(());
        }
        if self.ended {
            return Err(SqlError::QueryDenied(
                "multiple SQL statements are forbidden",
            ));
        }
        match byte {
            b';' => self.ended = true,
            b'\'' => self.state = SqlLexState::Single,
            b'"' => self.state = SqlLexState::Double(index.saturating_add(1)),
            b'`' if self.dialect == Dialect::Legacy => self.state = SqlLexState::Backtick,
            b'[' if self.dialect == Dialect::Legacy => self.state = SqlLexState::Bracket,
            b'e' | b'E'
                if self.dialect == Dialect::Postgres
                    && next == Some(b'\'')
                    && token_start(sql, index) =>
            {
                self.state = SqlLexState::Escape;
                self.skip = 1;
            }
            b'$' if self.dialect == Dialect::Postgres && token_start(sql, index) => {
                if let Some(delimiter) = dollar_delimiter(sql, index) {
                    self.state = SqlLexState::Dollar(delimiter);
                    self.skip = delimiter.len().saturating_sub(1);
                }
            }
            _ if self.dialect == Dialect::Postgres => reject_control_identifier(sql, index)?,
            _ => {}
        }
        Ok(())
    }
}

fn reject_control_identifier(sql: &[u8], index: usize) -> Result<(), SqlError> {
    if !token_start(sql, index) {
        return Ok(());
    }
    let tail = sql
        .get(index..)
        .ok_or(SqlError::QueryDenied("invalid SQL identifier"))?;
    if tail.starts_with(b"U&\"") || tail.starts_with(b"u&\"") {
        return Err(SqlError::QueryDenied(
            "Unicode-escaped SQL identifiers are forbidden",
        ));
    }
    let length = tail
        .iter()
        .copied()
        .take_while(|byte| {
            byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'$' || *byte >= 128
        })
        .count();
    let identifier = tail
        .get(..length)
        .ok_or(SqlError::QueryDenied("invalid SQL identifier"))?;
    reject_control_name(identifier.iter().copied())
}

fn quoted_bytes(identifier: &[u8]) -> impl Iterator<Item = u8> + Clone {
    let mut bytes = identifier.iter().copied();
    std::iter::from_fn(move || {
        let byte = bytes.next()?;
        if byte == b'"' {
            bytes.next()?;
        }
        Some(byte)
    })
}

// SQL execution and remote-connection entry points denied by the reader policy.
const SQL_EXECUTORS: &[&str] = &[
    "query_to_xml",
    "query_to_xmlschema",
    "query_to_xml_and_xmlschema",
    "ts_stat",
    "ts_rewrite",
    "crosstab",
    "crosstab2",
    "crosstab3",
    "crosstab4",
    "connectby",
    "dblink",
    "dblink_connect",
    "dblink_connect_u",
    "dblink_disconnect",
    "dblink_exec",
    "dblink_open",
    "dblink_fetch",
    "dblink_close",
    "dblink_send_query",
    "dblink_get_result",
    "dblink_cancel_query",
    "dblink_is_busy",
    "dblink_get_connections",
    "dblink_get_notify",
    "dblink_error_message",
    "dblink_get_pkey",
    "dblink_build_sql_insert",
    "dblink_build_sql_update",
    "dblink_build_sql_delete",
    "dblink_fdw_validator",
    "dblink_current_query",
];

fn reject_control_name(identifier: impl Iterator<Item = u8> + Clone) -> Result<(), SqlError> {
    let identifier = identifier.map(|byte| byte.to_ascii_lowercase());
    let names = [
        "pg_advisory_lock",
        "pg_advisory_lock_shared",
        "pg_advisory_xact_lock",
        "pg_advisory_xact_lock_shared",
        "pg_try_advisory_lock",
        "pg_try_advisory_lock_shared",
        "pg_try_advisory_xact_lock",
        "pg_try_advisory_xact_lock_shared",
        "pg_advisory_unlock",
        "pg_advisory_unlock_shared",
        "pg_advisory_unlock_all",
        "pg_terminate_backend",
        "pg_cancel_backend",
    ];
    if names.iter().any(|name| identifier.clone().eq(name.bytes())) {
        return Err(SqlError::QueryDenied(
            "advisory locks and backend control functions are forbidden",
        ));
    }
    if SQL_EXECUTORS
        .iter()
        .any(|name| identifier.clone().eq(name.bytes()))
    {
        return Err(SqlError::QueryDenied(
            "SQL-executing functions are forbidden",
        ));
    }
    if ["pg_notify", "pg_logical_emit_message"]
        .iter()
        .any(|name| identifier.clone().eq(name.bytes()))
    {
        return Err(SqlError::QueryDenied(
            "notification and logical message functions are forbidden",
        ));
    }
    if identifier.eq("set_config".bytes()) {
        return Err(SqlError::QueryDenied(
            "session configuration functions are forbidden",
        ));
    }
    Ok(())
}

fn token_start(sql: &[u8], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|previous| sql.get(previous))
        .is_none_or(|byte| {
            !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'$' && byte.is_ascii()
        })
}

fn dollar_delimiter(sql: &[u8], index: usize) -> Option<&[u8]> {
    let tail = sql.get(index.checked_add(1)?..)?;
    let end = tail.iter().position(|byte| *byte == b'$')?;
    let tag = tail.get(..end)?;
    if tag.first().is_some_and(u8::is_ascii_digit)
        || !tag
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || !byte.is_ascii())
    {
        return None;
    }
    sql.get(index..index.checked_add(end)?.checked_add(2)?)
}

fn validate_query_prefix(sql: &str, dialect: Dialect) -> Result<(), SqlError> {
    let mut lexer = Lexer {
        state: SqlLexState::Normal,
        skip: 0,
        ended: false,
        dialect,
    };
    let first = sql.bytes().enumerate().find_map(|(index, byte)| {
        if lexer.skip == 0
            && lexer.state == SqlLexState::Normal
            && !byte.is_ascii_whitespace()
            && !(byte == b'-' && sql.as_bytes().get(index.saturating_add(1)) == Some(&b'-'))
            && !(byte == b'/' && sql.as_bytes().get(index.saturating_add(1)) == Some(&b'*'))
        {
            return Some(index);
        }
        // Before the first token, only whitespace and comments are visited.
        match lexer.step(sql.as_bytes(), index, byte) {
            Ok(()) => None,
            Err(_) => Some(index),
        }
    });
    let Some(index) = first else {
        return Err(SqlError::QueryDenied(match dialect {
            Dialect::Legacy => "missing SELECT or WITH query",
            Dialect::Postgres => "missing SQL query",
        }));
    };
    let keyword = sql
        .get(index..)
        .ok_or(SqlError::QueryDenied("invalid SQL prefix"))?
        .split(|ch: char| !ch.is_ascii_alphabetic())
        .next()
        .ok_or(SqlError::QueryDenied("missing SQL keyword"))?;
    let allowed: &[&str] = match dialect {
        Dialect::Legacy => &["SELECT", "WITH"],
        Dialect::Postgres => &["SELECT", "WITH", "VALUES", "EXPLAIN", "TABLE", "SHOW"],
    };
    if allowed
        .iter()
        .any(|allowed| keyword.eq_ignore_ascii_case(allowed))
    {
        return Ok(());
    }
    Err(SqlError::QueryDenied(match dialect {
        Dialect::Legacy => "only SELECT or WITH queries are permitted",
        Dialect::Postgres => {
            "only SELECT, WITH, VALUES, EXPLAIN, TABLE or SHOW queries are permitted"
        }
    }))
}
