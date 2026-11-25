use super::{Identifier, Produced, SeedError, Table};
use inventory_core::catalog::{Member, Service, Shape, Target, ValueConstraints};
use inventory_core::record::ValueRef;
use toml::Value;

const MAX_NODES: usize = 4096;
const MAX_STRING: usize = 4096;

enum Task<'a> {
    Visit(Target, ValueConstraints<'a>, usize, String),
    Member(Member<'a>, String, usize),
    Finish(Option<Vec<String>>, usize),
}

pub(super) struct Synthesis<'a, 's> {
    pub service: Service<'a>,
    pub step: &'s str,
    pub identifiers: &'s [Identifier],
    pub earlier: &'s [Produced],
    pub own_input: Option<&'s str>,
}

impl Synthesis<'_, '_> {
    pub fn input(&self, target: Target) -> Result<Value, SeedError> {
        let mut tasks = vec![Task::Visit(
            target,
            ValueConstraints::default(),
            0,
            String::new(),
        )];
        let mut values = Vec::new();
        std::iter::from_fn(|| {
            let task = tasks.pop()?;
            Some(self.advance(task, &mut tasks, &mut values))
        })
        .take(MAX_NODES)
        .try_for_each(|result| result)?;
        if !tasks.is_empty() || values.len() != 1 {
            return Err(SeedError::Invalid("draft input exceeds node budget"));
        }
        values
            .pop()
            .ok_or(SeedError::Invalid("draft input is missing"))
    }

    fn advance<'a>(
        &self,
        task: Task<'a>,
        tasks: &mut Vec<Task<'a>>,
        values: &mut Vec<Value>,
    ) -> Result<(), SeedError>
    where
        Self: 'a,
    {
        match task {
            Task::Member(member, path, depth) => {
                if member.wire()?.sensitive()? {
                    return Err(SeedError::Message(format!("sensitive member {path}")));
                }
                let member_shape = shape(self.service, member.target)?;
                if member_shape
                    .map(|shape| shape.wire()?.sensitive())
                    .transpose()?
                    .is_some_and(|v| v)
                {
                    return Err(SeedError::Message(format!("sensitive member {path}")));
                }
                if let Some(value) = self.reference(member, member_shape, &path, depth)? {
                    values.push(value);
                } else {
                    tasks.push(Task::Visit(
                        member.target,
                        member.value_constraints()?,
                        depth,
                        path,
                    ));
                }
            }
            Task::Visit(target, constraints, depth, path) => {
                self.visit(target, constraints, depth, path, tasks, values)?;
            }
            Task::Finish(keys, count) => {
                let start = values
                    .len()
                    .checked_sub(count)
                    .ok_or(SeedError::Invalid("draft value stack is invalid"))?;
                let children = values.split_off(start);
                values.push(match keys {
                    Some(keys) => Value::Table(keys.into_iter().zip(children).collect()),
                    None => Value::Array(children),
                });
            }
        }
        Ok(())
    }

    fn visit<'a>(
        &self,
        target: Target,
        constraints: ValueConstraints<'a>,
        depth: usize,
        path: String,
        tasks: &mut Vec<Task<'a>>,
        values: &mut Vec<Value>,
    ) -> Result<(), SeedError>
    where
        Self: 'a,
    {
        if depth > 4 {
            return Err(SeedError::Invalid("draft required input exceeds depth 4"));
        }
        let shape = shape(self.service, target)?;
        let kind = match shape {
            Some(shape) => shape.kind()?,
            None => prelude(target)?,
        };
        if shape
            .map(|shape| shape.wire()?.sensitive())
            .transpose()?
            .is_some_and(|v| v)
        {
            return Err(SeedError::Invalid("sensitive input shape"));
        }
        if let Some(value) = shape.map(Shape::first_enum_value).transpose()?.flatten() {
            values.push(enum_value(value)?);
            return Ok(());
        }
        match (kind, shape) {
            ("structure", Some(shape)) => Self::structure(shape, depth, &path, tasks)?,
            ("list", Some(shape)) => {
                if shape.element_wire()?.sensitive()? {
                    return Err(SeedError::Invalid("sensitive list element"));
                }
                tasks.push(Task::Finish(None, 1));
                tasks.push(Task::Visit(
                    shape
                        .element()?
                        .ok_or(SeedError::Invalid("draft list element is missing"))?,
                    shape.element_constraints()?,
                    depth.saturating_add(1),
                    format!("{path}.0"),
                ));
            }
            _ => values.push(scalar(kind, shape, constraints, self.step)?),
        }
        Ok(())
    }

    fn structure<'a>(
        shape: Shape<'a>,
        depth: usize,
        path: &str,
        tasks: &mut Vec<Task<'a>>,
    ) -> Result<(), SeedError> {
        let members = shape
            .members()?
            .filter_map(|member| match member {
                Ok(member) if !member.required => None,
                other => Some(other),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if members.len() > 64 || tasks.len().saturating_add(members.len()) > MAX_NODES {
            return Err(SeedError::Invalid("draft structure exceeds input budget"));
        }
        let keys = members
            .iter()
            .map(|member| Ok(member.name()?.to_owned()))
            .collect::<Result<Vec<_>, SeedError>>()?;
        tasks.push(Task::Finish(Some(keys.clone()), members.len()));
        tasks.extend(members.into_iter().zip(keys).rev().map(|(member, name)| {
            let path = if path.is_empty() {
                name
            } else {
                format!("{path}.{name}")
            };
            Task::Member(member, path, depth.saturating_add(1))
        }));
        Ok(())
    }

    fn reference(
        &self,
        member: Member<'_>,
        member_shape: Option<Shape<'_>>,
        path: &str,
        depth: usize,
    ) -> Result<Option<Value>, SeedError> {
        let kind = match member_shape {
            Some(shape) => shape.kind()?,
            None => prelude(member.target)?,
        };
        if !matches!(kind, "string" | "enum" | "list") || self.own_input == Some(path) {
            return Ok(None);
        }
        let name = member.name()?;
        let name_is_identifier = self
            .identifiers
            .iter()
            .any(|identifier| super::identifier_matches(identifier, name))
            || matches!(name, "arn" | "id" | "arns" | "ids")
            || name.ends_with("Arn")
            || name.ends_with("Id")
            || name.ends_with("Arns")
            || name.ends_with("Ids");
        if !name_is_identifier {
            return Ok(None);
        }
        if depth > 4 {
            return Err(SeedError::Invalid("draft required input exceeds depth 4"));
        }
        if kind == "list" {
            let list = member_shape.ok_or(SeedError::Invalid("draft list shape is missing"))?;
            if !self.reference_list(list)? {
                return Ok(None);
            }
        }
        let value = self.producer(name)?;
        Ok(Some(if kind == "list" {
            Value::Array(vec![value])
        } else {
            value
        }))
    }

    fn reference_list(&self, list: Shape<'_>) -> Result<bool, SeedError> {
        if list.element_wire()?.sensitive()? {
            return Err(SeedError::Invalid("sensitive list element"));
        }
        let target = list
            .element()?
            .ok_or(SeedError::Invalid("draft list element is missing"))?;
        let element = shape(self.service, target)?;
        if element
            .map(|shape| shape.wire()?.sensitive())
            .transpose()?
            .is_some_and(|value| value)
        {
            return Err(SeedError::Invalid("sensitive input shape"));
        }
        let kind = match element {
            Some(shape) => shape.kind()?,
            None => prelude(target)?,
        };
        Ok(matches!(kind, "string" | "enum"))
    }

    fn producer(&self, name: &str) -> Result<Value, SeedError> {
        let mut matches = self.earlier.iter().filter(|output| output.name == name);
        let Some(output) = matches.next() else {
            return Err(SeedError::Message(format!(
                "identifier {name} needs exactly one earlier producer; found 0"
            )));
        };
        if !matches.clone().all(|other| other.step == output.step) {
            let count = matches
                .map(|other| other.step.as_str())
                .chain(std::iter::once(output.step.as_str()))
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            return Err(SeedError::Message(format!(
                "identifier {name} needs exactly one earlier producer; found {count}"
            )));
        }
        Ok(Value::Table(Table::from_iter([
            (String::from("from"), Value::String(output.step.clone())),
            (String::from("path"), Value::String(output.path.clone())),
        ])))
    }
}

pub(super) fn shape(service: Service<'_>, target: Target) -> Result<Option<Shape<'_>>, SeedError> {
    match target {
        Target::Shape(index) => Ok(Some(
            service
                .shape(index)?
                .ok_or(SeedError::Invalid("draft shape is missing"))?,
        )),
        Target::Prelude(_) => Ok(None),
        _ => Err(SeedError::Invalid("draft target is unknown")),
    }
}

fn prelude(target: Target) -> Result<&'static str, SeedError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        _ => Err(SeedError::Invalid("draft target is not a prelude")),
    }
}

