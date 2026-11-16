use super::{Case, Error, Fault, MAX_STEPS, Outcome, RuleSet, VerifiedEndpoint};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

pub(super) type Context = BTreeMap<String, Parameter>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Parameter {
    value: Value,
}

/// Stands in for the host label value every request supplies. It is a valid
/// DNS label that names no region, so a compiled route shows where the rules
/// place the label without resembling any real one.
const HOST_LABEL: &str = "request-host-label";

/// The one required input member that both forms a host label and binds an
/// endpoint parameter. `prefixed` when the operation's modeled host prefix is
/// exactly that label, which the runtime then prepends to the route itself.
pub(super) struct HostLabel {
    pub(super) parameter: String,
    pub(super) prefixed: bool,
}

/// What an operation's model fixes about its endpoint before any request.
pub(super) struct Operation {
    pub(super) context: Context,
    pub(super) label: Option<HostLabel>,
}

pub(super) fn compile(
    file: &Path,
    document: &Value,
    cases: &[Case],
    name: &str,
    operation: &Operation,
    signing_name: Option<&str>,
) -> Result<VerifiedEndpoint, Error> {
    let mut rules = RuleSet::parse(file, document)?;
    let context = &operation.context;
    let label = operation.label.as_ref();
    validate(&rules, document, context)?;
    if let Some(label) = label {
        validate_label(&rules, document, context, label)?;
    }
    // An operation's constants select only matching modeled endpoint evidence.
    // Omitting a constant is equivalent only when its original default matches.
    // A case without the required host label describes no request at all.
    let matched = cases
        .iter()
        .filter(|case| matches_context(&rules, case, context))
        .filter(|case| {
            label.is_none_or(|label| {
                case.params.get(&label.parameter).is_some_and(Value::is_string)
            })
        })
        .collect::<Vec<_>>();
    let fixed = context.iter()
        .map(|(name, input)| (name, input.value.clone()))
        .chain(label.map(|label| (&label.parameter, Value::String(HOST_LABEL.into()))));
    for (name, value) in fixed {
        let parameter = rules.document.parameters.get_mut(name)
            .ok_or_else(|| Error::Model {
                file: file.to_path_buf(),
                kind: Fault::Malformed("undeclared static endpoint parameter".into()),
            })?;
        parameter.required = true;
        parameter.default = Some(value);
    }
    let cases = matched
        .into_iter()
        .map(|case| scoped(&rules, case, name, operation))
        .collect::<Vec<_>>();
    let verified = rules.run_cases(&cases)?.verified_endpoint(signing_name)
        .map_err(|failure| rules.error(Fault::Malformed(format!(
            "unverified static endpoint context: {failure:?}"
        ))))?;
    match label {
        Some(label) => verified.without_host_label(HOST_LABEL, label.prefixed)
            .ok_or_else(|| rules.error(Fault::Malformed(
                "host label is not the modeled host prefix of the route".into(),
            ))),
        None => Ok(verified),
    }
}

/// Restates a published case as the request the runtime makes for `operation`:
/// constants and declared defaults are implied, the host label is whatever the
/// request carries, and inputs naming only this operation add no parameters.
fn scoped(rules: &RuleSet, case: &Case, name: &str, operation: &Operation) -> Case {
    let declared = |name: &str| rules.document.parameters.get(name);
    let params = case.params.iter()
        .filter(|(name, value)| {
            !operation.context.contains_key(*name)
                && declared(name).and_then(|parameter| parameter.default.as_ref()) != Some(*value)
        })
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect::<BTreeMap<_, _>>();
    let (params, expect) = match &operation.label {
        Some(label) => with_placeholder(rules, label, params, &case.expect),
        None => (params, case.expect.clone()),
    };
    Case {
        params: without_inert_flags(rules, &expect, params),
        expect,
        operation_inputs: if describes_only(case, name) {
            Vec::new()
        } else {
            case.operation_inputs.clone()
        },
    }
}

