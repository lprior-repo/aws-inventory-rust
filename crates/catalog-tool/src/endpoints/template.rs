use super::{Fault, MAX_STRING, eval::Bindings, functions::attr, model::tick};
use serde_json::Value;

enum State {
    Text,
    Open,
    Name(String),
    Close,
}

pub(super) fn expand(input: &str, env: &Bindings) -> Result<String, Fault> {
    let (output, state) =
        input
            .chars()
            .try_fold((String::new(), State::Text), |(mut output, state), ch| {
                let next = step(state, ch, env, &mut output)?;
                Ok::<_, Fault>((output, next))
            })?;
    if !matches!(state, State::Text) {
        return Err(Fault::Template(input.into()));
    }
    Ok(output)
}

fn step(state: State, ch: char, env: &Bindings, output: &mut String) -> Result<State, Fault> {
    match (state, ch) {
        (State::Text, '{') => Ok(State::Open),
        (State::Text, '}') => Ok(State::Close),
        (State::Text, c) | (State::Open, c @ '{') | (State::Close, c @ '}') => written(output, c),
        (State::Open, '}') | (State::Close, _) => Err(Fault::Template("unescaped brace".into())),
        (State::Open, c) => Ok(State::Name(c.to_string())),
        (State::Name(name), '}') => substitute(&name, env, output).map(|()| State::Text),
        (State::Name(_), '{') => Err(Fault::Template("nested brace".into())),
        (State::Name(name), c) => named(name, c),
    }
}

fn substitute(name: &str, env: &Bindings, output: &mut String) -> Result<(), Fault> {
    let value = if let Some((name, path)) = name.split_once('#') {
        env.get(name).and_then(|value| attr(value, path))
    } else {
        env.get(name)
    };
    let value = value
        .and_then(Value::as_str)
        .ok_or_else(|| Fault::Template(name.into()))?;
    append(output, value)
}

pub(super) fn append(output: &mut String, value: &str) -> Result<(), Fault> {
    if output
        .len()
        .checked_add(value.len())
        .is_none_or(|n| n > MAX_STRING)
    {
        return Err(Fault::Bound("expanded string bytes"));
    }
    output
        .try_reserve(value.len())
        .map_err(|_| Fault::Bound("allocation"))?;
    output.push_str(value);
    Ok(())
}

fn character(output: &mut String, value: char) -> Result<(), Fault> {
    append(output, value.encode_utf8(&mut [0; 4]))
}

pub(super) fn expand_value(
    value: &Value,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Value, Fault> {
    tick(depth, steps)?;
    match value {
        Value::String(s) => expand(s, env).map(Value::String),
        Value::Array(a) => a
            .iter()
            .map(|v| expand_value(v, env, depth.saturating_add(1), steps))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(o) => expand_properties(o, env, depth, steps),
        value => Ok(value.clone()),
    }
}

fn written(output: &mut String, ch: char) -> Result<State, Fault> {
    character(output, ch)?;
    Ok(State::Text)
}

fn named(mut name: String, ch: char) -> Result<State, Fault> {
    character(&mut name, ch)?;
    Ok(State::Name(name))
}

pub(super) fn expand_properties(
    object: &serde_json::Map<String, Value>,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Value, Fault> {
    object
        .iter()
        .map(|(key, value)| {
            Ok((
                key.clone(),
                expand_value(value, env, depth.saturating_add(1), steps)?,
            ))
        })
        .collect::<Result<_, Fault>>()
        .map(Value::Object)
}