fn enum_value(value: ValueRef<'_>) -> Result<Value, SeedError> {
    match value {
        ValueRef::Text(text) => Ok(Value::String(text.into())),
        ValueRef::Int(value) => Ok(Value::Integer(value)),
        _ => Err(SeedError::Invalid("draft enum value is not scalar")),
    }
}

fn scalar(
    kind: &str,
    shape: Option<Shape<'_>>,
    constraints: ValueConstraints<'_>,
    step: &str,
) -> Result<Value, SeedError> {
    match kind {
        "string" => {
            let target = shape
                .map(Shape::value_constraints)
                .transpose()?
                .unwrap_or_default();
            Ok(Value::String(string(step, target, constraints)?))
        }
        "byte" | "short" | "integer" | "long" | "bigInteger" => {
            let target = shape
                .map(Shape::value_constraints)
                .transpose()?
                .unwrap_or_default();
            integer(target, constraints)
        }
        "boolean" => Ok(Value::Boolean(false)),
        "unit" => Ok(Value::Table(Table::new())),
        _ => Err(SeedError::Message(format!(
            "cannot synthesize required {kind} shape"
        ))),
    }
}

fn integer(target: ValueConstraints<'_>, member: ValueConstraints<'_>) -> Result<Value, SeedError> {
    if target.integer_range_unrepresentable || member.integer_range_unrepresentable {
        return Err(SeedError::Invalid(
            "draft integer range is not representable as i64",
        ));
    }
    let minimum = target
        .min_integer
        .into_iter()
        .chain(member.min_integer)
        .max();
    let maximum = target
        .max_integer
        .into_iter()
        .chain(member.max_integer)
        .min();
    if matches!((minimum, maximum), (Some(min), Some(max)) if min > max) {
        return Err(SeedError::Invalid(
            "draft integer range cannot be satisfied",
        ));
    }
    let value = minimum.unwrap_or(1);
    Ok(Value::Integer(
        maximum.map_or(value, |maximum| value.min(maximum)),
    ))
}

