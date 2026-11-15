use super::*;

pub(super) fn bytes() -> Result<Vec<u8>, RecordError> {
    let services = ["Example", "Second", "Third"]
        .into_iter()
        .map(encoded_service)
        .collect::<Result<Vec<_>, _>>()?;
    let (field_names, field_offsets) = super::super::field_registry_values();
    let root = map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text("rich-fixture".into())),
        (CatalogKey::FieldNames, field_names),
        (CatalogKey::FieldNameOffsets, field_offsets),
        (CatalogKey::Services, Value::List(services)),
    ])?;
    Ok(Record::encode(&root)?.into_bytes())
}

fn encoded_service(name: &str) -> Result<Value, RecordError> {
    let mut fields = service_fields(name)?;
    fields.insert(
        Key::Field(CatalogKey::Operations.field()?),
        Value::List(vec![rich_operation()?]),
    );
    Ok(Value::Bytes(
        Record::encode(&Value::Map(fields))?.into_bytes(),
    ))
}

fn service_fields(name: &str) -> Result<BTreeMap<Key, Value>, RecordError> {
    let Value::Map(mut fields) = service()? else {
        return Err(RecordError::NotMap);
    };
    let shapes = shapes()?;
    let shape_offsets = super::super::shape_offsets(&shapes)?;
    fields.insert(
        Key::Field(CatalogKey::Name.field()?),
        Value::Text(name.into()),
    );
    fields.insert(
        Key::Field(CatalogKey::ServiceShape.field()?),
        Value::Text(format!("n#{name}")),
    );
    fields.insert(Key::Field(CatalogKey::Shapes.field()?), Value::List(shapes));
    fields.insert(Key::Field(CatalogKey::ShapeOffsets.field()?), shape_offsets);
    fields.insert(
        Key::Field(CatalogKey::Resources.field()?),
        Value::List(vec![Value::Text("Thing".into())]),
    );
    Ok(fields)
}

fn rich_operation() -> Result<Value, RecordError> {
    let Value::Map(mut operation) = operation()? else {
        return Err(RecordError::NotMap);
    };
    operation.insert(Key::Field(CatalogKey::Output.field()?), Value::Int(1));
    Ok(Value::Map(operation))
}

fn shapes() -> Result<Vec<Value>, RecordError> {
    Ok(vec![
        recursive()?,
        list_shape()?,
        map_shape()?,
        union()?,
        resource()?,
    ])
}

fn fixture_shape(
    name: &str,
    kind: i64,
    extra: Vec<(CatalogKey, Value)>,
) -> Result<Value, RecordError> {
    let fields = [
        (CatalogKey::Name, Value::Text(name.into())),
        (CatalogKey::Kind, Value::Int(kind)),
        (CatalogKey::Members, Value::List(vec![])),
    ];
    map(fields.into_iter().chain(extra))
}

fn member(id: i64, target: i64, required: bool) -> Result<Value, RecordError> {
    map([
        (CatalogKey::MemberId, Value::Int(id)),
        (CatalogKey::MemberTarget, Value::Int(target)),
        (CatalogKey::MemberRequired, Value::Bool(required)),
    ])
}

fn recursive() -> Result<Value, RecordError> {
    fixture_shape(
        "Recursive",
        14,
        vec![(CatalogKey::Members, Value::List(vec![member(7, 0, false)?]))],
    )
}

fn list_shape() -> Result<Value, RecordError> {
    fixture_shape("Things", 16, vec![(CatalogKey::Element, Value::Int(0))])
}

fn map_shape() -> Result<Value, RecordError> {
    fixture_shape(
        "Lookup",
        17,
        vec![
            (CatalogKey::MapKey, Value::Int(-3)),
            (CatalogKey::MapValue, Value::Int(1)),
        ],
    )
}

fn union() -> Result<Value, RecordError> {
    fixture_shape(
        "Choice",
        15,
        vec![(
            CatalogKey::Members,
            Value::List(vec![member(7, 2, true)?, member(8, -3, false)?]),
        )],
    )
}

fn resource() -> Result<Value, RecordError> {
    fixture_shape(
        "Thing",
        22,
        vec![(CatalogKey::Arn, Value::Text("thing/{id}".into()))],
    )
}
