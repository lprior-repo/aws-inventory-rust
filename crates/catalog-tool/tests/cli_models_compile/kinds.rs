use inventory_core::{
    catalog::{MAX_CATALOG_BYTES, runtime::RuntimeCatalog},
    record::{FieldId, ListRef, MapRef, ValueRef},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ORIGINAL: &[(&str, &str)] = &[
    ("ec2", "DescribeVpcs"),
    ("ec2", "DescribeSubnets"),
    ("ec2", "DescribeRegions"),
    ("sts", "GetCallerIdentity"),
];
const SERVICES: [&str; 8] = [
    "cloudwatch",
    "dynamodb",
    "iam",
    "kms",
    "lambda",
    "s3",
    "sns",
    "sqs",
];

fn read_catalog(path: &str) -> TestResult<Vec<u8>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    if fs::metadata(&path)?.len() > u64::try_from(MAX_CATALOG_BYTES)? {
        return Err("catalog fixture exceeds size bound".into());
    }
    Ok(fs::read(path)?)
}

#[test]
#[ignore = "reads the locally generated pinned-model catalog"]
fn given_compiled_catalog_when_plans_are_read_then_every_kind_has_a_schema() -> TestResult {
    let bytes = read_catalog("catalog/aws-api.catalog")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let kinds: BTreeSet<_> = catalog
        .schemas()
        .iter()
        .map(|schema| schema.kind())
        .collect();
    assert!(!catalog.plans().is_empty());
    for plan in catalog.plans() {
        assert!(
            kinds.contains(plan.kind()),
            "{} {}",
            plan.service_name(),
            plan.operation_name()
        );
    }
    println!(
        "plans_with_schemas={} schemas={}",
        catalog.plans().len(),
        kinds.len()
    );
    Ok(())
}

#[test]
#[ignore = "reads the locally generated pinned-model catalog"]
fn given_compiled_catalog_when_tables_are_read_then_every_name_is_distinct() -> TestResult {
    let bytes = read_catalog("catalog/aws-api.catalog")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let tables: BTreeSet<_> = catalog
        .schemas()
        .iter()
        .map(|schema| schema.table())
        .collect();
    assert!(!tables.is_empty());
    assert_eq!(tables.len(), catalog.schemas().len());
    assert!(tables.iter().all(|name| name.as_str().len() <= 63));
    println!("distinct_tables={}", tables.len());
    Ok(())
}

#[test]
#[ignore = "requires the ignored pre-wave catalog comparison fixture"]
fn given_original_four_plans_when_catalog_expands_then_encoded_bytes_do_not_change() -> TestResult {
    let before = read_catalog(".local/a2-before.catalog")?;
    let after = read_catalog("catalog/aws-api.catalog")?;
    assert_eq!(original_plans(&before)?, original_plans(&after)?);
    Ok(())
}

fn original_plans(bytes: &[u8]) -> TestResult<BTreeMap<(&str, &str), &[u8]>> {
    let mut selected = BTreeMap::new();
    for value in raw_plans(bytes)?.iter()? {
        let ValueRef::Map(plan) = value? else {
            return Err("plan is not a map".into());
        };
        let key = (text(plan, 1)?, text(plan, 2)?);
        if ORIGINAL.contains(&key) && selected.insert(key, plan.as_bytes()?).is_some() {
            return Err("duplicate original plan".into());
        }
    }
    assert_eq!(selected.len(), ORIGINAL.len());
    Ok(selected)
}

fn raw_plans(bytes: &[u8]) -> TestResult<ListRef<'_>> {
    let root = MapRef::from_container_bytes(bytes, MAX_CATALOG_BYTES)?;
    let Some(ValueRef::Map(extension)) = root.get_field(FieldId::new(60_000)?)? else {
        return Err("runtime extension absent".into());
    };
    let Some(ValueRef::List(plans)) = extension.get_field(FieldId::new(2)?)? else {
        return Err("runtime plans absent".into());
    };
    Ok(plans)
}

fn text(map: MapRef<'_>, field: u32) -> TestResult<&str> {
    match map.get_field(FieldId::new(field)?)? {
        Some(ValueRef::Text(value)) => Ok(value),
        _ => Err("plan text field absent".into()),
    }
}

#[test]
#[ignore = "prints pinned-model thin-slice kinds for the required manual review"]
fn given_thin_slice_when_catalog_is_compiled_then_all_eight_services_have_reviewable_kinds()
-> TestResult {
    let bytes = read_catalog("catalog/aws-api.catalog")?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let mut seen = BTreeSet::new();
    for plan in catalog
        .plans()
        .iter()
        .filter(|plan| SERVICES.contains(&plan.service_name()))
    {
        seen.insert(plan.service_name());
        println!(
            "{} {} {}",
            plan.service_name(),
            plan.operation_name(),
            plan.kind().as_str()
        );
    }
    assert_eq!(seen.len(), SERVICES.len());
    Ok(())
}
