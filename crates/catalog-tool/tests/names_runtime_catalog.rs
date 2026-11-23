use inventory_core::catalog::runtime::RuntimeCatalog;
use std::{collections::BTreeSet, fs, path::Path, time::Instant};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
#[ignore = "reads the locally generated pinned-model catalog"]
fn names_runtime_catalog_reports_identifier_counts() -> TestResult {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog");
    let bytes = fs::read(path)?;
    let start = Instant::now();
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    println!("catalog bytes={} load_ms={}", bytes.len(), start.elapsed().as_millis());
    let tables = catalog.schemas().iter().map(|schema| schema.table().as_str());
    let columns = catalog.schemas().iter().flat_map(|schema| schema.columns());
    let table_names: Vec<_> = tables.collect();
    let column_names: Vec<_> = columns.map(|column| column.name()).collect();
    print_counts("tables", &table_names);
    print_counts("columns", &column_names);
    assert_eq!(table_names.len(), table_names.iter().collect::<BTreeSet<_>>().len());
    assert!(!table_names.is_empty());
    Ok(())
}

fn print_counts(label: &str, names: &[&str]) {
    let long = names.iter().filter(|name| name.len() > 63).count();
    let max = names.iter().map(|name| name.len()).max();
    println!("{label} total={} over_63={long} max={max:?}", names.len());
}
