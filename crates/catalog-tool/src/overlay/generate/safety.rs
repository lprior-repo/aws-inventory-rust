use crate::plans::{
    Model,
    model::{ShapeRef, target},
};
use serde_json::Value;
use std::{collections::BTreeSet, ops::ControlFlow};

type Visit = ControlFlow<Result<(), String>>;
const SECRETS: &[&str] = &[
    "password",
    "secretaccesskey",
    "sessiontoken",
    "credentials",
    "privatekey",
    "clientsecret",
    "environment",
    "authorizationtoken",
    "apikey",
    "accesskey",
    "accesskeyid",
    "secret",
    "secretkey",
    "secretstring",
    "secretbinary",
    "token",
    "refreshtoken",
    "environmentvariables",
    "headers",
];

const CREDENTIAL_SUFFIXES: &[&str] = &[
    "password",
    "passworddata",
    "secretaccesskey",
    "sessiontoken",
    "credentials",
    "privatekey",
    "clientsecret",
    "authorizationtoken",
    "apikey",
    "accesskey",
    "accesskeyid",
    "secretkey",
    "secretstring",
    "secretbinary",
    "accesstoken",
    "refreshtoken",
    "idtoken",
    "keymaterial",
];

fn credential(name: &str) -> bool {
    CREDENTIAL_SUFFIXES.iter().any(|suffix| {
        name.as_bytes()
            .get(name.len().saturating_sub(suffix.len())..)
            .is_some_and(|end| end.eq_ignore_ascii_case(suffix.as_bytes()))
    })
}

pub(super) fn output(model: &Model<'_>, operation: &str) -> Result<(), String> {
    let root = model
        .side_root(operation, "output")
        .ok_or("output: missing")?;
    let mut pending = BTreeSet::from([root.id]);
    let mut visited = BTreeSet::new();
    match (0..131_072).try_for_each(|_| step(model, &mut pending, &mut visited)) {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => Err("output: traversal limit".to_owned()),
    }
}

fn step<'a>(
    model: &Model<'a>,
    pending: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Visit {
    let Some(id) = pending.pop_first() else {
        return ControlFlow::Break(Ok(()));
    };
    if !visited.insert(id) {
        return ControlFlow::Continue(());
    }
    let result = model
        .shape(id)
        .ok_or_else(|| "output: unresolved shape".to_owned())
        .and_then(|shape| inspect(shape, pending, visited));
    match result {
        Ok(()) => ControlFlow::Continue(()),
        Err(reason) => ControlFlow::Break(Err(reason)),
    }
}

fn inspect<'a>(
    shape: ShapeRef<'a>,
    pending: &mut BTreeSet<&'a str>,
    visited: &BTreeSet<&str>,
) -> Result<(), String> {
    reject("shape", shape.value)?;
    let members = shape.value.get("members").and_then(Value::as_object);
    members
        .into_iter()
        .flatten()
        .try_for_each(|(name, member)| {
            reject(name, member)?;
            enqueue(member, pending, visited);
            Ok::<(), String>(())
        })?;
    ["member", "key", "value"].into_iter().try_for_each(|key| {
        if let Some(member) = shape.value.get(key) {
            reject(key, member)?;
            enqueue(member, pending, visited);
        }
        Ok(())
    })
}

fn enqueue<'a>(member: &'a Value, pending: &mut BTreeSet<&'a str>, visited: &BTreeSet<&str>) {
    if let Some(id) = target(member).filter(|id| !visited.contains(id)) {
        pending.insert(id);
    }
}

fn reject(name: &str, value: &Value) -> Result<(), String> {
    let sensitive = value
        .get("traits")
        .and_then(|traits| traits.get("smithy.api#sensitive"))
        .is_some();
    if sensitive
        || credential(name)
        || SECRETS
            .iter()
            .any(|secret| name.eq_ignore_ascii_case(secret))
    {
        return Err(format!("output: secret-bearing member {name}"));
    }
    Ok(())
}