/// Restates the case's own host label value as the placeholder the route is
/// compiled with, including where the published URL leads with it. A published
/// error is restated only when the placeholder reproduces it; an error about
/// one particular value stays that case's own and proves nothing here.
fn with_placeholder(
    rules: &RuleSet,
    label: &HostLabel,
    params: BTreeMap<String, Value>,
    expect: &Outcome,
) -> (BTreeMap<String, Value>, Outcome) {
    let Some(value) = params.get(&label.parameter).and_then(Value::as_str) else {
        return (params, expect.clone());
    };
    let mut restated = expect.clone();
    if let Outcome::Endpoint(endpoint) = &mut restated {
        let rest = endpoint.url.strip_prefix("https://")
            .and_then(|host| host.strip_prefix(value))
            .and_then(|host| host.strip_prefix('.'));
        if let Some(rest) = rest {
            endpoint.url = format!("https://{HOST_LABEL}.{rest}");
        }
    }
    let mut implied = params.clone();
    implied.remove(&label.parameter);
    let specific = matches!(expect, Outcome::Error(_))
        && rules.evaluate(&implied).ok().as_ref() != Some(expect);
    if specific {
        (params, restated)
    } else {
        (implied, restated)
    }
}

/// An optional flag published as `false` is not the unset flag the runtime
/// sends. It is dropped only when the rules prove the two requests resolve to
/// the published expectation alike; otherwise the case keeps its parameters.
fn without_inert_flags(
    rules: &RuleSet,
    expect: &Outcome,
    params: BTreeMap<String, Value>,
) -> BTreeMap<String, Value> {
    let mut unset = params.clone();
    unset.retain(|name, value| {
        value != &Value::Bool(false)
            || rules.document.parameters.get(name)
                .is_none_or(|parameter| parameter.required || parameter.default.is_some())
    });
    let expected = |bindings: &BTreeMap<String, Value>| {
        rules.evaluate(bindings).is_ok_and(|outcome| &outcome == expect)
    };
    if unset.len() < params.len() && expected(&params) && expected(&unset) {
        unset
    } else {
        params
    }
}

/// A case's parameters are the complete rule inputs of the requests it names,
/// so inputs naming only this operation describe nothing its parameters lack.
fn describes_only(case: &Case, operation: &str) -> bool {
    case.operation_inputs.iter().all(|input| {
        input.get("operationName").and_then(Value::as_str) == Some(operation)
    })
}

fn builtin(document: &Value, name: &str) -> bool {
    document.get("parameters")
        .and_then(|parameters| parameters.get(name))
        .and_then(|parameter| parameter.get("builtIn"))
        .is_some()
        || matches!(name, "Region" | "UseFIPS" | "UseDualStack" | "Endpoint")
}

fn validate(rules: &RuleSet, document: &Value, context: &Context) -> Result<(), Error> {
    let mut steps = MAX_STEPS;
    for (name, input) in context {
        super::model::guard(&input.value, 0, &mut steps).map_err(|kind| rules.error(kind))?;
        let accepted = rules.document.parameters.get(name)
            .is_some_and(|parameter| parameter.kind.accepts(&input.value));
        if builtin(document, name) || !accepted {
            return Err(rules.error(Fault::Malformed(
                "invalid static endpoint parameter".into(),
            )));
        }
    }
    Ok(())
}

/// The label binds a string parameter the client never configures and the
/// operation does not also fix to a constant.
fn validate_label(
    rules: &RuleSet,
    document: &Value,
    context: &Context,
    label: &HostLabel,
) -> Result<(), Error> {
    let name = label.parameter.as_str();
    let text = rules.document.parameters.get(name)
        .is_some_and(|parameter| parameter.kind.accepts(&Value::String(HOST_LABEL.into())));
    if builtin(document, name) || context.contains_key(name) || !text {
        return Err(rules.error(Fault::Malformed(
            "invalid host label endpoint parameter".into(),
        )));
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
