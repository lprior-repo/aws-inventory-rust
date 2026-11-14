use super::*;
use inventory_core::catalog::CatalogError;

const SHAPES: [(&str, &str); 8] = [
    ("Choice", "union"),
    ("Input", "structure"),
    ("ListThings", "operation"),
    ("Lookup", "map"),
    ("Svc", "service"),
    ("Text", "string"),
    ("Thing", "resource"),
    ("Things", "list"),
];
const SCALARS: [&str; 15] = [
    "blob",
    "boolean",
    "string",
    "byte",
    "short",
    "integer",
    "long",
    "float",
    "double",
    "bigInteger",
    "bigDecimal",
    "timestamp",
    "document",
    "enum",
    "intEnum",
];

#[test]
fn complete_ordered_shape_names_and_kinds_survive_compilation() -> TestResult {
    // Given the complete fixture, each numeric position has exactly its modeled short name and kind.
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let shapes = service
        .shapes()?
        .map(|shape| {
            let shape = shape?;
            Ok((shape.name()?, shape.kind()?))
        })
        .collect::<Result<Vec<_>, CatalogError>>()?;
    assert_eq!(shapes, SHAPES);
    Ok(())
}

#[test]
fn operation_and_resource_names_survive_compilation_in_order() -> TestResult {
    // Given modeled operations and resources, their own name lists are not inferred from counts.
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let operations = service
        .operations()?
        .map(|operation| operation?.name())
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(operations, ["ListThings"]);
    assert_eq!(
        service.resources()?.collect::<Result<Vec<_>, _>>()?,
        ["Thing"]
    );
    Ok(())
}

#[test]
fn modeled_operation_input_output_and_flags_survive_compilation() -> TestResult {
    // Given ListThings' modeled input and output, the compiler must preserve both local references.
    let bytes = root(record(MODEL, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    assert_eq!(operation.name()?, "ListThings");
    assert_eq!(operation.flags()?, 31);
    assert_eq!(operation.input()?, Target::Shape(1));
    assert_eq!(operation.output()?, Target::Shape(7));
    Ok(())
}

#[test]
fn absent_operation_input_and_output_compile_to_unit() -> TestResult {
    // Given no modeled operation targets, both directions resolve to the Unit prelude kind.
    let model = MODEL.replace(
        "\"input\":{\"target\":\"n#Input\"},\"output\":{\"target\":\"n#Things\"},",
        "",
    );
    let bytes = root(record(&model, "Zulu\nAlpha\n")?)?;
    let service = Catalog::borrow(&bytes)?
        .service("example")?
        .ok_or("service")?;
    let operation = service.operations()?.next().ok_or("operation")??;
    assert_eq!(operation.input()?, Target::Prelude("unit"));
    assert_eq!(operation.output()?, Target::Prelude("unit"));
    Ok(())
}

fn scalar_model(kind: &str) -> Result<String, serde_json::Error> {
    serde_json::to_string(&serde_json::json!({"shapes": {
        "n#Svc": {"type": "service"},
        "n#Scalar": {"type": kind}
    }}))
}

#[test]
fn every_modeled_scalar_kind_round_trips_through_compiler_and_public_reader() -> TestResult {
    // Given every modeled scalar and enum kind, no mapping may silently substitute a different kind.
    SCALARS.into_iter().try_for_each(|kind| -> TestResult {
        let bytes = root(record(&scalar_model(kind)?, "")?)?;
        let service = Catalog::borrow(&bytes)?
            .service("example")?
            .ok_or("service")?;
        let scalar = service.shape(0)?.ok_or("scalar")?;
        assert_eq!(scalar.name()?, "Scalar");
        assert_eq!(scalar.kind()?, kind);
        Ok(())
    })
}
