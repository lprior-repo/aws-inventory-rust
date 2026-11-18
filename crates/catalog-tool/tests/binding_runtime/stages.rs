use super::*;

fn branches(producer: &str, stage: &str) -> TestResult<Vec<overlay::ExecutionBranch>> {
    let operation = overlay::OperationName::new(producer)?;
    Ok(vec![
        overlay::ExecutionBranch {
            discriminator: "majors".into(),
            fixed: BTreeMap::new(),
            bind: BTreeMap::new(),
            emission: Some(overlay::Emission::DiscoveryOnly),
            row_policy: None,
            scope: overlay::Scope::Region,
            request_region: None,
        },
        overlay::ExecutionBranch {
            discriminator: "minors".into(),
            fixed: BTreeMap::new(),
            bind: BTreeMap::from([
                (
                    overlay::MemberPath::new("EntityId")?,
                    Binding::FromBranch {
                        operation: operation.clone(),
                        branch: stage.into(),
                        path: overlay::MemberPath::selected("Id")?,
                    },
                ),
                (
                    overlay::MemberPath::new("OrganizationId")?,
                    Binding::ContextFromBranch {
                        operation,
                        branch: stage.into(),
                        source: ContextSource::Request,
                        path: overlay::MemberPath::new("OrganizationId")?,
                    },
                ),
            ]),
            emission: Some(overlay::Emission::EmitMetadata),
            row_policy: None,
            scope: overlay::Scope::Region,
            request_region: None,
        },
    ])
}
fn staged(producer: &str, stage: &str) -> TestResult<Vec<u8>> {
    let branches = branches(producer, stage)?;
    compiled(1, |child| child.branches = branches)
}

#[test]
fn given_explicit_acyclic_same_operation_stages_when_compiled_then_overrides_emission_and_depth_survive()
-> TestResult {
    let bytes = staged("ListChildren", "majors")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let child = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child absent")?;
    let major = child
        .branches()
        .iter()
        .find(|branch| branch.discriminator == "majors")
        .ok_or("major absent")?;
    let minor = child
        .branches()
        .iter()
        .find(|branch| branch.discriminator == "minors")
        .ok_or("minor absent")?;
    assert_eq!(child.facet_precedence(), 2);
    assert_eq!(
        child.effective_emission(Some(major)),
        inventory_core::catalog::runtime::Emission::DiscoveryOnly
    );
    assert_eq!(
        child.effective_emission(Some(minor)),
        inventory_core::catalog::runtime::Emission::EmitMetadata
    );
    assert_eq!(
        child
            .effective_bindings(Some(major))
            .filter(|binding| matches!(binding.source, BindingSource::Alternatives(_)))
            .count(),
        2
    );
    assert_eq!(
        child.effective_bindings(Some(minor)).count(),
        3,
        "base destinations must be replaced, not duplicated"
    );
    assert_eq!(child.effective_bindings(Some(minor)).filter(|binding| matches!(&binding.source,
        BindingSource::ContextFromBranch { operation, branch, .. } if operation.as_ref() == "ListChildren" && branch.as_ref() == "majors")).count(), 2);
    Ok(())
}

#[test]
fn given_missing_or_cyclic_stage_sources_when_checked_then_no_same_operation_cycle_waiver_exists()
-> TestResult {
    for (operation, stage) in [
        ("ListChildren", "missing"),
        ("ListChildren", "minors"),
        ("ListParents0", "majors"),
    ] {
        assert!(
            staged(operation, stage).is_err(),
            "accepted {operation}/{stage}"
        );
    }
    let mut stages = branches("ListChildren", "majors")?;
    let reverse = branches("ListChildren", "minors")?
        .pop()
        .ok_or("reverse stage absent")?;
    stages.first_mut().ok_or("major absent")?.bind = reverse.bind;
    assert!(
        compiled(1, |child| child.branches = stages).is_err(),
        "accepted majors -> minors -> majors"
    );
    Ok(())
}

fn corrupt_stage(bytes: &[u8], corruption: usize) -> TestResult<Record> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = map(map(&mut root)?
        .get_mut(&field(60_000)?)
        .ok_or("extension absent")?)?;
    let Value::List(plans) = extension.get_mut(&field(2)?).ok_or("plans absent")? else {
        return Err("not plans".into());
    };
    let child = plans
        .iter_mut()
        .find(|plan| match plan {
            Value::Map(fields) => fields
                .values()
                .any(|value| matches!(value, Value::Text(name) if name == "ListChildren")),
            _ => false,
        })
        .ok_or("child absent")?;
    let Value::List(stages) = map(child)?.get_mut(&field(16)?).ok_or("stages absent")? else {
        return Err("not stages".into());
    };
    let minor = stages.last_mut().ok_or("minor absent")?;
    if corruption == 5 {
        map(minor)?.insert(field(5)?, Value::Int(99));
    } else {
        let Value::List(bindings) = map(minor)?.get_mut(&field(2)?).ok_or("bindings absent")?
        else {
            return Err("not bindings".into());
        };
        let binding = map(bindings.first_mut().ok_or("binding absent")?)?;
        match corruption {
            0 => {
                binding.insert(field(6)?, Value::Text("minors".into()));
            }
            1 => {
                binding.insert(field(6)?, Value::Text("unknown".into()));
            }
            2 => {
                binding.insert(field(4)?, Value::Text("ListParents0".into()));
            }
            3 => {
                binding.insert(field(7)?, Value::Int(8));
            }
            _ => {
                binding.insert(field(5)?, Value::List(vec![Value::Int(999_999)]));
            }
        }
    }
    let encoded = Record::encode(&Value::Map(extension.clone()))?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    map(extension.get_mut(&field(7)?).ok_or("receipt absent")?)?
        .insert(field(7)?, Value::Bytes(digest.to_vec()));
    Ok(Record::encode(&root)?)
}

#[test]
fn given_rehashed_stage_corruption_when_runtime_loads_then_unknown_cyclic_or_unmodeled_stages_fail_closed()
-> TestResult {
    let bytes = staged("ListChildren", "majors")?;
    for corruption in 0..6 {
        let record = corrupt_stage(&bytes, corruption)?;
        assert!(
            RuntimeCatalog::from_bytes(record.as_bytes()).is_err(),
            "accepted stage corruption {corruption}"
        );
    }
    Ok(())
}
