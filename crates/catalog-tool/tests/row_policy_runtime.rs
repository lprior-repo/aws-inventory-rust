#[path = "support/bindings.rs"]
mod fixture;
use catalog_tool::overlay::{
    self, ContextSource, FixedValue, Identity, MemberPath, OperationName, OverlayClass, Paginator,
    RowOperand, RowPredicate, RowValueType, Scope,
};
use fixture::{TestResult, compiled};
use inventory_core::{
    catalog::runtime::{RowPredicatePlan, RuntimeCatalog, RuntimeCatalogError},
    record::{FieldId, Key, MapRef, Record, Value, ValueRef},
};
use catalog_tool::classes::DenyList;
use catalog_tool::kind_map;
use std::{collections::BTreeMap, path::Path};

fn absent(source: ContextSource, path: &str) -> TestResult<RowPredicate> {
    Ok(RowPredicate::Absent(RowOperand {
        source,
        path: overlay::MemberPath::selected(path)?,
        value_type: RowValueType::String,
    }))
}

fn equality() -> TestResult<RowPredicate> {
    Ok(RowPredicate::SelectedRequestEquals {
        selected: overlay::MemberPath::new("Id")?,
        request: overlay::MemberPath::new("OrganizationId")?,
        value_type: RowValueType::String,
        missing: overlay::MissingSelected::AllowAbsent,
    })
}

fn engine_predicate(allowed: bool) -> TestResult<RowPredicate> {
    let operand = RowOperand {
        source: ContextSource::SelectedItem,
        path: overlay::MemberPath::new("Id")?,
        value_type: RowValueType::String,
    };
    let values = vec!["docdb".to_owned(), "neptune".to_owned()];
    if allowed {
        Ok(RowPredicate::StringIn { operand, values })
    } else {
        Ok(RowPredicate::StringNotIn { operand, values })
    }
}



fn embedded_operation<'a>(
    services: &'a [overlay::ServiceOverlay],
    service: &str,
    name: &str,
) -> TestResult<&'a overlay::OperationOverlay> {
    services
        .iter()
        .find(|entry| entry.service() == service)
        .and_then(|entry| entry.operations().get(&OperationName::new(name).ok()?))
        .ok_or_else(|| format!("missing operation {service}/{name}").into())
}

fn assert_checked_operation(
    checked: &overlay::CheckedOverlays,
    service_name: &str,
    operation_name: &str,
) -> TestResult {
    let service = checked
        .services()
        .iter()
        .find(|service| service.service() == service_name)
        .ok_or_else(|| format!("checked service missing: {service_name}"))?;
    if service
        .callable()
        .iter()
        .any(|operation| operation.operation() == operation_name)
    {
        Ok(())
    } else {
        Err(format!("checked operation missing: {service_name}/{operation_name}").into())
    }
}

