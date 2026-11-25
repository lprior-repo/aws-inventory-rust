use std::collections::{BTreeMap, BTreeSet};

use crate::HarnessError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Missing,
    Extra,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Missing => "missing_from_sweep",
            Self::Extra => "extra_in_sweep",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Difference<'a> {
    pub direction: Direction,
    pub identity: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Comparison<'a> {
    pub matched: usize,
    pub differences: Vec<Difference<'a>>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("ambiguous identifier {identifier:?}: {candidates:?}")]
pub struct AmbiguousIdentity {
    pub identifier: String,
    pub candidates: Vec<String>,
}

pub fn compare<'a>(
    listed: &'a BTreeSet<String>,
    stored: &'a BTreeSet<String>,
) -> Result<Comparison<'a>, AmbiguousIdentity> {
    let exact = listed.intersection(stored).count();
    let remaining: BTreeSet<&str> = listed.difference(stored).map(String::as_str).collect();
    let mut candidates = BTreeMap::new();
    let mut extras = Vec::new();
    for id in stored.difference(listed) {
        if let Some(identifier) = candidate(id, &remaining) {
            if let Some(first) = candidates.insert(identifier, id.as_str()) {
                return Err(AmbiguousIdentity {
                    identifier: identifier.to_owned(),
                    candidates: vec![first.to_owned(), id.to_owned()],
                });
            }
        } else {
            extras.push(Difference {
                direction: Direction::Extra,
                identity: id,
            });
        }
    }
    let matched = exact.saturating_add(candidates.len());
    let differences = remaining
        .into_iter()
        .filter(|identity| !candidates.contains_key(identity))
        .map(|identity| Difference {
            direction: Direction::Missing,
            identity,
        })
        .chain(extras)
        .collect();
    Ok(Comparison {
        matched,
        differences,
    })
}

fn candidate<'a>(id: &str, remaining: &BTreeSet<&'a str>) -> Option<&'a str> {
    let resource = arn_resource(id).filter(|part| !part.starts_with("arn:"))?;
    remaining.get(resource).copied().or_else(|| {
        let (_, name) = resource.split_once([':', '/'])?;
        if name.contains([':', '/']) {
            return None;
        }
        remaining.get(name).copied()
    })
}

fn arn_resource(id: &str) -> Option<&str> {
    id.strip_prefix("arn:")?
        .splitn(5, ':')
        .nth(4)
        .filter(|part| !part.is_empty())
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rule<'a> {
    pub type_name: &'a str,
    direction: Direction,
    prefix: &'a str,
    pub reason: &'a str,
}

impl Rule<'_> {
    pub fn covers(&self, type_name: &str, difference: &Difference<'_>) -> bool {
        self.type_name == type_name
            && self.direction == difference.direction
            && difference.identity.starts_with(self.prefix)
    }
}

pub fn rules(text: &str) -> Result<Vec<Rule<'_>>, HarnessError> {
    text.lines()
        .enumerate()
        .try_fold(
            (Vec::<Rule<'_>>::new(), false),
            |(mut rules, seen_rule), (line, text)| {
                if text.trim().is_empty() {
                    return Ok((rules, seen_rule));
                }
                if text.starts_with('#') {
                    if seen_rule {
                        return Err(HarnessError::invalid(format!(
                            "Cloud Control explanation comment after first rule at line {}",
                            line.saturating_add(1)
                        )));
                    }
                    return Ok((rules, false));
                }
                let rule = parse_rule(text)?;
                let duplicate = rules.iter().any(|existing| {
                    existing.type_name == rule.type_name
                        && existing.direction == rule.direction
                        && existing.prefix == rule.prefix
                });
                if duplicate {
                    return Err(HarnessError::invalid(format!(
                        "Cloud Control duplicate explanation rule at line {}",
                        line.saturating_add(1)
                    )));
                }
                rules.push(rule);
                Ok((rules, true))
            },
        )
        .map(|(rules, _)| rules)
}

fn parse_rule(line: &str) -> Result<Rule<'_>, HarnessError> {
    let mut columns = line.split('\t');
    let type_name = columns.next().filter(|name| name.starts_with("AWS::"));
    let rule_text = columns.next();
    let reason = columns.next().filter(|reason| !reason.trim().is_empty());
    let (Some(type_name), Some(rule_text), Some(reason), None) =
        (type_name, rule_text, reason, columns.next())
    else {
        return Err(HarnessError::invalid(
            "Cloud Control explanation must have type, rule, reason columns",
        ));
    };
    let (direction, pattern) = rule_text
        .split_once(':')
        .ok_or_else(|| HarnessError::invalid("Cloud Control rule requires direction:prefix*"))?;
    let direction = match direction {
        "missing_from_sweep" => Direction::Missing,
        "extra_in_sweep" => Direction::Extra,
        _ => {
            return Err(HarnessError::invalid(
                "unknown Cloud Control difference direction",
            ));
        }
    };
    let prefix = pattern
        .strip_suffix('*')
        .filter(|prefix| !prefix.is_empty() && !prefix.contains('*'))
        .ok_or_else(|| {
            HarnessError::invalid(
                "Cloud Control rule must end in one wildcard, not name one identity",
            )
        })?;
    Ok(Rule {
        type_name,
        direction,
        prefix,
        reason,
    })
}

#[cfg(test)]
mod tests;
