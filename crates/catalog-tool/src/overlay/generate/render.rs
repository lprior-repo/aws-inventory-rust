use crate::overlay::{
    Identity, ItemSelector, OperationOverlay, OverlayClass, OverlayError, Paginator, Scope,
    ServiceOverlay, SkipEvidence,
};

#[derive(Default)]
struct Output(String);

impl Output {
    fn append(&mut self, text: &str) -> Result<(), OverlayError> {
        let length = self.0.len().checked_add(text.len()).ok_or_else(limit)?;
        if length > crate::overlay::MAX_FILE_BYTES {
            return Err(limit());
        }
        self.0.try_reserve(text.len()).map_err(|_| limit())?;
        self.0.push_str(text);
        Ok(())
    }

    fn quoted(&mut self, value: &str) -> Result<(), OverlayError> {
        self.append("\"")?;
        for character in value.chars() {
            match character {
                '\\' => self.append("\\\\")?,
                '"' => self.append("\\\"")?,
                character if character.is_control() => {
                    return Err(invalid("control character in generated string"));
                }
                character => self.append(character.encode_utf8(&mut [0; 4]))?,
            }
        }
        self.append("\"")
    }

    fn line(&mut self, key: &str, value: &str) -> Result<(), OverlayError> {
        self.append(key)?;
        self.append(" = ")?;
        self.quoted(value)?;
        self.append("\n")
    }

    fn table(&mut self, operation: &str, suffix: &str) -> Result<(), OverlayError> {
        self.operation_table(None, operation, suffix)
    }

    fn operation_table(
        &mut self,
        service: Option<&str>,
        operation: &str,
        suffix: &str,
    ) -> Result<(), OverlayError> {
        self.append("[")?;
        if let Some(service) = service {
            self.append("service.")?;
            self.quoted(service)?;
            self.append(".")?;
        }
        self.append("operation.")?;
        self.append(operation)?;
        self.append(suffix)?;
        self.append("]\n")
    }
}

pub(super) fn render(policy: &ServiceOverlay) -> Result<String, OverlayError> {
    if policy.operations().len() > crate::overlay::MAX_OPERATIONS {
        return Err(limit());
    }
    let mut output = Output::default();
    let format = if policy
        .operations()
        .values()
        .any(|entry| matches!(entry.items, Some(ItemSelector::Nested(_))))
    {
        2
    } else {
        1
    };
    output.append(&format!("format = {format}\n"))?;
    output.line("reviewed", super::GENERATED_RULES_DATE)?;
    output.line("reviewer", "generated")?;
    for (name, entry) in policy.operations() {
        output.append("\n")?;
        output.table(name.as_str(), "")?;
        operation(&mut output, name.as_str(), entry)?;
    }
    Ok(output.0)
}

pub fn render_skip_receipts(policy: &ServiceOverlay) -> Result<String, OverlayError> {
    let mut output = Output::default();
    output
        .append("# Merge the operation tables under this service into its existing overlay.\n")?;
    policy.operations().iter().try_for_each(|(parent, entry)| {
        let service = Some(policy.service());
        output.operation_table(service, parent.as_str(), "")?;
        skips(&mut output, entry)?;
        receipts(&mut output, service, parent.as_str(), entry)?;
        output.append("\n")
    })?;
    Ok(output.0)
}

fn operation(
    output: &mut Output,
    name: &str,
    entry: &OperationOverlay,
) -> Result<(), OverlayError> {
    let class = match entry.class {
        Some(OverlayClass::Enumerate) => "enumerate",
        Some(OverlayClass::Expand) => "expand",
        Some(OverlayClass::Defer) => "defer",
        _ => return Err(invalid("unsupported generated class")),
    };
    output.line("class", class)?;
    if let Some(reason) = entry.defer_reason.as_deref() {
        output.line("defer_reason", reason)?;
    }
    fields(output, entry)?;
    skips(output, entry)?;
    identity(output, name, entry.identity.as_ref())?;
    paginator(output, name, entry.paginator.as_ref())?;
    bindings(output, name, entry)?;
    receipts(output, None, name, entry)
}

fn receipts(
    output: &mut Output,
    service: Option<&str>,
    parent: &str,
    entry: &OperationOverlay,
) -> Result<(), OverlayError> {
    entry
        .skip_evidence
        .iter()
        .try_for_each(|(child, evidence)| {
            output.operation_table(service, parent, &format!(".skip_evidence.{child}"))?;
            receipt(output, evidence)
        })
}