fn assert_database_operation_kinds() -> TestResult {
    let kinds = kind_map::embedded(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    for (service, operation, expected_kind) in [
        ("rds", "DescribeDBClusters", "AWS::RDS::DBCluster"),
        ("rds", "DescribeDBInstances", "AWS::RDS::DBInstance"),
        ("docdb", "DescribeDBClusters", "AWS::DocDB::DBCluster"),
        ("docdb", "DescribeDBInstances", "AWS::ApiOnly::DocDBDBInstance"),
        ("neptune", "DescribeDBClusters", "AWS::Neptune::DBCluster"),
        ("neptune", "DescribeDBInstances", "AWS::Neptune::DBInstance"),
    ] {
        let mapping = kinds
            .operations()
            .iter()
            .find(|mapping| mapping.service == service && mapping.operation == operation)
            .ok_or_else(|| format!("kind mapping missing: {service}/{operation}"))?;
        assert_eq!(mapping.kind, expected_kind);
    }
    Ok(())
}

fn assert_engine_filter(plan: &overlay::OperationOverlay, expected: &str) -> TestResult {
    let filter = MemberPath::new("Filters")?;
    let expected = FixedValue::list([FixedValue::structure([
        ("Name", FixedValue::String("engine".to_owned())),
        ("Values", FixedValue::Strings(vec![expected.to_owned()])),
    ])?]);
    assert_eq!(plan.fixed.get(&filter), Some(&expected));
    Ok(())
}

fn assert_marker_pagination(plan: &overlay::OperationOverlay) -> TestResult {
    let Some(Paginator::Token(token)) = plan.paginator.as_ref() else {
        return Err("marker token paginator missing".into());
    };
    assert_eq!(token.input, MemberPath::new("Marker")?);
    assert_eq!(token.output, MemberPath::new("Marker")?);
    assert!(!token.optional);
    Ok(())
}

fn assert_database_instance(
    plan: &overlay::OperationOverlay,
    service_engine: &str,
) -> TestResult {
    assert_eq!(plan.class, Some(OverlayClass::Enumerate));
    assert_eq!(plan.scope, Some(Scope::Region));
    assert!(plan.items.is_some());
    assert_marker_pagination(plan)?;
    if service_engine == "docdb" {
        assert!(!plan.fixed.contains_key(&MemberPath::new("Filters")?));
        assert!(matches!(plan.row_policy.owned.as_ref(),
            Some(RowPredicate::StringIn { operand, values })
            if operand.source == ContextSource::SelectedItem && operand.path.as_str() == "Engine"
                && operand.value_type == RowValueType::String
                && values.iter().map(String::as_str).eq(["docdb"])));
    } else {
        assert_engine_filter(plan, service_engine)?;
    }
    assert_eq!(
        plan.identity,
        Some(Identity::Key(vec![MemberPath::new("DbiResourceId")?]))
    );
    assert_eq!(
        plan.exclude_response_members,
        vec![MemberPath::selected("DBInstances.PendingModifiedValues.MasterUserPassword")?]
    );
    Ok(())
}

fn assert_database_cluster(plan: &overlay::OperationOverlay, engine: &str) -> TestResult {
    assert_eq!(plan.class, Some(OverlayClass::Enumerate));
    assert_eq!(plan.scope, Some(Scope::Region));
    assert!(plan.items.is_some());
    assert_marker_pagination(plan)?;
    assert_engine_filter(plan, engine)?;
    assert_eq!(
        plan.identity,
        Some(Identity::Arn(MemberPath::selected("DBClusterArn")?))
    );
    assert!(plan.exclude_response_members.is_empty());
    Ok(())
}

fn assert_rds_resource(plan: &overlay::OperationOverlay, items: &str, stable_id: &str) -> TestResult {
    assert_eq!(plan.class, Some(OverlayClass::Enumerate));
    assert_eq!(plan.scope, Some(Scope::Region));
    assert!(plan.items.is_some());
    assert_eq!(plan.page_size, Some(100));
    assert_marker_pagination(plan)?;
    assert_eq!(
        plan.identity,
        Some(Identity::Key(vec![MemberPath::new(stable_id)?]))
    );
    assert_eq!(
        plan.exclude_response_members,
        vec![MemberPath::selected(&format!("{items}.PendingModifiedValues.MasterUserPassword"))?]
    );
    match plan.row_policy.owned.as_ref() {
        Some(RowPredicate::StringNotIn { operand, values })
            if operand.source == ContextSource::SelectedItem
                && operand.path.as_str() == "Engine"
                && operand.value_type == RowValueType::String
                && values.iter().map(String::as_str).eq(["docdb", "neptune"]) => Ok(()),
        _ => Err("RDS database engine exclusion is missing".into()),
    }
}

#[test]
fn given_database_catalogs_when_loaded_then_rds_filters_overlapping_engines_and_all_services_omit_passwords()
-> TestResult {
    let services = overlay::embedded()?;
    let rds_instances = embedded_operation(&services, "rds", "DescribeDBInstances")?;
    let rds_clusters = embedded_operation(&services, "rds", "DescribeDBClusters")?;
    assert_rds_resource(rds_instances, "DBInstances", "DbiResourceId")?;
    assert_rds_resource(rds_clusters, "DBClusters", "DbClusterResourceId")?;
    assert_database_cluster(
        embedded_operation(&services, "docdb", "DescribeDBClusters")?,
        "docdb",
    )?;
    assert_database_cluster(
        embedded_operation(&services, "neptune", "DescribeDBClusters")?,
        "neptune",
    )?;
    assert_database_instance(
        embedded_operation(&services, "docdb", "DescribeDBInstances")?,
        "docdb",
    )?;
    assert_database_instance(
        embedded_operation(&services, "neptune", "DescribeDBInstances")?,
        "neptune",
    )?;
    let policies = overlay::embedded()?
        .into_iter()
        .filter(|policy| matches!(policy.service(), "rds" | "docdb" | "neptune"))
        .collect::<Vec<_>>();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let checked = overlay::check_policies(
        &root.join(".local/aws-models"),
        &policies,
        &DenyList::builtin(),
    )?;
    for (service, operation) in [
        ("rds", "DescribeDBClusters"),
        ("rds", "DescribeDBInstances"),
        ("docdb", "DescribeDBClusters"),
        ("docdb", "DescribeDBInstances"),
        ("neptune", "DescribeDBClusters"),
        ("neptune", "DescribeDBInstances"),
    ] {
        assert_checked_operation(&checked, service, operation)?;
    }
    assert_database_operation_kinds()?;
    Ok(())
}
#[test]
fn given_string_membership_predicates_when_compiled_then_native_loader_retains_bounded_values() -> TestResult {
    let excluded_predicate = engine_predicate(false)?;
    let included_predicate = engine_predicate(true)?;
    let excluded = compiled(1, |plan| {
        plan.row_policy.owned = Some(excluded_predicate);
    })?;
    let included = compiled(1, |plan| {
        plan.row_policy.owned = Some(included_predicate);
    })?;
    let excluded = RuntimeCatalog::from_bytes(&excluded)?;
    let included = RuntimeCatalog::from_bytes(&included)?;
    let excluded = excluded
        .lookup("bindings", "ListChildren")
        .ok_or("excluded plan missing")?;
    let included = included
        .lookup("bindings", "ListChildren")
        .ok_or("included plan missing")?;
    assert!(matches!(
        excluded.row_policy().owned.as_ref(),
        Some(RowPredicatePlan::StringNotIn { values, .. })
            if values.iter().map(|value| value.as_ref()).eq(["docdb", "neptune"])
    ));
    assert!(matches!(
        included.row_policy().owned.as_ref(),
        Some(RowPredicatePlan::StringIn { values, .. })
            if values.iter().map(|value| value.as_ref()).eq(["docdb", "neptune"])
    ));
    Ok(())
}
#[test]
fn given_selected_request_response_policies_when_compiled_then_native_loader_retains_distinct_decisions()
-> TestResult {
    let owned = absent(ContextSource::SelectedItem, "Id")?;
    let assertion = equality()?;
    let response = absent(ContextSource::Response, "ResponseId")?;
    let child = overlay::ChildEligibility {
        operation: overlay::OperationName::new("ListParents0")?,
        predicate: absent(ContextSource::Response, "ResponseId")?,
    };
    let bytes = compiled(1, |plan| {
        plan.row_policy.owned = Some(owned);
        plan.row_policy.assertions = vec![assertion, response];
        plan.row_policy.child = Some(child);
    })?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let child = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child missing")?;
    let policy = child.row_policy();
    assert!(matches!(policy.owned, Some(RowPredicatePlan::Absent(_))));
    assert_eq!(policy.assertions.len(), 2);
    assert!(matches!(
        policy.assertions.first(),
        Some(RowPredicatePlan::SelectedRequestEquals {
            missing: inventory_core::catalog::runtime::MissingSelected::AllowAbsent,
            ..
        })
    ));
    assert_eq!(
        policy
            .child
            .as_ref()
            .ok_or("eligibility missing")?
            .operation
            .as_ref(),
        "ListParents0"
    );
    let parent = catalog
        .lookup("bindings", "ListParents0")
        .ok_or("parent missing")?;
    assert!(
        !parent.context_projection().response.is_empty(),
        "child eligibility response operand was not projected"
    );
    Ok(())
}

#[test]
fn given_invalid_paths_types_or_predicate_bounds_when_compiled_then_checker_rejects_exactly_the_row_policy()
-> TestResult {
    let invalid = [
        absent(ContextSource::SelectedItem, "Missing")?,
        absent(ContextSource::SelectedItem, "Count")?,
        absent(ContextSource::Request, "ResponseId")?,
        absent(ContextSource::Response, "OrganizationId")?,
        RowPredicate::BooleanEquals {
            operand: RowOperand {
                source: ContextSource::SelectedItem,
                path: overlay::MemberPath::new("Id")?,
                value_type: RowValueType::Boolean,
            },
            value: false,
        },
        RowPredicate::Absent(RowOperand {
            source: ContextSource::SelectedItem,
            path: overlay::MemberPath::new("Id")?,
            value_type: RowValueType::Structure,
        }),
        RowPredicate::StringNotIn {
            operand: RowOperand {
                source: ContextSource::SelectedItem,
                path: overlay::MemberPath::new("Id")?,
                value_type: RowValueType::Boolean,
            },
            values: vec!["docdb".to_owned()],
        },
        RowPredicate::StringIn {
            operand: RowOperand {
                source: ContextSource::SelectedItem,
                path: overlay::MemberPath::new("Id")?,
                value_type: RowValueType::String,
            },
            values: Vec::new(),
        },
    ];
    for predicate in invalid {
        let error = compiled(1, |plan| plan.row_policy.owned = Some(predicate))
            .err()
            .ok_or("invalid policy accepted")?;
        let error = error
            .downcast_ref::<overlay::CheckError>()
            .ok_or("wrong rejection boundary")?;
        assert!(
            matches!(error,overlay::CheckError::Violations(violations) if violations.iter().any(|v| v.key() == "row_policy" && v.rule() == overlay::ViolationRule::InvalidPath))
        );
    }
    let predicate = equality()?;
    let error = compiled(1, |plan| plan.row_policy.assertions = vec![predicate; 9])
        .err()
        .ok_or("unbounded assertions accepted")?;
    assert!(
        matches!(error.downcast_ref::<overlay::CheckError>(),Some(overlay::CheckError::Violations(violations)) if violations.iter().any(|v| v.key() == "row_policy"))
    );
    Ok(())
}

#[test]
fn given_child_eligibility_for_an_unbound_producer_when_compiled_then_it_cannot_gate_an_independent_row_set()
-> TestResult {
    let child = overlay::ChildEligibility {
        operation: overlay::OperationName::new("ListChildren")?,
        predicate: absent(ContextSource::SelectedItem, "Id")?,
    };
    let error = compiled(1, |plan| plan.row_policy.child = Some(child))
        .err()
        .ok_or("unbound predicate source accepted")?;
    assert!(
        matches!(error.downcast_ref::<overlay::CheckError>(),Some(overlay::CheckError::Violations(violations)) if violations.iter().any(|v| v.key() == "row_policy"))
    );
    Ok(())
}

fn key(number: u32) -> TestResult<Key> {
    Ok(Key::Field(FieldId::new(number)?))
}
fn map(value: &mut Value) -> TestResult<&mut BTreeMap<Key, Value>> {
    match value {
        Value::Map(fields) => Ok(fields),
        _ => Err("metadata is not a map".into()),
    }
}
fn get(value: &mut Value, number: u32) -> TestResult<&mut Value> {
    map(value)?
        .get_mut(&key(number)?)
        .ok_or("metadata field missing".into())
}

fn corrupt(bytes: &[u8], mutation: usize) -> TestResult<Record> {
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(bytes)?).to_value()?;
    let extension = get(&mut root, 60_000)?;
    let Value::List(plans) = get(extension, 2)? else {
        return Err("plans missing".into());
    };
    let child = plans
        .iter_mut()
        .find(|plan| match plan {
            Value::Map(fields) => fields
                .values()
                .any(|value| matches!(value,Value::Text(text) if text == "ListChildren")),
            _ => false,
        })
        .ok_or("child missing")?;
    let policy = get(child, 22)?;
    let owned = get(policy, 1)?;
    match mutation {
        0 => {
            *get(get(owned, 2)?, 3)? = Value::Int(1);
        }
        1 => {
            *get(get(owned, 2)?, 1)? = Value::Int(99);
        }
        2 => {
            *get(get(owned, 2)?, 2)? = Value::List(vec![Value::Int(999_999)]);
        }
        3 => {
            *get(owned, 1)? = Value::Int(99);
        }
        4 => {
            map(owned)?.insert(key(77)?, Value::Bool(true));
        }
        _ => return Err("unknown corruption".into()),
    }
    let encoded = Record::encode(extension)?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    *get(get(extension, 7)?, 7)? = Value::Bytes(digest.to_vec());
    Ok(Record::encode(&root)?)
}

