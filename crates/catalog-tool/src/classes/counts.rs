use super::{Class, Error, OperationFacts};
use std::{collections::BTreeMap, fmt::Write};

pub(super) const FIELDS: [&str; 13] = [
    "operations",
    "enumerate",
    "expand",
    "denied",
    "unclassified",
    "denied content",
    "denied secret",
    "denied paid",
    "denied mutating",
    "denied reference-data",
    "no_read_verb",
    "readonly_denied",
    "readonly_unclassified",
];

#[derive(Debug, Default)]
pub(super) struct Counts(BTreeMap<&'static str, u64>);

impl Counts {
    pub fn record(&mut self, facts: &OperationFacts<'_>, class: Class) -> Result<(), Error> {
        self.increment("operations")?;
        self.increment(match class {
            Class::Enumerate => "enumerate",
            Class::Expand => "expand",
            Class::Denied(_) => "denied",
            Class::Unclassified => "unclassified",
        })?;
        if let Class::Denied(reason) = class {
            self.increment(reason_key(reason))?;
        }
        self.diagnostics(facts, class)
    }

    fn diagnostics(&mut self, facts: &OperationFacts<'_>, class: Class) -> Result<(), Error> {
        if !crate::smithy::READ_PREFIXES
            .iter()
            .any(|p| facts.name.starts_with(p))
        {
            if class != Class::Denied(super::DenyReason::Mutating) {
                return Err(Error::CountMismatch);
            }
            self.increment("no_read_verb")?;
        }
        match (facts.readonly, class) {
            (true, Class::Denied(_)) => self.increment("readonly_denied"),
            (true, Class::Unclassified) => self.increment("readonly_unclassified"),
            _ => Ok(()),
        }
    }

    fn increment(&mut self, key: &'static str) -> Result<(), Error> {
        let count = self.0.entry(key).or_default();
        *count = count.checked_add(1).ok_or(Error::CountMismatch)?;
        Ok(())
    }

    pub fn count(&self, key: &str) -> u64 {
        let Some(count) = self.0.get(key) else {
            return 0;
        };
        *count
    }

    pub fn validate(&self) -> Result<(), Error> {
        let sum = ["enumerate", "expand", "denied", "unclassified"]
            .iter()
            .try_fold(0_u64, |sum, key| sum.checked_add(self.count(key)))
            .ok_or(Error::CountMismatch)?;
        if sum != self.count("operations") {
            return Err(Error::CountMismatch);
        }
        let denied = FIELDS
            .iter()
            .filter(|key| key.starts_with("denied "))
            .try_fold(0_u64, |sum, key| sum.checked_add(self.count(key)))
            .ok_or(Error::CountMismatch)?;
        if denied != self.count("denied") {
            return Err(Error::CountMismatch);
        }
        Ok(())
    }

    pub fn render_service(&self, text: &mut String, service: &str) -> Result<(), Error> {
        self.validate()?;
        writeln!(
            text,
            "service {service} {} {} {} {}",
            self.count("enumerate"),
            self.count("expand"),
            self.count("denied"),
            self.count("unclassified")
        )?;
        Ok(())
    }
}

fn reason_key(reason: super::DenyReason) -> &'static str {
    use super::DenyReason;
    match reason {
        DenyReason::Content => "denied content",
        DenyReason::Secret => "denied secret",
        DenyReason::Paid => "denied paid",
        DenyReason::Mutating => "denied mutating",
        DenyReason::ReferenceData => "denied reference-data",
    }
}

#[cfg(test)]
mod tests;
