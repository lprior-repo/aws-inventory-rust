use super::super::{DecodeError, DecodeLimits};
use minicbor::{Decoder, data::Type};
mod value;

enum Frame {
    Map {
        remaining: Option<u64>,
        pending: Option<bool>,
        root: bool,
    },
    Array {
        remaining: Option<u64>,
    },
}

enum Action {
    Key(bool),
    Value(bool),
    Pop,
}

pub(super) fn code(body: &[u8], limits: DecodeLimits) -> Result<Option<String>, DecodeError> {
    if body.is_empty() {
        return Ok(None);
    }
    let mut decoder = Decoder::new(body);
    check_root(&mut decoder)?;
    let mut scanner = Scanner::new(decoder.map().map_err(syntax)?, limits)?;
    while !scanner.frames.is_empty() {
        scanner.step(&mut decoder)?;
    }
    if decoder.position() != body.len() {
        return Err(DecodeError::Invalid("trailing CBOR service error data"));
    }
    Ok(scanner.found)
}

struct Scanner {
    frames: Vec<Frame>,
    nodes: usize,
    found: Option<String>,
    limits: DecodeLimits,
}

impl Scanner {
    fn new(remaining: Option<u64>, limits: DecodeLimits) -> Result<Self, DecodeError> {
        nodes_check(1, limits)?;
        let mut frames = Vec::new();
        frames.try_reserve(1)?;
        frames.push(Frame::Map {
            remaining,
            pending: None,
            root: true,
        });
        Ok(Self {
            frames,
            nodes: 1,
            found: None,
            limits,
        })
    }

    fn step(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        match next_action(&mut self.frames)? {
            Action::Pop => {
                self.frames.pop();
            }
            Action::Key(root) => self.key(decoder, root)?,
            Action::Value(candidate) => self.value(decoder, candidate)?,
        }
        Ok(())
    }

    fn key(&mut self, decoder: &mut Decoder<'_>, root: bool) -> Result<(), DecodeError> {
        if self.break_map(decoder)? {
            return Ok(());
        }
        nodes_add(&mut self.nodes, self.limits)?;
        if decoder.datatype().map_err(syntax)? != Type::String {
            return Err(DecodeError::Invalid(
                "service error CBOR map key is not text",
            ));
        }
        let candidate = root && error_key(decoder.str().map_err(syntax)?);
        let Some(Frame::Map { pending, .. }) = self.frames.last_mut() else {
            return Err(DecodeError::Invalid("CBOR error parser state"));
        };
        *pending = Some(candidate);
        Ok(())
    }

    fn break_map(&mut self, decoder: &mut Decoder<'_>) -> Result<bool, DecodeError> {
        if decoder.datatype().map_err(syntax)? != Type::Break {
            return Ok(false);
        }
        if !matches!(
            self.frames.last(),
            Some(Frame::Map {
                remaining: None,
                ..
            })
        ) {
            return Err(DecodeError::Invalid("unexpected CBOR break"));
        }
        decoder.skip().map_err(syntax)?;
        self.frames.pop();
        Ok(true)
    }

    fn value(&mut self, decoder: &mut Decoder<'_>, candidate: bool) -> Result<(), DecodeError> {
        if self.break_array(decoder)? {
            return Ok(());
        }
        decrement_array(&mut self.frames)?;
        nodes_add(&mut self.nodes, self.limits)?;
        self.consume(decoder, candidate)
    }

    fn break_array(&mut self, decoder: &mut Decoder<'_>) -> Result<bool, DecodeError> {
        if !matches!(self.frames.last(), Some(Frame::Array { remaining: None }))
            || decoder.datatype().map_err(syntax)? != Type::Break
        {
            return Ok(false);
        }
        decoder.skip().map_err(syntax)?;
        self.frames.pop();
        Ok(true)
    }
}

fn check_root(decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
    if !matches!(
        decoder.datatype().map_err(syntax)?,
        Type::Map | Type::MapIndef
    ) {
        return Err(DecodeError::Invalid("service error CBOR is not a map"));
    }
    Ok(())
}

fn error_key(key: &str) -> bool {
    matches!(key, "__type" | "code" | "Code")
}

fn decrement_array(frames: &mut [Frame]) -> Result<(), DecodeError> {
    if let Some(Frame::Array {
        remaining: Some(left),
    }) = frames.last_mut()
    {
        *left = left
            .checked_sub(1)
            .ok_or(DecodeError::Invalid("CBOR array count"))?;
    }
    Ok(())
}

fn next_action(frames: &mut [Frame]) -> Result<Action, DecodeError> {
    let Some(frame) = frames.last_mut() else {
        return Err(DecodeError::Invalid("CBOR error parser state"));
    };
    match frame {
        Frame::Map {
            remaining,
            pending,
            root,
        } => map_action(remaining, pending, *root),
        Frame::Array { remaining } => array_action(remaining),
    }
}

fn map_action(
    remaining: &mut Option<u64>,
    pending: &mut Option<bool>,
    root: bool,
) -> Result<Action, DecodeError> {
    if let Some(candidate) = pending.take() {
        decrement(remaining, "CBOR map count")?;
        return Ok(Action::Value(candidate));
    }
    Ok(if *remaining == Some(0) {
        Action::Pop
    } else {
        Action::Key(root)
    })
}

fn array_action(remaining: &Option<u64>) -> Result<Action, DecodeError> {
    Ok(if *remaining == Some(0) {
        Action::Pop
    } else {
        Action::Value(false)
    })
}

fn decrement(remaining: &mut Option<u64>, message: &'static str) -> Result<(), DecodeError> {
    if let Some(left) = remaining {
        *left = left.checked_sub(1).ok_or(DecodeError::Invalid(message))?;
    }
    Ok(())
}

fn nodes_add(nodes: &mut usize, limits: DecodeLimits) -> Result<(), DecodeError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(DecodeError::Limit("response nodes"))?;
    nodes_check(*nodes, limits)
}

fn nodes_check(nodes: usize, limits: DecodeLimits) -> Result<(), DecodeError> {
    if nodes > limits.nodes() {
        Err(DecodeError::Limit("response nodes"))
    } else {
        Ok(())
    }
}

fn depth_check(depth: usize, limits: DecodeLimits) -> Result<(), DecodeError> {
    if depth >= limits.depth() {
        Err(DecodeError::Limit("response depth"))
    } else {
        Ok(())
    }
}

fn syntax<E>(_: E) -> DecodeError {
    DecodeError::Invalid("service error CBOR syntax")
}