#[test]
fn given_resealed_corrupt_row_predicates_when_native_catalog_loads_then_model_and_bounded_codec_reject_each()
-> TestResult {
    let predicate = absent(ContextSource::SelectedItem, "Id")?;
    let bytes = compiled(1, |plan| plan.row_policy.owned = Some(predicate))?;
    for mutation in 0..5 {
        let corrupt = corrupt(&bytes, mutation)?;
        assert!(
            matches!(
                RuntimeCatalog::from_bytes(corrupt.as_bytes()),
                Err(RuntimeCatalogError::Capability)
            ),
            "accepted mutation {mutation}"
        );
    }
    Ok(())
}

#[test]
fn given_child_response_eligibility_when_its_producer_projection_is_removed_then_native_loading_rejects_the_resealed_plan()
-> TestResult {
    let eligibility = overlay::ChildEligibility {
        operation: overlay::OperationName::new("ListParents0")?,
        predicate: absent(ContextSource::Response, "ResponseId")?,
    };
    let bytes = compiled(1, |plan| plan.row_policy.child = Some(eligibility))?;
    RuntimeCatalog::from_bytes(&bytes)?;
    let mut root = ValueRef::Map(MapRef::from_bytes_bounded(&bytes)?).to_value()?;
    let extension = get(&mut root, 60_000)?;
    let Value::List(plans) = get(extension, 2)? else {
        return Err("plans missing".into());
    };
    let parent = plans
        .iter_mut()
        .find(|plan| match plan {
            Value::Map(fields) => fields
                .values()
                .any(|value| matches!(value,Value::Text(text) if text == "ListParents0")),
            _ => false,
        })
        .ok_or("parent missing")?;
    *get(get(parent, 20)?, 2)? = Value::List(Vec::new());
    let encoded = Record::encode(extension)?;
    let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
    *get(get(extension, 7)?, 7)? = Value::Bytes(digest.to_vec());
    let corrupt = Record::encode(&root)?;
    assert!(matches!(
        RuntimeCatalog::from_bytes(corrupt.as_bytes()),
        Err(RuntimeCatalogError::Capability)
    ));
    Ok(())
}

