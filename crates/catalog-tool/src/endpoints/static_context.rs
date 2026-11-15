use super::{Case, Error, Fault, MAX_STEPS, RuleSet, VerifiedEndpoint};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

pub(super) type Context = BTreeMap<String, Parameter>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Parameter {
    value: Value,
}

pub(super) fn compile(
    file: &Path,
    document: &Value,
    cases: &[Case],
    context: &Context,
    signing_name: Option<&str>,
) -> Result<VerifiedEndpoint, Error> {
    let mut rules = RuleSet::parse(file, document)?;
    validate(&rules, document, context)?;
    // An operation's constants select only matching modeled endpoint evidence.
    // Omitting a constant is equivalent only when its original default matches.
    let cases = cases
        .iter()
        .filter(|case| matches_context(&rules, case, context))
        .map(|case| Case {
            params: case.params.iter()
                .filter(|(name, _)| !context.contains_key(*name))
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
            expect: case.expect.clone(),
            operation_inputs: case.operation_inputs.clone(),
        })
        .collect::<Vec<_>>();
    for (name, input) in context {
        let parameter = rules.document.parameters.get_mut(name)
            .ok_or_else(|| Error::Model {
                file: file.to_path_buf(),
                kind: Fault::Malformed("undeclared static endpoint parameter".into()),
            })?;
        parameter.required = true;
        parameter.default = Some(input.value.clone());
    }
    rules.run_cases(&cases)?.verified_endpoint(signing_name)
        .map_err(|failure| rules.error(Fault::Malformed(format!(
            "unverified static endpoint context: {failure:?}"
        ))))
}

fn validate(rules: &RuleSet, document: &Value, context: &Context) -> Result<(), Error> {
    let mut steps = MAX_STEPS;
    for (name, input) in context {
        super::model::guard(&input.value, 0, &mut steps).map_err(|kind| rules.error(kind))?;
        let builtin = document.get("parameters")
            .and_then(|parameters| parameters.get(name))
            .and_then(|parameter| parameter.get("builtIn"))
            .is_some();
        let accepted = rules.document.parameters.get(name)
            .is_some_and(|parameter| parameter.kind.accepts(&input.value));
        if builtin || matches!(name.as_str(), "Region" | "UseFIPS" | "UseDualStack" | "Endpoint") || !accepted {
            return Err(rules.error(Fault::Malformed(
                "invalid static endpoint parameter".into(),
            )));
        }
    }
    Ok(())
}

fn matches_context(rules: &RuleSet, case: &Case, context: &Context) -> bool {
    context.iter().all(|(name, input)| {
        case.params.get(name)
            .or_else(|| rules.document.parameters.get(name).and_then(|parameter| parameter.default.as_ref()))
            == Some(&input.value)
    })
}
