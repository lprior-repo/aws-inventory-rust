use super::support::{TestResult, field, split};
use inventory_harness::proxy::{BodyStyle, Fault};

pub fn check(style: BodyStyle, bytes: &[u8], code: &str, fault: Fault) -> TestResult {
    let message = if fault == Fault::ExpiredSignature {
        "Signature expired: injected by deterministic proxy"
    } else {
        "Injected by deterministic proxy"
    };
    let body = split(bytes)?.1;
    match style {
        BodyStyle::Json => json(body, code, message)?,
        BodyStyle::Cbor => cbor(body, code, message)?,
        _ => check_xml(style, bytes, code, message)?,
    }
    assert_eq!(field(bytes, "content-type")?, content_type(style));
    Ok(())
}

fn json(body: &[u8], code: &str, message: &str) -> TestResult {
    let value: serde_json::Value = serde_json::from_slice(body)?;
    assert_eq!(
        value,
        serde_json::json!({"__type": code, "message": message})
    );
    Ok(())
}

fn cbor(body: &[u8], code: &str, message: &str) -> TestResult {
    let mut decoder = minicbor::Decoder::new(body);
    assert_eq!(decoder.map()?, Some(2));
    assert_eq!(decoder.str()?, "__type");
    assert_eq!(decoder.str()?, code);
    assert_eq!(decoder.str()?, "message");
    assert_eq!(decoder.str()?, message);
    assert_eq!(decoder.position(), body.len());
    Ok(())
}

fn xml(
    style: BodyStyle,
    code: &str,
    message: &str,
    id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(match style {
        BodyStyle::QueryXml => format!(
            "<ErrorResponse><Error><Type>Sender</Type><Code>{code}</Code><Message>{message}</Message></Error><RequestId>{id}</RequestId></ErrorResponse>"
        ),
        BodyStyle::RestXml => format!(
            "<Error><Code>{code}</Code><Message>{message}</Message><RequestId>{id}</RequestId></Error>"
        ),
        BodyStyle::Ec2Xml => format!(
            "<Response><Errors><Error><Code>{code}</Code><Message>{message}</Message></Error></Errors><RequestID>{id}</RequestID></Response>"
        ),
        _ => return Err("not an XML style".into()),
    })
}

fn content_type(style: BodyStyle) -> &'static str {
    match style {
        BodyStyle::Json => "application/x-amz-json-1.1",
        BodyStyle::Cbor => "application/cbor",
        _ => "application/xml",
    }
}

fn check_xml(style: BodyStyle, bytes: &[u8], code: &str, message: &str) -> TestResult {
    let id = field(bytes, "x-amzn-RequestId")?;
    assert_eq!(split(bytes)?.1, xml(style, code, message, id)?.as_bytes());
    Ok(())
}