#[test]
fn given_modeled_structure_absence_when_compiled_then_the_native_operand_retains_structure_type()
-> TestResult {
    let predicate = RowPredicate::Absent(RowOperand {
        source: ContextSource::SelectedItem,
        path: overlay::MemberPath::selected("$")?,
        value_type: RowValueType::Structure,
    });
    let bytes = compiled(1, |plan| plan.row_policy.owned = Some(predicate))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let policy = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child missing")?
        .row_policy();
    assert!(
        matches!(&policy.owned,Some(RowPredicatePlan::Absent(operand))
        if operand.value_type == inventory_core::catalog::runtime::RowValueType::Structure && operand.path.is_empty())
    );
    Ok(())
}

#[test]
fn given_additive_stage_policy_when_compiled_then_native_loader_preserves_base_and_branch_rules_and_rejects_branch_codec_tampering()
-> TestResult {
    let base = absent(ContextSource::SelectedItem, "Id")?;
    let owned = absent(ContextSource::Response, "ResponseId")?;
    let assertion = equality()?;
    let bytes = compiled(1, |plan| {
        plan.row_policy.owned = Some(base);
        plan.branches = vec![overlay::ExecutionBranch {
            discriminator: "PORTAL".to_owned(),
            fixed: BTreeMap::new(),
            bind: BTreeMap::new(),
            emission: None,
            scope: overlay::Scope::Region,
            request_region: None,
            row_policy: Some(overlay::RowPolicy {
                owned: Some(owned),
                assertions: vec![assertion],
                child: None,
            }),
        }];
    })?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let plan = catalog
        .lookup("bindings", "ListChildren")
        .ok_or("child missing")?;
    assert!(plan.row_policy().owned.is_some());
    let branch = plan.branches().first().ok_or("branch missing")?;
    let policy = branch.row_policy.as_ref().ok_or("branch policy missing")?;
    assert!(matches!(policy.owned, Some(RowPredicatePlan::Absent(_))));
    assert_eq!(policy.assertions.len(), 1);
    for mutation in 0..2 {
        let mut root = ValueRef::Map(MapRef::from_bytes_bounded(&bytes)?).to_value()?;
        let extension = get(&mut root, 60_000)?;
        let Value::List(plans) = get(extension, 2)? else {
            return Err("plans missing".into());
        };
        let child = plans
            .iter_mut()
            .find(|plan| match plan {
                Value::Map(fields) => fields
                    .values()
                    .any(|value| matches!(value,Value::Text(text) if text == "ListChildren")),
                _ => false,
            })
            .ok_or("child missing")?;
        let Value::List(branches) = get(child, 16)? else {
            return Err("branches missing".into());
        };
        let policy = get(branches.first_mut().ok_or("branch missing")?, 6)?;
        if mutation == 0 {
            map(policy)?.insert(key(77)?, Value::Bool(true));
        } else {
            let Value::List(assertions) = get(policy, 2)? else {
                return Err("assertions missing".into());
            };
            *get(assertions.first_mut().ok_or("assertion missing")?, 4)? = Value::Int(2);
        }
        let encoded = Record::encode(extension)?;
        let digest = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())?;
        *get(get(extension, 7)?, 7)? = Value::Bytes(digest.to_vec());
        let corrupt = Record::encode(&root)?;
        assert!(
            matches!(
                RuntimeCatalog::from_bytes(corrupt.as_bytes()),
                Err(RuntimeCatalogError::Capability)
            ),
            "accepted branch policy mutation {mutation}"
        );
    }
    Ok(())
}
