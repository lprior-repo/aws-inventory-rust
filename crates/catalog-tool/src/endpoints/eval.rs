use super::model::{Call, Condition, Expr, Rule, RuleKind, guard, tick};
use super::{Endpoint, Error, Fault, MAX_STEPS, Outcome, RuleSet, functions, template};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub(super) type Bindings = BTreeMap<String, Value>;

#[cfg(test)]
#[path = "eval_bounds_tests.rs"]
mod bounds_tests;

impl RuleSet {
    pub fn evaluate(&self, parameters: &Bindings) -> Result<Outcome, Error> {
        let mut steps = MAX_STEPS;
        let result = self
            .bind(parameters)
            .and_then(|env| evaluate_group(&self.document.rules, &env, 0, &mut steps));
        match result {
            Err(Fault::UnsupportedFunction { name }) => Ok(Outcome::UnsupportedFunction { name }),
            result => result.map_err(|kind| self.error(kind)),
        }
    }

    fn bind(&self, input: &Bindings) -> Result<Bindings, Fault> {
        let mut steps = MAX_STEPS;
        if input.keys().any(|key| key.len() > super::MAX_STRING) {
            return Err(Fault::Bound("string bytes"));
        }
        input
            .values()
            .try_for_each(|value| guard(value, 0, &mut steps))?;
        if input
            .keys()
            .any(|k| !self.document.parameters.contains_key(k))
        {
            return Err(Fault::Arguments("unknown parameter".into()));
        }
        self.document
            .parameters
            .iter()
            .try_fold(Bindings::new(), |mut env, (name, p)| {
                bind_parameter(&mut env, input, name, p)?;
                Ok(env)
            })
    }
}

fn evaluate_group(
    rules: &[Rule],
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Outcome, Fault> {
    tick(depth, steps)?;
    rules
        .iter()
        .find_map(|rule| {
            let mut local = env.clone();
            match conditions(&rule.conditions, &mut local, depth, steps) {
                Ok(false) => None,
                Err(e) => Some(Err(e)),
                Ok(true) => Some(resolve(&rule.kind, &local, depth, steps)),
            }
        })
        .ok_or(Fault::Exhausted)?
}

fn conditions(
    items: &[Condition],
    env: &mut Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<bool, Fault> {
    items.iter().try_fold(true, |matched, c| {
        if !matched {
            return Ok(false);
        }
        condition(c, env, depth, steps)
    })
}

fn resolve(
    kind: &RuleKind,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Outcome, Fault> {
    match kind {
        RuleKind::Tree { rules } => evaluate_group(rules, env, depth.saturating_add(1), steps),
        RuleKind::Error { error } => Ok(Outcome::Error(string(expression(
            error, env, depth, steps,
        )?)?)),
        RuleKind::Endpoint { endpoint } => endpoint_result(endpoint, env, depth, steps),
    }
}

fn expression(
    expr: &Expr,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Value, Fault> {
    tick(depth, steps)?;
    match expr {
        Expr::Ref { r#ref } => {
            let Some(value) = env.get(r#ref) else {
                return Ok(Value::Null);
            };
            Ok(value.clone())
        }
        Expr::Call(c) => call(c, env, depth.saturating_add(1), steps),
        Expr::Literal(Value::String(s)) => template::expand(s, env).map(Value::String),
        Expr::Literal(Value::Object(_)) => {
            Err(Fault::Malformed("invalid expression object".into()))
        }
        Expr::Literal(value) => Ok(value.clone()),
    }
}

fn call(call: &Call, env: &Bindings, depth: usize, steps: &mut usize) -> Result<Value, Fault> {
    tick(depth, steps)?;
    let function =
        functions::Function::parse(&call.function).ok_or_else(|| Fault::UnsupportedFunction {
            name: call.function.clone(),
        })?;
    let args = call
        .argv
        .iter()
        .map(|e| expression(e, env, depth.saturating_add(1), steps))
        .collect::<Result<Vec<_>, _>>()?;
    functions::apply(function, &args)
}

fn string(value: Value) -> Result<String, Fault> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(Fault::Arguments("expected string".into())),
    }
}

fn bind_parameter(
    env: &mut Bindings,
    input: &Bindings,
    name: &str,
    p: &super::model::Parameter,
) -> Result<(), Fault> {
    let value = input
        .get(name)
        .filter(|v| !v.is_null())
        .or(p.default.as_ref());
    let Some(value) = value else {
        if p.required {
            return Err(Fault::MissingParameter(name.into()));
        }
        return Ok(());
    };
    if !p.kind.accepts(value) {
        return Err(Fault::Arguments(name.into()));
    }
    env.insert(name.into(), value.clone());
    Ok(())
}

fn endpoint_result(
    endpoint: &super::model::EndpointExpr,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Outcome, Fault> {
    let url = string(expression(&endpoint.url, env, depth, steps)?)?;
    let properties = template::expand_properties(&endpoint.properties, env, depth, steps)?;
    let headers = headers(&endpoint.headers, env, depth, steps)?;
    Ok(Outcome::Endpoint(Endpoint {
        url,
        properties,
        headers,
    }))
}

fn condition(
    c: &Condition,
    env: &mut Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<bool, Fault> {
    let value = call(&c.call, env, depth.saturating_add(1), steps)?;
    if value.is_null() || value == Value::Bool(false) {
        return Ok(false);
    }
    if let Some(name) = &c.assign {
        assign(env, name, value)?;
    }
    Ok(true)
}

fn assign(env: &mut Bindings, name: &str, value: Value) -> Result<(), Fault> {
    if env.contains_key(name) || env.len() >= 256 {
        return Err(Fault::Malformed(format!(
            "duplicate or excessive assignment: {name}"
        )));
    }
    env.insert(name.into(), value);
    Ok(())
}

fn headers(
    items: &BTreeMap<String, Vec<Expr>>,
    env: &Bindings,
    depth: usize,
    steps: &mut usize,
) -> Result<Value, Fault> {
    items
        .iter()
        .map(|(name, values)| {
            let values = values
                .iter()
                .map(|v| string(expression(v, env, depth, steps)?).map(Value::String))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((name.clone(), Value::Array(values)))
        })
        .collect::<Result<Map<_, _>, Fault>>()
        .map(Value::Object)
}
