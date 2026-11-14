use super::{TestResult, compile_model, fixture_service};
use inventory_core::catalog::{ErrorKind, Target};
use inventory_core::record::FieldId;

const FIELDS: &str = "First\nSecond\nValue\n";
const MODEL: &str = r#"{"smithy":"2.0","shapes":{
"example#Service":{"type":"service","version":"1","errors":[{"target":"example#SharedFailure"}]},
"example#Read":{"type":"operation","errors":[{"target":"example#ClientFailure"}]},
"example#ClientFailure":{"type":"structure","traits":{"smithy.api#error":"client","smithy.api#httpError":409,"aws.protocols#awsQueryError":{"code":"ConflictException","httpResponseCode":422},"smithy.api#retryable":{"throttling":true}}},
"example#ServerFailure":{"type":"structure","traits":{"smithy.api#error":"server","smithy.api#httpError":503,"smithy.api#retryable":{}}},
"example#SharedFailure":{"type":"structure","traits":{"smithy.api#error":"client","smithy.api#httpError":404}},
"example#Input":{"type":"structure","members":{"First":{"target":"smithy.api#String"},"Second":{"target":"smithy.api#String"}}},
"example#Output":{"type":"structure","members":{"Value":{"target":"smithy.api#String"}}}
}}"#;

fn named_shape<'a>(
    service: inventory_core::catalog::Service<'a>,
    expected: &str,
) -> Result<inventory_core::catalog::Shape<'a>, Box<dyn std::error::Error>> {
    let expected = expected.rsplit_once('#').map_or(expected, |(_, name)| name);
    service
        .shapes()?
        .find_map(|shape| match shape {
            Ok(shape) if shape.name().ok() == Some(expected) => Some(Ok(shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or_else(|| format!("shape {expected} missing").into())
}
fn shape_index(
    service: inventory_core::catalog::Service<'_>,
    expected: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let expected = expected.rsplit_once('#').map_or(expected, |(_, name)| name);
    service
        .shapes()?
        .enumerate()
        .find_map(|(index, shape)| match shape {
            Ok(shape) if shape.name().ok() == Some(expected) => Some(Ok(index)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or_else(|| format!("shape {expected} missing").into())
}

#[test]
fn operation_errors_merge_local_and_service_common_references() -> TestResult {
    let (_source, bytes) = compile_model(MODEL, FIELDS)?;
    let service = fixture_service(&bytes)?;
    let mut operations = service.operations()?;
    let operation = operations.next().ok_or("operation missing")??;
    let errors = operation.errors()?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        errors,
        [
            Target::Shape(shape_index(service, "example#SharedFailure")?),
            Target::Shape(shape_index(service, "example#ClientFailure")?)
        ]
    );
    let client = named_shape(service, "example#ClientFailure")?
        .error()?
        .ok_or("client error missing")?;
    assert_eq!(client.kind, ErrorKind::Client);
    assert_eq!(client.code, Some("ConflictException"));
    assert_eq!(client.status, Some(409));
    assert_eq!(client.http_status, Some(409));
    assert_eq!(client.aws_query_status, Some(422));
    assert_eq!(client.retryable, true);
    assert_eq!(client.throttling, true);
    let server = named_shape(service, "example#ServerFailure")?
        .error()?
        .ok_or("server error missing")?;
    assert_eq!(server.kind, ErrorKind::Server);
    assert_eq!(server.code, None);
    assert_eq!(server.status, Some(503));
    assert_eq!(server.retryable, true);
    assert_eq!(server.throttling, false);
    assert_eq!(named_shape(service, "example#Input")?.error()?, None);
    Ok(())
}

#[test]
fn registry_ids_remain_registry_ordered_and_shared_field_names_resolve() -> TestResult {
    let (_source, bytes) = compile_model(MODEL, FIELDS)?;
    let service = fixture_service(&bytes)?;
    let mut shapes = service.shapes()?;
    let input = shapes
        .find_map(|shape| match shape {
            Ok(shape) if shape.name().ok() == Some("Input") => Some(Ok(shape)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("input missing")?;
    let members = input.members()?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        members.iter().map(|member| member.id).collect::<Vec<_>>(),
        [FieldId::new(1)?, FieldId::new(2)?]
    );
    assert_eq!(
        members
            .iter()
            .map(|member| member.name())
            .collect::<Result<Vec<_>, _>>()?,
        ["First", "Second"]
    );
    assert_eq!(service.field_name(FieldId::new(1)?)?, "First");
    assert_eq!(service.field_name(FieldId::new(2)?)?, "Second");
    assert_eq!(service.field_name(FieldId::new(3)?)?, "Value");
    Ok(())
}

#[test]
fn absent_error_and_wire_traits_return_empty_or_none_without_invented_ids() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Read":{"type":"operation"},"example#Input":{"type":"structure","members":{"Anything":{"target":"smithy.api#String"}}}}}"#;
    let (_source, bytes) = compile_model(model, "Anything\n")?;
    let service = fixture_service(&bytes)?;
    let shape = named_shape(service, "example#Input")?;
    let member = shape.members()?.next().ok_or("member missing")??;
    assert_eq!(member.id, FieldId::new(1)?);
    assert_eq!(member.name()?, "Anything");
    assert_eq!(member.wire()?.json_name()?, None);
    assert_eq!(member.wire()?.xml_name()?, None);
    assert_eq!(member.wire()?.ec2_query_name()?, None);
    assert_eq!(member.wire()?.media_type()?, None);
    assert_eq!(
        member.wire()?.binding()?,
        inventory_core::catalog::HttpBinding::Document
    );
    assert_eq!(member.wire()?.xml_flattened()?, false);
    assert_eq!(member.wire()?.xml_attribute()?, false);
    assert_eq!(member.wire()?.streaming()?, false);
    assert_eq!(member.wire()?.sensitive()?, false);
    assert_eq!(member.wire()?.idempotency_token()?, false);
    assert_eq!(shape.element_wire()?.xml_name()?, None);
    let mut operations = service.operations()?;
    assert_eq!(
        operations
            .next()
            .ok_or("operation missing")??
            .errors()?
            .count(),
        0
    );
    Ok(())
}

#[test]
fn modeled_error_statuses_preserve_redirects_and_http_status_boundaries() -> TestResult {
    for status in [100_u16, 308, 599] {
        let model = format!(
            r#"{{"smithy":"2.0","shapes":{{"example#Service":{{"type":"service","version":"1"}},"example#RedirectException":{{"type":"structure","traits":{{"smithy.api#error":"client","smithy.api#httpError":{status}}}}}}}}}"#
        );
        let (_source, bytes) = compile_model(&model, FIELDS)?;
        let service = fixture_service(&bytes)?;
        let error = named_shape(service, "example#RedirectException")?
            .error()?
            .ok_or("modeled error missing")?;
        assert_eq!(error.kind, ErrorKind::Client);
        assert_eq!(error.http_status, Some(status));
        assert_eq!(error.status, Some(status));
        assert_eq!(error.aws_query_status, None);
    }
    Ok(())
}
