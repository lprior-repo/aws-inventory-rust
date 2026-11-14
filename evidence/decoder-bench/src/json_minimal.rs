use crate::{
    error::{Error, Result},
    minimal_lex::Lexer,
    scalar::Kind,
    sink::Sink,
};
#[derive(Clone, Copy)]
enum Task {
    Value,
    Object(bool),
    Array(bool),
}
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut lexer = Lexer { input, pos: 0 };
    let mut tasks = vec![Task::Value];
    std::iter::from_fn(|| {
        tasks.pop().map(|task| match task {
            Task::Value => value(&mut lexer, sink, &mut tasks),
            Task::Object(first) => object(&mut lexer, sink, &mut tasks, first),
            Task::Array(first) => array(&mut lexer, sink, &mut tasks, first),
        })
    })
    .take(input.len().saturating_mul(2).saturating_add(1))
    .try_for_each(|r| r)?;
    if !tasks.is_empty() || lexer.peek().is_some() {
        return Err(Error::Shape("trailing data"));
    }
    Ok(())
}
fn value(l: &mut Lexer<'_>, s: &mut Sink, tasks: &mut Vec<Task>) -> Result<()> {
    match l.peek().ok_or(Error::Shape("value EOF"))? {
        b'{' => {
            l.eat(b'{')?;
            s.begin(true)?;
            tasks.push(Task::Object(true));
            Ok(())
        }
        b'[' => {
            l.eat(b'[')?;
            s.begin(false)?;
            tasks.push(Task::Array(true));
            Ok(())
        }
        b'"' => s.scalar(&l.string()?, Kind::Text),
        b't' => {
            l.literal(b"true")?;
            s.scalar("true", Kind::Bool)
        }
        b'f' => {
            l.literal(b"false")?;
            s.scalar("false", Kind::Bool)
        }
        b'n' => {
            l.literal(b"null")?;
            s.scalar("", Kind::Null)
        }
        b'-' | b'0'..=b'9' => s.scalar(l.number()?, Kind::Number),
        _ => Err(Error::Shape("baseline value")),
    }
}
fn object(l: &mut Lexer<'_>, s: &mut Sink, tasks: &mut Vec<Task>, first: bool) -> Result<()> {
    if l.peek() == Some(b'}') {
        l.eat(b'}')?;
        return s.end(true);
    }
    if !first {
        l.eat(b',')?;
    }
    s.key(&l.string()?)?;
    l.eat(b':')?;
    tasks.extend([Task::Object(false), Task::Value]);
    Ok(())
}
fn array(l: &mut Lexer<'_>, s: &mut Sink, tasks: &mut Vec<Task>, first: bool) -> Result<()> {
    if l.peek() == Some(b']') {
        l.eat(b']')?;
        return s.end(false);
    }
    if !first {
        l.eat(b',')?;
    }
    tasks.extend([Task::Array(false), Task::Value]);
    Ok(())
}