fn receipt(output: &mut Output, evidence: &SkipEvidence) -> Result<(), OverlayError> {
    output.line("service", &evidence.service)?;
    output.line("list", evidence.list.as_str())?;
    output.line("describe", evidence.describe.as_str())?;
    output.line("inputs_sha256", &evidence.inputs_sha256)?;
    output.line("compared", &evidence.compared)?;
    output.append(&format!("resources = {}\n", evidence.resources))?;
    output.line("when", &evidence.when)
}

fn fields(output: &mut Output, entry: &OperationOverlay) -> Result<(), OverlayError> {
    if let Some(scope) = entry.scope {
        let name = match scope {
            Scope::Region => "region",
            Scope::Account => "account",
            Scope::HomeRegion => "home_region",
        };
        output.line("scope", name)?;
    }
    if let Some(items) = entry.items.as_ref() {
        match items {
            ItemSelector::Root => output.line("items", "$")?,
            ItemSelector::Member(path) => output.line("items", path.as_str())?,
            ItemSelector::Explicit { .. } => return Err(invalid("explicit generated selection")),
            ItemSelector::Nested(nested) => {
                output.append("nested_items = { parents = ")?;
                output.quoted(nested.parents.as_str())?;
                output.append(", items = ")?;
                output.quoted(nested.items.as_str())?;
                output.append(", owner = ")?;
                output.quoted(nested.owner.as_str())?;
                output.append(" }\n")?;
            }
        }
    }
    if let Some(tags) = entry.tags.as_ref() {
        output.line("tags", tags.as_str())?;
    }
    if let Some(page_size) = entry.page_size {
        output.append(&format!("page_size = {page_size}\n"))?;
    }
    Ok(())
}

fn skips(output: &mut Output, entry: &OperationOverlay) -> Result<(), OverlayError> {
    if entry.skip.is_empty() {
        return Ok(());
    }
    output.append("skip = [")?;
    entry
        .skip
        .iter()
        .enumerate()
        .try_for_each(|(index, name)| {
            if index != 0 {
                output.append(", ")?;
            }
            output.quoted(name.as_str())
        })?;
    output.append("]\n")
}

fn identity(
    output: &mut Output,
    name: &str,
    identity: Option<&Identity>,
) -> Result<(), OverlayError> {
    let Some(identity) = identity else {
        return Ok(());
    };
    output.table(name, ".identity")?;
    match identity {
        Identity::Arn(path) => output.line("arn", path.as_str()),
        Identity::Key(paths) => {
            output.append("key = [")?;
            for (index, path) in paths.iter().enumerate() {
                if index > 0 {
                    output.append(", ")?;
                }
                output.quoted(path.as_str())?;
            }
            output.append("]\n")
        }
        _ => Err(invalid("unsupported generated identity")),
    }
}

fn paginator(
    output: &mut Output,
    name: &str,
    paginator: Option<&Paginator>,
) -> Result<(), OverlayError> {
    let Some(paginator) = paginator else {
        return Ok(());
    };
    output.table(name, ".paginator")?;
    match paginator {
        Paginator::None => output.line("kind", "none"),
        Paginator::Token(pair) if !pair.optional => {
            output.line("kind", "token")?;
            output.line("input", pair.input.as_str())?;
            output.line("output", pair.output.as_str())
        }
        _ => Err(invalid("unsupported generated paginator")),
    }
}

fn bindings(output: &mut Output, name: &str, entry: &OperationOverlay) -> Result<(), OverlayError> {
    if entry.bind.is_empty() {
        return Ok(());
    }
    output.table(name, ".bind")?;
    entry.bind.iter().try_for_each(|(input, binding)| {
        output.quoted(input.as_str())?;
        output.append(" = { ")?;
        match binding {
            crate::overlay::Binding::From { operation, path } => {
                output.append("from = ")?;
                output.quoted(operation.as_str())?;
                output.append(", path = ")?;
                output.quoted(path.as_str())?;
            }
            crate::overlay::Binding::Account => output.append("account = true")?,
            crate::overlay::Binding::Region => output.append("region = true")?,
            _ => return Err(invalid("unsupported generated binding")),
        }
        output.append(" }\n")
    })
}

fn invalid(reason: &str) -> OverlayError {
    OverlayError::Invalid(reason.to_owned())
}

fn limit() -> OverlayError {
    invalid("generated output limit")
}

#[cfg(test)]
mod tests;
