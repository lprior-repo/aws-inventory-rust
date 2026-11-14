use super::model::{convert, find_list_operation, find_operation};
use super::*;

pub(super) fn assert_query_limits(
    service: Service<'_>,
    protocol: Protocol,
    registry: &Registry,
) -> FixtureResult<()> {
    let empty = Value::Map(BTreeMap::new());
    let record = Record::encode(&empty)?;
    let operation = find_operation(service, "EmptyInputAndEmptyOutput")?;
    let limits = RequestLimits::new(8, 1024, 1024)?;
    let context = RequestContext {
        service,
        operation,
        protocol,
        input: record.root(),
        endpoint: "https://example.com",
        limits,
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    };
    assert!(matches!(
        encode(context),
        Err(crate::request::EncodeError::Limit("body"))
    ));

    let operation = find_list_operation(service)?;
    let Target::Shape(index) = operation.input()? else {
        return Err(fixture_error("list operation input is not a structure"));
    };
    let input_shape = service
        .shape(index)?
        .ok_or_else(|| fixture_error("list operation input shape absent"))?;
    let mut params = JsonMap::new();
    for item in input_shape.members()? {
        let member = item?;
        if let Target::Shape(list_index) = member.target {
            if let Some(shape) = service.shape(list_index)? {
                if shape.kind()? == "list" && is_string_list(service, shape)? {
                    let name = service.field_name(member.id)?;
                    params.insert(
                        name.to_owned(),
                        Json::Array(vec![Json::String("x".to_owned()); 4_097]),
                    );
                    break;
                }
            }
        }
    }
    assert!(
        !params.is_empty(),
        "QueryLists fixture has no string list member"
    );
    let input = convert(service, registry, operation.input()?, &Json::Object(params))?;
    let record = Record::encode(&input)?;
    let context = RequestContext {
        service,
        operation,
        protocol,
        input: record.root(),
        endpoint: "https://example.com",
        limits: RequestLimits::default(),
        idempotency_token: None,
        compression: CompressionPolicy::default(),
        addressing: None,
    };
    assert!(matches!(
        encode(context),
        Err(crate::request::EncodeError::Limit("query traversal"))
    ));
    Ok(())
}

fn is_string_list(
    service: Service<'_>,
    list: inventory_core::catalog::Shape<'_>,
) -> FixtureResult<bool> {
    match list.element()? {
        Some(Target::Prelude("string")) => Ok(true),
        Some(Target::Shape(index)) => match service.shape(index)? {
            Some(element) => Ok(element.kind()? == "string"),
            None => Err(fixture_error("list element shape absent")),
        },
        _ => Ok(false),
    }
}
