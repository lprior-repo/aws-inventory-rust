use super::{TestResult, compile_model, fixture_service};
use inventory_core::{
    catalog::{HttpBinding, TimestampFormat},
    record::FieldId,
};

const FIELDS: &str =
    "Body\nHeader\nLabel\nName\nPayload\nPrefix\nQuery\nStatus\nValue\nDate\nFlat\nSecret\n";

fn named_shape<'a>(
    service: inventory_core::catalog::Service<'a>,
    expected: &str,
) -> Result<inventory_core::catalog::Shape<'a>, Box<dyn std::error::Error>> {
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

#[test]
fn member_wire_names_flags_and_shape_wire_traits_survive_compilation() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{
"example#Service":{"type":"service","version":"1"},
"example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"smithy.api#jsonName":"wireJson","smithy.api#xmlName":"wireXml","aws.protocols#ec2QueryName":"wireEc2","smithy.api#httpLabel":{},"smithy.api#xmlNamespace":{"uri":"urn:member","prefix":"m"},"smithy.api#timestampFormat":"http-date","smithy.api#sensitive":{},"smithy.api#idempotencyToken":{},"smithy.api#hostLabel":{},"smithy.api#mediaType":"application/example"}},"Date":{"target":"smithy.api#Timestamp","traits":{"smithy.api#timestampFormat":"date-time"}},"Flat":{"target":"smithy.api#String","traits":{"smithy.api#xmlFlattened":{},"smithy.api#xmlAttribute":{}}},"Secret":{"target":"example#Secret"}}},
"example#Secret":{"type":"string","traits":{"smithy.api#sensitive":{},"smithy.api#mediaType":"application/secret"}},
"example#Packet":{"type":"structure","traits":{"smithy.api#xmlName":"PacketWire","smithy.api#xmlNamespace":{"uri":"urn:shape","prefix":"p"},"smithy.api#timestampFormat":"epoch-seconds","smithy.api#sensitive":{},"smithy.api#mediaType":"application/shape"},"members":{"Value":{"target":"smithy.api#String"}}}
}}"#;
    let (_source, bytes) = compile_model(model, FIELDS)?;
    let service = fixture_service(&bytes)?;
    let input = named_shape(service, "example#Input")?;
    let member = input
        .members()?
        .find_map(|member| match member {
            Ok(member) if member.name().ok() == Some("Name") => Some(Ok(member)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("name member missing")?;
    assert_eq!(member.id, FieldId::new(4)?);
    assert_eq!(member.name()?, "Name");
    assert_eq!(member.wire()?.json_name()?, Some("wireJson"));
    assert_eq!(member.wire()?.xml_name()?, Some("wireXml"));
    assert_eq!(member.wire()?.ec2_query_name()?, Some("wireEc2"));
    assert_eq!(member.wire()?.binding()?, HttpBinding::Label);
    assert_eq!(
        member
            .wire()?
            .xml_namespace()?
            .map(|ns| (ns.uri, ns.prefix)),
        Some(("urn:member", Some("m")))
    );
    assert_eq!(
        member.wire()?.timestamp_format()?,
        Some(TimestampFormat::HttpDate)
    );
    assert_eq!(member.wire()?.sensitive()?, true);
    assert_eq!(member.wire()?.idempotency_token()?, true);
    assert_eq!(member.wire()?.host_label()?, true);
    let date = input
        .members()?
        .find_map(|member| match member {
            Ok(member) if member.name().ok() == Some("Date") => Some(Ok(member)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("date member missing")?;
    assert_eq!(
        date.wire()?.timestamp_format()?,
        Some(TimestampFormat::DateTime)
    );
    let flat = input
        .members()?
        .find_map(|member| match member {
            Ok(member) if member.name().ok() == Some("Flat") => Some(Ok(member)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("flat member missing")?;
    assert_eq!(flat.wire()?.xml_flattened()?, true);
    assert_eq!(flat.wire()?.xml_attribute()?, true);
    let secret = input
        .members()?
        .find_map(|member| match member {
            Ok(member) if member.name().ok() == Some("Secret") => Some(Ok(member)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .transpose()?
        .ok_or("secret member missing")?;
    assert_eq!(secret.wire()?.sensitive()?, false);
    assert_eq!(
        named_shape(service, "example#Secret")?
            .wire()?
            .sensitive()?,
        true
    );
    assert_eq!(
        named_shape(service, "example#Secret")?
            .wire()?
            .media_type()?,
        Some("application/secret")
    );
    let packet = named_shape(service, "example#Packet")?;
    assert_eq!(packet.wire()?.xml_name()?, Some("PacketWire"));
    assert_eq!(
        packet
            .wire()?
            .xml_namespace()?
            .map(|ns| (ns.uri, ns.prefix)),
        Some(("urn:shape", Some("p")))
    );
    assert_eq!(
        packet.wire()?.timestamp_format()?,
        Some(TimestampFormat::EpochSeconds)
    );
    assert_eq!(packet.wire()?.sensitive()?, true);
    assert_eq!(packet.wire()?.media_type()?, Some("application/shape"));
    assert_eq!(packet.element_wire()?.xml_name()?, None);
    Ok(())
}

#[test]
fn each_http_binding_is_preserved_as_its_typed_binding() -> TestResult {
    let cases = [
        ("smithy.api#httpLabel", "{}", HttpBinding::Label),
        ("smithy.api#httpQuery", "\"q\"", HttpBinding::Query("q")),
        ("smithy.api#httpQueryParams", "{}", HttpBinding::QueryParams),
        (
            "smithy.api#httpHeader",
            "\"x-custom\"",
            HttpBinding::Header("x-custom"),
        ),
        (
            "smithy.api#httpPrefixHeaders",
            "\"x-meta-\"",
            HttpBinding::PrefixHeaders("x-meta-"),
        ),
        ("smithy.api#httpPayload", "{}", HttpBinding::Payload),
        (
            "smithy.api#httpResponseCode",
            "{}",
            HttpBinding::ResponseCode,
        ),
    ];
    for (trait_id, payload, expected) in cases {
        let model = r#"{"smithy":"2.0","shapes":{"example#Service":{"type":"service","version":"1"},"example#Input":{"type":"structure","members":{"Name":{"target":"smithy.api#String","traits":{"__TRAIT__":__PAYLOAD__}}}}}}"#
            .replace("__TRAIT__", trait_id)
            .replace("__PAYLOAD__", payload);
        let (_source, bytes) = compile_model(&model, FIELDS)?;
        let member = named_shape(fixture_service(&bytes)?, "example#Input")?
            .members()?
            .next()
            .ok_or("member missing")??;
        assert_eq!(member.wire()?.binding()?, expected, "{trait_id}");
    }
    Ok(())
}

#[test]
fn pseudo_member_traits_and_optional_member_metadata_are_retained() -> TestResult {
    let model = r#"{"smithy":"2.0","shapes":{
"example#Service":{"type":"service","version":"1"},
"example#Values":{"type":"list","traits":{"smithy.api#sparse":{}},"member":{"target":"smithy.api#String","traits":{"smithy.api#xmlName":"value-wire","smithy.api#xmlFlattened":{},"smithy.api#streaming":{}}}},
"example#Labels":{"type":"map","traits":{"smithy.api#sparse":{}},"key":{"target":"smithy.api#String","traits":{"smithy.api#xmlName":"key-wire"}},"value":{"target":"smithy.api#String","traits":{"smithy.api#xmlName":"value-wire","smithy.api#sensitive":{}}}}
}}"#;
    let (_source, bytes) = compile_model(model, "")?;
    let service = fixture_service(&bytes)?;
    let values = named_shape(service, "example#Values")?;
    assert_eq!(values.wire()?.sparse()?, true);
    assert_eq!(
        values.element()?,
        Some(inventory_core::catalog::Target::Prelude("string"))
    );
    assert_eq!(values.element_wire()?.xml_name()?, Some("value-wire"));
    assert_eq!(values.element_wire()?.binding()?, HttpBinding::Document);
    assert_eq!(values.element_wire()?.xml_flattened()?, true);
    assert_eq!(values.element_wire()?.streaming()?, true);
    assert_eq!(values.wire()?.sensitive()?, false);
    let labels = named_shape(service, "example#Labels")?;
    assert_eq!(labels.wire()?.sparse()?, true);
    assert_eq!(labels.map_key_wire()?.xml_name()?, Some("key-wire"));
    assert_eq!(labels.map_value_wire()?.xml_name()?, Some("value-wire"));
    assert_eq!(labels.map_value_wire()?.sensitive()?, true);
    assert_eq!(labels.map_key_wire()?.xml_attribute()?, false);
    Ok(())
}