pub(super) fn string(
    step: &str,
    target: ValueConstraints<'_>,
    member: ValueConstraints<'_>,
) -> Result<String, SeedError> {
    let minimum = target
        .min_length
        .into_iter()
        .chain(member.min_length)
        .max()
        .map_or(1, |v| v.max(1));
    let maximum = target
        .max_length
        .into_iter()
        .chain(member.max_length)
        .min()
        .map_or(MAX_STRING, |v| v.min(MAX_STRING));
    let (minimum, maximum, allowed) = [target.pattern, member.pattern]
        .into_iter()
        .flatten()
        .try_fold(
            (
                minimum,
                maximum,
                (b' '..=b'~').map(char::from).collect::<Vec<_>>(),
            ),
            |(minimum, maximum, allowed), pattern| {
                let (characters, low, high) = character_class(pattern)?.ok_or(
                    SeedError::Invalid("draft pattern is not a simple character class"),
                )?;
                Ok::<_, SeedError>((
                    minimum.max(low),
                    maximum.min(high),
                    allowed
                        .into_iter()
                        .filter(|c| characters.contains(c))
                        .collect(),
                ))
            },
        )?;
    if minimum > maximum || allowed.is_empty() {
        return Err(SeedError::Invalid(
            "draft string constraints cannot be satisfied",
        ));
    }
    string_template(step, minimum, maximum, &allowed)
}

fn string_template(
    step: &str,
    minimum: usize,
    maximum: usize,
    allowed: &[char],
) -> Result<String, SeedError> {
    let filler = allowed
        .iter()
        .copied()
        .find(|c| *c == 'a')
        .or_else(|| allowed.first().copied())
        .ok_or(SeedError::Invalid("draft character class is empty"))?;
    let base = format!("inv-{step}-1");
    let size = base.len().max(minimum).min(maximum);
    let prefix = size.saturating_sub(1);
    let text = base
        .chars()
        .chain(std::iter::repeat(filler))
        .take(prefix)
        .map(|c| if allowed.contains(&c) { c } else { filler })
        .collect::<String>();
    // Replacing one character by a u16 repeat number can add four characters.
    if size.saturating_add(4) <= maximum
        && (b'0'..=b'9').all(|digit| allowed.contains(&char::from(digit)))
    {
        return Ok(format!("{text}{{n}}"));
    }
    if allowed.contains(&'1') {
        return Ok(format!("{text}1"));
    }
    let text = format!("{text}{filler}");
    Ok(text)
}

