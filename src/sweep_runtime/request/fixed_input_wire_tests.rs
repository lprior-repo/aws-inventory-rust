use super::{TestResult, compiled_inputs, signed, signed_bytes};
use catalog_tool::overlay::FixedValue;
use serde_json::json;

const PROTOCOLS: [&str; 7] = [
    "aws.protocols#awsQuery",
    "aws.protocols#ec2Query",
    "aws.protocols#awsJson1_0",
    "aws.protocols#awsJson1_1",
    "aws.protocols#restJson1",
    "aws.protocols#restXml",
    "smithy.protocols#rpcv2Cbor",
];

fn pairs(body: &[u8]) -> Result<Vec<(&str, &str)>, Box<dyn std::error::Error>> {
    let mut pairs = std::str::from_utf8(body)?
        .split('&')
        .map(|pair| {
            pair.split_once('=')
                .ok_or_else(|| "invalid query pair".into())
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    pairs.sort_unstable();
    Ok(pairs)
}

#[test]
fn given_nested_literals_when_all_protocols_are_signed_then_complete_wire_bodies_match()
-> TestResult {
    for protocol in PROTOCOLS {
        let request = signed(protocol, "neptune")?;
        assert!(request.headers().contains_key("authorization"));
        assert!(request.headers().contains_key("x-amz-content-sha256"));
        match protocol {
            "aws.protocols#awsQuery" => assert_eq!(pairs(request.body())?, vec![
                ("Action", "ListClusters"), ("Attributes.entry.1.key", "owner"),
                ("Attributes.entry.1.value", "inventory"), ("Filters.Filter.1.Name", "engine"),
                ("Filters.Filter.1.Values.Value.1", "neptune"), ("Version", "1"),
            ]),
            "aws.protocols#ec2Query" => assert_eq!(pairs(request.body())?, vec![
                ("Action", "ListClusters"), ("Attributes.1.Key", "owner"),
                ("Attributes.1.Value", "inventory"), ("Filters.1.Name", "engine"),
                ("Filters.1.Values.1", "neptune"), ("Version", "1"),
            ]),
            "aws.protocols#restXml" => assert_eq!(request.body().as_ref(), b"<Input><Attributes><entry><key>owner</key><value>inventory</value></entry></Attributes><Filters><Filter><Name>engine</Name><Values><Value>neptune</Value></Values></Filter></Filters></Input>"),
            "smithy.protocols#rpcv2Cbor" => assert_eq!(request.body().as_ref(), b"\xbfjAttributes\xbfeowneriinventory\xffgFilters\x9f\xbfdNamefenginefValues\x9fgneptune\xff\xff\xff\xff"),
            _ => assert_eq!(serde_json::from_slice::<serde_json::Value>(request.body())?,
                json!({"Attributes":{"owner":"inventory"},"Filters":[{"Name":"engine","Values":["neptune"]}]})),
        }
    }
    Ok(())
}

#[test]
fn given_empty_list_and_map_when_all_protocols_are_signed_then_each_wire_empty_semantics_match()
-> TestResult {
    for protocol in PROTOCOLS {
        let bytes = compiled_inputs(
            protocol,
            FixedValue::list([]),
            FixedValue::map(std::iter::empty::<(&str, FixedValue)>())?,
        )?;
        let request = signed_bytes(&bytes)?;
        assert!(request.headers().contains_key("authorization"));
        match protocol {
            "aws.protocols#awsQuery" => assert_eq!(
                pairs(request.body())?,
                vec![
                    ("Action", "ListClusters"),
                    ("Filters", ""),
                    ("Version", "1"),
                ]
            ),
            "aws.protocols#ec2Query" => assert_eq!(
                pairs(request.body())?,
                vec![("Action", "ListClusters"), ("Version", "1")]
            ),
            "aws.protocols#restXml" => assert_eq!(
                request.body().as_ref(),
                b"<Input><Attributes></Attributes><Filters></Filters></Input>"
            ),
            "smithy.protocols#rpcv2Cbor" => assert_eq!(
                request.body().as_ref(),
                b"\xbfjAttributes\xbf\xffgFilters\x9f\xff\xff"
            ),
            _ => assert_eq!(
                serde_json::from_slice::<serde_json::Value>(request.body())?,
                json!({"Attributes":{},"Filters":[]})
            ),
        }
    }
    Ok(())
}
