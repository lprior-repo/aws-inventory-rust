use inventory_core::catalog::Catalog;
use serde_json::json;

fn fixture() -> Vec<u8> {
    let model = json!({
        "shapes": {
            "test#Service": {
                "type": "service",
                "version": "1",
                "operations": [{"target": "test#Inspect"}],
                "traits": {"aws.protocols#restXml": {}}
            },
            "test#Inspect": {"type": "operation", "input": {"target": "test#Input"}},
            "test#Input": {"type": "structure", "members": {
                "Records": {"target": "test#Records"},
                "Flat": {"target": "test#Flat", "traits": {"smithy.api#xmlFlattened": {}}},
                "Nested": {"target": "test#Nested"},
                "Labels": {"target": "test#Labels"},
                "Text": {"target": "smithy.api#String"},
                "Empty": {"target": "smithy.api#String"}
            }},
            "test#Records": {"type": "list", "member": {"target": "test#Pair"}},
            "test#Flat": {"type": "list", "member": {"target": "test#Pair"}},
            "test#Nested": {"type": "list", "member": {"target": "test#Words"}},
            "test#Words": {"type": "list", "member": {"target": "smithy.api#String"}},
            "test#Pair": {"type": "structure", "members": {
                "First": {"target": "smithy.api#String"},
                "Second": {"target": "smithy.api#String"}
            }},
            "test#Labels": {"type": "map", "key": {"target": "smithy.api#String"}, "value": {"target": "smithy.api#String"}}
        }
    });
    super::super::model::compile_service(&model, "test#Service")
}

fn equal(left: &str, right: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let bytes = fixture();
    let catalog = Catalog::borrow(&bytes)?;
    let service = catalog
        .service("Service")?
        .ok_or("fixture service exists")?;
    let mut inspect = None;
    for operation in service.operations()? {
        let operation = operation?;
        if operation.name()? == "Inspect" {
            inspect = Some(operation);
            break;
        }
    }
    let operation = inspect.ok_or("fixture Inspect operation exists")?;
    Ok(super::equal(
        left.as_bytes(),
        right.as_bytes(),
        service,
        operation,
    ))
}

#[test]
fn structures_ignore_member_order_and_xml_text_representation()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(equal(
        "<Input><Records><member><First>a</First><Second>b</Second></member></Records><Text>&amp;</Text><Empty/></Input>",
        "<Input><Empty></Empty><Text><![CDATA[&]]></Text><Records><member><Second>b</Second><First>a</First></member></Records></Input>"
    )?);
    Ok(())
}

#[test]
fn list_of_structures_preserves_element_order() -> Result<(), Box<dyn std::error::Error>> {
    let body = "<Input><Records><member><First>a</First><Second>1</Second></member><member><First>b</First><Second>2</Second></member></Records></Input>";
    assert!(equal(body, body)?);
    assert!(!equal(
        body,
        "<Input><Records><member><First>b</First><Second>2</Second></member><member><First>a</First><Second>1</Second></member></Records></Input>"
    )?);
    Ok(())
}

#[test]
fn flattened_lists_use_enclosing_member_names_and_preserve_order()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(equal(
        "<Input><Flat><First>a</First><Second>b</Second></Flat><Flat><First>c</First><Second>d</Second></Flat></Input>",
        "<Input><Flat><Second>b</Second><First>a</First></Flat><Flat><Second>d</Second><First>c</First></Flat></Input>"
    )?);
    assert!(!equal(
        "<Input><Flat><First>a</First><Second>b</Second></Flat><Flat><First>c</First><Second>d</Second></Flat></Input>",
        "<Input><Flat><First>c</First><Second>d</Second></Flat><Flat><First>a</First><Second>b</Second></Flat></Input>"
    )?);
    Ok(())
}

#[test]
fn flattened_and_wrapped_list_encodings_are_not_interchangeable()
-> Result<(), Box<dyn std::error::Error>> {
    let body = "<Input><Flat><First>a</First><Second>b</Second></Flat></Input>";
    assert!(equal(body, body)?);
    assert!(!equal(
        "<Input><Flat><member><First>a</First><Second>b</Second></member></Flat></Input>",
        body
    )?);
    Ok(())
}

#[test]
fn nested_lists_preserve_each_ordered_level() -> Result<(), Box<dyn std::error::Error>> {
    let body = "<Input><Nested><member><member>a</member><member>b</member></member><member><member>c</member></member></Nested></Input>";
    assert!(equal(body, body)?);
    assert!(!equal(
        "<Input><Nested><member><member>a</member><member>b</member></member></Nested></Input>",
        "<Input><Nested><member><member>b</member><member>a</member></member></Nested></Input>"
    )?);
    Ok(())
}

#[test]
fn map_entry_order_is_ignored_but_keys_and_values_are_exact()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(equal(
        "<Input><Labels><entry><key>a</key><value>one</value></entry><entry><key>b</key><value>two</value></entry></Labels></Input>",
        "<Input><Labels><entry><key>b</key><value>two</value></entry><entry><key>a</key><value>one</value></entry></Labels></Input>"
    )?);
    assert!(!equal(
        "<Input><Labels><entry><key>a</key><value>one</value></entry></Labels></Input>",
        "<Input><Labels><entry><key>a</key><value>wrong</value></entry></Labels></Input>"
    )?);
    assert!(!equal(
        "<Input><Labels><entry><key>a</key><value>one</value></entry><entry><key>a</key><value>two</value></entry></Labels></Input>",
        "<Input><Labels><entry><key>a</key><value>one</value></entry><entry><key>b</key><value>two</value></entry></Labels></Input>"
    )?);
    assert!(!equal(
        "<Input><Labels><entry><key>a</key><value>one</value></entry></Labels></Input>",
        "<Input><Labels><entry><value>one</value></entry></Labels></Input>"
    )?);
    Ok(())
}

#[test]
fn namespaces_and_document_root_are_significant() -> Result<(), Box<dyn std::error::Error>> {
    let body = "<Input><Text>x</Text></Input>";
    assert!(equal(body, body)?);
    assert!(!equal(
        "<Input xmlns=\"urn:left\"><Text>x</Text></Input>",
        "<Input xmlns=\"urn:right\"><Text>x</Text></Input>"
    )?);
    assert!(!equal(body, "<Other><Text>x</Text></Other>")?);
    assert!(!equal("<Input/><Input/>", "<Input/>")?);
    Ok(())
}