fn character_class(pattern: &str) -> Result<Option<(Vec<char>, usize, usize)>, SeedError> {
    let pattern = pattern.strip_prefix('^').map_or(pattern, |v| v);
    let pattern = pattern.strip_suffix('$').map_or(pattern, |v| v);
    let Some((class, count)) = pattern.strip_prefix('[').and_then(|p| p.split_once(']')) else {
        return Ok(None);
    };
    let (low, high) = match count {
        "" => (1, 1),
        "*" => (0, MAX_STRING),
        "+" => (1, MAX_STRING),
        "?" => (0, 1),
        count
            if count.starts_with('{')
                && count.ends_with('}')
                && count
                    .chars()
                    .all(|c| c.is_ascii_digit() || matches!(c, '{' | '}' | ',')) =>
        {
            class_count(count)?
        }
        _ => return Ok(None),
    };
    let (class, negated) = class
        .strip_prefix('^')
        .map_or((class, false), |v| (v, true));
    let atoms = match class_atoms(class) {
        Ok(atoms) => atoms,
        Err(SeedError::Invalid(
            "draft pattern character class is unsupported" | "draft pattern escape is unsupported",
        )) => return Ok(None),
        Err(error) => return Err(error),
    };
    let allowed = (b' '..=b'~')
        .map(char::from)
        .filter(|c| atoms.iter().any(|atom| atom.accepts(*c)) != negated)
        .collect();
    Ok(Some((allowed, low, high)))
}

#[derive(Clone, Copy)]
enum Atom {
    Literal(char),
    Escaped(char),
    Range(char, char),
    Word,
    Digit,
    Space,
}

impl Atom {
    fn accepts(self, character: char) -> bool {
        match self {
            Self::Literal(value) | Self::Escaped(value) => value == character,
            Self::Range(start, end) => start <= character && character <= end,
            Self::Word => character.is_ascii_alphanumeric() || character == '_',
            Self::Digit => character.is_ascii_digit(),
            Self::Space => character.is_ascii_whitespace(),
        }
    }
}

fn class_atoms(class: &str) -> Result<Vec<Atom>, SeedError> {
    if !class.is_ascii() {
        return Err(SeedError::Invalid(
            "draft pattern character class is unsupported",
        ));
    }
    let mut chars = class.chars();
    let tokens = std::iter::from_fn(|| {
        chars.next().map(|character| {
            if character != '\\' {
                return Ok(Atom::Literal(character));
            }
            match chars.next() {
                Some('w') => Ok(Atom::Word),
                Some('d') => Ok(Atom::Digit),
                Some('s') => Ok(Atom::Space),
                Some(value) if value.is_ascii_punctuation() => Ok(Atom::Escaped(value)),
                _ => Err(SeedError::Invalid("draft pattern escape is unsupported")),
            }
        })
    })
    .collect::<Result<Vec<_>, _>>()?;
    tokens
        .iter()
        .enumerate()
        .scan(0usize, |skip, (index, atom)| {
            if *skip != 0 {
                *skip = skip.saturating_sub(1);
                return Some(None);
            }
            if let (Atom::Literal(start), Some(Atom::Literal('-')), Some(Atom::Literal(end))) = (
                atom,
                tokens.get(index.saturating_add(1)),
                tokens.get(index.saturating_add(2)),
            ) {
                *skip = 2;
                return Some(Some(if start <= end {
                    Ok(Atom::Range(*start, *end))
                } else {
                    Err(SeedError::Invalid("draft pattern range is invalid"))
                }));
            }
            Some(Some(Ok(*atom)))
        })
        .flatten()
        .collect()
}

fn class_count(count: &str) -> Result<(usize, usize), SeedError> {
    let count = count
        .strip_prefix('{')
        .and_then(|c| c.strip_suffix('}'))
        .ok_or(SeedError::Invalid(
            "draft pattern quantifier is unsupported",
        ))?;
    let parse = |text: &str| {
        text.parse::<usize>()
            .map_err(|_| SeedError::Invalid("draft pattern count is invalid"))
    };
    match count.split_once(',') {
        Some((low, "")) => Ok((parse(low)?, MAX_STRING)),
        Some((low, high)) => Ok((parse(low)?, parse(high)?)),
        None => Ok((parse(count)?, parse(count)?)),
    }
}
