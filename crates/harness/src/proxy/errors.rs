use super::{BodyStyle, Decision, Fault, ProxyError, protocol::Connection, response::Response};

type Envelope = (&'static str, Vec<u8>);

fn descriptor(fault: Fault, style: BodyStyle) -> Option<(u16, &'static str)> {
    use BodyStyle::{Ec2Xml, QueryXml, RestXml};
    Some(match (fault, style) {
        (Fault::Throttle, RestXml) => (503, "SlowDown"),
        (Fault::Throttle, Ec2Xml) => (503, "RequestLimitExceeded"),
        (Fault::Throttle, _) => (400, "ThrottlingException"),
        (Fault::InternalFailure, _) => (500, "InternalFailure"),
        (Fault::ServiceUnavailable, _) => (503, "ServiceUnavailable"),
        (Fault::AccessDenied, RestXml) => (403, "AccessDenied"),
        (Fault::AccessDenied, Ec2Xml) => (403, "UnauthorizedOperation"),
        (Fault::AccessDenied, _) => (403, "AccessDeniedException"),
        (Fault::ExpiredPaginationToken, _) => (400, "ExpiredNextTokenException"),
        (Fault::ExpiredCredentials, RestXml) => (403, "ExpiredToken"),
        (Fault::ExpiredCredentials, Ec2Xml) => (403, "RequestExpired"),
        (Fault::ExpiredCredentials, _) => (403, "ExpiredTokenException"),
        (Fault::ExpiredSignature, RestXml) => (403, "RequestTimeTooSkewed"),
        (Fault::ExpiredSignature, QueryXml) => (403, "SignatureDoesNotMatch"),
        (Fault::ExpiredSignature, _) => (403, "InvalidSignatureException"),
        _ => return None,
    })
}

pub(crate) fn canned(
    decision: Decision,
    style: BodyStyle,
    ordinal: u64,
) -> Result<Option<Response>, ProxyError> {
    let Some(fault) = decision.fault else {
        return Ok(None);
    };
    let Some((status, code)) = descriptor(fault, style) else {
        return Ok(None);
    };
    let request_id = format!("proxy-{ordinal:016x}-{:016x}", decision.entropy);
    let (content_type, body) = envelope(style, code, message(fault), &request_id)?;
    Ok(Some(wire_response(
        status,
        code,
        &request_id,
        content_type,
        body,
    )))
}

fn message(fault: Fault) -> &'static str {
    if fault == Fault::ExpiredSignature {
        "Signature expired: injected by deterministic proxy"
    } else {
        "Injected by deterministic proxy"
    }
}

fn wire_response(
    status: u16,
    code: &str,
    request_id: &str,
    content_type: &str,
    body: Vec<u8>,
) -> Response {
    let head = format!(
        "HTTP/1.1 {status} Injected\r\nx-amzn-ErrorType: {code}\r\nx-amzn-RequestId: {request_id}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    Response {
        head: head.into_bytes(),
        body,
        connection: Connection::KeepAlive,
    }
}

fn envelope(style: BodyStyle, code: &str, message: &str, id: &str) -> Result<Envelope, ProxyError> {
    let xml = match style {
        BodyStyle::Json => return Ok(json(code, message)),
        BodyStyle::Cbor => return Ok(("application/cbor", cbor(code, message)?)),
        BodyStyle::QueryXml => format!(
            "<ErrorResponse><Error><Type>Sender</Type><Code>{code}</Code><Message>{message}</Message></Error><RequestId>{id}</RequestId></ErrorResponse>"
        ),
        BodyStyle::RestXml => format!(
            "<Error><Code>{code}</Code><Message>{message}</Message><RequestId>{id}</RequestId></Error>"
        ),
        BodyStyle::Ec2Xml => format!(
            "<Response><Errors><Error><Code>{code}</Code><Message>{message}</Message></Error></Errors><RequestID>{id}</RequestID></Response>"
        ),
    };
    Ok(("application/xml", xml.into_bytes()))
}

fn json(code: &str, message: &str) -> Envelope {
    let body = format!(r#"{{"__type":"{code}","message":"{message}"}}"#);
    ("application/x-amz-json-1.1", body.into_bytes())
}

fn cbor(code: &str, message: &str) -> Result<Vec<u8>, ProxyError> {
    const ERROR_BODY_CAPACITY: usize = 512;
    let mut buffer = [0u8; ERROR_BODY_CAPACITY];
    let mut encoder = minicbor::Encoder::new(buffer.as_mut_slice());
    encoder
        .map(2)
        .and_then(|e| e.str("__type"))
        .and_then(|e| e.str(code))
        .and_then(|e| e.str("message"))
        .and_then(|e| e.str(message))
        .map_err(|_| ProxyError::Limit("CBOR error body"))?;
    let used = ERROR_BODY_CAPACITY.saturating_sub(encoder.writer().len());
    let body = buffer.get(..used).ok_or(ProxyError::Framing)?;
    Ok(body.to_vec())
}

pub(crate) fn rejection(status: u16, code: &'static str) -> Vec<u8> {
    let reason = match status {
        417 => "Expectation Failed",
        502 => "Bad Gateway",
        _ => "Proxy Error",
    };
    format!("HTTP/1.1 {status} {reason}\r\nx-proxy-error: {code}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").into_bytes()
}
