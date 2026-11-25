use super::super::{DecodeError, DecodeLimits};
use super::Field;
use jiter::{Jiter, Peek};

enum Frame {
    Object { next: Option<bool>, root: bool },
    Array { pending: bool },
}

enum Action {
    Value(bool),
    ObjectNext(bool),
    ArrayNext,
}

pub(super) fn scan(
    body: &[u8],
    limits: DecodeLimits,
    field: Field,
) -> Result<Option<String>, DecodeError> {
    if body.is_empty() {
        return Ok(None);
    }
    let mut parser = Jiter::new(body);
    check_root(&mut parser)?;
    let first = parser.next_object().map_err(syntax)?;
    let mut scanner = Scanner::new(first, limits, field)?;
    while !scanner.frames.is_empty() {
        scanner.step(&mut parser)?;
    }
    parser.finish().map_err(syntax)?;
    Ok(scanner.found)
}

struct Scanner {
    frames: Vec<Frame>,
    nodes: usize,
    found: Option<String>,
    limits: DecodeLimits,
    field: Field,
}

impl Scanner {
    fn new(first: Option<&str>, limits: DecodeLimits, field: Field) -> Result<Self, DecodeError> {
        let mut frames = Vec::new();
        if let Some(first) = first {
            frames.try_reserve(1)?;
            frames.push(Frame::Object {
                next: Some(field.matches(first)),
                root: true,
            });
        }
        let nodes = usize::from(!frames.is_empty());
        nodes_check(nodes, limits)?;
        Ok(Self {
            frames,
            nodes,
            found: None,
            limits,
            field,
        })
    }

    fn step(&mut self, parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
        match action(&mut self.frames)? {
            Action::ObjectNext(root) => object_next(parser, &mut self.frames, root, self.field),
            Action::ArrayNext => array_next(parser, &mut self.frames),
            Action::Value(candidate) => self.value(parser, candidate),
        }
    }

    fn value(&mut self, parser: &mut Jiter<'_>, candidate: bool) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        self.scan_value(parser, candidate)
    }

    fn scan_value(&mut self, parser: &mut Jiter<'_>, candidate: bool) -> Result<(), DecodeError> {
        if candidate {
            return self.scan_candidate(parser);
        }
        match parser.peek().map_err(syntax)? {
            Peek::Object => push_object(parser, &mut self.frames, self.nodes, self.limits),
            Peek::Array => push_array(parser, &mut self.frames, self.nodes, self.limits),
            _ => parser.next_skip().map_err(syntax),
        }
    }

    fn scan_candidate(&mut self, parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
        if parser.peek().map_err(syntax)? != Peek::String {
            return Err(self.field.invalid());
        }
        self.field
            .record(&mut self.found, parser.next_str().map_err(syntax)?)
    }
}

fn check_root(parser: &mut Jiter<'_>) -> Result<(), DecodeError> {
    if parser.peek().map_err(syntax)? != Peek::Object {
        return Err(DecodeError::Invalid("service error JSON is not an object"));
    }
    Ok(())
}

fn action(frames: &mut [Frame]) -> Result<Action, DecodeError> {
    let Some(frame) = frames.last_mut() else {
        return Err(DecodeError::Invalid("JSON error parser state"));
    };
    match frame {
        Frame::Object { next, root } => match next.take() {
            Some(candidate) => Ok(Action::Value(candidate)),
            None => Ok(Action::ObjectNext(*root)),
        },
        Frame::Array { pending } => {
            let value = *pending;
            *pending = false;
            Ok(if value {
                Action::Value(false)
            } else {
                Action::ArrayNext
            })
        }
    }
}

fn object_next(
    parser: &mut Jiter<'_>,
    frames: &mut Vec<Frame>,
    root: bool,
    field: Field,
) -> Result<(), DecodeError> {
    match parser.next_key().map_err(syntax)? {
        Some(key) => {
            let Some(Frame::Object { next, .. }) = frames.last_mut() else {
                return Err(DecodeError::Invalid("JSON error parser state"));
            };
            *next = Some(root && field.matches(key));
        }
        None => {
            frames.pop();
        }
    }
    Ok(())
}

fn array_next(parser: &mut Jiter<'_>, frames: &mut Vec<Frame>) -> Result<(), DecodeError> {
    match parser.array_step().map_err(syntax)? {
        Some(_) => {
            let Some(Frame::Array { pending }) = frames.last_mut() else {
                return Err(DecodeError::Invalid("JSON error parser state"));
            };
            *pending = true;
        }
        None => {
            frames.pop();
        }
    }
    Ok(())
}

fn push_object(
    parser: &mut Jiter<'_>,
    frames: &mut Vec<Frame>,
    nodes: usize,
    limits: DecodeLimits,
) -> Result<(), DecodeError> {
    depth_check(frames.len(), limits)?;
    let first = parser.next_object().map_err(syntax)?;
    if first.is_some() {
        push_frame(
            frames,
            Frame::Object {
                next: Some(false),
                root: false,
            },
        )?;
    }
    nodes_check(nodes, limits)
}

fn push_array(
    parser: &mut Jiter<'_>,
    frames: &mut Vec<Frame>,
    nodes: usize,
    limits: DecodeLimits,
) -> Result<(), DecodeError> {
    depth_check(frames.len(), limits)?;
    let first = parser.next_array().map_err(syntax)?;
    if first.is_some() {
        push_frame(frames, Frame::Array { pending: true })?;
    }
    nodes_check(nodes, limits)
}

fn push_frame(frames: &mut Vec<Frame>, frame: Frame) -> Result<(), DecodeError> {
    frames.try_reserve(1)?;
    frames.push(frame);
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
    DecodeError::Invalid("service error JSON syntax")
}
