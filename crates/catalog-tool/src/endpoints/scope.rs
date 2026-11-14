use super::model::{Condition, Document, Rule, RuleKind};
use super::{Fault, MAX_DEPTH};
use std::collections::BTreeSet;

pub(super) fn validate(document: &Document) -> Result<(), Fault> {
    let parameters = document.parameters.keys().map(String::as_str).collect();
    rules(&document.rules, &parameters, 0)
}

fn rules<'a>(items: &'a [Rule], inherited: &BTreeSet<&'a str>, depth: usize) -> Result<(), Fault> {
    if depth > MAX_DEPTH {
        return Err(Fault::Bound("depth"));
    }
    items.iter().try_for_each(|rule| {
        let mut scope = inherited.clone();
        assignments(&rule.conditions, &mut scope)?;
        if let RuleKind::Tree { rules: children } = &rule.kind {
            rules(children, &scope, depth.saturating_add(1))?;
        }
        Ok(())
    })
}

fn assignments<'a>(
    conditions: &'a [Condition],
    scope: &mut BTreeSet<&'a str>,
) -> Result<(), Fault> {
    conditions
        .iter()
        .filter_map(|condition| condition.assign.as_deref())
        .try_for_each(|name| {
            if !scope.insert(name) {
                return Err(Fault::Malformed(format!("duplicate assignment: {name}")));
            }
            Ok(())
        })
}
